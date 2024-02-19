/// Domain
///
/// Should:
/// - handle extrinsic events to update the internal state
/// - handle extrinsic events to actuate on outside systems
/// - execute commands and emit intrinsic events
use anyhow::{bail, Result};
use tracing::info;

use crate::driven::event_dispatch::EventDispatch;
use crate::driven::fabric_state::{AccountDelta, FabricState};

mod auth;
mod events;

pub use auth::*;
pub use events::*;

pub struct Config {
    pub cluster: ClusterUuid,
}

pub struct Domain {
    pub config: Config,
    pub event_dispatch: EventDispatch,
    pub fabric_state: FabricState,
}

pub type SignatureValue = String;
pub type AuthTimestamp = u64;
pub type SecretValue = Vec<u8>;
pub type HashDigest = [u8; 32];
pub type HashSalt = Vec<u8>;

#[derive(Clone)]
pub enum Credential {
    OwnerSignatureV1(SignatureValue, AuthTimestamp),
    ApiKeyV1(SecretValue),
}

pub struct RegisterApiKeyCmd {
    pub auth: Credential,
    pub namespace: NamespaceName,
    pub secret: SecretValue,
}

pub struct CreateResourceCmd {
    pub auth: Credential,
    pub namespace: String,
    pub name: String,
    pub kind: String,
    pub spec: Blob,
}

pub struct CreateResourceAck {
    pub event_receipt: Vec<u8>,
    pub resource_uuid: Vec<u8>,
}

pub struct ListResourcesQuery {
    pub auth: Credential,
    pub resource_name: String,
    pub namespace_name: String,
}

pub struct ListResourcesItem {
    pub metadata: ResourceMetadataV1,
    pub spec: Blob,
    pub status: Blob,
}

pub struct ReadBalanceQuery {
    pub auth: Credential,
    pub namespace_name: String,
}

#[derive(Debug)]
pub struct ReadBalanceOutput {
    pub accounts: Vec<(u64, u64, u64)>,
}

impl Domain {
    async fn assert_available_namespace(&self, ns: &NamespaceName) -> Result<()> {
        let exists = self.fabric_state.namespace_exists(&ns).await?;

        if exists {
            bail!("namespace isn't available")
        }

        Ok(())
    }

    async fn on_namespace_minted(&self, evt: NamespaceMintedV1) -> Result<()> {
        info!("namespace minted");

        // TODO: how do we handle business invariants? eg, if the namespace isn't
        // available, then something is inconsistent at a global scale.
        self.assert_available_namespace(&evt.name).await?;

        self.fabric_state.insert_namespace(&evt.name).await?;

        Ok(())
    }

    async fn assert_existing_namespace(&self, ns: &NamespaceName) -> Result<()> {
        let exists = self.fabric_state.namespace_exists(&ns).await?;

        if !exists {
            bail!("invalid namespace")
        }

        Ok(())
    }

    async fn assert_valid_api_key(&self, ns: &NamespaceName, secret: SecretValue) -> Result<()> {
        let keys = self
            .fabric_state
            .get_all_api_keys_for_namespace(&ns)
            .await?;

        for key in keys {
            let redigest = auth::digest(&secret, &key.salt)?;
            let digest = key.digest.as_slice();

            if digest == redigest {
                return Ok(());
            }
        }

        bail!("invalid api key")
    }

    async fn assert_valid_credentials(
        &self,
        ns: &NamespaceName,
        credential: Credential,
    ) -> Result<()> {
        match credential {
            Credential::ApiKeyV1(secret) => self.assert_valid_api_key(ns, secret).await,
            Credential::OwnerSignatureV1(_, _) => {
                // TODO
                Ok(())
            }
        }
    }

