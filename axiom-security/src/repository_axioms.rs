use axiom_core::{Axiom, AxiomId, AxiomResult, Priority, SubstrateState};

/// AXIOM_REPO_ACCESS_CONSISTENCY
/// Ensures repository access state is logically consistent
pub struct RepositoryAccessConsistency;

impl Axiom for RepositoryAccessConsistency {
    fn id(&self) -> AxiomId {
        AxiomId::from_name("AXIOM_REPO_ACCESS_CONSISTENCY")
    }

    fn name(&self) -> &str {
        "Repository Access Consistency"
    }

    fn priority(&self) -> Priority {
        Priority::CRITICAL
    }

    fn evaluate(&self, state: &SubstrateState) -> AxiomResult {
        // Check: HTTP status codes should be valid (0-599)
        if let Some(status) = state.http_status {
            if status > 599 {
                return AxiomResult::Violation {
                    code: "INVALID_HTTP_STATUS".to_string(),
                    message: format!("HTTP status {} is outside valid range 0-599", status),
                    remediation: Some("Verify HTTP response is correctly formed".to_string()),
                };
            }
        }

        AxiomResult::Pass
    }
}

/// AXIOM_REPO_PUBLIC_PRIVATE_MATCH
/// Ensures public/private visibility labels match access state
pub struct RepositoryPublicPrivateMatch;

impl Axiom for RepositoryPublicPrivateMatch {
    fn id(&self) -> AxiomId {
        AxiomId::from_name("AXIOM_REPO_PUBLIC_PRIVATE_MATCH")
    }

    fn name(&self) -> &str {
        "Repository Visibility-Access Match"
    }

    fn priority(&self) -> Priority {
        Priority::CRITICAL
    }

    fn evaluate(&self, state: &SubstrateState) -> AxiomResult {
        let is_public = state
            .visibility_label
            .as_ref()
            .map(|v| v.to_lowercase().contains("public"))
            .unwrap_or(false);

        let http_status = state.http_status.unwrap_or(0);

        // If marked public AND (404 or 403), that's suspicious
        if is_public && (http_status == 403 || http_status == 404) {
            return AxiomResult::Violation {
                code: "VISIBILITY_ACCESS_MISMATCH".to_string(),
                message: format!(
                    "Repository marked PUBLIC but returns HTTP {} (Access Denied)",
                    http_status
                ),
                remediation: Some(
                    "Check repository visibility settings or contact repository admin".to_string(),
                ),
            };
        }

        AxiomResult::Pass
    }
}

/// AXIOM_REPO_MISCONFIGURATION
/// Detects common repository misconfigurations
pub struct RepositoryMisconfiguration;

impl Axiom for RepositoryMisconfiguration {
    fn id(&self) -> AxiomId {
        AxiomId::from_name("AXIOM_REPO_MISCONFIGURATION")
    }

    fn name(&self) -> &str {
        "Repository Misconfiguration Detection"
    }

    fn priority(&self) -> Priority {
        Priority::HIGH
    }

    fn evaluate(&self, state: &SubstrateState) -> AxiomResult {
        let is_public = state
            .visibility_label
            .as_ref()
            .map(|v| v.to_lowercase().contains("public"))
            .unwrap_or(false);

        let is_forbidden = state.http_status == Some(403);

        // The canonical misconfiguration: Public label + Forbidden status
        if is_public && is_forbidden {
            return AxiomResult::Violation {
                code: "REPO_CONFIG_ERROR".to_string(),
                message:
                    "Configuration Error: Public-labeled repository returning 403 Forbidden"
                        .to_string(),
                remediation: Some(
                    "Verify repository permission settings match visibility label. This is likely a misconfiguration, not a security test.".to_string(),
                ),
            };
        }

        AxiomResult::Pass
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_consistency_valid() {
        let axiom = RepositoryAccessConsistency;
        let state = SubstrateState::new().with_http_status(200);

        let result = axiom.evaluate(&state);
        assert!(result.is_pass());
    }

    #[test]
    fn test_access_consistency_violation() {
        let axiom = RepositoryAccessConsistency;
        let state = SubstrateState::new().with_http_status(999);

        let result = axiom.evaluate(&state);
        assert!(result.is_violation());
    }

    #[test]
    fn test_public_private_match_violation() {
        let axiom = RepositoryPublicPrivateMatch;
        let state = SubstrateState::new()
            .with_http_status(403)
            .with_visibility("public".to_string());

        let result = axiom.evaluate(&state);
        assert!(result.is_violation());
    }

    #[test]
    fn test_public_private_match_pass() {
        let axiom = RepositoryPublicPrivateMatch;
        let state = SubstrateState::new()
            .with_http_status(200)
            .with_visibility("public".to_string());

        let result = axiom.evaluate(&state);
        assert!(result.is_pass());
    }

    #[test]
    fn test_misconfiguration_detection() {
        let axiom = RepositoryMisconfiguration;
        let state = SubstrateState::new()
            .with_http_status(403)
            .with_visibility("public".to_string());

        let result = axiom.evaluate(&state);
        assert!(result.is_violation());
    }
}
