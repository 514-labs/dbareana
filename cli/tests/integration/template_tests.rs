/// Integration tests for template save/export/import
/// Run with: cargo test --test integration -- --ignored

use std::time::Duration;

use dbarena::cli::commands::template::{
    handle_template_delete, handle_template_save,
};
use dbarena::config::TemplateManager;
use dbarena::container::{ContainerConfig, DatabaseType};

#[path = "../common/mod.rs"]
mod common;
use common::{create_and_start_container, cleanup_container, docker_available, unique_container_name};

#[tokio::test]
#[ignore]
async fn test_template_save_and_delete() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-template-container"));
    let test_container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    let template_name = unique_container_name("test-template");

    handle_template_save(test_container.name.clone(), template_name.clone(), None)
        .await
        .expect("Failed to save template");

    let manager = TemplateManager::new().expect("Failed to create template manager");
    let template = manager
        .load(&template_name)
        .expect("Failed to load template");

    assert_eq!(template.name, template_name);
    assert!(template.config.network.is_some(), "Network should be captured");
    assert!(
        !template.config.env_vars.is_empty(),
        "Env vars should be captured"
    );

    handle_template_delete(template_name.clone(), true)
        .await
        .expect("Failed to delete template");

    cleanup_container(&test_container.id)
        .await
        .expect("Failed to cleanup container");
}
