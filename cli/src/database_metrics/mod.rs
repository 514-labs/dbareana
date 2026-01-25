//! Database-specific performance metrics collection
//!
//! This module provides collectors for gathering database-specific metrics
//! (connections, queries, transactions, cache hit ratios, replication lag, etc.)
//! from PostgreSQL, MySQL, and SQL Server containers using Docker exec.

pub mod collector;
pub mod models;
mod mysql;
mod postgres;
mod sqlserver;

pub use collector::{DatabaseMetricsCollector, DockerDatabaseMetricsCollector};
pub use models::{DatabaseMetrics, QueryBreakdown};
