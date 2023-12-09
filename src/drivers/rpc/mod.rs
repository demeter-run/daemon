use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;
use tonic::{codegen::InterceptedService, transport::Server};

mod auth;
mod ops;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    listen_address: String,
    auth: auth_v3::Config,
}

#[derive(Error, Debug)]
enum Error {
    #[error("server error {0}")]
    Server(String),
}

impl Error {
    fn server(err: impl Display) -> Self {
        Self::Server(err.to_string())
    }
}

async fn serve(config: Config) -> Result<(), Error> {
    let addr = config.listen_address.parse().unwrap();

    let mut server = Server::builder();

    let service = InterceptedService::new(
        ops::OpsServiceImpl::new(),
        auth_v3::build_interceptor(&config.auth),
    );

    let server = server.add_service(service);

    server.serve(addr).await.map_err(Error::server)?;

    Ok(())
}
