//! Security Analyzer
//!
//! Analyzes generated policies for security issues and compliance with least privilege principles

use super::types::SecurityWarning;
use super::{CedarPolicy, PolicyEffect, PolicyScope};
use std::collections::HashSet;

/// Security analysis report
#[derive(Debug, Clone)]
pub struct SecurityReport {
    /// Overall security score (0-100, higher is better)
    score: u8,
    /// List of warnings
    warnings: Vec<SecurityWarning>,
    /// List of analyses
    analyses: Vec<SecurityAnalysis>,
    /// Risk level
    risk_level: RiskLevel,
}

/// Risk level for security issues
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskLevel {
    /// Low risk - minor issues
    Low,
    /// Medium risk - should be addressed
    Medium,
    /// High risk - must be addressed
    High,
    /// Critical risk - immediate action required
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::High => write!(f, "High"),
            RiskLevel::Critical => write!(f, "Critical"),
        }
    }
}

/// A security analysis result
#[derive(Debug, Clone)]
pub struct SecurityAnalysis {
    /// Name of the analysis
    pub name: String,
    /// Result description
    pub description: String,
    /// Whether the analysis passed
    pub passed: bool,
    /// Optional recommendation
    pub recommendation: Option<String>,
}

impl SecurityAnalysis {
    /// Create a new security analysis
    pub fn new(name: String, description: String, passed: bool) -> Self {
        Self {
            name,
            description,
            passed,
            recommendation: None,
        }
    }

    /// Create a security analysis with a recommendation
    pub fn with_recommendation(
        name: String,
        description: String,
        passed: bool,
        recommendation: String,
    ) -> Self {
        Self {
            name,
            description,
            passed,
            recommendation: Some(recommendation),
        }
    }
}

impl SecurityReport {
    /// Create a new security report
    pub fn new() -> Self {
        Self {
            score: 100,
            warnings: Vec::new(),
            analyses: Vec::new(),
            risk_level: RiskLevel::Low,
        }
    }

    /// Analyze policies for security issues
    pub fn analyze(policies: &[CedarPolicy]) -> Self {
        let mut report = Self::new();

        // Run all security checks
        report.check_wildcard_actions(policies);
        report.check_overly_permissive_principals(policies);
        report.check_default_deny(policies);
        report.check_crud_symmetry(policies);
        report.check_policy_ids(policies);
        report.check_audit_logging(policies);

        // Calculate overall risk level and score
        report.calculate_score_and_risk();

        report
    }

    /// Get the security score
    pub fn score(&self) -> u8 {
        self.score
    }

    /// Get the risk level
    pub fn risk_level(&self) -> &RiskLevel {
        &self.risk_level
    }

    /// Get all warnings
    pub fn warnings(&self) -> &[SecurityWarning] {
        &self.warnings
    }

    /// Get all analyses
    pub fn analyses(&self) -> &[SecurityAnalysis] {
        &self.analyses
    }

    /// Check for wildcard actions
    fn check_wildcard_actions(&mut self, policies: &[CedarPolicy]) {
        let mut has_wildcard = false;
        let mut wildcard_count = 0;

        for policy in policies {
            // Check if policy allows wildcard actions
            if policy.content.contains("action in") {
                // Look for "*" in action list
                if policy.content.contains('*') {
                    has_wildcard = true;
                    wildcard_count += policy.content.matches('*').count();
                }
            }
        }

        if has_wildcard {
            self.warnings.push(SecurityWarning::WildcardActions {
                policy_id: "multiple".to_string(),
                count: wildcard_count,
            });

            self.analyses.push(SecurityAnalysis::with_recommendation(
                "Wildcard Actions Check".to_string(),
                format!(
                    "Found {} wildcard action(s) in policies",
                    wildcard_count
                ),
                false,
                "Replace wildcard actions with specific actions to enforce least privilege".to_string(),
            ));

            self.score -= 20;
        } else {
            self.analyses.push(SecurityAnalysis::new(
                "Wildcard Actions Check".to_string(),
                "No wildcard actions found".to_string(),
                true,
            ));
        }
    }

