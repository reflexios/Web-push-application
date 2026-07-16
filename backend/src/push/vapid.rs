use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use p256::EncodedPoint;
use p256::ecdsa::SigningKey;
use p256::pkcs8::{EncodePrivateKey, LineEnding};
use rand_core::OsRng;

#[derive(Debug, Clone)]
pub struct VapidKeyPair {
    pub private_pem: String,
    pub public_base64: String,
}

pub fn generate_vapid() -> Result<VapidKeyPair> {
    let signing_key = SigningKey::random(&mut OsRng);

    let private_pem = signing_key
        .to_pkcs8_pem(LineEnding::LF)
        .context("failed to serialize private key to PKCS#8 PEM")?
        .to_string();

    let encoded_point = EncodedPoint::from(signing_key.verifying_key());
    let public_bytes = encoded_point.as_bytes();

    let public_base64 = URL_SAFE_NO_PAD.encode(public_bytes);

    Ok(VapidKeyPair {
        private_pem,
        public_base64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_keys_have_correct_shape() {
        let pair = generate_vapid().expect("generate keys");
        assert!(
            pair.public_base64.len() >= 80 && pair.public_base64.len() <= 90,
            "unexpected base64 length: {}",
            pair.public_base64.len()
        );
        assert!(pair.private_pem.contains("BEGIN PRIVATE KEY"));
    }
}
