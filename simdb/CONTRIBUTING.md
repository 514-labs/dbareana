# Contributing to simDB

Thank you for your interest in contributing to simDB! This document provides guidelines and instructions for contributing.

## Getting Started

### Prerequisites

- Rust 1.70 or higher
- Docker Desktop or Docker Engine
- Git

### Setting Up Development Environment

```bash
# Clone the repository
git clone https://github.com/yourusername/simdb.git
cd simdb

# Build the project
cargo build

# Run tests
cargo test

# Run integration tests (requires Docker)
cargo test --test integration -- --ignored
```

## Development Workflow

### Branch Strategy

- `main` - Stable releases only
- `develop` - Integration branch for features
- `feature/<name>` - Feature branches

### Making Changes

1. Fork the repository
2. Create a feature branch from `develop`:
   ```bash
   git checkout develop
   git checkout -b feature/my-feature
   ```
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass:
   ```bash
   cargo test --all
   ```
6. Format your code:
   ```bash
   cargo fmt --all
   ```
7. Run clippy:
   ```bash
   cargo clippy -- -D warnings
   ```
8. Commit your changes with a descriptive message
9. Push to your fork
10. Open a Pull Request to `develop`

## Code Style

### Rust Style

- Follow Rust style guidelines
- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Write idiomatic Rust code

### Code Organization

- Keep functions focused and small
- Add documentation comments for public APIs
- Use meaningful variable and function names
- Group related functionality in modules

### Error Handling

- Use the `Result` type for fallible operations
- Create specific error types in `error.rs`
- Provide helpful error messages
- Don't panic in library code

## Testing

### Test Categories

1. **Unit Tests**: Test individual functions and methods
   ```rust
   #[cfg(test)]
   mod tests {
       #[test]
       fn test_something() {
           // Test code
       }
   }
   ```

2. **Integration Tests**: Test end-to-end workflows
   ```rust
   #[tokio::test]
   #[ignore] // Requires Docker
   async fn test_container_lifecycle() {
       // Test code
   }
   ```

3. **Benchmarks**: Measure performance
   ```rust
   #[tokio::test]
   #[ignore]
   async fn bench_warm_start() {
       // Benchmark code
   }
   ```

### Running Tests

```bash
# Unit tests only
cargo test --lib

# Integration tests
cargo test --test integration -- --ignored

# Benchmarks
cargo test --test benchmarks -- --ignored --nocapture

# All tests
cargo test --all -- --ignored
```

## Documentation

### Code Documentation

- Add doc comments for all public APIs
- Include examples in doc comments
- Explain complex algorithms
- Document error conditions

Example:
```rust
/// Creates a new database container with the given configuration.
///
/// # Arguments
///
/// * `config` - The container configuration
///
/// # Returns
///
/// Returns the created container on success.
///
/// # Errors
///
/// Returns an error if:
/// - Docker is not available
/// - The image cannot be pulled
/// - Container creation fails
///
/// # Example
///
/// ```no_run
/// use simdb::container::{ContainerConfig, ContainerManager, DatabaseType, DockerClient};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = DockerClient::new()?;
///     let manager = ContainerManager::new(client);
///     let config = ContainerConfig::new(DatabaseType::Postgres);
///     let container = manager.create_container(config).await?;
///     Ok(())
/// }
/// ```
pub async fn create_container(&self, config: ContainerConfig) -> Result<Container> {
    // Implementation
}
```

### README Updates

- Update README.md for new features
- Add examples for new functionality
- Update the roadmap as features are completed

## Commit Messages

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Examples

```
feat(cli): add support for custom environment variables

Added --env flag to create command to allow setting custom
environment variables in containers.

Closes #123
```

```
fix(health): improve PostgreSQL health check reliability

The health check now waits for the database to accept connections
before returning success, preventing false positives.
```

## Pull Request Process

1. **Title**: Use a clear, descriptive title
2. **Description**: Explain what changes were made and why
3. **Tests**: Ensure all tests pass
4. **Documentation**: Update relevant documentation
5. **Review**: Address review comments promptly
6. **Squash**: Consider squashing commits before merging

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Benchmarks pass (if applicable)
- [ ] Manual testing performed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex code
- [ ] Documentation updated
- [ ] No new warnings introduced
```

## Performance Considerations

### Benchmarks

When making changes that could affect performance:

1. Run benchmarks before and after changes
2. Document performance impact
3. Ensure targets are still met:
   - Warm start: <5s
   - Cold start: <30s
   - Health check: <5s
   - Destruction: <3s

### Profiling

Use profiling tools when optimizing:

```bash
# Profile with cargo-flamegraph
cargo install flamegraph
cargo flamegraph --test benchmarks -- --ignored
```

## Adding New Database Support

To add support for a new database:

1. Add database type to `DatabaseType` enum in `config.rs`
2. Implement `docker_image()`, `default_port()`, and `as_str()` methods
3. Create health checker in `health/implementations.rs`
4. Add environment variables in `manager.rs`
5. Add tests and benchmarks
6. Update documentation

## Questions?

- Open an issue for questions
- Join discussions in Issues
- Check existing documentation

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers
- Focus on constructive feedback
- Help others learn and grow

Thank you for contributing to simDB!