    /// Check for overly permissive principals
    fn check_overly_permissive_principals(&mut self, policies: &[CedarPolicy]) {
        let mut permissive_policies = Vec::new();

        for policy in policies {
            // Check for policies that apply to all principals
            if policy.content.contains("principal,") && !policy.content.contains("principal in") {
                permissive_policies.push(policy.id.clone());
            }
        }

        if !permissive_policies.is_empty() {
            self.warnings.push(SecurityWarning::OverlyPermissivePrincipals {
                policy_id: "multiple".to_string(),
                principals: permissive_policies.clone(),
            });

            self.analyses.push(SecurityAnalysis::with_recommendation(
                "Permissive Principals Check".to_string(),
                format!(
                    "Found {} policy(ies) that apply to all principals",
                    permissive_policies.len()
                ),
                false,
                "Use specific principal groups or roles instead of wildcards".to_string(),
            ));

            self.score -= 15;
        } else {
            self.analyses.push(SecurityAnalysis::new(
                "Permissive Principals Check".to_string(),
                "No overly permissive principals found".to_string(),
                true,
            ));
        }
    }

    /// Check for default deny policy
    fn check_default_deny(&mut self, policies: &[CedarPolicy]) {
        let has_default_deny = policies.iter().any(|p| {
            p.effect == PolicyEffect::Forbid && p.scope == PolicyScope::Global
        });

        if !has_default_deny {
            self.warnings.push(SecurityWarning::MissingDefaultDeny);

            self.analyses.push(SecurityAnalysis::with_recommendation(
                "Default Deny Check".to_string(),
                "No default deny policy found".to_string(),
                false,
                "Add a global forbid policy as the last policy to ensure default deny".to_string(),
            ));

            self.score -= 25;
        } else {
            self.analyses.push(SecurityAnalysis::new(
                "Default Deny Check".to_string(),
                "Default deny policy found".to_string(),
                true,
            ));
        }
    }

    /// Check CRUD symmetry (create, read, update, delete should have matching policies)
    fn check_crud_symmetry(&mut self, policies: &[CedarPolicy]) {
        let mut create_count = 0;
        let mut read_count = 0;
        let mut update_count = 0;
        let mut delete_count = 0;

        for policy in policies {
            if policy.content.contains("POST") || policy.content.contains("PUT") {
                create_count += 1;
            }
            if policy.content.contains("GET") || policy.content.contains("HEAD") {
                read_count += 1;
            }
            if policy.content.contains("PATCH") {
                update_count += 1;
            }
            if policy.content.contains("DELETE") {
                delete_count += 1;
            }
        }

        let min_count = create_count.min(read_count).min(update_count).min(delete_count);
        let max_count = create_count.max(read_count).max(update_count).max(delete_count);

        if max_count > 0 && max_count > min_count * 2 {
            self.analyses.push(SecurityAnalysis::with_recommendation(
                "CRUD Symmetry Check".to_string(),
                format!(
                    "Unbalanced CRUD operations: C:{}, R:{}, U:{}, D:{}",
                    create_count, read_count, update_count, delete_count
                ),
                false,
                "Review CRUD operations to ensure they are balanced and follow least privilege".to_string(),
            ));

            self.score -= 10;
        } else {
            self.analyses.push(SecurityAnalysis::new(
                "CRUD Symmetry Check".to_string(),
                format!(
                    "CRUD operations are balanced: C:{}, R:{}, U:{}, D:{}",
                    create_count, read_count, update_count, delete_count
                ),
                true,
            ));
        }
    }

    /// Check policy IDs for consistency
    fn check_policy_ids(&mut self, policies: &[CedarPolicy]) {
        let mut ids: HashSet<String> = HashSet::new();
        let mut duplicates = 0;

        for policy in policies {
            if !ids.insert(policy.id.clone()) {
                duplicates += 1;
            }
        }

        if duplicates > 0 {
            self.analyses.push(SecurityAnalysis::with_recommendation(
                "Policy ID Check".to_string(),
                format!("Found {} duplicate policy ID(s)", duplicates),
                false,
                "Ensure all policy IDs are unique".to_string(),
            ));

            self.score -= 5;
        } else {
            self.analyses.push(SecurityAnalysis::new(
                "Policy ID Check".to_string(),
                "All policy IDs are unique".to_string(),
                true,
            ));
        }
    }

