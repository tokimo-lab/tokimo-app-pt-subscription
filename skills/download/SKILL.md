---
name: download
description: "Search PT (private-tracker) torrents and download a chosen result (e.g. the 4K version) to a download client in one shot. Use to find a torrent by keyword, read its TorrentID + Site, then grab it."
when-to-use: "When the user wants to find and download a specific movie/TV torrent from their PT sites right now (one-shot), e.g. 'download the 4K version of <title>'."
argument-hint: "<keyword> [--resolution 2160p] [--category movie]"
version: "0.1.0"
context: inline
---

# Download a PT Torrent (one-shot)

Search PT sites for a torrent, read its `TorrentID` and `Site`, then download
it to a download client. The binary name is `tokimo-app-pt-subscription`.

## Prerequisites

- At least one **PT site configured** with working auth (cookie or api_key).
  Check with `tokimo-app-pt-subscription pt-sites list --status`.
- At least one **download client configured**
  (`tokimo-app-pt-subscription clients list`). If one client is marked default,
  `--client` is optional and the default is used automatically.

## Quick Reference

| Step | Command |
|------|---------|
| (Optional) Confirm sites work | `tokimo-app-pt-subscription pt-sites list --status` |
| (Optional) Confirm clients | `tokimo-app-pt-subscription clients list` |
| Search (4K only) | `tokimo-app-pt-subscription search "<keyword>" --resolution 2160p` |
| Search free 4K only | `tokimo-app-pt-subscription search "<keyword>" --resolution 2160p --free` |
| Download a result | `tokimo-app-pt-subscription download --site <Site> --torrent-id <TorrentID> --category movie` |

The `search` table columns are:
`TorrentID, Site, Resolution, Size, Seeders, Free, Category, Title`.

- `TorrentID` — pass to `download --torrent-id`.
- `Site` — pass to `download --site` (it is the exact value the site resolver accepts).

## Workflow

1. **Search, filtering to 4K.** `--resolution 2160p` keeps only 2160p/4K
   results (`4k` is an alias for `2160p`; it matches either the resolution
   field or the title). Add `--free` to keep only free/discounted torrents,
   `--site <site>` and `--category <slug>` to narrow further.

   ```bash
   tokimo-app-pt-subscription search "<keyword>" --resolution 2160p
   ```

   Read the `TorrentID` and `Site` of the row you want.

2. **Download it.** Use the `Site` and `TorrentID` from step 1.

   ```bash
   tokimo-app-pt-subscription download --site <Site> --torrent-id <TorrentID> --category movie
   ```

   - `--client <name|id>` — optional; defaults to the **default** download client.
   - `--category <slug>` — canonical slug (e.g. `movie`, `tv`); resolves the
     save path from the client's configured paths (falls back to `global/<slug>`).
   - `--save-path <path>` — explicit override (skips category-based resolution).
   - `--season <N> --episodes <1,2,3>` — for TV: only download matching episode files.
   - `--tags a,b` and `--paused` — optional.

   On success it prints the torrent name, the client, and the resolved save path.

## Worked Example — download the 4K version of 速度与激情6

```bash
# 1. Search for the movie, restricting to 4K (2160p) releases
tokimo-app-pt-subscription search "速度与激情6" --resolution 2160p
#   TorrentID  Site     Resolution  Size     Seeders  Free   Category  Title
#   123456     m-team   2160p       55.4 GB  42       FREE   movie     Fast.&.Furious.6.2013.2160p.UHD.BluRay...
#   ...

# 2. Download the chosen result (TorrentID 123456 from site "m-team") into the movie path
tokimo-app-pt-subscription download --site m-team --torrent-id 123456 --category movie
#   ✓ Added 'Fast.&.Furious.6.2013.2160p...' to qBittorrent (site: m-team, save path: /data/movies)
```

If multiple clients exist and none is default, add `--client <name|id>`:

```bash
tokimo-app-pt-subscription download --site m-team --torrent-id 123456 --category movie --client qBittorrent
```

## Notes

- `--site` accepts the site DB id, the `site_id` (e.g. `m-team`), or the site
  display name — the same value shown in the search `Site` column.
- If `download` reports no default client, run `clients list` and pass an
  explicit `--client`.
- For a recurring "auto-download new matches on a schedule" flow instead of a
  one-shot grab, use the **subscribe** skill (`subscriptions create`).
