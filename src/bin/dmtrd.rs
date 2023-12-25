use std::sync::Arc;

use clap::Parser;
use dmtrd::{
    domain::Domain,
    driven::{event_dispatch::EventDispatch, fabric_state::FabricState},
};
use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::info;

#[derive(Parser)]
#[clap(name = "Demeter Operator", version = "")]
struct App {}

#[derive(Deserialize, Debug)]
struct ConfigRoot {}

impl ConfigRoot {
    pub fn new(explicit_file: &Option<std::path::PathBuf>) -> Result<Self, config::ConfigError> {
        let mut s = config::Config::builder();

        // but we can override it by having a file in the working dir
        s = s.add_source(config::File::with_name("config.toml").required(false));

        // if an explicit file was passed, then we load it as mandatory
        if let Some(explicit) = explicit_file.as_ref().and_then(|x| x.to_str()) {
            s = s.add_source(config::File::with_name(explicit).required(true));
        }

        // finally, we use env vars to make some last-step overrides
        s = s.add_source(config::Environment::with_prefix("DMTR").separator("_"));

        s.build()?.try_deserialize()
    }
}

async fn seed_dummy_data(domain: &mut Domain) {
    domain
        .on_namespace_minted(dmtrd::domain::NamespaceMintedV1 {
            name: "ns1".into(),
            root_public_key: "123".into(),
        })
        .await
        .unwrap();

    let pwd = hex::decode("6d7962616470617373776f7264").unwrap();
    let salt = b"somesaltforyou";

    domain
        .on_apikey_registered(dmtrd::domain::ApiKeyRegisteredV1 {
            namespace: "ns1".into(),
            digest: dmtrd::domain::digest(&pwd, salt).unwrap(),
            salt: salt.to_vec(),
        })
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    let _ = App::parse();

    tracing_subscriber::fmt::init();

    let fabric_state = FabricState::ephemeral().await.unwrap();
    let event_dispatch = EventDispatch::ephemeral(100);

    let mut domain = Domain {
        fabric_state,
        event_dispatch,
    };

    seed_dummy_data(&mut domain).await;

    let domain = Arc::new(Mutex::new(domain));

    let domain1 = domain.clone();
    let thread1 = tokio::spawn(async move {
        info!("starting rpc driver");

        dmtrd::drivers::rpc::serve(
            dmtrd::drivers::rpc::Config {
                listen_address: "[::]:50051".into(),
            },
            domain1,
        )
        .await
    });

    let domain2 = domain.clone();
    let thread2 = tokio::spawn(async move {
        info!("starting fabric sync");

        dmtrd::drivers::fabric_sync::run(domain2).await
    });

    let (res1, res2) = tokio::try_join!(thread1, thread2).unwrap();

    res1.and(res2).unwrap();
}
