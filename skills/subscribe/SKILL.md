---
name: subscribe
description: "Create a PT subscription that auto-downloads matching torrents on a schedule (e.g. the 4K version of a movie as soon as it appears). Use to set up recurring auto-grabbing rather than a one-shot download."
when-to-use: "When the user wants to automatically download matching torrents over time (a standing subscription), e.g. 'subscribe to the 4K release of <title>'."
argument-hint: "--title <name> --media-type movie --resolutions 2160p"
version: "0.1.0"
context: inline
---

# Subscribe to a PT Title (auto-download on a schedule)

Create a subscription that periodically searches your PT sites and
auto-downloads torrents matching its filters. The binary name is
`tokimo-app-pt-subscription`.

## Prerequisites

- At least one **PT site configured** with working auth
  (`tokimo-app-pt-subscription pt-sites list --status`).
- A **download client** to receive matches. Pass its id via
  `--download-client-id`, or rely on the configured default client.
- Know the **site DB ids** you want to search (`pt-sites list`) if you want to
  restrict to specific sites.

## Quick Reference

| Step | Command |
|------|---------|
| List sites (get ids) | `tokimo-app-pt-subscription pt-sites list` |
| List clients (get ids) | `tokimo-app-pt-subscription clients list` |
| Create subscription | `tokimo-app-pt-subscription subscriptions create --title "<name>" --media-type movie --category movie --resolutions 2160p` |
| Run it immediately | `tokimo-app-pt-subscription subscriptions execute <id\|title>` |
| Check what it did | `tokimo-app-pt-subscription subscriptions logs <id\|title>` |
| List subscriptions | `tokimo-app-pt-subscription subscriptions list` |

## Create flags (from `subscriptions create`)

Identity / scope:

- `--title <name>` — subscription title (also used as the search keyword).
- `--media-type <movie|tv>` — required kind.
- `--category <slug>` — canonical category (e.g. `movie`, `tv`); drives save path.
- `--tmdb-id <id>`, `--year <YYYY>`, `--season <N>`, `--episodes <1,2,3>` — optional metadata / TV scoping.

Match filters (comma-separated where plural):

- `--resolutions 2160p` — resolution tags (use `2160p` for 4K). Repeat values comma-separated, e.g. `2160p,1080p`.
- `--sources <a,b>`, `--codecs <a,b>`, `--release-groups <a,b>`.
- `--include-keywords <...>`, `--exclude-keywords <...>`.
- `--min-size <GB>`, `--max-size <GB>`, `--min-seeders <n>`, `--max-seeders <n>`.
- `--free-only` — only match free torrents. `--exclude-hr` — skip HR torrents.

Scheduling / routing:

- `--interval-minutes <N>` — how often the subscription runs.
- `--max-downloads-per-run <N>` — cap downloads per execution.
- `--site-ids <id1,id2>` — restrict to specific PT site DB ids (omit = all sites).
- `--download-client-id <id>` — target download client (omit = default client).

> You can also pass the full raw body via `--json '<CreateSubscriptionInput JSON>'`.

## Workflow

1. **(Optional) Get site and client ids.**

   ```bash
   tokimo-app-pt-subscription pt-sites list
   tokimo-app-pt-subscription clients list
   ```

2. **Create the subscription** with the desired filters.

3. **(Optional) Execute immediately** instead of waiting for the interval, then
   inspect logs.

   ```bash
   tokimo-app-pt-subscription subscriptions execute "<title>"
   tokimo-app-pt-subscription subscriptions logs "<title>"
   ```

## Worked Example — auto-download the 4K version of 速度与激情6

```bash
# 1. (optional) find site + client ids
tokimo-app-pt-subscription pt-sites list
tokimo-app-pt-subscription clients list

# 2. Create a 4K movie subscription that runs every 60 minutes, free torrents only
tokimo-app-pt-subscription subscriptions create \
  --title "速度与激情6" \
  --media-type movie \
  --category movie \
  --resolutions 2160p \
  --free-only \
  --min-size 20 \
  --interval-minutes 60 \
  --site-ids <site_id> \
  --download-client-id <client_id>

# 3. Run it right away and check what matched
tokimo-app-pt-subscription subscriptions execute "速度与激情6"
tokimo-app-pt-subscription subscriptions logs "速度与激情6"
```

## Notes

- `--site-ids` and `--download-client-id` take the **DB ids** shown by
  `pt-sites list` / `clients list` (not display names).
- Omit `--download-client-id` to route matches to the default download client.
- For a single immediate grab of one specific torrent (not a standing rule),
  use the **download** skill instead.
