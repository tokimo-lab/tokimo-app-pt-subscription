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
