use std::sync::Arc;

use kube::{CustomResource, ResourceExt};
use pasetors::{claims::Claims, keys::AsymmetricSecretKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rektor::{ContextData, DerivedResourceState};

#[derive(Deserialize, Debug)]
pub struct AuthConfig {
    #[serde(with = "hex")]
    pub secret_key: Vec<u8>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, Default)]
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

fn build_new_paseto(token: &AuthToken, ctx: &ContextData) -> String {
    let cfg = ctx
        .extensions
        .get::<AuthConfig>()
        .expect("missing auth config in context");

    let secret_key =
        AsymmetricSecretKey::from(&cfg.secret_key).expect("invalid asymmetric secret key");

    let ns = token.namespace().unwrap();

    let mut c = Claims::new().unwrap();
    c.subject(&ns).unwrap();

    let msg = c.to_string().unwrap();

    pasetors::version2::PublicToken::sign(&secret_key, msg.as_bytes(), None).unwrap()
}

fn is_expired(token: &AuthToken) -> bool {
    // TODO
    false
}

pub fn derive_state(
    spec: Arc<AuthToken>,
    ctx: &ContextData,
) -> DerivedResourceState<AuthToken, AuthTokenStatus> {
    let mut status = spec.status.clone().unwrap_or_default();

    if !status.emitted {
        status = AuthTokenStatus {
            secret: Some(build_new_paseto(&spec, &ctx)),
            emitted: true,
        }
    }

    if spec.spec.viewed {
        status = AuthTokenStatus {
            secret: None,
            emitted: true,
        }
    }

    let finalizer = match is_expired(&spec) {
        true => Some("demeter.run/wait-for-expired".into()),
        false => None,
    };

    DerivedResourceState {
        spec,
        finalizer,
        status,
    }
}
