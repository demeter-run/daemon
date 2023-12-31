use anyhow::{bail, Result};
use tracing::info;

use crate::driven::event_dispatch::EventDispatch;
use crate::driven::fabric_state::FabricState;

mod auth;
mod events;

pub use auth::*;
pub use events::*;

pub struct Domain {
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

impl Domain {
    async fn assert_available_namespace(&self, ns: &NamespaceName) -> Result<()> {
        let exists = self.fabric_state.namespace_exists(&ns).await?;

        if exists {
            bail!("namespace isn't available")
        }

        Ok(())
    }

    pub async fn on_namespace_minted(&self, evt: NamespaceMintedV1) -> Result<()> {
        info!("namespace minted");

        // TODO: how do we handle business invariants? eg, if the namespace isn't
        // available, then something in inconsistent at a global scale.
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

    pub async fn assert_valid_credentials(
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

    pub async fn on_apikey_registered(&mut self, evt: ApiKeyRegisteredV1) -> Result<()> {
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

    pub async fn on_resource_created(&mut self, evt: ResourceCreatedV1) -> Result<()> {
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
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::sync::{broadcast::Receiver, Mutex};

    use crate::driven::event_dispatch::EventWrapper;

    use super::*;

    async fn basic_event_watch_loop(
        mut subscription: Receiver<EventWrapper>,
        domain: Arc<Mutex<Domain>>,
    ) {
        while let Ok(EventWrapper(evt, _)) = subscription.recv().await {
            let mut domain = domain.lock().await;

            match evt {
                Event::ApiKeyRegisteredV1(evt) => domain.on_apikey_registered(evt).await.unwrap(),
                Event::ResourceCreatedV1(evt) => domain.on_resource_created(evt).await.unwrap(),
                Event::NamespaceMintedV1(_) => todo!(),
            }
        }
    }

    #[tokio::test]
    async fn happy_path() {
        let fabric_state = FabricState::ephemeral().await.unwrap();
        let event_dispatch = EventDispatch::ephemeral(100);

        let mut domain = Domain {
            fabric_state,
            event_dispatch,
        };

        let subscription = domain.event_dispatch.subscribe();

        let domain = Arc::new(Mutex::new(domain));

        let domain2 = domain.clone();
        let watcher = tokio::spawn(async move { basic_event_watch_loop(subscription, domain2) });

        domain
            .lock()
            .await
            .on_namespace_minted(NamespaceMintedV1 {
                name: "ns1".into(),
                root_public_key: "123".into(),
            })
            .await
            .unwrap();

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

        domain
            .lock()
            .await
            .create_resource(CreateResourceCmd {
                auth: Credential::ApiKeyV1(b"mybadpassword".to_vec()),
                namespace: "ns1".into(),
                name: "res1".into(),
                kind: "workers.demeter.run/v1Allpha1".into(),
                spec: b"abc".into(),
            })
            .await
            .unwrap();

        watcher.abort();
    }
}
