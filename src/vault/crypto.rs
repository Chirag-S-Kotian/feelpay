use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use zeroize::Zeroize;

use crate::error::FeelPayError;

/// Low-level AES-256-GCM encrypt/decrypt for PCI vault scope.
pub struct VaultCrypto {
    cipher: Aes256Gcm,
    pub key_id: String,
}

impl VaultCrypto {
    pub fn new(encryption_key_hex: &str, key_id: String) -> Result<Self, FeelPayError> {
        let key_bytes = hex::decode(encryption_key_hex).map_err(|_| {
            FeelPayError::Internal("Invalid vault encryption key format (not hex)".into())
        })?;
        if key_bytes.len() != 32 {
            return Err(FeelPayError::Internal(
                "Vault encryption key must be 32 bytes (AES-256)".into(),
            ));
        }
        let key = *Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(&key);
        Ok(Self { cipher, key_id })
    }

    /// Encrypt plaintext → (ciphertext_b64, nonce_b64). Zeroizes input.
    pub fn encrypt(&self, mut plaintext: String) -> Result<(String, String), FeelPayError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| FeelPayError::Internal("AES encrypt failed".into()))?;
        plaintext.zeroize();
        Ok((B64.encode(ciphertext), B64.encode(nonce)))
    }

    /// Decrypt (ciphertext_b64, nonce_b64) → plaintext
    pub fn decrypt(&self, ciphertext_b64: &str, nonce_b64: &str) -> Result<String, FeelPayError> {
        let ciphertext = B64
            .decode(ciphertext_b64)
            .map_err(|_| FeelPayError::Internal("Corrupt encrypted payload".into()))?;
        let nonce_bytes = B64
            .decode(nonce_b64)
            .map_err(|_| FeelPayError::Internal("Corrupt nonce payload".into()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext_bytes = self
            .cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| FeelPayError::Internal("AES decrypt failed".into()))?;
        String::from_utf8(plaintext_bytes)
            .map_err(|_| FeelPayError::Internal("Decrypted data is not valid UTF-8".into()))
    }

    /// Derive a deterministic HMAC fingerprint for duplicate-PAN detection (future use).
    pub fn fingerprint(pan: &str) -> String {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(pan.as_bytes());
        hex::encode(hash)
    }
}
