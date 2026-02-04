use dbarena::config::Template;
use dbarena::container::{ContainerConfig, DatabaseType, VolumeMount};

#[test]
fn test_template_from_container_config_includes_volumes() {
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_volume(VolumeMount::volume(
            "data-volume".to_string(),
            "/var/lib/postgresql/data".to_string(),
            false,
        ));

    let template = Template::from_container_config("pg-template".to_string(), None, &config);

    assert_eq!(template.config.volumes.len(), 1);
    assert_eq!(template.config.volumes[0].source, "data-volume");
    assert_eq!(
        template.config.volumes[0].target,
        "/var/lib/postgresql/data"
    );
}

#[test]
fn test_template_toml_roundtrip_preserves_volumes() {
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_volume(VolumeMount::volume(
            "data-volume".to_string(),
            "/var/lib/postgresql/data".to_string(),
            true,
        ));

    let template = Template::from_container_config("pg-template".to_string(), None, &config);

    let toml = toml::to_string_pretty(&template).expect("Failed to serialize template");
    let parsed: Template = toml::from_str(&toml).expect("Failed to parse template");

    assert_eq!(parsed.config.volumes.len(), 1);
    assert!(parsed.config.volumes[0].read_only);
}
