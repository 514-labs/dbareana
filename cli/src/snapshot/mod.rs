//! Container snapshot module
//!
//! Provides functionality to create, restore, and manage container snapshots.
//! Snapshots are stored as Docker images with metadata labels.

pub mod metadata;
pub mod storage;
pub mod manager;

pub use metadata::Snapshot;
pub use storage::SnapshotStorage;
pub use manager::SnapshotManager;
