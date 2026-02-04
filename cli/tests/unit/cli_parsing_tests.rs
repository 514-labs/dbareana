use clap::Parser;
use dbarena::cli::{Cli, Commands, SnapshotCommands};

#[test]
fn test_query_container_alias_parses() {
    let cli = Cli::parse_from([
        "dbarena",
        "query",
        "--container",
        "test-db",
        "--script",
        "SELECT 1",
    ]);

    match cli.command {
        Some(Commands::Query {
            container,
            container_flag,
            ..
        }) => {
            assert!(container.is_none());
            assert_eq!(container_flag, Some("test-db".to_string()));
        }
        _ => panic!("Expected query command with container flag"),
    }
}

#[test]
fn test_query_container_positional_parses() {
    let cli = Cli::parse_from([
        "dbarena",
        "query",
        "test-db",
        "--script",
        "SELECT 1",
    ]);

    match cli.command {
        Some(Commands::Query {
            container,
            container_flag,
            ..
        }) => {
            assert_eq!(container, Some("test-db".to_string()));
            assert!(container_flag.is_none());
        }
        _ => panic!("Expected query command with positional container"),
    }
}

#[test]
fn test_snapshot_restore_alias_parses() {
    let cli = Cli::parse_from([
        "dbarena",
        "snapshot",
        "restore",
        "--snapshot",
        "snap-123",
    ]);

    match cli.command {
        Some(Commands::Snapshot(SnapshotCommands::Restore {
            snapshot,
            snapshot_flag,
            ..
        })) => {
            assert!(snapshot.is_none());
            assert_eq!(snapshot_flag, Some("snap-123".to_string()));
        }
        _ => panic!("Expected snapshot restore with snapshot flag"),
    }
}

#[test]
fn test_snapshot_create_container_alias_parses() {
    let cli = Cli::parse_from([
        "dbarena",
        "snapshot",
        "create",
        "--container",
        "test-db",
        "--name",
        "baseline",
    ]);

    match cli.command {
        Some(Commands::Snapshot(SnapshotCommands::Create {
            container,
            container_flag,
            name,
            ..
        })) => {
            assert!(container.is_none());
            assert_eq!(container_flag, Some("test-db".to_string()));
            assert_eq!(name, "baseline".to_string());
        }
        _ => panic!("Expected snapshot create with container flag"),
    }
}
