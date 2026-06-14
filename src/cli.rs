//! CLI entrypoints — direct database access, no server needed.

#![allow(clippy::print_stdout, clippy::print_literal)]

use std::path::Path;

use anyhow::{Context, bail};
use sea_orm::{ConnectOptions, Database};
use tokimo_bus_cli::TokimoAuthArgs;
use tokimo_package_client_api::downloaders::traits::AddTorrentOptions;

use crate::cli_types::{ClientsCmd, TorrentsCmd};
use tokimo_app_pt_subscription::db::repos::download_client_repo::{
    CreateDownloadClientInput, DownloadClientDto, DownloadClientRepo, DownloadPath, UpdateDownloadClientInput,
};
use tokimo_app_pt_subscription::services::DownloadClientService;

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

// ── Helpers ──────────────────────────────────────────────────────────────────

fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes)
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
