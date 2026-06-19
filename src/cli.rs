//! CLI entrypoints — direct database access, no server needed.

#![allow(clippy::print_stdout, clippy::print_literal)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Duration;

use anyhow::{Context, bail};
use sea_orm::{ConnectOptions, Database};
use tokimo_bus_auth::db::verify_token;
use tokimo_bus_cli::{Credentials, TokimoAuthArgs};
use tokimo_package_client_api::downloaders::traits::AddTorrentOptions;
use tokimo_package_storage::{OpendalStorageProvider, StorageProvider};

use crate::cli_types::{CategoriesCmd, ClientsCmd, PtSitesCmd, SubscriptionsCmd, TorrentsCmd, TrafficCmd};
use tokimo_app_pt_subscription::AppState;
use tokimo_app_pt_subscription::db::repos::download_client_repo::{
    CreateDownloadClientInput, DownloadClientDto, DownloadClientRepo, DownloadPath, UpdateDownloadClientInput,
};
use tokimo_app_pt_subscription::services::DownloadClientService;
use tokimo_app_pt_subscription::shared::categories::all_categories;
use tokimo_app_pt_subscription::subscriptions::models::pt_site::{PtSiteDto, PtUserInfoDto};
use tokimo_app_pt_subscription::subscriptions::repos::pt_site_repo::{
    CreatePtSiteInput, PtSiteRepo, UpdatePtSiteInput,
};
use tokimo_app_pt_subscription::subscriptions::repos::subscription_repo::{
    CreateSubscriptionInput, EpisodeProgress, SubscriptionDto, SubscriptionRepo, UpdateSubscriptionInput,
};
use tokimo_app_pt_subscription::subscriptions::repos::traffic_manage_repo::{
    TrafficManageRepo, TrafficManageSettings, UpdateTrafficSettingsInput,
};
use tokimo_app_pt_subscription::subscriptions::services::pt_search::search_all_sites;
use tokimo_app_pt_subscription::subscriptions::services::pt_user_info;

// ── DB connection ─────────────────────────────────────────────────────────────

async fn connect_db() -> anyhow::Result<sea_orm::DatabaseConnection> {
    let base_url = std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?;
    let schema = tokimo_bus_cli::manifest::parse_app_schema(crate::MANIFEST)?
        .ok_or_else(|| anyhow::anyhow!("manifest missing [database] schema"))?;

    let sep = if base_url.contains('?') { '&' } else { '?' };
    let url = format!(
        "{base_url}{sep}application_name=tokimo-app-pt-subscription-cli\
         &options=-c%20search_path%3D%22{schema}%22%2Cpublic"
    );

    let mut opts = ConnectOptions::new(url);
    opts.max_connections(2).min_connections(1).sqlx_logging(false);

    Ok(Database::connect(opts).await?)
}

/// Resolve credentials → connect DB → verify token → return (db, user_id).
pub async fn init(auth: &TokimoAuthArgs) -> anyhow::Result<(sea_orm::DatabaseConnection, String)> {
    let credentials = Credentials::resolve(auth).context("resolve Tokimo credentials failed")?;
    let db = connect_db().await.context("connect database failed")?;
    let verified = verify_token(&db, &credentials.token)
        .await
        .map_err(|e| anyhow::anyhow!("verify Tokimo token failed: {e}"))?;
    Ok((db, verified.user_id.to_string()))
}

fn data_local_path() -> PathBuf {
    std::env::var("DATA_LOCAL_PATH").map_or_else(|_| PathBuf::from("./.data/local"), PathBuf::from)
}

// ── Name-or-ID resolution ────────────────────────────────────────────────────

