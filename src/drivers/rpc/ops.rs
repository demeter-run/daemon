use demeter_core_spec::demeter::core::v1alpha::*;
use tonic::async_trait;

pub struct OpsServiceImpl;

impl OpsServiceImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ops_service_server::OpsService for OpsServiceImpl {
    async fn create_resource(
        &self,
        request: tonic::Request<CreateResourceRequest>,
    ) -> Result<tonic::Response<CreateResourceResponse>, tonic::Status> {
    }

    async fn list_resources(
        &self,
        request: tonic::Request<ListResourcesRequest>,
    ) -> Result<tonic::Response<ListResourcesResponse>, tonic::Status> {
        todo!()
    }

    async fn read_resource(
        &self,
        request: tonic::Request<ReadResourceRequest>,
    ) -> Result<tonic::Response<ReadResourceResponse>, tonic::Status> {
        todo!()
    }

    async fn patch_resource(
        &self,
        request: tonic::Request<PatchResourceRequest>,
    ) -> Result<tonic::Response<PatchResourceResponse>, tonic::Status> {
        todo!()
    }

    async fn delete_resource(
        &self,
        request: tonic::Request<DeleteResourceRequest>,
    ) -> Result<tonic::Response<DeleteResourceResponse>, tonic::Status> {
        todo!()
    }
}
