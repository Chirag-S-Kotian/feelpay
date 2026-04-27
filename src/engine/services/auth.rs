use crate::error::FeelPayError;
use totp_rs::{Algorithm, Secret, TOTP};

pub struct MfaService;

impl MfaService {
    /// Generates a new random TOTP secret in Base32 encoding
    pub fn generate_secret() -> String {
        use rand::RngCore;
        let mut bytes = [0u8; 20];
        rand::thread_rng().fill_bytes(&mut bytes);
        Secret::Raw(bytes.to_vec()).to_encoded().to_string()
    }

    /// Generates a QR code URL for the given user email and secret
    pub fn generate_qr_code(email: &str, secret: &str) -> Result<String, FeelPayError> {
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret.to_string())
                .to_bytes()
                .map_err(|e| FeelPayError::Internal(format!("Secret error: {}", e)))?,
            Some("FeelPay".to_string()),
            email.to_string(),
        )
        .map_err(|e| FeelPayError::Internal(format!("TOTP init error: {}", e)))?;

        Ok(totp.get_url())
    }

    /// Verifies a TOTP token against a secret
    pub fn verify_token(secret: &str, token: &str) -> Result<bool, FeelPayError> {
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret.to_string())
                .to_bytes()
                .map_err(|e| FeelPayError::Internal(format!("Secret error: {}", e)))?,
            Some("FeelPay".to_string()),
            "".to_string(), // Account name not needed for verification
        )
        .map_err(|e| FeelPayError::Internal(format!("TOTP init error: {}", e)))?;

        totp.check_current(token)
            .map_err(|e| FeelPayError::Internal(format!("TOTP verification error: {}", e)))
    }
}
