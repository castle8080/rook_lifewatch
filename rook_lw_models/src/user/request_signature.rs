use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RequestSignature {
    /// ID of the user making the request
    pub user_id: String,
    
    /// When the request was created (UTC timestamp)
    pub created_at: DateTime<Utc>,
    
    /// Random salt bytes (for replay attack prevention)
    pub salt: Vec<u8>,
    
    /// The computed HMAC signature bytes
    pub signature: Vec<u8>,
}

impl RequestSignature {
    /// Encode the signature to base64url format (URL-safe, no padding)
    pub fn to_base64url(&self) -> Result<String, serde_json::Error> {
        let json = serde_json::to_vec(self)?;
        Ok(URL_SAFE_NO_PAD.encode(&json))
    }
    
    /// Decode from base64url format
    pub fn from_base64url(encoded: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let decoded = URL_SAFE_NO_PAD.decode(encoded)?;
        let sig = serde_json::from_slice(&decoded)?;
        Ok(sig)
    }
    
    /// Check if the signature has expired (5 minute window)
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.created_at);
        age.num_seconds() > 300 // 5 minutes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_to_base64url_and_from_base64url() {
        let sig = RequestSignature {
            user_id: "test-user-123".to_string(),
            created_at: Utc::now(),
            salt: vec![1, 2, 3, 4, 5, 6, 7, 8],
            signature: vec![10, 20, 30, 40, 50, 60, 70, 80],
        };

        // Encode to base64url
        let encoded = sig.to_base64url().expect("Failed to encode");
        
        // Should not contain padding characters
        assert!(!encoded.contains('='));
        
        // Decode back
        let decoded = RequestSignature::from_base64url(&encoded).expect("Failed to decode");
        
        // Should match original
        assert_eq!(sig.user_id, decoded.user_id);
        assert_eq!(sig.salt, decoded.salt);
        assert_eq!(sig.signature, decoded.signature);
        // Times should match (allowing for serialization precision)
        assert_eq!(sig.created_at.timestamp(), decoded.created_at.timestamp());
    }

    #[test]
    fn test_roundtrip_with_special_characters() {
        let sig = RequestSignature {
            user_id: "user+with/special=chars".to_string(),
            created_at: Utc::now(),
            salt: vec![255, 254, 253, 0, 1, 2],
            signature: vec![128, 127, 126, 125, 124, 123],
        };

        let encoded = sig.to_base64url().expect("Failed to encode");
        let decoded = RequestSignature::from_base64url(&encoded).expect("Failed to decode");
        
        assert_eq!(sig.user_id, decoded.user_id);
        assert_eq!(sig.salt, decoded.salt);
        assert_eq!(sig.signature, decoded.signature);
    }

    #[test]
    fn test_url_safe_encoding() {
        let sig = RequestSignature {
            user_id: "test".to_string(),
            created_at: Utc::now(),
            salt: vec![0; 16],
            signature: vec![255; 32],
        };

        let encoded = sig.to_base64url().expect("Failed to encode");
        
        // URL-safe base64 should not contain + or /
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
        
        // Should use - and _ instead
        // (may or may not be present depending on the data, but shouldn't cause errors)
        let decoded = RequestSignature::from_base64url(&encoded).expect("Failed to decode");
        assert_eq!(sig.signature, decoded.signature);
    }

    #[test]
    fn test_is_expired() {
        use chrono::Duration;
        
        // Create a signature from 6 minutes ago (expired)
        let expired_sig = RequestSignature {
            user_id: "test".to_string(),
            created_at: Utc::now() - Duration::minutes(6),
            salt: vec![1, 2, 3],
            signature: vec![4, 5, 6],
        };
        assert!(expired_sig.is_expired());
        
        // Create a signature from 4 minutes ago (not expired)
        let valid_sig = RequestSignature {
            user_id: "test".to_string(),
            created_at: Utc::now() - Duration::minutes(4),
            salt: vec![1, 2, 3],
            signature: vec![4, 5, 6],
        };
        assert!(!valid_sig.is_expired());
        
        // Create a signature from right now (not expired)
        let fresh_sig = RequestSignature {
            user_id: "test".to_string(),
            created_at: Utc::now(),
            salt: vec![1, 2, 3],
            signature: vec![4, 5, 6],
        };
        assert!(!fresh_sig.is_expired());
    }

    #[test]
    fn test_from_base64url_invalid_input() {
        // Test with invalid base64
        let result = RequestSignature::from_base64url("not valid base64!!!");
        assert!(result.is_err());
        
        // Test with valid base64 but invalid JSON
        let result = RequestSignature::from_base64url("aGVsbG8gd29ybGQ");
        assert!(result.is_err());
    }
}
