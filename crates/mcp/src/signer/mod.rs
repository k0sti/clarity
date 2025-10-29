//! Nostr signer wrapper

pub use nostr_sdk::prelude::{Keys, NostrSigner, PublicKey};

/// Create a new signer from a private key (hex string or nsec)
pub fn from_sk(sk: &str) -> Result<Keys, nostr_sdk::key::Error> {
    Keys::parse(sk)
}

/// Generate a new random signer
pub fn generate() -> Keys {
    Keys::generate()
}
