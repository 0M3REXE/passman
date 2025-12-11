use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use crate::model::{Entry, Vault};
use crate::utils::{analyze_password_strength, PasswordStrength};

/// Password health status for an entry
#[derive(Debug, Clone, PartialEq)]
pub enum PasswordHealth {
    Excellent,
    Good,
    Warning { issues: Vec<String> },
    Critical { issues: Vec<String> },
}

/// Password health analysis result
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HealthReport {
    pub entry_id: String,
    pub health: PasswordHealth,
    pub last_changed: DateTime<Utc>,
    pub age_days: i64,
    pub strength: PasswordStrength,
    pub recommendations: Vec<String>,
}

/// Password health analyzer
pub struct PasswordHealthAnalyzer {
    breach_database: HashMap<String, DateTime<Utc>>, // Simulated breach database
}

impl PasswordHealthAnalyzer {
    pub fn new() -> Self {
        Self {
            breach_database: Self::create_mock_breach_database(),
        }
    }

    /// Create a mock breach database for demonstration
    fn create_mock_breach_database() -> HashMap<String, DateTime<Utc>> {
        let mut db = HashMap::new();
        
        // Common breached passwords
        let common_passwords = vec![
            "password123",
            "admin",
            "123456",
            "password",
            "qwerty",
            "letmein",
            "welcome",
            "monkey",
        ];

        let breach_date = Utc::now() - Duration::days(30);
        for password in common_passwords {
            db.insert(password.to_string(), breach_date);
        }

        db
    }

    /// Analyze the health of all passwords in a vault
    pub fn analyze_vault(&self, vault: &Vault) -> Vec<HealthReport> {
        let mut reports = Vec::new();

        for (id, entry) in &vault.entries {
            let report = self.analyze_entry(id, entry);
            reports.push(report);
        }

        // Sort by health status (worst first)
        reports.sort_by(|a, b| self.health_priority(&a.health).cmp(&self.health_priority(&b.health)));

        reports
    }

    /// Analyze the health of a single password entry
    pub fn analyze_entry(&self, id: &str, entry: &Entry) -> HealthReport {
        let (strength, _) = analyze_password_strength(entry.password_str());
        let age_days = (Utc::now() - entry.created_at).num_days();
        
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Check for breached passwords
        if self.is_password_breached(entry.password_str()) {
            issues.push("Password found in data breach".to_string());
            recommendations.push("Change password immediately".to_string());
        }

        // Check password age
        if age_days > 365 {
            issues.push(format!("Password is {} days old", age_days));
            recommendations.push("Consider changing old passwords".to_string());
        } else if age_days > 180 {
            recommendations.push("Password is getting old, consider changing".to_string());
        }

        // Check password strength
        match strength {
            PasswordStrength::VeryWeak | PasswordStrength::Weak => {
                issues.push("Weak password".to_string());
                recommendations.push("Use a stronger password".to_string());
            }
            PasswordStrength::Fair => {
                recommendations.push("Password could be stronger".to_string());
            }
            _ => {}
        }

        // Check for common patterns
        if self.has_common_patterns(entry.password_str()) {
            issues.push("Password uses common patterns".to_string());
            recommendations.push("Avoid predictable patterns".to_string());
        }

        // Determine overall health
        let health = if issues.iter().any(|i| i.contains("breach") || i.contains("Weak")) {
            PasswordHealth::Critical { issues: issues.clone() }
        } else if !issues.is_empty() {
            PasswordHealth::Warning { issues: issues.clone() }
        } else if !recommendations.is_empty() {
            PasswordHealth::Good
        } else {
            PasswordHealth::Excellent
        };

        HealthReport {
            entry_id: id.to_string(),
            health,
            last_changed: entry.modified_at,
            age_days,
            strength,
            recommendations,
        }
    }

    /// Check if password is in breach database
    fn is_password_breached(&self, password: &str) -> bool {
        self.breach_database.contains_key(password)
    }

    /// Check for common password patterns
    fn has_common_patterns(&self, password: &str) -> bool {
        let password_lower = password.to_lowercase();
        
        // Check for common patterns
        let patterns = [
            "123", "abc", "qwe", "asd", "zxc",
            "password", "admin", "user", "test",
        ];

        patterns.iter().any(|pattern| password_lower.contains(pattern))
    }

    /// Get priority for sorting (lower number = higher priority)
    fn health_priority(&self, health: &PasswordHealth) -> u8 {
        match health {
            PasswordHealth::Critical { .. } => 0,
            PasswordHealth::Warning { .. } => 1,
            PasswordHealth::Good => 2,
            PasswordHealth::Excellent => 3,
        }
    }

    /// Generate summary statistics
    pub fn generate_summary(&self, reports: &[HealthReport]) -> HealthSummary {
        let total = reports.len();
        let mut critical = 0;
        let mut warning = 0;
        let mut good = 0;
        let mut excellent = 0;

        for report in reports {
            match report.health {
                PasswordHealth::Critical { .. } => critical += 1,
                PasswordHealth::Warning { .. } => warning += 1,
                PasswordHealth::Good => good += 1,
                PasswordHealth::Excellent => excellent += 1,
            }
        }

        HealthSummary {
            total,
            critical,
            warning,
            good,
            excellent,
            score: self.calculate_health_score(critical, warning, good, excellent, total),
        }
    }

    /// Calculate overall health score (0-100)
    fn calculate_health_score(&self, _critical: usize, warning: usize, good: usize, excellent: usize, total: usize) -> u8 {
        if total == 0 {
            return 100;
        }

        // Critical passwords contribute 0 to score (already penalized by being critical)
        let score = (excellent * 100 + good * 75 + warning * 40) / total;
        score.min(100) as u8
    }
}

/// Summary of password health for a vault
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HealthSummary {
    pub total: usize,
    pub critical: usize,
    pub warning: usize,
    pub good: usize,
    pub excellent: usize,
    pub score: u8, // 0-100 overall health score
}

impl Default for PasswordHealthAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Entry;

    #[test]
    fn test_password_health_analysis() {
        let analyzer = PasswordHealthAnalyzer::new();
        let entry = Entry::new(
            "test_user".to_string(),
            "password123".to_string(), // This should be flagged as breached
            None,
        );

        let report = analyzer.analyze_entry("test", &entry);
        
        match report.health {
            PasswordHealth::Critical { issues } => {
                assert!(issues.iter().any(|i| i.contains("breach")));
            }
            _ => panic!("Expected critical health status"),
        }
    }

    #[test]
    fn test_health_summary() {
        let analyzer = PasswordHealthAnalyzer::new();
        let reports = vec![
            HealthReport {
                entry_id: "1".to_string(),
                health: PasswordHealth::Excellent,
                last_changed: Utc::now(),
                age_days: 30,
                strength: PasswordStrength::Strong,
                recommendations: vec![],
            },
            HealthReport {
                entry_id: "2".to_string(),
                health: PasswordHealth::Critical { issues: vec!["breach".to_string()] },
                last_changed: Utc::now(),
                age_days: 400,
                strength: PasswordStrength::Weak,
                recommendations: vec![],
            },
        ];

        let summary = analyzer.generate_summary(&reports);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.excellent, 1);
        assert_eq!(summary.critical, 1);
        assert!(summary.score < 100);
    }
}
