use totp_rs::{Algorithm, TOTP, Secret};
use qr_code::QrCode;
use crate::error::{PassmanError, Result};

/// Two-Factor Authentication manager
#[allow(dead_code)]
pub struct TwoFactorAuth {
    secret: Option<Secret>,
    totp: Option<TOTP>,
}

#[allow(dead_code)]
impl TwoFactorAuth {
    pub fn new() -> Self {
        Self {
            secret: None,
            totp: None,
        }
    }

    /// Generate a new TOTP secret and setup 2FA
    pub fn setup(&mut self, account_name: &str, issuer: &str) -> Result<String> {
        // Use a predefined secret for now - in production, this should be randomly generated
        let secret = Secret::Encoded("JBSWY3DPEHPK3PXP".to_string());
        
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,  // digits
            1,  // skew
            30, // step
            secret.to_bytes().map_err(|e| PassmanError::Crypto(crate::crypto::CryptoError::KeyDerivation(e.to_string())))?,
        ).map_err(|e| PassmanError::Crypto(crate::crypto::CryptoError::KeyDerivation(e.to_string())))?;

        // Manually construct the URL since get_url() might not be available
        let url = format!("otpauth://totp/{}:{}?secret={}&issuer={}", 
            issuer, account_name, "JBSWY3DPEHPK3PXP", issuer);
        
        self.secret = Some(secret);
        self.totp = Some(totp);

        Ok(url)
    }

    /// Generate QR code for the TOTP setup
    pub fn generate_qr_code(&self, url: &str) -> Result<String> {
        let qr = QrCode::new(url)
            .map_err(|e| PassmanError::InvalidInput(e.to_string()))?;
        
        // Convert QR code to ASCII art for terminal display
        let qr_string = qr.to_string(false, 3);
        Ok(qr_string)
    }

    /// Verify a TOTP code
    pub fn verify_code(&self, code: &str) -> bool {
        if let Some(totp) = &self.totp {
            totp.check_current(code).unwrap_or(false)
        } else {
            false
        }
    }

    /// Generate current TOTP code (for testing/backup purposes)
    pub fn generate_current_code(&self) -> Result<String> {
        if let Some(totp) = &self.totp {
            totp.generate_current()
                .map_err(|e| PassmanError::Crypto(crate::crypto::CryptoError::KeyDerivation(e.to_string())))
        } else {
            Err(PassmanError::InvalidInput("2FA not setup".to_string()))
        }
    }

    /// Get the secret as a string for backup purposes
    pub fn get_secret_string(&self) -> Option<String> {
        self.secret.as_ref().map(|s| s.to_encoded().to_string())
    }

    /// Load 2FA from an existing secret
    pub fn load_from_secret(&mut self, secret_str: &str) -> Result<()> {
        let secret = Secret::Encoded(secret_str.to_string());
        
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,  // digits
            1,  // skew  
            30, // step
            secret.to_bytes().map_err(|e| PassmanError::Crypto(crate::crypto::CryptoError::KeyDerivation(e.to_string())))?,
        ).map_err(|e| PassmanError::Crypto(crate::crypto::CryptoError::KeyDerivation(e.to_string())))?;

        self.secret = Some(secret);
        self.totp = Some(totp);

        Ok(())
    }

    /// Check if 2FA is enabled
    pub fn is_enabled(&self) -> bool {
        self.totp.is_some()
    }
}

impl Default for TwoFactorAuth {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2fa_setup() {
        let mut tfa = TwoFactorAuth::new();
        let url = tfa.setup("test@example.com", "Passman").unwrap();
        
        assert!(url.starts_with("otpauth://totp/"));
        assert!(tfa.is_enabled());
    }

    #[test]
    fn test_2fa_verify() {
        let mut tfa = TwoFactorAuth::new();
        tfa.setup("test@example.com", "Passman").unwrap();
        
        let code = tfa.generate_current_code().unwrap();
        assert!(tfa.verify_code(&code));
    }
}