    pub async fn register_apikey(&mut self, cmd: RegisterApiKeyCmd) -> Result<()> {
        info!("registering apikey");

        self.assert_existing_namespace(&cmd.namespace).await?;

        self.assert_valid_credentials(&cmd.namespace, cmd.auth)
            .await?;

        let salt = b"somesaltforyou";
        let digest = auth::digest(&cmd.secret, salt)?;

        self.event_dispatch.submit_event(ApiKeyRegisteredV1 {
            namespace: cmd.namespace,
            digest,
            salt: salt.to_vec(),
        })?;

        Ok(())
    }

    async fn on_apikey_registered(&mut self, evt: ApiKeyRegisteredV1) -> Result<()> {
        info!("apikey registered");

        self.fabric_state
            .insert_api_key(&evt.namespace, &evt.digest, &evt.salt)
            .await?;

        Ok(())
    }

    pub async fn create_resource(&mut self, cmd: CreateResourceCmd) -> Result<CreateResourceAck> {
        info!("creating resource");

        self.assert_existing_namespace(&cmd.namespace).await?;

        self.assert_valid_credentials(&cmd.namespace, cmd.auth)
            .await?;

        // TODO: assert permissions

        // assert_resource_type_is_valid(cmd);
        // assert_resource_manifest_is_valid(cmd);
        // assert_resource_doesnt_exist(cmd);

        // define a new uuid for the resource
        let resource_uuid = uuid::Uuid::new_v4().into_bytes().to_vec();

        let event_receipt = self.event_dispatch.submit_event(ResourceCreatedV1 {
            metadata: ResourceMetadataV1 {
                namespace: cmd.namespace,
                kind: cmd.kind,
                name: cmd.name,
                uuid: resource_uuid.clone(),
            },
            manifest: cmd.spec,
        })?;

        let ack = CreateResourceAck {
            event_receipt,
            resource_uuid,
        };

        Ok(ack)
    }

    async fn on_resource_created(&mut self, evt: ResourceCreatedV1) -> Result<()> {
        info!("resource created");

        self.fabric_state
            .insert_resource(
                &evt.metadata.namespace,
                &evt.metadata.kind,
                &evt.metadata.uuid,
                &evt.metadata.name,
                &evt.manifest,
            )
            .await?;

        Ok(())
    }

    pub async fn list_resources(
        &self,
        query: ListResourcesQuery,
    ) -> Result<Vec<ListResourcesItem>> {
        self.assert_existing_namespace(&query.namespace_name)
            .await?;

        // assert_namespace_read_access(query);
        self.assert_valid_credentials(&query.namespace_name, query.auth)
            .await?;

        let items = self
            .fabric_state
            .list_resources(&query.namespace_name)
            .await?
            .into_iter()
            .map(|x| ListResourcesItem {
                metadata: ResourceMetadataV1 {
                    namespace: query.namespace_name.clone(),
                    kind: x.kind,
                    name: x.name,
                    uuid: x.uuid,
                },
                spec: vec![],
                status: vec![],
            })
            .collect();

        Ok(items)
    }

    pub async fn read_balance(&self, query: ReadBalanceQuery) -> Result<ReadBalanceOutput> {
        self.assert_existing_namespace(&query.namespace_name)
            .await?;

        // assert_namespace_balance_access(query);
        self.assert_valid_credentials(&query.namespace_name, query.auth)
            .await?;

        let accounts = self
            .fabric_state
            .read_balance(&query.namespace_name)
            .await?
            .into_iter()
            .map(|(a, b, c)| (a as u64, b as u64, c as u64))
            .collect();

        Ok(ReadBalanceOutput { accounts })
    }

    async fn on_resource_usage(&mut self, evt: ResourceUsageV1) -> Result<()> {
        info!("resource usage");

        self.fabric_state
            .insert_accounting(
                evt.epoch as i64,
                &evt.entry,
                &evt.cluster,
                &evt.namespace,
                Some(&evt.resource),
                vec![
                    AccountDelta {
                        account: 1,
                        debit: Some(evt.units as i64),
                        credit: None,
                    },
                    AccountDelta {
                        account: 2,
                        debit: None,
                        credit: Some(evt.units as i64),
                    },
                ],
            )
            .await?;

        Ok(())
    }

