//! ts-rs type export — run with `cargo test -p tokimo-app-pt-subscription -- export_bindings`
//! Generates TypeScript types to `ui/src/generated/rust-types/`.
#![allow(unused_imports)]

// Trigger ts-rs export by referencing all DTO types.
use tokimo_app_pt_subscription::db::repos::download_client_repo::DownloadClientDto;