    /// Check for audit logging
    fn check_audit_logging(&mut self, policies: &[CedarPolicy]) {
        let mut policies_with_comments = 0;
        let total_policies = policies.len();

        for policy in policies {
            if policy.content.contains("//") {
                policies_with_comments += 1;
            }
        }

        if total_policies > 0 {
            let comment_percentage = (policies_with_comments as f64 / total_policies as f64) * 100.0;

            if comment_percentage < 50.0 {
                self.analyses.push(SecurityAnalysis::with_recommendation(
                    "Audit Logging Check".to_string(),
                    format!(
                        "Only {:.0}% of policies have comments",
                        comment_percentage
                    ),
                    false,
                    "Add comments to all policies for better audit trail".to_string(),
                ));

                self.score -= 5;
            } else {
                self.analyses.push(SecurityAnalysis::new(
                    "Audit Logging Check".to_string(),
                    format!(
                        "{:.0}% of policies have comments",
                        comment_percentage
                    ),
                    true,
                ));
            }
        }
    }

    /// Calculate overall score and risk level
    fn calculate_score_and_risk(&mut self) {
        // Determine risk level based on score and critical warnings
        let has_critical_warning = self.warnings.iter().any(|w| matches!(w, SecurityWarning::MissingDefaultDeny));

        self.risk_level = if self.score >= 90 {
            RiskLevel::Low
        } else if self.score >= 75 {
            RiskLevel::Medium
        } else if self.score >= 60 {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        };

        if has_critical_warning {
            self.risk_level = RiskLevel::Critical;
            self.score = self.score.saturating_sub(10);
        }
    }

    /// Get a summary of the report
    pub fn summary(&self) -> String {
        format!(
            "Security Score: {}/100 (Risk: {})\nWarnings: {}\nAnalyses: {}",
            self.score,
            self.risk_level,
            self.warnings.len(),
            self.analyses.len()
        )
    }
}

impl Default for SecurityReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Risk assessment for a policy
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    /// The severity of the risk
    pub severity: RiskLevel,
    /// Description of the risk
    pub description: String,
    /// Recommendation for mitigation
    pub recommendation: String,
}

/// Analyze policy risks
pub struct RiskAnalyzer;

impl RiskAnalyzer {
    /// Analyze risks in a collection of policies
    pub fn analyze(policies: &[CedarPolicy]) -> Vec<RiskAssessment> {
        let mut risks = Vec::new();

        // Check for over-privileged actions
        risks.extend(Self::detect_over_privileged_actions(policies));

        // Check for missing default deny
        if !Self::has_default_deny(policies) {
            risks.push(RiskAssessment {
                severity: RiskLevel::High,
                description: "Missing default deny policy".to_string(),
                recommendation: "Add explicit default deny policy as final policy".to_string(),
            });
        }

        // Check for too many wildcards
        let wildcard_count = Self::count_wildcards(policies);
        if wildcard_count > 3 {
            risks.push(RiskAssessment {
                severity: RiskLevel::Medium,
                description: format!("High number of wildcards ({})", wildcard_count),
                recommendation: "Replace wildcards with specific resources".to_string(),
            });
        }

        risks
    }

    /// Detect over-privileged actions
    fn detect_over_privileged_actions(policies: &[CedarPolicy]) -> Vec<RiskAssessment> {
        let mut risks = Vec::new();

        for policy in policies {
            // Check if policy allows all resources
            if policy.content.contains("resource in") && policy.content.contains("::") {
                // This is specific, not over-privileged
                continue;
            }

            // Check for wildcard resources
            if policy.content.contains("resource") && !policy.content.contains("in") {
                risks.push(RiskAssessment {
                    severity: RiskLevel::High,
                    description: format!(
                        "Policy '{}' allows access to any resource",
                        policy.id
                    ),
                    recommendation: "Specify resource types instead of using wildcards".to_string(),
                });
            }
        }

        risks
    }

    /// Check if policies include a default deny
    fn has_default_deny(policies: &[CedarPolicy]) -> bool {
        policies.iter().any(|p| {
            p.effect == PolicyEffect::Forbid && p.scope == PolicyScope::Global
        })
    }

    /// Count wildcards in policies
    fn count_wildcards(policies: &[CedarPolicy]) -> usize {
        policies
            .iter()
            .map(|p| p.content.matches('*').count())
            .sum()
    }
}
