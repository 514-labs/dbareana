# Version 0.2.0 - Configuration Management

## Release Summary

This release introduces a comprehensive configuration management system that allows users to define database schemas, configurations, and deployment settings in declarative TOML files. Users can now define database-agnostic schemas and generate database-specific DDL, manage multiple configuration profiles, and interactively generate configurations through the CLI.

## Key Features

- **TOML-Based Configuration**: Declarative database and schema configuration using TOML syntax
- **Interactive Configuration Generator**: CLI wizard to create configurations without manual file editing
- **DDL Generation**: Automatic generation of database-specific DDL from generic schema definitions
- **Configuration Templates**: Pre-built templates for common CDC and testing scenarios
- **Database-Agnostic Schemas**: Define schemas once, deploy to PostgreSQL, MySQL, or SQL Server
- **Configuration Validation**: Syntax and semantic validation before deployment

## Value Proposition

This release eliminates the need to manually write database-specific setup scripts and enables reusable, version-controlled database configurations. Users can now:
- Define a schema once and deploy it to multiple database types without rewriting SQL
- Share team configurations through version control
- Quickly spin up databases with pre-seeded schemas for testing
- Generate DDL for review before applying to databases
- Maintain consistency across development, testing, and CI environments

## Target Users

- **Development Teams**: Teams needing consistent database configurations across environments
- **CDC Developers**: Engineers testing CDC solutions across multiple database types with identical schemas
- **Database Architects**: Teams evaluating how schemas behave across different databases
- **QA Engineers**: Testers requiring reproducible database states for test scenarios

## Dependencies

**Previous Version:**
- v0.1.0 (Docker Container Management + Rust CLI Foundation)

**System Requirements:**
- Docker Engine 20.10+
- Rust 1.92+
- 4GB RAM minimum

## Success Criteria

- [ ] User can create a basic configuration file using the interactive CLI wizard in <2 minutes
- [ ] User can define a schema with tables, columns, constraints, and indexes in TOML format
- [ ] DDL generation produces valid SQL for all three supported databases (PostgreSQL, MySQL, SQL Server)
- [ ] Generated DDL correctly handles data type mappings across databases
- [ ] Configuration validation catches common errors (invalid data types, missing required fields)
- [ ] User can deploy a configuration to a running container and verify schema creation
- [ ] Configuration templates cover common scenarios (basic CDC, e-commerce, time-series)

## Next Steps

**v0.3.0 - Resource Monitoring** will introduce:
- CPU, memory, and disk I/O monitoring per container
- Real-time resource usage tracking
- Resource consumption history and trends
- Foundation for performance profiling
