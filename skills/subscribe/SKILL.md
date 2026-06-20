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
| Create subscription (auto-runs once) | `tokimo-app-pt-subscription subscriptions create --title "<name>" --media-type movie --category movie --resolutions 2160p` |
| Trigger an extra run later | `tokimo-app-pt-subscription subscriptions execute <id\|title>` |
| Check what it did | `tokimo-app-pt-subscription subscriptions logs <id\|title>` |
| List subscriptions | `tokimo-app-pt-subscription subscriptions list` |

> **Creating a subscription immediately runs one matching pass automatically** —
> there is no separate manual execute step. Pass `--no-run` to skip that initial
> run (it will then only run on its schedule), or run
> `subscriptions execute <id|title>` any time to trigger additional runs later.

## Create flags (from `subscriptions create`)

**Required:**

- `--title <name>` — subscription title. This is the **search keyword** matched
  against your PT sites, so it must be present (the only required flag).

**Identity / scope (optional):**

- `--media-type <movie|tv>` — kind; defaults to `tv` if omitted.
- `--category <slug>` — canonical category (e.g. `movie`, `tv`); drives save path.
- `--tmdb-id <id>`, `--year <YYYY>` — **optional display metadata only**
  (poster / year). Matching is by `--title`, **not** by tmdb-id, so a
  subscription with only `--title` works.
- `--season <N>`, `--episodes <1,2,3>` — optional TV scoping.

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
- `--no-run` — skip the automatic initial matching run after creation.

> You can also pass the full raw body via `--json '<CreateSubscriptionInput JSON>'`.

## Workflow

1. **(Optional) Get site and client ids.**

   ```bash
   tokimo-app-pt-subscription pt-sites list
   tokimo-app-pt-subscription clients list
   ```

2. **Create the subscription** with the desired filters. It **runs one matching
   pass immediately** (unless you pass `--no-run`), then continues on its
   `--interval-minutes` schedule.

3. **Inspect the initial run**, and optionally trigger more runs later.

   ```bash
   tokimo-app-pt-subscription subscriptions logs "<title>"
   tokimo-app-pt-subscription subscriptions execute "<title>"   # extra run on demand
   ```

## Worked Example — auto-download the 4K version of 速度与激情6

```bash
# 1. (optional) find site + client ids
tokimo-app-pt-subscription pt-sites list
tokimo-app-pt-subscription clients list

# 2. Create a 4K movie subscription that runs every 60 minutes, free torrents
#    only. This auto-runs one matching pass right after creation.
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

# 3. Check what the automatic initial run matched (no manual execute needed)
tokimo-app-pt-subscription subscriptions logs "速度与激情6"

# 4. (optional) trigger an extra run on demand later
tokimo-app-pt-subscription subscriptions execute "速度与激情6"
```

## Notes

- `--site-ids` and `--download-client-id` take the **DB ids** shown by
  `pt-sites list` / `clients list` (not display names).
- Omit `--download-client-id` to route matches to the default download client.
- For a single immediate grab of one specific torrent (not a standing rule),
  use the **download** skill instead.
