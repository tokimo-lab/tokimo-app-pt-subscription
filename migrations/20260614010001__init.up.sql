-- pt_subscription schema: PT 追剧助手
-- Creates all tables for the standalone pt-subscription app

CREATE SCHEMA IF NOT EXISTS pt_subscription;

-- Drop old wrong schema if it exists (from previous buggy migration)
DROP SCHEMA IF EXISTS download_clients CASCADE;

-- Download clients
CREATE TABLE IF NOT EXISTS pt_subscription.download_clients (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    url TEXT NOT NULL,
    username TEXT,
    password TEXT,
    is_default BOOLEAN NOT NULL DEFAULT false,
    require_auth BOOLEAN NOT NULL DEFAULT true,
    monitor_enabled BOOLEAN NOT NULL DEFAULT false,
    sort_order INTEGER NOT NULL DEFAULT 0,
    poll_interval TEXT NOT NULL DEFAULT '5',
    download_paths JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);

-- PT sites
CREATE TABLE IF NOT EXISTS pt_subscription.pt_sites (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    site_id TEXT NOT NULL,
    domain TEXT NOT NULL,
    auth_type TEXT NOT NULL DEFAULT 'cookie',
    cookies TEXT,
    api_key TEXT,
    config_yaml TEXT,
    config_url TEXT,
    auto_stop_minutes TEXT,
    traffic_manage_enabled BOOLEAN NOT NULL DEFAULT false,
    traffic_manage_mode TEXT NOT NULL DEFAULT 'off',
    traffic_manage_target TEXT,
    adult_enabled BOOLEAN NOT NULL DEFAULT false,
    sort_order INTEGER NOT NULL DEFAULT 0,
    last_checked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);

-- Subscriptions
CREATE TABLE IF NOT EXISTS pt_subscription.subscriptions (
    id UUID PRIMARY KEY,
    media_type TEXT NOT NULL,
    tmdb_id TEXT,
    title TEXT NOT NULL,
    year TEXT,
    poster_path TEXT,
    season TEXT,
    episodes JSONB,
    category TEXT,
    sources JSONB,
    resolutions JSONB,
    codecs JSONB,
    release_groups JSONB,
    min_size TEXT NOT NULL DEFAULT '0',
    max_size TEXT NOT NULL DEFAULT '0',
    min_seeders TEXT NOT NULL DEFAULT '0',
    max_seeders TEXT NOT NULL DEFAULT '0',
    include_keywords TEXT,
    exclude_keywords TEXT,
    free_only BOOLEAN NOT NULL DEFAULT false,
    exclude_hr BOOLEAN NOT NULL DEFAULT false,
    max_downloads_per_run INTEGER NOT NULL DEFAULT 3,
    status TEXT NOT NULL DEFAULT 'active',
    interval_minutes TEXT NOT NULL DEFAULT '60',
    site_ids JSONB,
    download_client_id UUID,
    last_checked_at TIMESTAMPTZ,
    next_check_at TIMESTAMPTZ,
    created_by UUID,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);

-- Download records
CREATE TABLE IF NOT EXISTS pt_subscription.download_records (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'downloading',
    downloader_type TEXT NOT NULL,
    file_size TEXT,
    downloaded_bytes BIGINT,
    app_metadata JSONB,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);

-- Seed data: PT site (馒头)
INSERT INTO pt_subscription.pt_sites (id, name, site_id, domain, auth_type, cookies, api_key, config_yaml, config_url, auto_stop_minutes, traffic_manage_enabled, traffic_manage_mode, traffic_manage_target, adult_enabled, sort_order, last_checked_at, created_at, updated_at)
VALUES (
    'daf771bf-04a1-4de4-9b9f-14a85a36bd12',
    '馒头',
    'm-team',
    'https://api.m-team.cc/',
    'api_key',
    NULL,
    '64448da0-a2db-4450-9cf7-3e34d009dc83',
    NULL,
    NULL,
    NULL,
    false,
    'active',
    NULL,
    false,
    0,
    '2026-06-14 08:31:21.128445+00',
    '2026-06-13 20:13:41.569199+00',
    '2026-06-14 08:31:21.128445+00'
) ON CONFLICT (id) DO NOTHING;

-- Seed data: Download client (qBittorrent)
INSERT INTO pt_subscription.download_clients (id, name, type, url, username, password, is_default, require_auth, monitor_enabled, sort_order, poll_interval, download_paths, created_at, updated_at)
VALUES (
    'b41286d8-1ad5-4189-936d-d0add78bcb3d',
    'qb',
    'qbittorrent',
    'https://qb.williamchan.me:10443/',
    'root',
    '@#woshizhu22',
    false,
    true,
    false,
    0,
    '5',
    '[{"path": "/mnt/pt", "type": "global", "description": ""}]',
    '2026-06-14 08:03:46.525525+00',
    '2026-06-14 08:30:18.760327+00'
) ON CONFLICT (id) DO NOTHING;
