# Version 2.0.0 - OLAP Database Support

## Release Summary

Introduces support for analytical (OLAP) databases including ClickHouse, Apache Druid, DuckDB, and TimescaleDB. Extends simDB's capabilities beyond OLTP to enable testing of data warehousing, time-series, and analytics workloads.

## Key Features

- **ClickHouse Support**: Column-oriented database for analytics
- **Apache Druid Support**: Real-time analytics database
- **DuckDB Support**: Embedded analytics database
- **TimescaleDB Support**: Time-series extension for PostgreSQL
- **OLAP-Specific Configuration**: Column definitions, partitioning, compression
- **Analytics Data Generation**: Time-series and analytical data patterns

## Value Proposition

Expands testing to analytical workloads:
- Test OLTP-to-OLAP data pipelines
- Compare analytical query performance
- Validate CDC to data warehouse flows
- Benchmark columnar vs row-oriented databases
- Test hybrid OLTP/OLAP scenarios

## Target Users

- **Data Engineers**: Building analytical data pipelines
- **BI Engineers**: Testing data warehouse configurations
- **Platform Teams**: Evaluating analytical databases
- **CDC Developers**: Testing CDC to data warehouse integration

## Dependencies

- v1.0.0 (Complete CDC testing platform)
- v1.x (P1 features)

## Success Criteria

- [ ] User can create ClickHouse, Druid, DuckDB, TimescaleDB containers
- [ ] OLAP-specific configurations supported
- [ ] Analytics data generation patterns available
- [ ] Monitoring includes OLAP-specific metrics
- [ ] TUI displays OLAP database status

## Next Steps

**v2.1.0 - Analytics Workloads** will introduce analytical query patterns and benchmarks.
