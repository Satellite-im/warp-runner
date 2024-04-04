use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{debug, error, info, trace, warn};
use warp::{
    blink::Blink,
    constellation::{file::FileType, Constellation},
    multipass::{identity::Identity, MultiPass},
    raygun::RayGun,
    tesseract::Tesseract,
};
use warp_ipfs::{
    config::{Bootstrap, Config, Discovery, UpdateEvents},
    WarpIpfsBuilder,
};

use crate::cli_args::STATIC_ARGS;

#[derive(Clone)]
/// The Warp state.
pub struct Warp {
    inner: Arc<WarpInner>,
}

/// The Warp state.
pub struct WarpInner {
    tesseract: Tesseract,
    multipass: Mutex<Box<dyn MultiPass>>,
    raygun: Box<dyn RayGun>,
    constellation: Box<dyn Constellation>,
    blink: Box<dyn Blink>,
}

impl Warp {
    /// Initialize the Warp state.
    pub async fn init() -> Result<Self, warp::error::Error> {
        let tesseract = init_tesseract(false)
            .await
            .expect("failed to initialize tesseract");

        let path = &STATIC_ARGS.warp_path;
        let mut config = Config::production(path);

        // Discovery is disabled by default for now but may offload manual discovery through a separate service
        // in the near future
        config.store_setting.discovery = Discovery::from(&STATIC_ARGS.discovery);

        config.save_phrase = true; // TODO: This should be bound to a setting within Uplink so that the user can choose not to reveal the phrase for increased security.``
        config.bootstrap = Bootstrap::None;
        config.ipfs_setting.disable_quic = !STATIC_ARGS.enable_quic;
        config.ipfs_setting.portmapping = true;
        config.ipfs_setting.agent_version = Some(format!("uplink/{}", env!("CARGO_PKG_VERSION")));
        config.store_setting.emit_online_event = true;
        config.store_setting.share_platform = true;
        config.store_setting.update_events = UpdateEvents::Enabled;
        config.store_setting.default_profile_picture = Some(Arc::new(|identity| {
            let mut content =
                plot_icon::generate_png(identity.did_key().to_string().as_bytes(), 512)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            content.extend([11, 00, 23]);

            Ok((
                content,
                FileType::Mime("image/png".parse().expect("Correct mime")),
            ))
        }));
        config.thumbnail_size = (500, 500);

        let (multipass, raygun, constellation) = WarpIpfsBuilder::default()
            .set_tesseract(tesseract.clone())
            .set_config(config)
            .finalize()
            .await?;

        let blink = warp_blink_wrtc::BlinkImpl::new(multipass.clone()).await?;

        Ok(Self {
            inner: Arc::new(WarpInner {
                tesseract,
                multipass: Mutex::new(multipass),
                raygun,
                constellation,
                blink,
            }),
        })
    }
}

async fn init_tesseract(overwrite_old_account: bool) -> Result<Tesseract, warp::error::Error> {
    trace!("initializing tesseract");

    if overwrite_old_account {
        // delete old account data
        if let Err(e) = tokio::fs::remove_dir_all(&STATIC_ARGS.uplink_path).await {
            warn!("failed to delete uplink directory: {}", e);
        }

        // create directories
        if let Err(e) = tokio::fs::create_dir_all(&STATIC_ARGS.warp_path).await {
            warn!("failed to create warp directory: {}", e);
        }
    }

    // open existing file or create new one
    Tesseract::open_or_create(&STATIC_ARGS.warp_path, &STATIC_ARGS.tesseract_file)
}
