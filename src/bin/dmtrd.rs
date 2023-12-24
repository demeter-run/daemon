use clap::Parser;
use serde::Deserialize;
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

#[tokio::main]
async fn main() {
    let _ = App::parse();

    tracing_subscriber::fmt::init();

    info!("starting rpc driver");

    dmtrd::drivers::rpc::serve(dmtrd::drivers::rpc::Config {
        listen_address: "[::]:50051".into(),
    })
    .await
    .unwrap();
}
