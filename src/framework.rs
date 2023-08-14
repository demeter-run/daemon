use k8s_openapi::NamespaceResourceScope;
use kube::api::Patch;
use kube::api::PatchParams;
use kube::Resource;
use kube::ResourceExt;
use kube::{client::Client, Api};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use std::sync::Arc;

/// Context injected with each `reconcile` and `on_error` method invocation.
pub struct ContextData {
    /// Kubernetes client to make Kubernetes API requests with. Required for K8S resource management.
    pub client: Client,
}

impl ContextData {
    /// Constructs a new instance of ContextData.
    ///
    /// # Arguments:
    /// - `client`: A Kubernetes client to make Kubernetes REST API requests with. Resources
    /// will be created and deleted with this client.
    pub fn new(client: Client) -> Self {
        ContextData { client }
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

pub struct NamespacedAgent<T> {
    api: Api<T>,
    res: Arc<T>,
}

impl<K> NamespacedAgent<K>
where
    K: Resource<Scope = NamespaceResourceScope> + DeserializeOwned,
    K::DynamicType: Default,
{
    pub fn build(res: Arc<K>, ctx: &ContextData) -> Result<Self, Error> {
        let client = ctx.client.clone();

        let namespace: String = res
            .namespace()
            .ok_or(Error::UserInputError("missing namespace".into()))?;

        Ok(Self {
            res,
            api: Api::namespaced(client, &namespace),
        })
    }

    pub async fn patch_status<S>(&self, status: S) -> Result<K, Error>
    where
        S: Serialize,
    {
        let name = self.res.name_any();
        let data = json!({ "status": status });
        let patch: Patch<Value> = Patch::Merge(data);

        self.api
            .patch_status(&name, &PatchParams::default(), &patch)
            .await
            .map_err(|source| Error::KubeError { source })
    }
}
