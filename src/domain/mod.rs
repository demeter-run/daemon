use anyhow::{bail, Result};

use crate::driven::event_dispatch::EventDispatch;
use crate::driven::fabric_state::FabricState;

mod auth;
mod events;

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
pub type NamespaceName = String;

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
    pub namespace: NamespaceName,
    pub resource_type: String,
    pub resource_spec: String,
}

pub struct ListResourcesQuery {
    pub auth: Credential,
    pub resource_name: String,
    pub namespace_name: String,
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
        self.assert_existing_namespace(&cmd.namespace).await?;

        self.assert_valid_credentials(&cmd.namespace, cmd.auth)
            .await?;

        let salt = b"somesaltforyou";
        let digest = auth::digest(&cmd.secret, salt)?;

        self.event_dispatch
            .submit_event(ApiKeyRegisteredV1 {
                namespace: cmd.namespace,
                digest,
                salt: salt.to_vec(),
            })
            .await?;

        Ok(())
    }

    pub async fn on_apikey_registered(&mut self, evt: ApiKeyRegisteredV1) -> Result<()> {
        self.fabric_state
            .insert_api_key(&evt.namespace, &evt.digest, &evt.salt)
            .await?;

        Ok(())
    }

    pub async fn create_resource(&mut self, cmd: CreateResourceCmd) -> Result<()> {
        self.assert_existing_namespace(&cmd.namespace).await?;

        self.assert_valid_credentials(&cmd.namespace, cmd.auth)
            .await?;

        // TODO: assert permissions

        // assert_resource_type_is_valid(cmd);
        // assert_resource_manifest_is_valid(cmd);
        // assert_resource_doesnt_exist(cmd);

        // dispatch_resource_created_event(cmd);
        self.event_dispatch
            .submit_event(ResourceCreatedV1 {})
            .await?;

        Ok(())
    }

    pub fn list_resources(query: ListResourcesQuery) {
        // assert_namespace_is_valid(query);
        // assert_namespace_read_access(query);
        // fetch_resources_by_namespace(query);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn happy_path() {
        let fabric_state = FabricState::ephemeral().await.unwrap();
        let event_dispatch = EventDispatch::ephemeral();

        let mut domain = Domain {
            fabric_state,
            event_dispatch,
        };

        domain
            .on_namespace_minted(NamespaceMintedV1 {
                name: "ns1".into(),
                root_public_key: "123".into(),
            })
            .await
            .unwrap();

        domain
            .register_apikey(RegisterApiKeyCmd {
                auth: Credential::OwnerSignatureV1("123".into(), 1234),
                namespace: "ns1".into(),
                secret: b"mybadpassword".to_vec(),
            })
            .await
            .unwrap();

        while let Some(evt) = domain.event_dispatch.entries.pop_front() {
            match evt {
                Event::ApiKeyRegisteredV1(evt) => domain.on_apikey_registered(evt).await.unwrap(),
                Event::NamespaceMintedV1(_) => todo!(),
                Event::ResourceCreatedV1(_) => todo!(),
            }
        }

        domain
            .create_resource(CreateResourceCmd {
                auth: Credential::ApiKeyV1(b"mybadpassword".to_vec()),
                namespace: "ns1".into(),
                resource_type: "workers.demeter.run/v1Allpha1".into(),
                resource_spec: "{}".into(),
            })
            .await
            .unwrap();

        let Domain { event_dispatch, .. } = domain;

        dbg!(event_dispatch.entries);
    }
}
