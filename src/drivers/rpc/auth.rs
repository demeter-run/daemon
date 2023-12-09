use dmtrd::domain::fabric_inbox::AuthCredential;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Authenticator {}

fn extract_required_metadata_string(
    request: &tonic::Request<()>,
    key: &str,
) -> Result<String, tonic::Status> {
    request
        .metadata()
        .get(key)
        .ok_or_else(|| tonic::Status::unauthenticated("missing required auth value"))?
        .to_str()
        .map(|x| x.to_owned())
        .map_err(|_| tonic::Status::unauthenticated("malformed auth value"))
}

impl tonic::service::Interceptor for Authenticator {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        let ns = extract_required_metadata_string(&request, "x-dmtr-ns")?;
        let token = extract_required_metadata_string(&request, "x-dmtr-token")?;

        let creds = AuthCredential::ApiKeyV1(ns, token);

        request.extensions_mut().insert(creds);

        Ok(request)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {}

pub fn build_interceptor(config: &Config) -> Authenticator {
    Authenticator {}
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn build_token() {}
}
