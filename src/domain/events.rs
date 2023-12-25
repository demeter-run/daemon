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

#[derive(Debug, Clone)]
pub struct NamespaceMintedV1 {
    pub name: String,
    pub root_public_key: String,
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
    pub resource_uuid: Vec<u8>,
}

into_event!(ResourceCreatedV1);

#[derive(Debug, Clone)]
pub enum Event {
    NamespaceMintedV1(NamespaceMintedV1),
    ApiKeyRegisteredV1(ApiKeyRegisteredV1),
    ResourceCreatedV1(ResourceCreatedV1),
}
