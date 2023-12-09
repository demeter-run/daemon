use anyhow::{anyhow, Context, Result};
use argon2::Argon2;

use super::HashDigest;

pub fn digest(pwd: &[u8], salt: &[u8]) -> Result<HashDigest> {
    let argon2 = Argon2::default();

    let mut digest = [0u8; 32];
    argon2
        .hash_password_into(pwd, salt, &mut digest)
        .map_err(|err| anyhow!(err.to_string()))
        .context("error computing hash digest")?;

    Ok(digest)
}
