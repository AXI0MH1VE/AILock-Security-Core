use axiom_core::{AxiomContext, AxiomHiveError, SubstrateState};
use serde::{Deserialize, Serialize};

/// Pre-inference constraint handler
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreInferenceHandler {
    name: String,
    enabled: bool,
}

impl PreInferenceHandler {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enabled: true,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check substrate state before allowing inference to proceed
    pub fn check_substrate(
        &self,
        substrate_state: &SubstrateState,
        _context: &AxiomContext,
    ) -> Result<bool, AxiomHiveError> {
        // Verify substrate is in valid state
        if let Some(status) = substrate_state.http_status {
            if status >= 400 && status < 500 {
                // 4xx errors might indicate configuration issues
                return Ok(true); // Return true to allow axioms to evaluate further
            }
        }
        Ok(true)
    }
}

/// Post-inference constraint handler
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostInferenceHandler {
    name: String,
    enabled: bool,
}

impl PostInferenceHandler {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enabled: true,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Validate LLM output for constraint violations
    pub fn validate_output(
        &self,
        output: &str,
        substrate_state: &SubstrateState,
        _context: &AxiomContext,
    ) -> Result<bool, AxiomHiveError> {
        // Check for dangerous patterns in LLM output
        let dangerous_patterns = vec![
            ("try these approaches", "Suggesting attack vectors"),
            ("brute force", "Suggesting brute force attack"),
            ("SQL injection", "Suggesting SQL injection"),
            ("header manipulation", "Suggesting header manipulation"),
        ];

        for (pattern, reason) in dangerous_patterns {
            if output.to_lowercase().contains(pattern) {
                // Check if substrate state indicates this is production
                if let Some(status) = substrate_state.http_status {
                    if status == 403 && substrate_state.is_internal_network {
                        return Err(AxiomHiveError::Unknown(format!(
                            "Output contains dangerous pattern: {}",
                            reason
                        )));
                    }
                }
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_inference_handler_creation() {
        let handler = PreInferenceHandler::new("test_handler");
        assert_eq!(handler.name(), "test_handler");
        assert!(handler.is_enabled());
    }

    #[test]
    fn test_post_inference_handler_creation() {
        let handler = PostInferenceHandler::new("test_handler");
        assert_eq!(handler.name(), "test_handler");
        assert!(handler.is_enabled());
    }

    #[test]
    fn test_handler_enable_disable() {
        let mut handler = PreInferenceHandler::new("test");
        assert!(handler.is_enabled());
        handler.set_enabled(false);
        assert!(!handler.is_enabled());
    }
}
