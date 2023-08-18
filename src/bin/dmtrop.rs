use std::{io::stdout, sync::Arc};

use clap::Parser;
use kube::CustomResourceExt;
use serde::Deserialize;

#[derive(Parser)]
#[clap(name = "Demeter Operator", version = "")]
struct App {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    #[clap(about = "Runs the daemon")]
    Daemon(Daemon),
    #[clap(about = "Manages the CRDs")]
    Crds(Crds),
}

#[derive(Parser)]
struct Daemon {
    // Add any arguments or options for the "daemon" subcommand here
}

#[derive(Parser)]
struct Crds {
    // Add any arguments or options for the "crds" subcommand here
}

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
    let app = App::parse();

    match app.subcmd {
        SubCommand::Daemon(_) => {
            tracing_subscriber::fmt::init();

            //let kp = pasetors::keys::AsymmetricKeyPair::<pasetors::version2::V2>::generate().unwrap();
            //dbg!(hex::encode(kp.secret.as_bytes()));
            //dbg!(hex::encode(kp.public.as_bytes()));

            let config = ConfigRoot::new(&None).expect("couldn't load config");

            let kubernetes_client = kube::Client::try_default()
                .await
                .expect("Expected a valid KUBECONFIG environment variable.");

            let mut extensions = anymap::Map::new();
            extensions.insert(config.auth);

            let context = Arc::new(demeter_operator::rektor::ContextData::new(
                kubernetes_client.clone(),
                extensions,
            ));

            demeter_operator::rektor::run(
                context.clone(),
                demeter_operator::authtokens::derive_state,
            )
            .await;
        }
        SubCommand::Crds(_) => {
            let stdout = stdout();
            serde_yaml::to_writer(stdout, &demeter_operator::authtokens::AuthToken::crd()).unwrap();
        }
    }
}
