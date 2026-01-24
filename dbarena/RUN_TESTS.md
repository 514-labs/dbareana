# How to Run dbarena Tests

## Quick Start

```bash
# Run all unit tests (fast, no Docker required)
cargo test

# Compile integration tests (requires Docker to run)
cargo test --no-run
```

## Unit Tests (No Docker Required)

Unit tests run quickly and don't need Docker:

```bash
# Run all unit tests
cargo test --lib
cargo test --test unit_tests

# Run specific unit test file
cargo test --lib config::
cargo test --test unit_tests config_tests

# Run a single test
cargo test test_database_type_from_string

# Show test output
cargo test -- --nocapture
```

## Integration Tests (Requires Docker)

Integration tests create real Docker containers and require Docker daemon to be running:

```bash
# Check Docker is running
docker ps

# Run all integration tests
cargo test --test integration_tests -- --ignored

# Run specific module
cargo test --test integration_tests v0_1_0 -- --ignored
cargo test --test integration_tests config -- --ignored
cargo test --test integration_tests init_script -- --ignored
cargo test --test integration_tests exec -- --ignored

# Run single integration test
cargo test --test integration_tests test_postgres_init_script_success -- --ignored --nocapture
```

## Test Categories

### v0.1.0 Features
```bash
# Unit tests
cargo test --test unit_tests v0_1_0_tests

# Integration tests  
cargo test --test integration_tests v0_1_0_integration_tests -- --ignored
```

### v0.2.0 Config Features
```bash
# Unit tests
cargo test --test unit_tests config_tests

# Integration tests
cargo test --test integration_tests config_integration_tests -- --ignored
cargo test --test integration_tests config_commands_tests -- --ignored
```

### v0.2.0 Init Script Features
```bash
# Unit tests
cargo test --test unit_tests init_tests

# Integration tests
cargo test --test integration_tests init_script_tests -- --ignored
```

### Exec Command
```bash
# Integration tests only
cargo test --test integration_tests exec_tests -- --ignored
```

## Benchmarks

```bash
# Run performance benchmarks (requires Docker)
cargo test --test benchmarks -- --ignored --nocapture
```

## Test Coverage

```bash
# Install tarpaulin (first time only)
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --out Html --output-dir coverage

# Open report
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```

## Troubleshooting

### "Docker not available"
Integration tests will skip if Docker is not running:
```bash
# Start Docker daemon
# macOS: Open Docker Desktop
# Linux: sudo systemctl start docker

# Verify
docker ps
```

### Test Cleanup
If tests leave containers behind:
```bash
# List dbarena containers
docker ps -a --filter label=dbarena-managed

# Remove all dbarena containers
docker ps -a --filter label=dbarena-managed -q | xargs docker rm -f
```

### Verbose Output
```bash
# Show all test output
cargo test -- --nocapture

# Show specific test output
cargo test test_name -- --nocapture

# Show Rust backtrace on panic
RUST_BACKTRACE=1 cargo test
```

## Test Count Summary

- **Unit Tests**: 99 tests (~0.01s execution)
- **Integration Tests**: 118+ tests (Docker required)
- **Benchmarks**: 3 benchmarks (Docker required)

Total: 220+ tests

## Continuous Integration

Example CI commands:
```bash
# Unit tests only (fast, for every commit)
cargo test --lib
cargo test --test unit_tests

# Integration tests (for PRs/releases, requires Docker)
cargo test --test integration_tests -- --ignored

# Full test suite
cargo test
cargo test --test integration_tests -- --ignored
```
