# tokimo-app-pt-subscription

PT site subscription manager for Tokimo. Automatically search and download torrents from private trackers based on configurable subscriptions.

## Features

- **Subscription Management** — Create, edit, pause/resume subscriptions with filters (resolution, codec, source, category)
- **Multi-site Search** — Search across configured PT sites with rate limiting
- **Smart Scoring** — Rank torrents by seeders, resolution, freeleech status
- **Download Client Integration** — Push torrents to qBittorrent/Transmission/etc.
- **Traffic Management** — Monitor site traffic quotas and user stats
- **Execution Logs** — View detailed per-subscription execution history

## Architecture

```
Browser → tokimo-server :5678 (auth/CORS)
           │ transparent proxy → UDS
           ▼
     tokimo-app-pt-subscription (axum)
       ├─ REST API (/subscriptions, /pt-sites, /download-clients, /search)
       ├─ Scheduler (polling active subscriptions)
       ├─ PT site scrapers (HTML + API)
       └─ PostgreSQL (schema=pt_subscription)
```

## Development

```bash
# Build Rust backend
cargo build -p tokimo-app-pt-subscription

# Build UI
cd ui && pnpm build

# Run in Tokimo
bun dev --apps=pt-subscription
```

## License

MIT OR Apache-2.0.
