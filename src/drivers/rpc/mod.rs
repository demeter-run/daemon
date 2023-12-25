use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Server;

use dmtri::demeter::ops::v1alpha::ops_service_server::OpsServiceServer;

use crate::domain::Domain;

pub mod auth;
mod ops;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub listen_address: String,
}

pub async fn serve(config: Config, domain: Arc<Mutex<Domain>>) -> Result<()> {
    let addr = config.listen_address.parse().unwrap();

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(dmtri::demeter::ops::v1alpha::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(protoc_wkt::google::protobuf::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let inner = ops::OpsServiceImpl::new(domain);
    let auth = auth::build_interceptor();

    let server = OpsServiceServer::with_interceptor(inner, auth);

    Server::builder()
        .add_service(server)
        .add_service(reflection)
        .serve(addr)
        .await
        .context("running grpc server")?;

    Ok(())
}
