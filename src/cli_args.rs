use std::path::PathBuf;

use clap::Parser;
use once_cell::sync::Lazy;

use crate::discovery_mode::DiscoveryMode;

#[derive(Debug, Parser)]
#[clap(name = "")]
pub struct Args {
    /// The location to store the .uplink directory, within which a .warp, state.json, and other useful logs will be located
    #[clap(long)]
    path: Option<PathBuf>,
    #[clap(long)]
    discovery: Option<DiscoveryMode>,
    #[clap(long)]
    enable_quic: bool,
    #[clap(long)]
    discovery_point: Option<String>,
    #[cfg(debug_assertions)]
    #[clap(long, default_value_t = false)]
    with_mock: bool,
    /// tells the app that it was installed via an installer, not built locally. Uplink will look for an `extra.zip` file based on
    /// the platform-specific installer.
    #[clap(long, default_value_t = false)]
    pub production_mode: bool,
    /// configures log output
    #[clap(long, default_value_t = false)]
    pub log_to_file: bool,
}

#[derive(Debug)]
pub struct StaticArgs {
    /// ~/.uplink
    /// contains the following: extra (folder), extensions (folder), themes (folder), fonts (folder), .user
    pub dot_uplink: PathBuf,
    /// ~/.uplink/.user
    /// contains the following: warp (folder), state.json, debug.log
    pub uplink_path: PathBuf,
    /// Directory for temporary files and deleted everytime app is closed or opened
    pub temp_files: PathBuf,
    /// custom themes for the user
    pub themes_path: PathBuf,
    /// custom fonts for the user
    pub fonts_path: PathBuf,
    /// state.json: a serialized version of State which gets saved every time state is modified
    pub cache_path: PathBuf,
    /// a fake tesseract_path to prevent anything from mutating the tesseract keypair after it has been created (probably not necessary)
    pub mock_cache_path: PathBuf,
    /// houses warp specific data
    pub warp_path: PathBuf,
    /// a debug log which is only written to when the settings are enabled. otherwise logs are only sent to stdout
    pub logger_path: PathBuf,
    /// contains the keypair used for IPFS
    pub tesseract_file: String,
    /// the unlock and auth pages don't have access to State but need to know if they should play a notification.
    /// part of state is serialized and saved here
    pub login_config_path: PathBuf,
    /// path to custom plugins
    pub extensions_path: PathBuf,
    /// crash logs
    pub crash_logs: PathBuf,
    /// recordings
    pub recordings: PathBuf,
    /// seconds
    pub typing_indicator_refresh: u64,
    /// seconds
    pub typing_indicator_timeout: u64,
    /// used only for testing the UI. generates fake friends, conversations, and messages
    pub use_mock: bool,
    /// Disable discovery
    pub discovery: DiscoveryMode,
    /// Enable quic transport
    pub enable_quic: bool,
    // some features aren't ready for release. This field is used to disable such features.
    pub production_mode: bool,
}

pub static STATIC_ARGS: Lazy<StaticArgs> = Lazy::new(|| {
    let args = Args::parse();
    #[allow(unused_mut)]
    #[allow(unused_assignments)]
    let mut use_mock = false;
    #[cfg(debug_assertions)]
    {
        use_mock = args.with_mock;
    }

    let uplink_container = match args.path {
        Some(path) => path,
        _ => dirs::home_dir().unwrap_or_default().join(".uplink"),
    };

    let uplink_path = uplink_container.join(".user");
    let warp_path = uplink_path.join("warp");
    StaticArgs {
        dot_uplink: uplink_container.clone(),
        uplink_path: uplink_path.clone(), // TODO: Should this be "User path" instead?
        temp_files: uplink_container.join("temp_files"),
        themes_path: uplink_container.join("themes"),
        fonts_path: uplink_container.join("fonts"),
        cache_path: uplink_path.join("state.json"),
        extensions_path: uplink_container.join("extensions"),
        crash_logs: uplink_container.join("crash-logs"),
        recordings: uplink_container.join("recordings"),
        mock_cache_path: uplink_path.join("mock-state.json"),
        warp_path: warp_path.clone(),
        logger_path: uplink_path.join("debug.log"),
        typing_indicator_refresh: 5,
        typing_indicator_timeout: 6,
        tesseract_file: "tesseract.json".into(),
        login_config_path: uplink_path.join("login_config.json"),
        use_mock,
        discovery: args.discovery.unwrap_or_default(),
        enable_quic: args.enable_quic,
        production_mode: cfg!(feature = "production_mode"),
    }
});
