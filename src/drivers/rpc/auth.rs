use tonic::Status;
use tracing::warn;

use crate::domain;

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
        let token = extract_required_metadata_string(&request, "x-dmtr-apikey")?;

        let (hrp, token, _) = bech32::decode(&token).map_err(|error| {
            warn!(?error, "invalid bech32");
            tonic::Status::permission_denied("invalid apikey")
        })?;

        if hrp != "dmtr_apikey" {
            return Err(Status::permission_denied("invalid apikey"));
        }

        let token = bech32::convert_bits(&token, 5, 8, true).map_err(|error| {
            warn!(?error, "invalid bech32");
            tonic::Status::permission_denied("invalid apikey")
        })?;

        let creds = domain::Credential::ApiKeyV1(token);

        request.extensions_mut().insert(creds);

        Ok(request)
    }
}

pub fn build_interceptor() -> Authenticator {
    Authenticator {}
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn build_token() {}
}
