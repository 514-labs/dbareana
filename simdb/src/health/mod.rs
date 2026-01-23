mod checker;
mod implementations;

pub use checker::HealthChecker;
pub use implementations::{MySQLHealthChecker, PostgresHealthChecker, SQLServerHealthChecker};

use crate::Result;
use std::time::Duration;

pub async fn wait_for_healthy(
    _container_id: &str,
    _checker: &dyn HealthChecker,
    _timeout: Duration,
) -> Result<()> {
    // Implementation will be added later
    Ok(())
}
