use anyhow::{Context, Result};
use dmtri::demeter::ops::v1alpha::ops_service_server::OpsServiceServer;
use serde::{Deserialize, Serialize};
use tonic::transport::Server;

pub mod auth;
mod ops;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub listen_address: String,
}

pub async fn serve(config: Config) -> Result<()> {
    let addr = config.listen_address.parse().unwrap();
    let inner = ops::OpsServiceImpl::new();
    let auth = auth::build_interceptor();

    let server = OpsServiceServer::with_interceptor(inner, auth);

    Server::builder()
        .add_service(server)
        .serve(addr)
        .await
        .context("running grpc server")?;

    Ok(())
}
