use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{async_trait, Status};

use crate::domain;
use dmtri::demeter::ops::v1alpha as proto;

pub struct OpsServiceImpl {
    domain: Arc<Mutex<domain::Domain>>,
}

impl OpsServiceImpl {
    pub fn new(domain: Arc<Mutex<domain::Domain>>) -> Self {
        Self { domain }
    }
}

#[async_trait]
impl proto::ops_service_server::OpsService for OpsServiceImpl {
    async fn create_resource(
        &self,
        request: tonic::Request<proto::CreateResourceRequest>,
    ) -> Result<tonic::Response<proto::CreateResourceResponse>, tonic::Status> {
        let credential = request.extensions().get::<domain::Credential>();

        let credential = match credential {
            None => return Err(Status::permission_denied("invalid credential")),
            Some(x) => x.clone(),
        };

        let req = request.into_inner();

        let mut domain = self.domain.lock().await;

        let ack = domain
            .create_resource(domain::CreateResourceCmd {
                auth: credential,
                metadata: req
                    .metadata
                    .map(|proto| domain::ResourceMetadata {
                        namespace: proto.namespace,
                        name: proto.name,
                    })
                    .ok_or(Status::invalid_argument("missing metadata"))?,
                spec: req
                    .spec
                    .map(|x| domain::AnyResource {
                        kind: x.type_url,
                        manifest: x.value.into(),
                    })
                    .ok_or(Status::invalid_argument("missing resource"))?,
            })
            .await
            .map_err(|err| Status::unknown(err.to_string()))?;

        let res = proto::CreateResourceResponse {
            event_receipt: ack.event_receipt.into(),
            resource_uuid: ack.resource_uuid.into(),
        };

        Ok(tonic::Response::new(res))
    }

    async fn list_resources(
        &self,
        request: tonic::Request<proto::ListResourcesRequest>,
    ) -> Result<tonic::Response<proto::ListResourcesResponse>, tonic::Status> {
        todo!()
    }

    async fn read_resource(
        &self,
        request: tonic::Request<proto::ReadResourceRequest>,
    ) -> Result<tonic::Response<proto::ReadResourceResponse>, tonic::Status> {
        todo!()
    }

    async fn patch_resource(
        &self,
        request: tonic::Request<proto::PatchResourceRequest>,
    ) -> Result<tonic::Response<proto::PatchResourceResponse>, tonic::Status> {
        todo!()
    }

    async fn delete_resource(
        &self,
        request: tonic::Request<proto::DeleteResourceRequest>,
    ) -> Result<tonic::Response<proto::DeleteResourceResponse>, tonic::Status> {
        todo!()
    }
}
