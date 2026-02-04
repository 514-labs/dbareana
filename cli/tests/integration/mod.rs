mod container_lifecycle;
mod v0_1_0_integration_tests;
mod config_integration_tests;
mod init_script_tests;
mod exec_tests;
mod config_commands_tests;
mod snapshot_tests;
mod volume_tests;
mod network_tests;
mod template_tests;

// Integration tests require Docker to be running
// Run with: cargo test --test integration -- --ignored
