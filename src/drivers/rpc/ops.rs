use k8s_openapi::http::request;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{async_trait, Status};

use crate::domain;
use dmtri::demeter::ops::v1alpha::{self as proto, ListResourcesResponse};

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

        let proto_meta = req
            .metadata
            .ok_or(Status::invalid_argument("missing metadata"))?;

        let proto_spec = req.spec.ok_or(Status::invalid_argument("missing spec"))?;

        let ack = domain
            .create_resource(domain::CreateResourceCmd {
                auth: credential,
                namespace: proto_meta.namespace,
                name: proto_meta.name,
                kind: proto_spec.type_url,
                spec: proto_spec.value.into(),
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
        let credential = request.extensions().get::<domain::Credential>();

        let credential = match credential {
            None => return Err(Status::permission_denied("invalid credential")),
            Some(x) => x.clone(),
        };

        let domain = self.domain.lock().await;

        let items = domain
            .list_resources(domain::ListResourcesQuery {
                auth: credential,
                namespace_name: request.into_inner().namespace,
                resource_name: "".into(),
            })
            .await
            .map_err(|err| Status::unknown(err.to_string()))?
            .into_iter()
            .map(|x| proto::Resource {
                metadata: Some(proto::ResourceMetadata {
                    namespace: x.metadata.namespace,
                    name: x.metadata.name,
                }),
                spec: None,
                status: None,
            })
            .collect();

        let res = proto::ListResourcesResponse { items };

        Ok(tonic::Response::new(res))
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
