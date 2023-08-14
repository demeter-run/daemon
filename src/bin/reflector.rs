use futures::{StreamExt, TryStreamExt};
use tracing::*;

use kube::{
    api::Api,
    runtime::{reflector, watcher, WatchStreamExt},
    Client, ResourceExt,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let client = Client::try_default().await?;

    // 1. Run a reflector against the installed CRD
    let (reader, writer) = reflector::store::<demeter_operator::authtokens::AuthToken>();

    let foos: Api<demeter_operator::authtokens::AuthToken> = Api::all(client);
    let wc = watcher::Config::default().any_semantic();
    let mut stream = watcher(foos, wc)
        .default_backoff()
        .reflect(writer)
        .applied_objects()
        .boxed();

    // tokio::spawn(async move {
    //     reader.wait_until_ready().await.unwrap();
    //     loop {
    //         // Periodically read our state
    //         // while this runs you can kubectl apply -f crd-baz.yaml or crd-qux.yaml and see it works
    //         tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    //         let crds = reader
    //             .state()
    //             .iter()
    //             .map(|r| r.name_any())
    //             .collect::<Vec<_>>();
    //         info!("Current crds: {:?}", crds);
    //     }
    // });

    while let Some(event) = stream.try_next().await? {
        info!("saw {:?}", event.spec);
    }

    Ok(())
}
