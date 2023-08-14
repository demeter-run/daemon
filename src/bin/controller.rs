use std::sync::Arc;

use demeter_operator::framework::ContextData;
use kube::client::Client;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // First, a Kubernetes client must be obtained using the `kube` crate
    // The client will later be moved to the custom controller
    let kubernetes_client: Client = Client::try_default()
        .await
        .expect("Expected a valid KUBECONFIG environment variable.");

    let context: Arc<ContextData> = Arc::new(ContextData::new(kubernetes_client.clone()));

    demeter_operator::authtokens::controller::run(context.clone()).await;
}