    async fn on_usage_payment(&mut self, evt: UsagePaymentV1) -> Result<()> {
        info!("usage payment");

        self.fabric_state
            .insert_accounting(
                evt.epoch as i64,
                &evt.entry,
                &evt.cluster,
                &evt.namespace,
                None,
                vec![
                    AccountDelta {
                        account: 2,
                        debit: Some(evt.units as i64),
                        credit: None,
                    },
                    AccountDelta {
                        account: 3,
                        debit: None,
                        credit: Some(evt.units as i64),
                    },
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn handle(&mut self, event: Event) -> Result<()> {
        info!(?event, "event recevied");

        match event {
            Event::NamespaceMintedV1(evt) => self.on_namespace_minted(evt).await,
            Event::ApiKeyRegisteredV1(evt) => self.on_apikey_registered(evt).await,
            Event::ResourceCreatedV1(evt) => self.on_resource_created(evt).await,
            Event::ResourceUsageV1(evt) => self.on_resource_usage(evt).await,
            Event::UsagePaymentV1(evt) => self.on_usage_payment(evt).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};
    use tokio::sync::Mutex;

    use crate::driven::event_dispatch::EventWrapper;

    use super::*;

    #[tokio::test]
    async fn happy_path() {
        tracing_subscriber::fmt::init();

        let fabric_state = FabricState::ephemeral().await.unwrap();
        let event_dispatch = EventDispatch::ephemeral(100);

        let mut domain = Domain {
            config: Config {
                cluster: b"123".into(),
            },
            fabric_state,
            event_dispatch,
        };

        let mut subscription = domain.event_dispatch.subscribe();

        let domain = Arc::new(Mutex::new(domain));

        let domain2 = domain.clone();
        let watcher = tokio::spawn(async move {
            while let Ok(EventWrapper(evt, _)) = subscription.recv().await {
                domain2.lock().await.handle(evt).await.unwrap();
            }
        });

        // extrinsic event
        domain
            .lock()
            .await
            .event_dispatch
            .submit_event(NamespaceMintedV1 {
                name: "ns1".into(),
                root_public_key: "123".into(),
            })
            .unwrap();

        tokio::time::sleep(Duration::from_secs(3)).await;

        domain
            .lock()
            .await
            .register_apikey(RegisterApiKeyCmd {
                auth: Credential::OwnerSignatureV1("123".into(), 1234),
                namespace: "ns1".into(),
                secret: b"mybadpassword".to_vec(),
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_secs(3)).await;

        let res_ack = domain
            .lock()
            .await
            .create_resource(CreateResourceCmd {
                auth: Credential::ApiKeyV1(b"mybadpassword".to_vec()),
                namespace: "ns1".into(),
                name: "res1".into(),
                kind: "workers.demeter.run/v1alpha1".into(),
                spec: b"abc".into(),
            })
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_secs(3)).await;

        // extrinsic event
        domain
            .lock()
            .await
            .event_dispatch
            .submit_event(ResourceUsageV1 {
                entry: b"1".into(),
                epoch: 123,
                namespace: "ns1".into(),
                resource: res_ack.resource_uuid,
                cluster: b"cluster1".into(),
                units: 500,
            })
            .unwrap();

        // extrinsic event
        domain
            .lock()
            .await
            .event_dispatch
            .submit_event(UsagePaymentV1 {
                entry: b"1".into(),
                epoch: 123,
                namespace: "ns1".into(),
                cluster: b"cluster1".into(),
                units: 400,
            })
            .unwrap();

        tokio::time::sleep(Duration::from_secs(3)).await;

        let balance = domain
            .lock()
            .await
            .read_balance(ReadBalanceQuery {
                auth: Credential::ApiKeyV1(b"mybadpassword".to_vec()),
                namespace_name: "ns1".into(),
            })
            .await
            .unwrap();

        dbg!(balance);

        watcher.abort();
    }
}
