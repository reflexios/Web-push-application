use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use rand::RngCore;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

const NONCE_LEN: usize = 12;

pub fn derive_key(secret: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(secret);
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest);
    key
}

#[allow(deprecated)]
fn cipher(key_bytes: &[u8; 32]) -> Aes256Gcm {
    Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key_bytes.as_slice()))
}

#[allow(deprecated)]
pub fn encrypt_private_key(key_bytes: &[u8; 32], plaintext: &str) -> Result<String> {
    let cipher = cipher(key_bytes);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow!("failed to encrypt vapid private key: {e}"))?;

    let mut payload = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    payload.extend_from_slice(&nonce_bytes);
    payload.extend_from_slice(&ciphertext);

    Ok(STANDARD.encode(payload))
}

#[allow(deprecated)]
pub fn decrypt_private_key(key_bytes: &[u8; 32], stored: &str) -> Result<String> {
    let cipher = cipher(key_bytes);

    let payload = STANDARD
        .decode(stored)
        .context("vapid_private_key is not valid base64")?;

    if payload.len() < NONCE_LEN {
        return Err(anyhow!("stored vapid_private_key payload is too short"));
    }

    let (nonce_bytes, ciphertext) = payload.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!("failed to decrypt vapid private key: {e}"))?;

    String::from_utf8(plaintext).context("decrypted vapid private key is not valid UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_then_decrypt_roundtrip() {
        let key = derive_key(b"some-test-secret");
        let plaintext = "-----BEGIN PRIVATE KEY-----\nabc123\n-----END PRIVATE KEY-----\n";

        let encrypted = encrypt_private_key(&key, plaintext).expect("encrypt");
        assert_ne!(encrypted, plaintext);

        let decrypted = decrypt_private_key(&key, &encrypted).expect("decrypt");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn wrong_key_fails_to_decrypt() {
        let key = derive_key(b"secret-a");
        let other_key = derive_key(b"secret-b");
        let encrypted = encrypt_private_key(&key, "top-secret").expect("encrypt");

        assert!(decrypt_private_key(&other_key, &encrypted).is_err());
    }
}
