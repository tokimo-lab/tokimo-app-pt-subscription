//! ts-rs type export — run with `cargo test -p tokimo-app-download-clients -- export_bindings`
//! Generates TypeScript types to `ui/src/generated/rust-types/`.
#![allow(unused_imports)]

// Trigger ts-rs export by referencing all DTO types.
use tokimo_app_download_clients::db::repos::download_client_repo::DownloadClientDto;
