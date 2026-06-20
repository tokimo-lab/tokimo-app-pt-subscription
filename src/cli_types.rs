//! CLI command types — shared between main.rs and cli.rs.

use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum ClientsCmd {
    /// List all download clients
    List,

    /// Add a new download client
    Add {
        /// Display name for the client
        #[arg(long)]
        name: String,

        /// Client type (qbittorrent, transmission, aria2, deluge, rtorrent, xunlei, pan115)
        #[arg(long)]
        r#type: String,

        /// WebUI or API URL
        #[arg(long)]
        url: String,

        /// Username (optional)
        #[arg(long)]
        username: Option<String>,

        /// Password (optional)
        #[arg(long)]
        password: Option<String>,

        /// Download paths as JSON array: [{"path":"/data/video","description":"Videos"}]
        #[arg(long)]
        download_paths: String,

        /// Set as default client
        #[arg(long, default_value = "false")]
        r#default: bool,
    },

    /// Update an existing download client
    Update {
        /// Client name or ID
        client: String,

        /// Display name
        #[arg(long)]
        name: Option<String>,

        /// WebUI or API URL
        #[arg(long)]
        url: Option<String>,

        /// Username
        #[arg(long)]
        username: Option<String>,

        /// Password
        #[arg(long)]
        password: Option<String>,

        /// Download paths as JSON array
        #[arg(long)]
        download_paths: Option<String>,
    },

    /// Delete a download client
    Delete {
        /// Client name or ID
        client: String,
    },

    /// Test connection to a download client
    Test {
        /// Client name or ID
        client: String,
    },

    /// Show connection status for all clients
    Status,
}

#[derive(Subcommand, Debug)]
pub enum TorrentsCmd {
    /// List torrents for a client
    List {
        /// Client name or ID
        client: String,

        /// Filter by state (e.g. downloading, seeding, paused)
        #[arg(long)]
        filter: Option<String>,

        /// Filter by category
        #[arg(long)]
        category: Option<String>,
    },

    /// Add a download (URL, magnet link, or .torrent file)
    Add {
        /// Client name or ID
        client: String,

        /// URL, magnet link, or path to .torrent file (can be repeated)
        #[arg(required = true, num_args = 1..)]
        source: Vec<String>,

        /// Download path (must be one of the client's configured paths)
        #[arg(long)]
        path: String,

        /// Category
        #[arg(long)]
        category: Option<String>,

        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Add in paused state
        #[arg(long, default_value = "false")]
        paused: bool,
    },

    /// Pause torrents by hash
    Pause {
        /// Client name or ID
        client: String,

        /// Torrent hashes
        #[arg(required = true)]
        hashes: Vec<String>,
    },

    /// Resume torrents by hash
    Resume {
        /// Client name or ID
        client: String,

        /// Torrent hashes
        #[arg(required = true)]
        hashes: Vec<String>,
    },

    /// Delete torrents by hash
    Delete {
        /// Client name or ID
        client: String,

        /// Torrent hashes
        #[arg(required = true)]
        hashes: Vec<String>,

        /// Also delete downloaded files
        #[arg(long, default_value = "false")]
        with_files: bool,
    },

