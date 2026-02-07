use axiom_core::{Axiom, AxiomContext, AxiomId, AxiomResult, Priority, SubstrateState};
use sha2::{Digest, Sha256};

/// AXIOM_ASSET_ALLOCATION_SAFETY
/// Enforces human-in-the-loop cryptographic signing for transactions deviating from MA5 by > 2%
pub struct AssetAllocationAxiom {
    supervisor_public_key: String,
}

impl AssetAllocationAxiom {
    pub fn new(public_key: impl Into<String>) -> Self {
        Self {
            supervisor_public_key: public_key.into(),
        }
    }

    fn verify_signature(&self, value: f64, signature: &str) -> bool {
        // In a real implementation, this would use ed25519 or similar
        // For this deterministic substrate, we use a SHA256-based proof of the value + public key
        let expected_data = format!("{}:{}", value, self.supervisor_public_key);
        let mut hasher = Sha256::new();
        hasher.update(expected_data.as_bytes());
        let expected_sig = hex::encode(hasher.finalize());
        
        signature == expected_sig
    }
}

impl Axiom for AssetAllocationAxiom {
    fn id(&self) -> AxiomId {
        AxiomId::from_name("AXIOM_ASSET_ALLOCATION_SAFETY")
    }

    fn name(&self) -> &str {
        "Asset Allocation Safety (HITL)"
    }

    fn priority(&self) -> Priority {
        Priority::CRITICAL
    }

    fn evaluate(&self, state: &SubstrateState) -> AxiomResult {
        let transaction = match state.transaction_value {
            Some(v) => v,
            None => return AxiomResult::Pass, // No transaction to evaluate
        };

        let ma5 = match state.ma5_value {
            Some(v) => v,
            None => return AxiomResult::Pass, // No MA5 baseline
        };

        let deviation = (transaction - ma5).abs() / ma5;

        if deviation > 0.02 {
            // Check for human signature
            match &state.human_signature {
                Some(sig) if self.verify_signature(transaction, sig) => AxiomResult::Pass,
                _ => AxiomResult::Violation {
                    code: "CRITICAL_ACCOUNTABILITY_GAP".to_string(),
                    message: format!(
                        "Transaction value ({}) deviates from MA5 ({}) by {:.2}%, exceeding 2% threshold.",
                        transaction, ma5, deviation * 100.0
                    ),
                    remediation: Some("Cryptographic human supervisor signature required to authorize this transaction.".to_string()),
                },
            }
        } else {
            AxiomResult::Pass
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hitl_violation_without_signature() {
        let axiom = AssetAllocationAxiom::new("supervisor_key");
        let state = SubstrateState::new()
            .with_transaction(105.0) // 5% deviation
            .with_ma5(100.0);
        
        let result = axiom.evaluate(&state);
        assert!(result.is_violation());
    }

    #[test]
    fn test_hitl_pass_with_valid_signature() {
        let axiom = AssetAllocationAxiom::new("supervisor_key");
        
        // Generate valid signature for 105.0
        let expected_data = format!("{}:{}", 105.0, "supervisor_key");
        let mut hasher = Sha256::new();
        hasher.update(expected_data.as_bytes());
        let sig = hex::encode(hasher.finalize());

        let state = SubstrateState::new()
            .with_transaction(105.0)
            .with_ma5(100.0)
            .with_human_signature(sig);
        
        let result = axiom.evaluate(&state);
        assert!(result.is_pass());
    }

    #[test]
    fn test_hitl_pass_within_threshold() {
        let axiom = AssetAllocationAxiom::new("supervisor_key");
        let state = SubstrateState::new()
            .with_transaction(101.0) // 1% deviation
            .with_ma5(100.0);
        
        let result = axiom.evaluate(&state);
        assert!(result.is_pass());
    }
}

// Extension trait for SubstrateState to support transaction fields in tests
trait SubstrateStateExt {
    fn with_transaction(self, val: f64) -> Self;
    fn with_ma5(self, val: f64) -> Self;
    fn with_human_signature(self, sig: String) -> Self;
}

impl SubstrateStateExt for SubstrateState {
    fn with_transaction(mut self, val: f64) -> Self {
        self.transaction_value = Some(val);
        self
    }
    fn with_ma5(mut self, val: f64) -> Self {
        self.ma5_value = Some(val);
        self
    }
    fn with_human_signature(mut self, sig: String) -> Self {
        self.human_signature = Some(sig);
        self
    }
}
