use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// Physical substrate verification results
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubstrateState {
    pub http_status: Option<u16>,
    pub http_headers: std::collections::HashMap<String, String>,
    pub visibility_label: Option<String>,
    pub dns_resolution: Option<IpAddr>,
    pub tls_valid: bool,
    pub tls_cert_subject: Option<String>,
    pub is_internal_network: bool,
    pub content_hash: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub signature: String,
}

impl SubstrateState {
    pub fn new() -> Self {
        Self {
            http_status: None,
            http_headers: std::collections::HashMap::new(),
            visibility_label: None,
            dns_resolution: None,
            tls_valid: false,
            tls_cert_subject: None,
            is_internal_network: false,
            content_hash: None,
            timestamp: chrono::Utc::now(),
            signature: String::new(),
        }
    }

    pub fn with_http_status(mut self, status: u16) -> Self {
        self.http_status = Some(status);
        self
    }

    pub fn with_visibility(mut self, label: String) -> Self {
        self.visibility_label = Some(label);
        self
    }

    pub fn with_dns_resolution(mut self, ip: IpAddr) -> Self {
        self.dns_resolution = Some(ip);
        self
    }

    pub fn with_internal_network(mut self, is_internal: bool) -> Self {
        self.is_internal_network = is_internal;
        self
    }

    pub fn sign(&mut self) {
        let data = format!(
            "{}:{}:{}",
            self.http_status.unwrap_or(0),
            self.visibility_label.as_deref().unwrap_or("unknown"),
            self.timestamp.to_rfc3339()
        );
        self.signature = hash_data(&data);
    }
}

impl Default for SubstrateState {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of substrate verification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubstrateVerification {
    pub state: SubstrateState,
    pub verified: bool,
    pub error: Option<String>,
}

impl SubstrateVerification {
    pub fn success(state: SubstrateState) -> Self {
        Self {
            state,
            verified: true,
            error: None,
        }
    }

    pub fn failure(state: SubstrateState, error: String) -> Self {
        Self {
            state,
            verified: false,
            error: Some(error),
        }
    }
}

fn hash_data(data: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substrate_state_builder() {
        let state = SubstrateState::new()
            .with_http_status(403)
            .with_visibility("public".to_string())
            .with_internal_network(false);

        assert_eq!(state.http_status, Some(403));
        assert_eq!(state.visibility_label, Some("public".to_string()));
        assert!(!state.is_internal_network);
    }

    #[test]
    fn test_substrate_signing() {
        let mut state = SubstrateState::new().with_http_status(403);
        state.sign();
        assert!(!state.signature.is_empty());
        assert_eq!(state.signature.len(), 64); // SHA256 hex is 64 chars
    }
}