    /// Show transfer info for a client
    Info {
        /// Client name or ID
        client: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum SubscriptionsCmd {
    /// List subscriptions for the current user
    List,

    /// Get one subscription
    Get {
        /// Subscription title or ID
        subscription: String,
    },

    /// Create a subscription
    Create {
        /// Raw JSON body for CreateSubscriptionInput
        #[arg(long)]
        json: Option<String>,

        /// Media type (movie or tv)
        #[arg(long)]
        media_type: Option<String>,

        /// TMDB ID
        #[arg(long)]
        tmdb_id: Option<i64>,

        /// Subscription title
        #[arg(long)]
        title: Option<String>,

        /// Release year
        #[arg(long)]
        year: Option<String>,

        /// Season number
        #[arg(long)]
        season: Option<i32>,

        /// Episode list (comma-separated)
        #[arg(long)]
        episodes: Option<String>,

        /// Canonical category slug
        #[arg(long)]
        category: Option<String>,

        /// Source tags (comma-separated)
        #[arg(long)]
        sources: Option<String>,

        /// Resolution tags (comma-separated)
        #[arg(long)]
        resolutions: Option<String>,

        /// Codec tags (comma-separated)
        #[arg(long)]
        codecs: Option<String>,

        /// Release groups (comma-separated)
        #[arg(long)]
        release_groups: Option<String>,

        /// Minimum size in GB
        #[arg(long)]
        min_size: Option<f64>,

        /// Maximum size in GB
        #[arg(long)]
        max_size: Option<f64>,

        /// Minimum seeders
        #[arg(long)]
        min_seeders: Option<f64>,

        /// Maximum seeders
        #[arg(long)]
        max_seeders: Option<f64>,

        /// Include keywords
        #[arg(long)]
        include_keywords: Option<String>,

        /// Exclude keywords
        #[arg(long)]
        exclude_keywords: Option<String>,

        /// Only match free torrents
        #[arg(long, default_value_t = false)]
        free_only: bool,

        /// Exclude HR torrents
        #[arg(long, default_value_t = false)]
        exclude_hr: bool,

        /// Max downloads per run
        #[arg(long)]
        max_downloads_per_run: Option<i32>,

        /// Execute interval in minutes
        #[arg(long)]
        interval_minutes: Option<i32>,

        /// Site IDs (comma-separated)
        #[arg(long)]
        site_ids: Option<String>,

        /// Download client ID
        #[arg(long)]
        download_client_id: Option<String>,

        /// Skip the immediate matching run after creation
        #[arg(long, default_value_t = false)]
        no_run: bool,
    },

    /// Update a subscription
    Update {
        /// Subscription title or ID
        subscription: String,

        /// Raw JSON body for UpdateSubscriptionInput
        #[arg(long)]
        json: Option<String>,

        /// Episode list (comma-separated, empty string clears)
        #[arg(long)]
        episodes: Option<String>,

        /// Canonical category slug (empty string clears)
        #[arg(long)]
        category: Option<String>,

        /// Source tags (comma-separated, empty string clears)
        #[arg(long)]
        sources: Option<String>,

        /// Resolution tags (comma-separated, empty string clears)
        #[arg(long)]
        resolutions: Option<String>,

        /// Codec tags (comma-separated, empty string clears)
        #[arg(long)]
        codecs: Option<String>,

        /// Release groups (comma-separated, empty string clears)
        #[arg(long)]
        release_groups: Option<String>,

        /// Minimum size in GB
        #[arg(long)]
        min_size: Option<f64>,

        /// Maximum size in GB
        #[arg(long)]
        max_size: Option<f64>,

        /// Minimum seeders
        #[arg(long)]
        min_seeders: Option<f64>,

        /// Maximum seeders
        #[arg(long)]
        max_seeders: Option<f64>,

        /// Include keywords (empty string clears)
        #[arg(long)]
        include_keywords: Option<String>,

        /// Exclude keywords (empty string clears)
        #[arg(long)]
        exclude_keywords: Option<String>,

        /// Free-only filter
        #[arg(long)]
        free_only: Option<bool>,

        /// Exclude-HR filter
        #[arg(long)]
        exclude_hr: Option<bool>,

        /// Subscription status
        #[arg(long)]
        status: Option<String>,

        /// Execute interval in minutes
        #[arg(long)]
        interval_minutes: Option<i32>,

        /// Max downloads per run
        #[arg(long)]
        max_downloads_per_run: Option<i32>,

        /// Site IDs (comma-separated, empty string clears)
        #[arg(long)]
        site_ids: Option<String>,

        /// Download client ID (empty string clears)
        #[arg(long)]
        download_client_id: Option<String>,
    },

    /// Delete a subscription
    Delete {
        /// Subscription title or ID
        subscription: String,
    },

    /// Execute a subscription immediately
    Execute {
        /// Subscription title or ID
        subscription: String,
    },

    /// Read subscription logs
    Logs {
        /// Subscription title or ID
        subscription: String,

        /// Limit to last N log lines
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Read episode download progress
    EpisodeProgress {
        /// Subscription title or ID
        subscription: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum PtSitesCmd {
    /// List PT sites
    List {
        /// Include connectivity status
        #[arg(long, default_value_t = false)]
        status: bool,
    },

    /// Get one PT site
    Get {
        /// Site ID, site_id, or name
        site: String,
    },

    /// Add a PT site
    Add {
        /// Display name
        #[arg(long)]
        name: String,

        /// Site identifier (e.g. m-team)
        #[arg(long)]
        site_id: String,

        /// Site domain URL (required here; CLI cannot access server-only site-domain registry)
        #[arg(long)]
        domain: String,

        /// Auth type (none, cookies, api_key)
        #[arg(long)]
        auth_type: Option<String>,

        /// Cookie header value
        #[arg(long)]
        cookies: Option<String>,

        /// API key
        #[arg(long)]
        api_key: Option<String>,

        /// Auto stop minutes
        #[arg(long)]
        auto_stop_minutes: Option<i64>,

        /// Enable adult content
        #[arg(long, default_value_t = false)]
        adult_enabled: bool,
    },

    /// Update a PT site
    Update {
        /// Site ID, site_id, or name
        site: String,

        /// Site identifier
        #[arg(long)]
        site_id: Option<String>,

        /// Display name
        #[arg(long)]
        name: Option<String>,

        /// Site domain URL
        #[arg(long)]
        domain: Option<String>,

        /// Auth type (none, cookies, api_key)
        #[arg(long)]
        auth_type: Option<String>,

        /// Cookie header value (empty string clears)
        #[arg(long)]
        cookies: Option<String>,

        /// API key (empty string clears)
        #[arg(long)]
        api_key: Option<String>,

        /// Auto stop minutes (empty string clears)
        #[arg(long)]
        auto_stop_minutes: Option<String>,

        /// Enable site-level traffic management
        #[arg(long)]
        traffic_manage_enabled: Option<bool>,

        /// Site traffic mode
        #[arg(long)]
        traffic_manage_mode: Option<String>,

        /// Site traffic target (empty string clears)
        #[arg(long)]
        traffic_manage_target: Option<String>,

        /// Enable adult content
        #[arg(long)]
        adult_enabled: Option<bool>,
    },

    /// Delete a PT site
    Delete {
        /// Site ID, site_id, or name
        site: String,
    },

    /// Check PT site status
    Status {
        /// Site ID, site_id, or name (omit to check all sites)
        site: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum TrafficCmd {
    /// Show traffic-manage settings
    Settings,

    /// Update traffic-manage settings
    UpdateSettings {
        /// Download path for traffic-managed torrents
        #[arg(long)]
        download_path: Option<String>,

        /// Minimal free disk space (GB)
        #[arg(long)]
        min_free_disk_space_gb: Option<i32>,

        /// Stats window in minutes
        #[arg(long)]
        stats_window_minutes: Option<i32>,

        /// Maximum upload rate (Mbps)
        #[arg(long)]
        max_upload_rate_mbps: Option<i32>,

        /// Maximum active torrents
        #[arg(long)]
        max_active_torrents: Option<i32>,

        /// Scan interval in minutes
        #[arg(long)]
        scan_interval_minutes: Option<i32>,

        /// Cleanup interval in minutes
        #[arg(long)]
        cleanup_interval_minutes: Option<i32>,

        /// Download client ID
        #[arg(long, conflicts_with = "clear_download_client")]
        download_client_id: Option<String>,

        /// Clear download client binding
        #[arg(long, default_value_t = false)]
        clear_download_client: bool,

        /// Enable or disable traffic management
        #[arg(long)]
        enabled: Option<bool>,
    },

    /// List traffic-manage logs
    Logs {
        /// Site ID, site_id, or name filter
        #[arg(long)]
        site: Option<String>,

        /// Number of rows to return
        #[arg(long, default_value_t = 50)]
        limit: u64,

        /// Pagination offset
        #[arg(long, default_value_t = 0)]
        offset: u64,
    },

    /// Show traffic-manage stats
    Stats,

    /// Trigger an immediate scan
    TriggerScan,

    /// Trigger an immediate cleanup
    TriggerCleanup,
}

#[derive(Subcommand, Debug)]
pub enum CategoriesCmd {
    /// List all canonical category slugs
    List,
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Manage download clients
    #[command(subcommand)]
    Clients(ClientsCmd),

    /// Manage torrents
    #[command(subcommand)]
    Torrents(TorrentsCmd),

    /// Manage subscriptions
    #[command(subcommand)]
    Subscriptions(SubscriptionsCmd),

    /// Manage PT sites
    #[command(name = "pt-sites", subcommand)]
    PtSites(PtSitesCmd),

    /// Search torrents across PT sites
    Search {
        /// Search keyword
        keyword: String,

        /// Restrict search to one or more sites (repeatable)
        #[arg(long = "site")]
        sites: Vec<String>,

        /// Restrict search to canonical categories (repeatable)
        #[arg(long = "category")]
        categories: Vec<String>,

        /// Filter by resolution token, e.g. 2160p / 1080p (repeatable; "4k" aliases 2160p)
        #[arg(long = "resolution")]
        resolutions: Vec<String>,

        /// Keep only free / discounted torrents
        #[arg(long, default_value_t = false)]
        free: bool,
    },

    /// Download a torrent from a PT site to a download client (one-shot)
    Download {
        /// PT site: DB id, site_id, or name (resolved like other commands)
        #[arg(long = "site")]
        site: String,

        /// Torrent id (the TorrentID column from `search`)
        #[arg(long = "torrent-id")]
        torrent_id: String,

        /// Download client: name or id. Defaults to the configured default client
        #[arg(long = "client")]
        client: Option<String>,

        /// Canonical category slug (e.g. movie, tv) — used to resolve the save path
        #[arg(long)]
        category: Option<String>,

        /// Explicit save-path override (skips category-based resolution)
        #[arg(long = "save-path")]
        save_path: Option<String>,

        /// Season number to filter (TV)
        #[arg(long)]
        season: Option<i32>,

        /// Episode list to keep (comma-separated, TV)
        #[arg(long)]
        episodes: Option<String>,

        /// Tags to attach (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Add in paused state
        #[arg(long, default_value_t = false)]
        paused: bool,
    },

    /// Manage traffic-control settings and reports
    #[command(subcommand)]
    Traffic(TrafficCmd),

    /// Manage canonical categories
    #[command(subcommand)]
    Categories(CategoriesCmd),
}
