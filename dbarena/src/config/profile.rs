use super::schema::DBArenaConfig;
use crate::container::DatabaseType;
use crate::error::{DBArenaError, Result};
use std::collections::HashMap;

/// Resolve environment variables for a specific profile and database type
///
/// Merging order (later overrides earlier):
/// 1. Global profile env vars
/// 2. Database-specific profile env vars
pub fn resolve_profile(
    config: &DBArenaConfig,
    profile_name: &str,
    db_type: DatabaseType,
) -> Result<HashMap<String, String>> {
    let mut env_vars = HashMap::new();

    // Check if global profile exists
    let global_profile = config.profiles.get(profile_name);
    let db_key = db_type.to_string().to_lowercase();
    let db_config = config.databases.get(&db_key);
    let db_profile = db_config.and_then(|cfg| cfg.profiles.get(profile_name));

    // If neither profile exists, return error
    if global_profile.is_none() && db_profile.is_none() {
        // Suggest similar profile names
        let available: Vec<String> = config
            .profiles
            .keys()
            .chain(
                db_config
                    .map(|cfg| cfg.profiles.keys())
                    .into_iter()
                    .flatten(),
            )
            .cloned()
            .collect();

        let suggestion = suggest_profile_name(profile_name, &available);

        return Err(DBArenaError::ProfileNotFound(format!(
            "Profile '{}' not found{}",
            profile_name,
            suggestion
                .map(|s| format!(". Did you mean '{}'?", s))
                .unwrap_or_else(|| format!(
                    ". Available profiles: {}",
                    if available.is_empty() {
                        "none".to_string()
                    } else {
                        available.join(", ")
                    }
                ))
        )));
    }

    // Apply global profile env vars
    if let Some(profile) = global_profile {
        env_vars.extend(profile.env.clone());
    }

    // Apply database-specific profile env vars (overrides global)
    if let Some(profile) = db_profile {
        env_vars.extend(profile.env.clone());
    }

    Ok(env_vars)
}

/// Get base environment variables for a database type (from config)
pub fn get_database_env(config: &DBArenaConfig, db_type: DatabaseType) -> HashMap<String, String> {
    let db_key = db_type.to_string().to_lowercase();
    config
        .databases
        .get(&db_key)
        .map(|cfg| cfg.env.clone())
        .unwrap_or_default()
}

/// Suggest a profile name based on simple string distance
fn suggest_profile_name(target: &str, available: &[String]) -> Option<String> {
    if available.is_empty() {
        return None;
    }

    // Simple suggestion: find the profile with the most matching characters
    available
        .iter()
        .map(|name| {
            let distance = levenshtein_distance(target, name);
            (name, distance)
        })
        .min_by_key(|(_, distance)| *distance)
        .filter(|(_, distance)| *distance <= 3) // Only suggest if reasonably close
        .map(|(name, _)| name.clone())
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(matrix[i][j + 1] + 1, matrix[i + 1][j] + 1),
                matrix[i][j] + cost,
            );
        }
    }

    matrix[len1][len2]
}

/// List all available profiles for a database type
pub fn list_profiles(config: &DBArenaConfig, db_type: DatabaseType) -> Vec<String> {
    let db_key = db_type.to_string().to_lowercase();
    let mut profiles: Vec<String> = config.profiles.keys().cloned().collect();

    // Add database-specific profiles
    if let Some(db_config) = config.databases.get(&db_key) {
        for profile_name in db_config.profiles.keys() {
            if !profiles.contains(profile_name) {
                profiles.push(profile_name.clone());
            }
        }
    }

    profiles.sort();
    profiles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_global_profile() {
        let toml = r#"
            [profiles.dev]
            env = { LOG_LEVEL = "debug", ENV = "dev" }
        "#;
        let config: DBArenaConfig = toml::from_str(toml).unwrap();

        let env = resolve_profile(&config, "dev", DatabaseType::Postgres).unwrap();
        assert_eq!(env.get("LOG_LEVEL"), Some(&"debug".to_string()));
        assert_eq!(env.get("ENV"), Some(&"dev".to_string()));
    }

    #[test]
    fn test_resolve_database_specific_profile() {
        let toml = r#"
            [profiles.dev]
            env = { LOG_LEVEL = "debug" }

            [databases.postgres.profiles.dev]
            env = { POSTGRES_DB = "myapp_dev" }
        "#;
        let config: DBArenaConfig = toml::from_str(toml).unwrap();

        let env = resolve_profile(&config, "dev", DatabaseType::Postgres).unwrap();
        assert_eq!(env.get("LOG_LEVEL"), Some(&"debug".to_string()));
        assert_eq!(env.get("POSTGRES_DB"), Some(&"myapp_dev".to_string()));
    }

    #[test]
    fn test_database_profile_overrides_global() {
        let toml = r#"
            [profiles.dev]
            env = { LOG_LEVEL = "debug", DB_NAME = "global" }

            [databases.postgres.profiles.dev]
            env = { DB_NAME = "database_specific" }
        "#;
        let config: DBArenaConfig = toml::from_str(toml).unwrap();

        let env = resolve_profile(&config, "dev", DatabaseType::Postgres).unwrap();
        assert_eq!(env.get("LOG_LEVEL"), Some(&"debug".to_string()));
        assert_eq!(env.get("DB_NAME"), Some(&"database_specific".to_string()));
    }

    #[test]
    fn test_profile_not_found() {
        let config = DBArenaConfig::default();
        let result = resolve_profile(&config, "nonexistent", DatabaseType::Postgres);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_profiles() {
        let toml = r#"
            [profiles.dev]
            env = { LOG_LEVEL = "debug" }

            [profiles.prod]
            env = { LOG_LEVEL = "error" }

            [databases.postgres.profiles.staging]
            env = { POSTGRES_DB = "staging" }
        "#;
        let config: DBArenaConfig = toml::from_str(toml).unwrap();

        let profiles = list_profiles(&config, DatabaseType::Postgres);
        assert_eq!(profiles.len(), 3);
        assert!(profiles.contains(&"dev".to_string()));
        assert!(profiles.contains(&"prod".to_string()));
        assert!(profiles.contains(&"staging".to_string()));
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("dev", "dev"), 0);
        assert_eq!(levenshtein_distance("dev", "Dev"), 1);
        assert_eq!(levenshtein_distance("dev", "dav"), 1);
        assert_eq!(levenshtein_distance("dev", "dev1"), 1); // Insert 1
        assert_eq!(levenshtein_distance("dev", "prod"), 4); // All chars different
        assert_eq!(levenshtein_distance("test", "tst"), 1); // Delete 'e'
    }

    #[test]
    fn test_suggest_profile_name() {
        let available = vec!["dev".to_string(), "test".to_string(), "prod".to_string(), "development".to_string()];

        assert_eq!(suggest_profile_name("dav", &available), Some("dev".to_string()));
        assert_eq!(suggest_profile_name("tst", &available), Some("test".to_string()));
        assert_eq!(suggest_profile_name("dev", &available), Some("dev".to_string())); // Exact match
        assert_eq!(suggest_profile_name("deve", &available), Some("dev".to_string())); // Close to "dev"
        assert_eq!(suggest_profile_name("xyzabc", &available), None); // Too different (distance > 3)
    }
}
