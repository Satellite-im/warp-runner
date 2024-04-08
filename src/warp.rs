use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{debug, error, trace, warn};
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
    tesseract: Mutex<Tesseract>,
    multipass: Mutex<Box<dyn MultiPass>>,
    raygun: Mutex<Box<dyn RayGun>>,
    constellation: Mutex<Box<dyn Constellation>>,
    blink: Mutex<Box<dyn Blink>>,

    config: Config,
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
            .set_config(config.clone())
            .finalize()
            .await?;

        let blink = warp_blink_wrtc::BlinkImpl::new(multipass.clone()).await?;

        Ok(Self {
            inner: Arc::new(WarpInner {
                tesseract: Mutex::new(tesseract),
                multipass: Mutex::new(multipass),
                raygun: Mutex::new(raygun),
                constellation: Mutex::new(constellation),
                blink: Mutex::new(blink),
                config,
            }),
        })
    }

    pub async fn create_identity(
        &self,
        username: String,
        tesseract_passphrase: String,
        seed_words: String,
    ) -> Result<Identity, warp::error::Error> {
        let mut tesseract = self.inner.tesseract.lock().await;
        if tesseract.exist("keypair") {
            debug!("attempting to overwrite old account");
            *tesseract = init_tesseract(true)
                .await
                .expect("failed to initialize tesseract");
            self.reinit(tesseract.clone()).await?;
        }

        tesseract.unlock(tesseract_passphrase.as_bytes())?;
        let _id = self
            .inner
            .multipass
            .lock()
            .await
            .create_identity(Some(&username), Some(&seed_words))
            .await
            .expect("create_identity failed. should never happen");
        let ident = self.wait_for_multipass().await.map_err(|e| {
            tesseract.lock();

            e
        })?;

        Ok(ident)
    }

    async fn reinit(&self, tesseract: Tesseract) -> Result<(), warp::error::Error> {
        let inner = &self.inner;
        let (multipass, raygun, constellation) = WarpIpfsBuilder::default()
            .set_tesseract(tesseract.clone())
            .set_config(inner.config.clone())
            .finalize()
            .await?;
        let blink = warp_blink_wrtc::BlinkImpl::new(multipass.clone()).await?;

        *inner.multipass.lock().await = multipass;
        *inner.raygun.lock().await = raygun;
        *inner.constellation.lock().await = constellation;
        *inner.blink.lock().await = blink;

        Ok(())
    }

    async fn wait_for_multipass(&self) -> Result<Identity, warp::error::Error> {
        loop {
            match self.inner.multipass.lock().await.get_own_identity().await {
                Ok(ident) => return Ok(ident),
                Err(warp::error::Error::MultiPassExtensionUnavailable) => {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }
                Err(e) => {
                    error!("multipass.get_own_identity failed: {}", e);
                    return Err(e);
                }
            }
        }
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
