# OLAP Database Support

## Feature Overview

Support for analytical databases (ClickHouse, Druid, DuckDB, TimescaleDB) with OLAP-specific configuration, data generation, and monitoring.

## Problem Statement

Testing analytical workloads requires:
- Different database types (columnar, time-series)
- OLAP-specific schemas (wide tables, partitioning)
- Time-series data generation
- Analytical query patterns

OLTP-focused tools don't address analytical database testing.

## User Stories

**As a data engineer**, I want to:
- Test ClickHouse as target for CDC pipeline
- Compare analytical query performance across databases
- Generate realistic time-series data
- Benchmark data ingestion rates

## Technical Requirements

### Functional Requirements

**FR-1: Database Support**
- ClickHouse: Latest version with Docker image
- Druid: Latest version with Docker compose
- DuckDB: Embedded or server mode
- TimescaleDB: PostgreSQL extension

**FR-2: OLAP Configuration**
- Column definitions (sortKey, primaryKey for ClickHouse)
- Partitioning schemes (time-based, hash-based)
- Compression settings
- Aggregation tables (materialized views)

**FR-3: Analytics Data Generation**
- Time-series data with realistic timestamps
- Wide tables (100+ columns)
- Hierarchical data (for drill-down)
- Large cardinality dimensions

**FR-4: OLAP Metrics**
- Query execution time (analytical queries)
- Data ingestion rate (rows/second)
- Storage size and compression ratio
- Cache efficiency

## CLI Interface Design

```bash
# Create ClickHouse instance
simdb create clickhouse --version latest

# Create TimescaleDB instance
simdb create timescaledb --version 2.10

# Deploy OLAP schema
simdb config deploy olap-schema.toml --container <name>

# Generate time-series data
simdb seed --config timeseries.toml --container <name> --size large
```

## Implementation Details

OLAP-specific Docker images, schema generators for columnar databases, time-series data generators, analytics query templates.

## Future Enhancements
- More OLAP databases (Pinot, StarRocks)
- CDC to OLAP direct integration
- Query result caching
- Materialized view management
