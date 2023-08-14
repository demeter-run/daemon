use futures::stream::StreamExt;
use kube::runtime::watcher::Config;
use kube::Resource;
use kube::{runtime::controller::Action, runtime::Controller, Api};
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{info, warn};

use crate::framework::ContextData;
use crate::framework::Error;
use crate::framework::NamespacedAgent;

use super::AuthToken;
use super::AuthTokenStatus;

pub async fn run(context: Arc<ContextData>) {
    // Preparation of resources used by the `kube_runtime::Controller`
    let crd_api: Api<AuthToken> = Api::all(context.client.clone());

    // The controller comes from the `kube_runtime` crate and manages the reconciliation process.
    // It requires the following information:
    // - `kube::Api<T>` this controller "owns". In this case, `T = Echo`, as this controller owns the `Echo` resource,
    // - `kube::runtime::watcher::Config` can be adjusted for precise filtering of `Echo` resources before the actual reconciliation, e.g. by label,
    // - `reconcile` function with reconciliation logic to be called each time a resource of `Echo` kind is created/updated/deleted,
    // - `on_error` function to call whenever reconciliation fails.
    Controller::new(crd_api.clone(), Config::default())
        .run(reconcile, on_error, context)
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

#[derive(Debug)]
enum DomainAction {
    GenerateToken,
    ForgetToken,
    AttemptCleanup,
    NoOp,
}

async fn reconcile(res: Arc<AuthToken>, context: Arc<ContextData>) -> Result<Action, Error> {
    let action = determine_action(&res);
    let agent = NamespacedAgent::<AuthToken>::build(res, &context)?;

    info!(?action, "auth token domain action detected");

    match action {
        DomainAction::GenerateToken => {
            // TODO: generate paseto and patch status
            // TODO: add finalizer too

            let patched = agent
                .patch_status(AuthTokenStatus {
                    emitted: true,
                    secret: Some("something".into()),
                })
                .await?;

            warn!(?patched, "generated token");

            Ok(Action::await_change())
        }
        DomainAction::ForgetToken => {
            let patched = agent
                .patch_status(AuthTokenStatus {
                    emitted: true,
                    secret: None,
                })
                .await?;

            warn!(?patched, "removed secret from data");

            Ok(Action::await_change())
        }
        DomainAction::AttemptCleanup => {
            // TODO: check if expired and remove finalizer
            Ok(Action::requeue(Duration::from_secs(10)))
        }
        DomainAction::NoOp => Ok(Action::await_change()),
    }
}

fn determine_action(token: &AuthToken) -> DomainAction {
    if token.meta().deletion_timestamp.is_some() {
        return DomainAction::AttemptCleanup;
    }

    if token.status.is_none() {
        return DomainAction::GenerateToken;
    }

    if token
        .status
        .as_ref()
        .map(|s| !s.emitted)
        .unwrap_or_default()
    {
        return DomainAction::GenerateToken;
    }

    if token.spec.viewed
        && token
            .status
            .as_ref()
            .map(|s| s.secret.is_some())
            .unwrap_or_default()
    {
        return DomainAction::ForgetToken;
    }

    DomainAction::NoOp
}

/// Actions to be taken when a reconciliation fails - for whatever reason.
/// Prints out the error to `stderr` and requeues the resource for another reconciliation after
/// five seconds.
///
/// # Arguments
/// - `echo`: The erroneous resource.
/// - `error`: A reference to the `kube::Error` that occurred during reconciliation.
/// - `_context`: Unused argument. Context Data "injected" automatically by kube-rs.
fn on_error(echo: Arc<AuthToken>, error: &Error, _context: Arc<ContextData>) -> Action {
    eprintln!("Reconciliation error:\n{:?}.\n{:?}", error, echo);
    Action::requeue(Duration::from_secs(5))
}
