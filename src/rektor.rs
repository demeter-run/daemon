use anymap::any::Any;
use anymap::AnyMap;
use futures::StreamExt;
use k8s_openapi::NamespaceResourceScope;
use kube::api::Patch;
use kube::api::PatchParams;
use kube::runtime::controller::Action;
use kube::runtime::watcher::Config;
use kube::runtime::Controller;
use kube::Resource;
use kube::ResourceExt;
use kube::{client::Client, Api};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;
use tracing::warn;

pub struct ContextData {
    pub client: Client,
    pub extensions: anymap::Map<dyn Any + Send + Sync>,
}

impl ContextData {
    pub fn new(client: Client, extensions: anymap::Map<dyn Any + Send + Sync>) -> Self {
        ContextData { client, extensions }
    }
}

/// All errors possible to occur during reconciliation
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Any error originating from the `kube-rs` crate
    #[error("Kubernetes reported error: {source}")]
    KubeError {
        #[from]
        source: kube::Error,
    },
    /// Error in user input or Echo resource definition, typically missing fields.
    #[error("Invalid Echo CRD: {0}")]
    UserInputError(String),
}

pub struct DerivedResourceState<S>
where
    S: Send + Sync,
{
    pub status: S,
    pub finalizer: Option<String>,
}

impl<S> DerivedResourceState<S>
where
    S: Serialize + Send + Sync,
{
    fn status_patch(&self) -> Patch<Value> {
        Patch::Merge(json!({ "status": self.status }))
    }
}

pub async fn apply<K, S>(
    owner: &K,
    state: DerivedResourceState<S>,
    context: Arc<ContextData>,
) -> Result<(), Error>
where
    K: Resource<Scope = NamespaceResourceScope> + DeserializeOwned,
    K::DynamicType: Default,
    S: Serialize + Send + Sync,
{
    let client = context.client.clone();

    let ns: String = owner
        .namespace()
        .ok_or(Error::UserInputError("missing namespace".into()))?;

    let api = Api::<K>::namespaced(client, &ns);

    let name = owner.name_any();

    api.patch_status(&name, &PatchParams::default(), &state.status_patch())
        .await
        .map_err(|source| Error::KubeError { source })?;

    Ok(())
}

pub type DeriveStateFn<K, S> = fn(res: &K, ctx: &ContextData) -> DerivedResourceState<S>;

pub async fn run<K, S>(context: Arc<ContextData>, derive: DeriveStateFn<K, S>)
where
    K: Resource<Scope = NamespaceResourceScope>
        + DeserializeOwned
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + 'static,
    K::DynamicType: Default + Eq + Hash + Clone + std::fmt::Debug + Unpin,
    S: Serialize + Send + Sync + 'static,
{
    // Preparation of resources used by the `kube_runtime::Controller`
    let api: Api<K> = Api::all(context.client.clone());

    // The controller comes from the `kube_runtime` crate and manages the reconciliation process.
    // It requires the following information:
    // - `kube::Api<T>` this controller "owns". In this case, `T = Echo`, as this controller owns the `Echo` resource,
    // - `kube::runtime::watcher::Config` can be adjusted for precise filtering of `Echo` resources before the actual reconciliation, e.g. by label,
    // - `reconcile` function with reconciliation logic to be called each time a resource of `Echo` kind is created/updated/deleted,
    // - `on_error` function to call whenever reconciliation fails.
    Controller::new(api.clone(), Config::default())
        .run(
            |r, c| async move {
                let state = derive(&r, &c);
                apply(r.as_ref(), state, c).await?;

                Ok(Action::await_change())
            },
            on_error,
            context,
        )
        .for_each(|reconciliation_result| async move {
            match reconciliation_result {
                Ok(echo_resource) => {
                    info!("Reconciliation successful. Resource: {:?}", echo_resource);
                }
                Err(reconciliation_err) => {
                    warn!("Reconciliation error: {:?}", reconciliation_err)
                }
            }
        })
        .await;
}

fn on_error<K>(spec: Arc<K>, error: &Error, _context: Arc<ContextData>) -> Action {
    warn!(%error, "reconciliation error");
    Action::requeue(Duration::from_secs(5))
}
