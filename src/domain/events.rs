use super::{HashDigest, HashSalt};

macro_rules! into_event {
    ($name:ident) => {
        impl Into<Event> for $name {
            fn into(self) -> Event {
                Event::$name(self)
            }
        }
    };
}

pub type NamespaceName = String;
pub type DCU = u64;
pub type Epoch = u64;
pub type ResourceUuid = Blob;
pub type ClusterUuid = Blob;

#[derive(Debug, Clone)]
pub struct ResourceMetadataV1 {
    pub namespace: NamespaceName,
    pub kind: String,
    pub name: String,
    pub uuid: ResourceUuid,
}

pub type Blob = Vec<u8>;

#[derive(Debug, Clone)]
pub struct NamespaceMintedV1 {
    pub name: String,
    pub root_public_key: Blob,
}

into_event!(NamespaceMintedV1);

#[derive(Debug, Clone)]
pub struct ApiKeyRegisteredV1 {
    pub namespace: String,
    pub digest: HashDigest,
    pub salt: HashSalt,
}

into_event!(ApiKeyRegisteredV1);

#[derive(Debug, Clone)]
pub struct ResourceCreatedV1 {
    pub metadata: ResourceMetadataV1,
    pub manifest: Vec<u8>,
}

into_event!(ResourceCreatedV1);

#[derive(Debug, Clone)]
pub struct ResourceUsageV1 {
    pub entry: Blob,
    pub epoch: Epoch,
    pub namespace: NamespaceName,
    pub resource: ResourceUuid,
    pub cluster: ClusterUuid,
    pub units: DCU,
}

into_event!(ResourceUsageV1);

#[derive(Debug, Clone)]
pub struct UsagePaymentV1 {
    pub entry: Blob,
    pub epoch: Epoch,
    pub namespace: NamespaceName,
    pub cluster: ClusterUuid,
    pub units: DCU,
}

into_event!(UsagePaymentV1);

#[derive(Debug, Clone)]
pub enum Event {
    NamespaceMintedV1(NamespaceMintedV1),
    ApiKeyRegisteredV1(ApiKeyRegisteredV1),
    ResourceCreatedV1(ResourceCreatedV1),
    ResourceUsageV1(ResourceUsageV1),
    UsagePaymentV1(UsagePaymentV1),
}