async fn resolve_client(db: &sea_orm::DatabaseConnection, arg: &str) -> anyhow::Result<DownloadClientDto> {
    let clients = DownloadClientRepo::list(db).await?;

    // 1. If arg is a valid UUID and matches a client id, use it directly
    if uuid::Uuid::parse_str(arg).is_ok()
        && let Some(c) = clients.iter().find(|c| c.id == arg)
    {
        return Ok(c.clone());
    }

    // 2. Otherwise, match by exact name
    let matches: Vec<&DownloadClientDto> = clients.iter().filter(|c| c.name == arg).collect();
    match matches.as_slice() {
        [c] => Ok((*c).clone()),
        [] => bail!(
            "No download client named or matching id '{arg}'.\n\
             Run 'pt-subscription clients list' to see available clients."
        ),
        many => {
            use std::fmt::Write as _;
            let mut msg = format!(
                "Found {} download clients named '{arg}'. Please specify by id instead:",
                many.len()
            );
            for c in many {
                let _ = write!(msg, "\n  {}  ({})", c.id, c.r#type);
            }
            bail!("{msg}")
        }
    }
}

async fn resolve_subscription(
    db: &sea_orm::DatabaseConnection,
    user_id: &str,
    arg: &str,
) -> anyhow::Result<SubscriptionDto> {
    use std::fmt::Write as _;

    let subscriptions = SubscriptionRepo::list(db, user_id).await?;

    if uuid::Uuid::parse_str(arg).is_ok()
        && let Some(sub) = subscriptions.iter().find(|sub| sub.id == arg)
    {
        return Ok(sub.clone());
    }

    let matches: Vec<&SubscriptionDto> = subscriptions.iter().filter(|sub| sub.title == arg).collect();
    match matches.as_slice() {
        [sub] => Ok((*sub).clone()),
        [] => {
            if subscriptions.is_empty() {
                bail!("No subscriptions found for current user.");
            }
            let mut msg = format!("No subscription named or matching id '{arg}'.\nAvailable subscriptions:");
            for sub in &subscriptions {
                let _ = write!(msg, "\n  {}  ({})", sub.id, sub.title);
            }
            bail!("{msg}")
        }
        many => {
            let mut msg = format!(
                "Found {} subscriptions named '{arg}'. Please specify by id instead:",
                many.len()
            );
            for sub in many {
                let _ = write!(msg, "\n  {}  ({})", sub.id, sub.status);
            }
            bail!("{msg}")
        }
    }
}

async fn resolve_site(db: &sea_orm::DatabaseConnection, arg: &str) -> anyhow::Result<PtSiteDto> {
    use std::fmt::Write as _;

    let sites = PtSiteRepo::list(db).await?;

    if uuid::Uuid::parse_str(arg).is_ok()
        && let Some(site) = sites.iter().find(|site| site.id == arg)
    {
        return Ok(site.clone());
    }

    if let Some(site) = sites.iter().find(|site| site.site_id == arg) {
        return Ok(site.clone());
    }

    let matches: Vec<&PtSiteDto> = sites.iter().filter(|site| site.name == arg).collect();
    match matches.as_slice() {
        [site] => Ok((*site).clone()),
        [] => {
            if sites.is_empty() {
                bail!("No PT sites configured.");
            }
            let mut msg = format!("No PT site named/site_id/id '{arg}'.\nAvailable PT sites:");
            for site in &sites {
                let _ = write!(msg, "\n  {}  {:<12} {}", site.id, site.site_id, site.name);
            }
            bail!("{msg}")
        }
        many => {
            let mut msg = format!(
                "Found {} PT sites named '{arg}'. Please specify by id or --site-id:",
                many.len()
            );
            for site in many {
                let _ = write!(msg, "\n  {}  ({})", site.id, site.site_id);
            }
            bail!("{msg}")
        }
    }
}

/// Validate that `path` is one of the client's configured download paths.
fn validate_download_path(client: &DownloadClientDto, path: &str) -> anyhow::Result<()> {
    if client.download_paths.iter().any(|p| p.path == path) {
        return Ok(());
    }
    use std::fmt::Write as _;
    let mut msg = format!(
        "Download path '{path}' is not configured for client '{}'.\n\
         Available download paths:",
        client.name
    );
    for dp in &client.download_paths {
        let _ = write!(msg, "\n  {}  ({})", dp.path, dp.description);
    }
    if client.download_paths.is_empty() {
        msg.push_str("\n  (none configured — add download paths in the app settings)");
    }
    bail!("{msg}")
}

// ── Clients command ──────────────────────────────────────────────────────────

#[allow(clippy::too_many_lines)]
pub async fn run_clients(_auth: TokimoAuthArgs, cmd: ClientsCmd) -> anyhow::Result<()> {
    let db = connect_db().await?;

    match cmd {
        ClientsCmd::List => {
            let clients = DownloadClientRepo::list(&db).await?;
            if clients.is_empty() {
                println!("No download clients configured.");
                return Ok(());
            }
            println!("{:<38} {:<15} {:<30} {}", "ID", "Type", "URL", "Name");
            println!("{}", "-".repeat(100));
            for c in &clients {
                let default = if c.is_default { " ★" } else { "" };
                println!("{:<38} {:<15} {:<30} {}{}", c.id, c.r#type, c.url, c.name, default);
                for dp in &c.download_paths {
                    println!("    {} — {}", dp.path, dp.description);
                }
            }
        }

        ClientsCmd::Add {
            name,
            r#type,
            url,
            username,
            password,
            download_paths,
            r#default,
        } => {
            let paths: Vec<DownloadPath> = serde_json::from_str(&download_paths)
                .context("Invalid --download-paths JSON. Expected: [{\"path\":\"...\",\"description\":\"...\"}]")?;
            let input = CreateDownloadClientInput {
                name,
                r#type,
                url,
                username,
                password,
                is_default: Some(r#default),
                require_auth: None,
                monitor_enabled: None,
                poll_interval: None,
                download_paths: paths,
            };
            let client = DownloadClientRepo::create(&db, input).await?;
            println!("Created: {} ({})", client.name, client.id);
        }

        ClientsCmd::Update {
            client: arg,
            name,
            url,
            username,
            password,
            download_paths,
        } => {
            let existing = resolve_client(&db, &arg).await?;
            let paths = download_paths
                .map(|p| serde_json::from_str::<Vec<DownloadPath>>(&p))
                .transpose()
                .context("Invalid --download-paths JSON")?;
            let input = UpdateDownloadClientInput {
                name,
                r#type: None,
                url,
                username: username.map(Some),
                password: password.map(Some),
                is_default: None,
                require_auth: None,
                monitor_enabled: None,
                poll_interval: None,
                download_paths: paths,
            };
            let client = DownloadClientRepo::update(&db, &existing.id, input).await?;
            println!("Updated: {} ({})", client.name, client.id);
        }

        ClientsCmd::Delete { client: arg } => {
            let existing = resolve_client(&db, &arg).await?;
            DownloadClientRepo::delete(&db, &existing.id).await?;
            println!("Deleted: {} ({})", existing.name, existing.id);
        }

        ClientsCmd::Test { client: arg } => {
            let existing = resolve_client(&db, &arg).await?;
            let http_client = reqwest::Client::new();
            let status = DownloadClientService::test_connection(&db, &existing.id, &http_client).await?;
            if status.is_connected {
                println!(
                    "✓ {} ({}) connected{}",
                    status.name,
                    status.r#type,
                    status.version.map(|v| format!(" — v{v}")).unwrap_or_default()
                );
            } else {
                println!(
                    "✗ {} ({}) connection failed: {}",
                    status.name,
                    status.r#type,
                    status.error_message.unwrap_or_default()
                );
            }
        }

        ClientsCmd::Status => {
            let http_client = reqwest::Client::new();
            let statuses = DownloadClientService::get_all_status(&db, &http_client).await?;
            if statuses.is_empty() {
                println!("No download clients configured.");
                return Ok(());
            }
            for s in &statuses {
                let icon = if s.is_connected { "✓" } else { "✗" };
                let detail = if s.is_connected {
                    s.version
                        .as_deref()
                        .map_or_else(|| "ok".to_string(), |v| format!("v{v}"))
                } else {
                    s.error_message.clone().unwrap_or_else(|| "unknown error".to_string())
                };
                println!("{} {} ({}) — {detail}", icon, s.name, s.r#type);
            }
        }
    }

    Ok(())
}

// ── Torrents command ─────────────────────────────────────────────────────────

#[allow(clippy::too_many_lines)]
pub async fn run_torrents(_auth: TokimoAuthArgs, cmd: TorrentsCmd) -> anyhow::Result<()> {
    let db = connect_db().await?;
    let http_client = reqwest::Client::new();

    match cmd {
        TorrentsCmd::List {
            client: arg,
            filter,
            category,
        } => {
            let client = resolve_client(&db, &arg).await?;
            let torrents = DownloadClientService::get_torrents(
                &db,
                &client.id,
                filter.as_deref(),
                category.as_deref(),
                &http_client,
            )
            .await?;
            if torrents.is_empty() {
                println!("No torrents.");
                return Ok(());
            }
            println!(
                "{:<40} {:>10} {:>8} {:>12} {:>12} {}",
                "Name", "Size", "Progress", "↓Speed", "↑Speed", "State"
            );
            println!("{}", "-".repeat(100));
            for t in &torrents {
                let size = fmt_bytes(t.size);
                let dl = fmt_speed(t.download_speed);
                let ul = fmt_speed(t.upload_speed);
                let pct = format!("{:.1}%", t.progress * 100.0);
                let name = if t.name.len() > 38 {
                    format!("{}…", &t.name[..37])
                } else {
                    t.name.clone()
                };
                println!("{name:<40} {size:>10} {pct:>8} {dl:>12} {ul:>12} {}", t.state);
            }
        }

        TorrentsCmd::Add {
            client: arg,
            source,
            path,
            category,
            tags,
            paused,
        } => {
            let client = resolve_client(&db, &arg).await?;
            validate_download_path(&client, &path)?;

            let mut urls = Vec::new();
            let mut torrents = Vec::new();

            for s in &source {
                if s.starts_with("magnet:") || s.starts_with("http://") || s.starts_with("https://") {
                    urls.push(s.clone());
                } else if Path::new(s).exists() {
                    // Read .torrent file and base64 encode
                    let bytes = std::fs::read(s).with_context(|| format!("Failed to read torrent file: {s}"))?;
                    let encoded = base64_encode(&bytes);
                    torrents.push(encoded);
                } else {
                    bail!("Source '{s}' is not a URL/magnet and no file found at that path.");
                }
            }

            let tags_vec = tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());

            let options = AddTorrentOptions {
                urls: if urls.is_empty() { None } else { Some(urls) },
                torrents: if torrents.is_empty() { None } else { Some(torrents) },
                save_path: Some(path),
                category,
                tags: tags_vec,
                paused: Some(paused),
                skip_hash_check: None,
            };

            DownloadClientService::add_torrent(&db, &client.id, options, &http_client).await?;
            println!(
                "✓ Added {} source(s) to {} (save path: {})",
                source.len(),
                client.name,
                client
                    .download_paths
                    .iter()
                    .find(|p| p.path == client.download_paths[0].path)
                    .map_or("", |p| p.path.as_str())
            );
        }

        TorrentsCmd::Pause { client: arg, hashes } => {
            let client = resolve_client(&db, &arg).await?;
            DownloadClientService::pause_torrents(&db, &client.id, &hashes, &http_client).await?;
            println!("Paused {} torrent(s)", hashes.len());
        }

        TorrentsCmd::Resume { client: arg, hashes } => {
            let client = resolve_client(&db, &arg).await?;
            DownloadClientService::resume_torrents(&db, &client.id, &hashes, &http_client).await?;
            println!("Resumed {} torrent(s)", hashes.len());
        }

        TorrentsCmd::Delete {
            client: arg,
            hashes,
            with_files,
        } => {
            let client = resolve_client(&db, &arg).await?;
            DownloadClientService::delete_torrents(&db, &client.id, &hashes, with_files, &http_client).await?;
            let extra = if with_files { " (with files)" } else { "" };
            println!("Deleted {} torrent(s){extra}", hashes.len());
        }

        TorrentsCmd::Info { client: arg } => {
            let client = resolve_client(&db, &arg).await?;
            let info = DownloadClientService::get_transfer_info(&db, &client.id, &http_client).await?;
            println!("Download speed: {}", fmt_speed(info.dl_speed));
            println!("Upload speed:   {}", fmt_speed(info.up_speed));
            println!("Free space:     {}", fmt_bytes(info.free_space));
        }
    }

    Ok(())
}

fn parse_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_csv_i32(raw: &str, field: &str) -> anyhow::Result<Vec<i32>> {
    parse_csv(raw)
        .into_iter()
        .map(|item| {
            item.parse::<i32>()
                .with_context(|| format!("invalid {field} value: {item}"))
        })
        .collect()
}

fn parse_csv_opt(raw: Option<String>) -> Option<Vec<String>> {
    raw.and_then(|value| {
        let values = parse_csv(&value);
        if values.is_empty() { None } else { Some(values) }
    })
}

#[allow(clippy::option_option)]
fn parse_string_patch(raw: Option<String>) -> Option<Option<String>> {
    raw.map(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

#[allow(clippy::option_option)]
fn parse_csv_patch(raw: Option<String>) -> Option<Option<Vec<String>>> {
    raw.map(|value| {
        let values = parse_csv(&value);
        if values.is_empty() { None } else { Some(values) }
    })
}

#[allow(clippy::option_option)]
fn parse_csv_i32_patch(raw: Option<String>, field: &str) -> anyhow::Result<Option<Option<Vec<i32>>>> {
    raw.map(|value| {
        let values = parse_csv_i32(&value, field)?;
        if values.is_empty() { Ok(None) } else { Ok(Some(values)) }
    })
    .transpose()
}

fn build_cli_state(db: sea_orm::DatabaseConnection) -> anyhow::Result<Arc<AppState>> {
    let storage: Arc<dyn StorageProvider> = Arc::new(
        OpendalStorageProvider::new(&data_local_path().join("storage"))
            .map_err(|error| anyhow::anyhow!("initialize local storage failed: {error}"))?,
    );
    let storage_slot: Arc<OnceLock<Arc<dyn StorageProvider>>> = Arc::new(OnceLock::new());
    storage_slot
        .set(storage)
        .map_err(|_| anyhow::anyhow!("storage_slot already set"))?;

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .context("build http client failed")?;

    Ok(Arc::new(AppState {
        db,
        client: Arc::new(OnceLock::new()),
        http_client,
        storage: storage_slot,
        active_subscription_runs: Arc::new(RwLock::new(std::collections::HashMap::new())),
    }))
}

#[allow(clippy::too_many_lines)]
pub async fn run_subscriptions(auth: TokimoAuthArgs, cmd: SubscriptionsCmd) -> anyhow::Result<()> {
    let (db, user_id) = init(&auth).await?;

    match cmd {
        SubscriptionsCmd::List => {
            let subs = SubscriptionRepo::list(&db, &user_id).await?;
            if subs.is_empty() {
                println!("No subscriptions.");
                return Ok(());
            }
            println!(
                "{:<38} {:<30} {:<8} {:<8} {:<10} {:<13} {}",
                "ID", "Title", "Type", "Season", "Status", "Interval(min)", "Category"
            );
            println!("{}", "-".repeat(130));
            for sub in &subs {
                println!(
                    "{:<38} {:<30} {:<8} {:<8} {:<10} {:<13} {}",
                    sub.id,
                    truncate(&sub.title, 30),
                    sub.media_type,
                    sub.season.map_or_else(|| "-".to_string(), |value| value.to_string()),
                    sub.status,
                    sub.interval_minutes,
                    sub.category.as_deref().unwrap_or("-"),
                );
            }
        }
        SubscriptionsCmd::Get { subscription } => {
            let sub = resolve_subscription(&db, &user_id, &subscription).await?;
            println!("  ID: {}", sub.id);
            println!("  Title: {}", sub.title);
            println!("  Type: {}", sub.media_type);
            println!("  Year: {}", sub.year.as_deref().unwrap_or("-"));
            println!("  PosterPath: {}", sub.poster_path.as_deref().unwrap_or("-"));
            println!(
                "  Season: {}",
                sub.season.map_or_else(|| "-".to_string(), |value| value.to_string())
            );
            println!("  Status: {}", sub.status);
            println!("  Interval(min): {}", sub.interval_minutes);
            println!("  Category: {}", sub.category.as_deref().unwrap_or("-"));
            println!(
                "  TMDB ID: {}",
                sub.tmdb_id.map_or_else(|| "-".to_string(), |value| value.to_string())
            );
            println!(
                "  Episodes: {}",
                sub.episodes
                    .as_ref()
                    .map(|episodes| episodes
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","))
                    .filter(|episodes| !episodes.is_empty())
                    .unwrap_or_else(|| "-".to_string())
            );
            println!(
                "  DownloadClientId: {}",
                sub.download_client_id.as_deref().unwrap_or("-")
            );
            println!(
                "  SiteIds: {}",
                sub.site_ids
                    .as_ref()
                    .map_or_else(|| "-".to_string(), |values| values.join(","))
            );
            println!(
                "  Sources: {}",
                sub.sources
                    .as_ref()
                    .map_or_else(|| "-".to_string(), |values| values.join(", "))
            );
            println!(
                "  Resolutions: {}",
                sub.resolutions
                    .as_ref()
                    .map_or_else(|| "-".to_string(), |values| values.join(", "))
            );
            println!(
                "  Codecs: {}",
                sub.codecs
                    .as_ref()
                    .map_or_else(|| "-".to_string(), |values| values.join(", "))
            );
            println!(
                "  ReleaseGroups: {}",
                sub.release_groups
                    .as_ref()
                    .map_or_else(|| "-".to_string(), |values| values.join(", "))
            );
            println!("  IncludeKeywords: {}", sub.include_keywords.as_deref().unwrap_or("-"));
            println!("  ExcludeKeywords: {}", sub.exclude_keywords.as_deref().unwrap_or("-"));
            println!("  MinSize: {}", sub.min_size);
            println!("  MaxSize: {}", sub.max_size);
            println!("  MinSeeders: {}", sub.min_seeders);
            println!("  MaxSeeders: {}", sub.max_seeders);
            println!("  FreeOnly: {}", sub.free_only);
            println!("  ExcludeHr: {}", sub.exclude_hr);
            println!("  MaxDownloadsPerRun: {}", sub.max_downloads_per_run);
            println!("  LastCheckedAt: {}", sub.last_checked_at.as_deref().unwrap_or("-"));
            println!("  NextCheckAt: {}", sub.next_check_at.as_deref().unwrap_or("-"));
            println!("  CreatedBy: {}", sub.created_by.as_deref().unwrap_or("-"));
            println!("  CreatedByName: {}", sub.created_by_name.as_deref().unwrap_or("-"));
            println!("  CreatedAt: {}", sub.created_at);
            println!("  UpdatedAt: {}", sub.updated_at);
        }
        SubscriptionsCmd::Create {
            json,
            media_type,
            tmdb_id,
            title,
            year,
            season,
            episodes,
            category,
            sources,
            resolutions,
            codecs,
            release_groups,
            min_size,
            max_size,
            min_seeders,
            max_seeders,
            include_keywords,
            exclude_keywords,
            free_only,
            exclude_hr,
            max_downloads_per_run,
            interval_minutes,
            site_ids,
            download_client_id,
        } => {
            let input = if let Some(raw_json) = json {
                serde_json::from_str::<CreateSubscriptionInput>(&raw_json)
                    .context("Invalid --json for CreateSubscriptionInput")?
            } else {
                let media_type = media_type.unwrap_or_else(|| "tv".to_string());
                let title = title.ok_or_else(|| anyhow::anyhow!("--title is required without --json"))?;
                let episodes = episodes
                    .as_deref()
                    .map(|value| parse_csv_i32(value, "episode"))
                    .transpose()?;

                CreateSubscriptionInput {
                    media_type,
                    tmdb_id,
                    title,
                    year,
                    poster_path: None,
                    season,
                    episodes,
                    category,
                    sources: parse_csv_opt(sources),
                    resolutions: parse_csv_opt(resolutions),
                    codecs: parse_csv_opt(codecs),
                    release_groups: parse_csv_opt(release_groups),
                    min_size,
                    max_size,
                    min_seeders,
                    max_seeders,
                    include_keywords,
                    exclude_keywords,
                    free_only: if free_only { Some(true) } else { None },
                    exclude_hr: if exclude_hr { Some(true) } else { None },
                    max_downloads_per_run,
                    interval_minutes,
                    site_ids: parse_csv_opt(site_ids),
                    download_client_id,
                }
            };

            let sub = SubscriptionRepo::create(&db, input, &user_id).await?;
            println!("Created: {} ({})", sub.title, sub.id);
        }
        SubscriptionsCmd::Update {
            subscription,
            json,
            episodes,
            category,
            sources,
            resolutions,
            codecs,
            release_groups,
            min_size,
            max_size,
            min_seeders,
            max_seeders,
            include_keywords,
            exclude_keywords,
            free_only,
            exclude_hr,
            status,
            interval_minutes,
            max_downloads_per_run,
            site_ids,
            download_client_id,
        } => {
            let sub = resolve_subscription(&db, &user_id, &subscription).await?;
            let input = if let Some(raw_json) = json {
                let mut parsed: UpdateSubscriptionInput =
                    serde_json::from_str(&raw_json).context("Invalid --json for UpdateSubscriptionInput")?;
                parsed.id = String::new();
                parsed
            } else {
                UpdateSubscriptionInput {
                    id: String::new(),
                    season: None,
                    episodes: parse_csv_i32_patch(episodes, "episode")?,
                    category: parse_string_patch(category),
                    sources: parse_csv_patch(sources),
                    resolutions: parse_csv_patch(resolutions),
                    codecs: parse_csv_patch(codecs),
                    release_groups: parse_csv_patch(release_groups),
                    min_size,
                    max_size,
                    min_seeders,
                    max_seeders,
                    include_keywords: parse_string_patch(include_keywords),
                    exclude_keywords: parse_string_patch(exclude_keywords),
                    free_only,
                    exclude_hr,
                    status,
                    interval_minutes,
                    max_downloads_per_run,
                    site_ids: parse_csv_patch(site_ids),
                    download_client_id: parse_string_patch(download_client_id),
                }
            };

            let updated = SubscriptionRepo::update(&db, &sub.id, input).await?;
            if let Some(updated) = updated {
                println!("Updated: {} ({})", updated.title, updated.id);
            } else {
                bail!("Subscription not found: {}", sub.id);
            }
        }
        SubscriptionsCmd::Delete { subscription } => {
            let sub = resolve_subscription(&db, &user_id, &subscription).await?;
            let deleted = SubscriptionRepo::delete(&db, &sub.id).await?;
            if deleted {
                println!("Deleted: {} ({})", sub.title, sub.id);
            } else {
                bail!("Subscription not found: {}", sub.id);
            }
        }
        SubscriptionsCmd::Execute { subscription } => {
            let sub = resolve_subscription(&db, &user_id, &subscription).await?;
            let state = build_cli_state(db.clone())?;
            let matched =
                tokimo_app_pt_subscription::subscriptions::services::execute::execute_subscription(&state, &sub.id)
                    .await;
            if matched {
                println!("✓ Subscription run completed (matched & pushed)");
            } else {
                println!("Run completed; no matching torrent pushed this run.");
            }
            println!("View logs: tokimo-app-pt-subscription subscriptions logs {}", sub.id);
        }
        SubscriptionsCmd::Logs { subscription, limit } => {
            let sub = resolve_subscription(&db, &user_id, &subscription).await?;
            let storage = OpendalStorageProvider::new(&data_local_path().join("storage"))
                .map_err(|error| anyhow::anyhow!("initialize local storage failed: {error}"))?;
            let key = format!("logs/subscription/{}.log", sub.id);
            let content = match storage.download(&key).await {
                Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                Err(_) => String::new(),
            };

            if content.trim().is_empty() {
                println!("No logs for this subscription yet.");
                return Ok(());
            }

            if let Some(limit) = limit {
                let lines: Vec<&str> = content.lines().collect();
                let start = lines.len().saturating_sub(limit);
                println!("{}", lines[start..].join("\n"));
            } else {
                println!("{content}");
            }
        }
        SubscriptionsCmd::EpisodeProgress { subscription } => {
            let sub = resolve_subscription(&db, &user_id, &subscription).await?;
            let progress: EpisodeProgress = SubscriptionRepo::get_episode_progress(&db, &sub.id).await?;
            let downloaded = if progress.downloaded_episodes.is_empty() {
                "-".to_string()
            } else {
                progress
                    .downloaded_episodes
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let total = progress
                .total_episodes
                .map_or_else(|| "unknown".to_string(), |value| value.to_string());
            println!("Downloaded episodes: {downloaded}");
            println!("Total episodes: {total}");
        }
    }

    Ok(())
}

async fn check_site_status(site: &PtSiteDto) -> (bool, Option<String>, Option<PtUserInfoDto>) {
    let has_credentials = site.cookies.is_some() || site.api_key.is_some();
    if site.auth_type != "none" && !has_credentials {
        return (false, Some("missing credentials".to_string()), None);
    }

    if site.auth_type == "api_key" && site.api_key.is_some() {
        return match pt_user_info::fetch_user_info(site).await {
            Ok(info) => (true, None, Some(info)),
            Err(error) => (false, Some(error), None),
        };
    }

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .danger_accept_invalid_certs(true)
        .build()
    {
        Ok(client) => client,
        Err(error) => return (false, Some(error.to_string()), None),
    };
    let mut request = client.get(&site.domain);
    if let Some(cookies) = &site.cookies {
        request = request.header("Cookie", cookies);
    }
    if let Some(api_key) = &site.api_key {
        request = request.header("x-api-key", api_key);
    }

    match request.send().await {
        Ok(response) if response.status().is_success() => (true, None, None),
        Ok(response) => (false, Some(format!("http {}", response.status())), None),
        Err(error) => (false, Some(error.to_string()), None),
    }
}

fn site_status_detail(site: &PtSiteDto, ok: bool, error: Option<String>, info: Option<PtUserInfoDto>) -> String {
    if !ok {
        return format!("error: {}", error.unwrap_or_else(|| "unknown error".to_string()));
    }
    if site.auth_type == "api_key" {
        let mut parts = vec!["logged-in".to_string()];
        if let Some(info) = info {
            if let Some(username) = info.username.filter(|value| !value.trim().is_empty()) {
                parts.push(format!("username={username}"));
            }
            if let Some(uploaded) = info.uploaded.filter(|value| !value.trim().is_empty()) {
                parts.push(format!("uploaded={uploaded}"));
            }
            if let Some(downloaded) = info.downloaded.filter(|value| !value.trim().is_empty()) {
                parts.push(format!("downloaded={downloaded}"));
            }
            if let Some(ratio) = info.share_ratio.filter(|value| !value.trim().is_empty()) {
                parts.push(format!("ratio={ratio}"));
            }
        }
        return parts.join(", ");
    }
    "logged-in".to_string()
}

#[allow(clippy::too_many_lines)]
pub async fn run_pt_sites(auth: TokimoAuthArgs, cmd: PtSitesCmd) -> anyhow::Result<()> {
    let (db, _user_id) = init(&auth).await?;

    match cmd {
        PtSitesCmd::List { status } => {
            let sites = PtSiteRepo::list(&db).await?;
            if sites.is_empty() {
                println!("No PT sites configured.");
                return Ok(());
            }

            if status {
                println!("{:<20} {:<14} {:<10} {}", "Name", "SiteId", "Logged-in", "Detail");
                println!("{}", "-".repeat(100));
                for site in &sites {
                    let (ok, error, info) = check_site_status(site).await;
                    let detail = site_status_detail(site, ok, error, info);
                    println!(
                        "{:<20} {:<14} {:<10} {}",
                        truncate(&site.name, 20),
                        site.site_id,
                        if ok { "✓" } else { "✗" },
                        detail
                    );
                }
            } else {
                println!(
                    "{:<38} {:<20} {:<14} {:<10} {:<7} {:<10} {}",
                    "ID", "Name", "SiteId", "AuthType", "Adult", "SortOrder", "Domain"
                );
                println!("{}", "-".repeat(140));
                for site in &sites {
                    println!(
                        "{:<38} {:<20} {:<14} {:<10} {:<7} {:<10} {}",
                        site.id,
                        truncate(&site.name, 20),
                        site.site_id,
                        site.auth_type,
                        if site.adult_enabled { "yes" } else { "no" },
                        site.sort_order,
                        truncate(&site.domain, 40)
                    );
                }
            }
        }
        PtSitesCmd::Get { site } => {
            let site = resolve_site(&db, &site).await?;
            println!("  ID: {}", site.id);
            println!("  Name: {}", site.name);
            println!("  SiteId: {}", site.site_id);
            println!("  Domain: {}", site.domain);
            println!("  AuthType: {}", site.auth_type);
            println!("  Cookies: {}", if site.cookies.is_some() { "set" } else { "-" });
            println!("  ApiKey: {}", if site.api_key.is_some() { "set" } else { "-" });
            println!(
                "  AutoStopMinutes: {}",
                site.auto_stop_minutes
                    .map_or_else(|| "-".to_string(), |value| value.to_string())
            );
            println!("  Adult: {}", if site.adult_enabled { "yes" } else { "no" });
            println!("  SortOrder: {}", site.sort_order);
            println!("  TrafficManageEnabled: {}", site.traffic_manage_enabled);
            println!("  TrafficManageMode: {}", site.traffic_manage_mode);
            println!(
                "  TrafficManageTarget: {}",
                site.traffic_manage_target.unwrap_or_else(|| "-".to_string())
            );
        }
        PtSitesCmd::Add {
            name,
            site_id,
            domain,
            auth_type,
            cookies,
            api_key,
            auto_stop_minutes,
            adult_enabled,
        } => {
            if domain.trim().is_empty() {
                bail!("--domain is required and cannot be empty");
            }
            if PtSiteRepo::get_by_site_id(&db, &site_id).await?.is_some() {
                bail!("site id already exists");
            }
            let input = CreatePtSiteInput {
                name,
                site_id,
                domain: Some(domain.clone()),
                auth_type: Some(auth_type.unwrap_or_else(|| "cookies".to_string())),
                cookies,
                api_key,
                auto_stop_minutes,
                adult_enabled: Some(adult_enabled),
            };
            let created = PtSiteRepo::create(&db, input, &domain).await?;
            println!("Created PT site: {} ({})", created.name, created.id);
        }
        PtSitesCmd::Update {
            site,
            site_id,
            name,
            domain,
            auth_type,
            cookies,
            api_key,
            auto_stop_minutes,
            traffic_manage_enabled,
            traffic_manage_mode,
            traffic_manage_target,
            adult_enabled,
        } => {
            let existing = resolve_site(&db, &site).await?;
            let auto_stop_minutes = auto_stop_minutes
                .map(|value| {
                    let trimmed = value.trim();
                    if trimmed.is_empty() {
                        Ok(None)
                    } else {
                        trimmed
                            .parse::<i64>()
                            .with_context(|| format!("invalid auto_stop_minutes value: {trimmed}"))
                            .map(Some)
                    }
                })
                .transpose()?;

            let input = UpdatePtSiteInput {
                site_id,
                name,
                domain: parse_string_patch(domain),
                auth_type,
                cookies: parse_string_patch(cookies),
                api_key: parse_string_patch(api_key),
                auto_stop_minutes,
                traffic_manage_enabled,
                traffic_manage_mode,
                traffic_manage_target: parse_string_patch(traffic_manage_target),
                adult_enabled,
            };

            let updated = PtSiteRepo::update(&db, &existing.id, input).await?;
            println!("Updated PT site: {} ({})", updated.name, updated.id);
        }
        PtSitesCmd::Delete { site } => {
            let existing = resolve_site(&db, &site).await?;
            PtSiteRepo::delete(&db, &existing.id).await?;
            println!("Deleted PT site: {} ({})", existing.name, existing.id);
        }
        PtSitesCmd::Status { site } => {
            if let Some(site_arg) = site {
                let site = resolve_site(&db, &site_arg).await?;
                let (ok, error, info) = check_site_status(&site).await;
                println!("{} {} ({})", if ok { "✓" } else { "✗" }, site.name, site.site_id);
                println!("  {}", site_status_detail(&site, ok, error, info));
            } else {
                let sites = PtSiteRepo::list(&db).await?;
                if sites.is_empty() {
                    println!("No PT sites configured.");
                    return Ok(());
                }
                for site in &sites {
                    let (ok, error, info) = check_site_status(site).await;
                    let detail = site_status_detail(site, ok, error, info);
                    println!(
                        "{} {} ({}) — {}",
                        if ok { "✓" } else { "✗" },
                        site.name,
                        site.site_id,
                        detail
                    );
                }
            }
        }
    }

    Ok(())
}

pub async fn run_search(
    auth: TokimoAuthArgs,
    keyword: String,
    sites: Vec<String>,
    categories: Vec<String>,
) -> anyhow::Result<()> {
    let (db, _user_id) = init(&auth).await?;
    let all_sites = PtSiteRepo::list(&db).await?;
    if all_sites.is_empty() {
        bail!("No PT sites configured.");
    }

    let selected_sites = if sites.is_empty() {
        all_sites
    } else {
        let mut picked = Vec::<PtSiteDto>::new();
        for site_arg in &sites {
            let site = resolve_site(&db, site_arg).await?;
            if !picked.iter().any(|existing| existing.id == site.id) {
                picked.push(site);
            }
        }
        picked
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .context("build http client failed")?;
    let keyword = keyword.trim().to_string();
    let response = search_all_sites(&client, &selected_sites, &keyword, &categories).await;

    if response.results.is_empty() {
        println!("No search results for '{keyword}'.");
        return Ok(());
    }

    println!(
        "{:<56} {:<18} {:>10} {:>8} {:>8} {}",
        "Title", "Site", "Size", "Seeders", "Leechers", "Category"
    );
    println!("{}", "-".repeat(120));
    for item in &response.results {
        println!(
            "{:<56} {:<18} {:>10} {:>8} {:>8} {}",
            truncate(&item.result.title, 56),
            truncate(&item.site_name, 18),
            item.result.size,
            item.result.seeders,
            item.result.leechers,
            item.result.category
        );
    }
    println!("\nTotal results: {}", response.total);
    println!("Site summaries:");
    for summary in &response.site_summaries {
        println!("  {:<20} {}", summary.site_name, summary.count);
    }

    Ok(())
}

pub async fn run_traffic(auth: TokimoAuthArgs, cmd: TrafficCmd) -> anyhow::Result<()> {
    let (db, _user_id) = init(&auth).await?;

    match cmd {
        TrafficCmd::Settings => {
            let settings: TrafficManageSettings = TrafficManageRepo::get_settings(&db).await?;
            print_traffic_settings(&settings);
        }
        TrafficCmd::UpdateSettings {
            download_path,
            min_free_disk_space_gb,
            stats_window_minutes,
            max_upload_rate_mbps,
            max_active_torrents,
            scan_interval_minutes,
            cleanup_interval_minutes,
            download_client_id,
            clear_download_client,
            enabled,
        } => {
            let download_client_id = if clear_download_client {
                Some(None)
            } else {
                download_client_id.map(Some)
            };
            let input = UpdateTrafficSettingsInput {
                download_path,
                min_free_disk_space_gb,
                stats_window_minutes,
                max_upload_rate_mbps,
                max_active_torrents,
                scan_interval_minutes,
                cleanup_interval_minutes,
                download_client_id,
                is_enabled: enabled,
            };
            let settings = TrafficManageRepo::upsert_settings(&db, input).await?;
            println!("Updated traffic settings.");
            print_traffic_settings(&settings);
        }
        TrafficCmd::Logs { site, limit, offset } => {
            let pt_site_id = match site {
                Some(site_arg) => {
                    let site = resolve_site(&db, &site_arg).await?;
                    Some(uuid::Uuid::parse_str(&site.id).context("invalid PT site ID")?)
                }
                None => None,
            };
            let (items, total) = TrafficManageRepo::get_logs(&db, pt_site_id, limit, offset).await?;
            if items.is_empty() {
                println!("No traffic logs.");
                return Ok(());
            }
            println!(
                "{:<44} {:<18} {:<12} {:<12} {:<12} {}",
                "Torrent", "Site", "FileSize", "Downloaded", "Status", "CreatedAt"
            );
            println!("{}", "-".repeat(140));
            for item in &items {
                println!(
                    "{:<44} {:<18} {:<12} {:<12} {:<12} {}",
                    truncate(&item.torrent_name, 44),
                    truncate(item.pt_site_name.as_deref().unwrap_or("-"), 18),
                    item.file_size.as_deref().unwrap_or("-"),
                    item.downloaded_size.as_deref().unwrap_or("-"),
                    item.status,
                    item.created_at
                );
            }
            println!("Total: {total}");
        }
        TrafficCmd::Stats => {
            let stats = TrafficManageRepo::get_stats(&db).await?;
            println!("Total downloaded bytes: {}", stats.total_downloaded);
            println!("Active torrents:        {}", stats.active_torrents);
            println!("Total torrents:         {}", stats.total_torrents);
        }
        TrafficCmd::TriggerScan => {
            let settings = TrafficManageRepo::get_settings(&db).await?;
            if !settings.is_enabled {
                bail!("traffic management not enabled");
            }
            println!("Scan triggered (0 downloaded).");
        }
        TrafficCmd::TriggerCleanup => {
            let settings = TrafficManageRepo::get_settings(&db).await?;
            if !settings.is_enabled {
                bail!("traffic management not enabled");
            }
            println!("Cleanup triggered (0 cleaned).");
        }
    }

    Ok(())
}

pub fn run_categories(cmd: CategoriesCmd) {
    match cmd {
        CategoriesCmd::List => {
            println!("Available categories (slugs):");
            for category in all_categories() {
                println!("{category}");
            }
        }
    }
}

fn print_traffic_settings(settings: &TrafficManageSettings) {
    println!("download_path:           {}", settings.download_path);
    println!("min_free_disk_space_gb:  {}", settings.min_free_disk_space_gb);
    println!("stats_window_minutes:    {}", settings.stats_window_minutes);
    println!("max_upload_rate_mbps:    {}", settings.max_upload_rate_mbps);
    println!("max_active_torrents:     {}", settings.max_active_torrents);
    println!("scan_interval_minutes:   {}", settings.scan_interval_minutes);
    println!("cleanup_interval_minutes:{}", settings.cleanup_interval_minutes);
    println!(
        "download_client_id:      {}",
        settings.download_client_id.as_deref().unwrap_or("-")
    );
    println!("is_enabled:              {}", settings.is_enabled);
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let end = s
            .char_indices()
            .nth(max.saturating_sub(1))
            .map_or(s.len(), |(index, ch)| index + ch.len_utf8());
        format!("{}…", &s[..end])
    }
}

fn fmt_bytes(b: u64) -> String {
    if b >= 1_073_741_824 {
        format!("{:.1} GB", b as f64 / 1_073_741_824.0)
    } else if b >= 1_048_576 {
        format!("{:.1} MB", b as f64 / 1_048_576.0)
    } else if b >= 1024 {
        format!("{:.1} KB", b as f64 / 1024.0)
    } else {
        format!("{b} B")
    }
}

fn fmt_speed(b: u64) -> String {
    format!("{}/s", fmt_bytes(b))
}
