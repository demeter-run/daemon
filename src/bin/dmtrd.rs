use std::{io::stdout, sync::Arc};

use clap::Parser;
use serde::Deserialize;

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
    let app = App::parse();

    tracing_subscriber::fmt::init();

    // let mut opt = ConnectOptions::new("protocol://username:password@host/database");
    // opt.max_connections(100)
    //     .min_connections(5)
    //     .connect_timeout(Duration::from_secs(8))
    //     .acquire_timeout(Duration::from_secs(8))
    //     .idle_timeout(Duration::from_secs(8))
    //     .max_lifetime(Duration::from_secs(8))
    //     .sqlx_logging(true)
    //     .sqlx_logging_level(log::LevelFilter::Info)
    //     .set_schema_search_path("my_schema"); // Setting default PostgreSQL schema

    // let db = Database::connect(opt).await?;

    // dmtrd::drivers::rpc::serve(Config {
    //     listen_address: "[::]:50051".into(),
    //     auth: auth_v3::Config {},
    // })
    // .await
    // .unwrap();
}
