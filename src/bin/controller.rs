use std::sync::Arc;

use demeter_operator::rektor::ContextData;
use kube::client::Client;
use pasetors::keys::Generate;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct ConfigRoot {
    auth: demeter_operator::authtokens::AuthConfig,
}

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
        s = s.add_source(config::Environment::with_prefix("DEMETER").separator("_"));

        s.build()?.try_deserialize()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let kp = pasetors::keys::AsymmetricKeyPair::<pasetors::version2::V2>::generate().unwrap();
    dbg!(hex::encode(kp.secret.as_bytes()));
    dbg!(hex::encode(kp.public.as_bytes()));

    let config = ConfigRoot::new(&None).expect("couldn't load config");

    dbg!(&config.auth.secret_key.len());

    let kubernetes_client: Client = Client::try_default()
        .await
        .expect("Expected a valid KUBECONFIG environment variable.");

    let mut extensions = anymap::Map::new();
    extensions.insert(config.auth);

    let context: Arc<ContextData> =
        Arc::new(ContextData::new(kubernetes_client.clone(), extensions));

    demeter_operator::rektor::run(context.clone(), demeter_operator::authtokens::derive_state)
        .await;
}
