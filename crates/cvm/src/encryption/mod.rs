//! Encryption and gift wrapping for ContextVM
//!
//! This module provides wrappers around nostr-sdk's NIP-44 and NIP-59 functionality.
//! The actual gift wrapping is done via the Client, not directly here.

use crate::core::error::{Error, Result};
use nostr_sdk::prelude::*;

/// Encrypt a message using NIP-44
pub async fn encrypt_nip44<T>(
    signer: &T,
    receiver_pubkey: &PublicKey,
    plaintext: &str,
) -> Result<String>
where
    T: NostrSigner,
{
    signer
        .nip44_encrypt(receiver_pubkey, plaintext)
        .await
        .map_err(|e| Error::Encryption(e.to_string()))
}

/// Decrypt a message using NIP-44
pub async fn decrypt_nip44<T>(
    signer: &T,
    sender_pubkey: &PublicKey,
    ciphertext: &str,
) -> Result<String>
where
    T: NostrSigner,
{
    signer
        .nip44_decrypt(sender_pubkey, ciphertext)
        .await
        .map_err(|e| Error::Decryption(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nip44_encryption() {
        let keys1 = Keys::generate();
        let keys2 = Keys::generate();

        let plaintext = "Hello, Nostr!";

        let ciphertext = encrypt_nip44(&keys1, &keys2.public_key(), plaintext)
            .await
            .unwrap();

        let decrypted = decrypt_nip44(&keys2, &keys1.public_key(), &ciphertext)
            .await
            .unwrap();

        assert_eq!(plaintext, decrypted);
    }
}
