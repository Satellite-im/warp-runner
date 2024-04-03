use tracing::debug;
use warp_ipfs::config::Discovery;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum DiscoveryMode {
    /// Enable full discovery
    Full,

    /// Use warp specific discovery
    #[default]
    Shuttle,

    /// Address to a specific discovery point
    RzPoint { address: String },

    /// Disable discovery
    Disable,
}

impl std::str::FromStr for DiscoveryMode {
    type Err = warp::error::Error;
    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        match mode.to_lowercase().as_str() {
            "full" => Ok(DiscoveryMode::Full),
            "shuttle" => Ok(DiscoveryMode::Shuttle),
            "disable" => Ok(DiscoveryMode::Disable),
            _ => Err(warp::error::Error::Other),
        }
    }
}

impl From<&DiscoveryMode> for Discovery {
    fn from(mode: &DiscoveryMode) -> Self {
        match mode {
            DiscoveryMode::Full => Discovery::Namespace {
                namespace: None,
                discovery_type: warp_ipfs::config::DiscoveryType::DHT,
            },
            DiscoveryMode::RzPoint { address } => Discovery::Namespace {
                namespace: None,
                discovery_type: warp_ipfs::config::DiscoveryType::RzPoint {
                    addresses: vec![address.parse().expect("Valid multiaddr address")],
                },
            },
            DiscoveryMode::Shuttle => {
                let env_addrs = std::env::var("SHUTTLE_ADDR_POINT")
                    .map(|val| {
                        val.split(',')
                            .filter_map(|addr_str| addr_str.parse::<_>().ok())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                let addresses = match env_addrs.is_empty() {
                    true => Vec::from_iter(["/ip4/159.65.41.31/tcp/8848/p2p/12D3KooWRF2bz3KDRPvBs1FASRDRk7BfdYc1RUcfwKsz7UBEu7mL"
                                .parse()
                                .expect("valid addr")]),
                    false => env_addrs
                };

                debug!("shuttle addresses: {:?}", addresses);

                Discovery::Shuttle { addresses }
            }
            DiscoveryMode::Disable => Discovery::None,
        }
    }
}

impl From<DiscoveryMode> for Discovery {
    fn from(mode: DiscoveryMode) -> Self {
        Self::from(&mode)
    }
}
