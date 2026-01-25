// Integration test runner for tests in tests/integration/ directory
// All integration tests require Docker to be running
// Run with: cargo test --test integration_tests -- --ignored

mod integration {
    mod container_lifecycle;
    mod v0_1_0_integration_tests;
    mod config_integration_tests;
    mod init_script_tests;
    mod exec_tests;
    mod config_commands_tests;
    mod stats_tests;
}
