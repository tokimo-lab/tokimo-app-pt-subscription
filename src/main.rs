//! PT 追剧助手 app — 双模 binary（CLI + server）。
//!
//! 启动流程：
//! 1. 连接 broker（supervisor 健康检查 + cross-app 调用通道）
//! 2. 起 axum router 监听 `<runtime_dir>/apps/pt-subscription.sock`
//! 3. 把 sock 上报 broker（server 用它做反代目的地）

const MANIFEST: &str = include_str!("../tokimo-app.toml");

mod app_server;
mod assets;
mod cli;
mod cli_types;

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use clap::Parser;
use cli_types::Command;
use tokimo_app_pt_subscription::AppState;
use tokimo_bus_cli::TokimoAuthArgs;
use tokimo_bus_client::{BusClient, ClientConfig};
use tracing::{error, info};

fn data_local_path() -> PathBuf {
    std::env::var("DATA_LOCAL_PATH").map_or_else(|_| PathBuf::from("./.data/local"), PathBuf::from)
}

// ── CLI ───────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "tokimo-app-pt-subscription",
    about = "PT 追剧助手 — 订阅 + 下载器 + PT 站点 + 剧集搜索",
    long_about = "PT 追剧助手 CLI — 管理订阅、下载客户端、PT 站点和剧集搜索。\n\n支持：qBittorrent, Transmission, Aria2, Deluge, rTorrent, Xunlei, Pan115 等下载工具。",
    term_width = 100
)]
struct Cli {
    /// Tokimo authentication options.
    #[command(flatten)]
    auth: TokimoAuthArgs,

    /// Top-level command.
    #[command(subcommand)]
    command: Option<Command>,
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
#[allow(clippy::print_stderr)]
async fn main() -> anyhow::Result<()> {
    let Cli { auth, command } = Cli::parse();

    match command {
        None if std::env::var_os("TOKIMO_BUS_SOCKET").is_some() => {
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "info,tokimo_bus_client=info,tokimo_app_pt_subscription=debug".into()),
                )
                .init();
            if let Err(error) = run_server().await {
                error!(%error, "pt-subscription: fatal");
                std::process::exit(1);
            }
        }
        None => {
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            tokimo_bus_cli::print_help_unified(&mut cmd);
            std::process::exit(0);
        }
        Some(cmd) => {
            let result = match cmd {
                Command::Clients(clients_cmd) => cli::run_clients(auth, clients_cmd).await,
                Command::Torrents(torrents_cmd) => cli::run_torrents(auth, torrents_cmd).await,
                Command::Subscriptions(sub_cmd) => cli::run_subscriptions(auth, sub_cmd).await,
                Command::PtSites(pt_sites_cmd) => cli::run_pt_sites(auth, pt_sites_cmd).await,
                Command::Search {
                    keyword,
                    sites,
                    categories,
                } => cli::run_search(auth, keyword, sites, categories).await,
                Command::Traffic(traffic_cmd) => cli::run_traffic(auth, traffic_cmd).await,
                Command::Categories(categories_cmd) => {
                    cli::run_categories(categories_cmd);
                    Ok(())
                }
            };
            if let Err(error) = result {
                eprintln!("Error: {error:#}");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

async fn run_server() -> anyhow::Result<()> {
    let cfg = ClientConfig::from_env().map_err(|e| anyhow::anyhow!("ClientConfig: {e}"))?;
    info!(endpoint = ?cfg.endpoint, "pt-subscription: connecting to broker");

    let db = tokimo_app_pt_subscription::db::init_pool().await?;
    info!("pt-subscription: db connected (schema managed by host)");

    let client_slot: Arc<OnceLock<Arc<BusClient>>> = Arc::new(OnceLock::new());
    let storage_slot: Arc<OnceLock<Arc<dyn tokimo_package_storage::StorageProvider>>> = Arc::new(OnceLock::new());
    storage_slot
        .set(Arc::new(
            tokimo_package_storage::OpendalStorageProvider::new(&data_local_path().join("storage"))
                .expect("storage init"),
        ))
        .map_err(|_| anyhow::anyhow!("storage_slot already set"))?;
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let ctx = Arc::new(AppState {
        db,
        client: Arc::clone(&client_slot),
        http_client,
        storage: storage_slot,
        active_subscription_runs: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
    });

    let app_socket =
        app_server::spawn("pt-subscription", Arc::clone(&ctx)).map_err(|e| anyhow::anyhow!("app_server spawn: {e}"))?;

    let client = BusClient::builder(cfg)
        .service("pt-subscription", env!("CARGO_PKG_VERSION"))
        .data_plane(app_socket)
        .build()
        .await
        .map_err(|e| anyhow::anyhow!("bus build: {e}"))?;
    client_slot
        .set(Arc::clone(&client))
        .map_err(|_| anyhow::anyhow!("client_slot already set"))?;

    info!("pt-subscription: registered with broker");

    let shutdown = {
        let client = Arc::clone(&client);
        tokio::spawn(async move { client.run_until_shutdown().await })
    };

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("pt-subscription: SIGINT received");
            client.shutdown();
        }
        _ = shutdown => info!("pt-subscription: broker sent Shutdown"),
    }

    Ok(())
}
