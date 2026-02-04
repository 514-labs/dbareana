use crate::seed::config::SeedRule;

/// Size preset for scaling row counts
#[derive(Debug, Clone, Copy)]
pub enum SizePreset {
    Small,
    Medium,
    Large,
}

impl SizePreset {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "small" | "s" => Some(SizePreset::Small),
            "medium" | "m" | "med" => Some(SizePreset::Medium),
            "large" | "l" | "lg" => Some(SizePreset::Large),
            _ => None,
        }
    }

    /// Get the multiplier for this preset
    pub fn multiplier(&self) -> f64 {
        match self {
            SizePreset::Small => 0.1,    // 10% of configured count
            SizePreset::Medium => 1.0,   // 100% of configured count
            SizePreset::Large => 10.0,   // 1000% of configured count
        }
    }

    /// Apply this preset to a set of seed rules
    pub fn apply_to_rules(&self, rules: &mut [SeedRule]) {
        let multiplier = self.multiplier();

        for rule in rules {
            let original_count = rule.count;
            rule.count = (original_count as f64 * multiplier).max(1.0) as usize;
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SizePreset::Small => "small",
            SizePreset::Medium => "medium",
            SizePreset::Large => "large",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::config::ColumnRule;
    use std::collections::HashMap;

    fn make_rule(name: &str, count: usize) -> SeedRule {
        SeedRule {
            name: name.to_string(),
            count,
            columns: vec![ColumnRule {
                name: "id".to_string(),
                generator: "sequential".to_string(),
                options: HashMap::new(),
            }],
        }
    }

    #[test]
    fn test_from_str() {
        assert!(matches!(SizePreset::from_str("small"), Some(SizePreset::Small)));
        assert!(matches!(SizePreset::from_str("s"), Some(SizePreset::Small)));
        assert!(matches!(SizePreset::from_str("medium"), Some(SizePreset::Medium)));
        assert!(matches!(SizePreset::from_str("m"), Some(SizePreset::Medium)));
        assert!(matches!(SizePreset::from_str("large"), Some(SizePreset::Large)));
        assert!(matches!(SizePreset::from_str("l"), Some(SizePreset::Large)));
        assert!(SizePreset::from_str("invalid").is_none());
    }

    #[test]
    fn test_multipliers() {
        assert_eq!(SizePreset::Small.multiplier(), 0.1);
        assert_eq!(SizePreset::Medium.multiplier(), 1.0);
        assert_eq!(SizePreset::Large.multiplier(), 10.0);
    }

    #[test]
    fn test_apply_small_preset() {
        let mut rules = vec![
            make_rule("users", 1000),
            make_rule("orders", 5000),
        ];

        SizePreset::Small.apply_to_rules(&mut rules);

        assert_eq!(rules[0].count, 100);  // 1000 * 0.1
        assert_eq!(rules[1].count, 500);  // 5000 * 0.1
    }

    #[test]
    fn test_apply_medium_preset() {
        let mut rules = vec![
            make_rule("users", 1000),
            make_rule("orders", 5000),
        ];

        SizePreset::Medium.apply_to_rules(&mut rules);

        assert_eq!(rules[0].count, 1000);  // unchanged
        assert_eq!(rules[1].count, 5000);  // unchanged
    }

    #[test]
    fn test_apply_large_preset() {
        let mut rules = vec![
            make_rule("users", 1000),
            make_rule("orders", 5000),
        ];

        SizePreset::Large.apply_to_rules(&mut rules);

        assert_eq!(rules[0].count, 10000);   // 1000 * 10
        assert_eq!(rules[1].count, 50000);   // 5000 * 10
    }

    #[test]
    fn test_minimum_count() {
        let mut rules = vec![make_rule("users", 5)];

        SizePreset::Small.apply_to_rules(&mut rules);

        // 5 * 0.1 = 0.5, but minimum is 1
        assert_eq!(rules[0].count, 1);
    }
}
