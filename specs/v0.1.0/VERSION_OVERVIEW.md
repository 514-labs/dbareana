# Version 0.1.0 - Foundation Release

## Release Summary

The foundation release establishes the core infrastructure for simDB: Docker-based container management and a Rust CLI interface. This version enables users to spin up database instances (PostgreSQL, MySQL, SQL Server) with minimal configuration and manage their lifecycle through an intuitive command-line interface.

## Key Features

- **Docker Container Management**: Programmatic control over database container lifecycle (start, stop, restart, destroy)
- **Rust CLI Foundation**: Interactive command-line interface for database instance management
- **Multi-Database Support**: Initial support for PostgreSQL, MySQL, and SQL Server
- **Health Checking**: Automated readiness detection to ensure databases are operational before use
- **Volume Management**: Data persistence options and cleanup capabilities

## Value Proposition

This release eliminates the manual overhead of Docker commands and provides a unified interface for managing multiple database types. Users can now:
- Spin up a test database in seconds with a single command
- Quickly tear down and recreate instances for clean-slate testing
- Manage multiple database instances simultaneously
- Trust that databases are fully ready before running tests

## Target Users

- **CDC Developers**: Engineers building Change Data Capture solutions who need quick database instances for testing
- **Database Engineers**: Teams comparing database configurations or versions
- **QA Engineers**: Testers requiring isolated database environments for test scenarios

## Dependencies

**System Requirements:**
- Docker Engine 20.10+ installed and running
- Rust 1.92+ (for building from source)
- 4GB RAM minimum (recommended 8GB for multiple instances)

**No previous versions required** - this is the initial release.

## Success Criteria

- [ ] User can spin up a PostgreSQL instance with a single command in <10 seconds
- [ ] User can spin up a MySQL instance with a single command in <10 seconds
- [ ] User can spin up a SQL Server instance with a single command in <15 seconds
- [ ] Health checks correctly detect when databases are ready to accept connections
- [ ] Container lifecycle commands (start/stop/restart/destroy) work reliably
- [ ] Data persists across container restarts when persistence is enabled
- [ ] CLI provides clear error messages for common failure scenarios (Docker not running, port conflicts, etc.)

## Next Steps

**v0.2.0 - Configuration Management System** will introduce:
- TOML-based database configuration files
- Schema definition and DDL generation
- Configuration templates for common scenarios
- Database-agnostic configuration format
