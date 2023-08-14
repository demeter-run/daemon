use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod controller;

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct AuthTokenStatus {
    emitted: bool,
    secret: Option<String>,
}

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    group = "demeter.run",
    version = "v1alpha",
    kind = "AuthToken",
    status = "AuthTokenStatus",
    namespaced
)]
pub struct AuthTokenSpec {
    issuer: String,
    expiration: Option<u64>,
    viewed: bool,
    revoked: bool,
}
