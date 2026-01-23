# Snapshot & Restore

## Feature Overview

Database state management through snapshots and point-in-time restoration. Enables quick database resets, A/B testing, and reproducible test environments.

## Problem Statement

Testing requires returning to known states:
- Resetting database between tests is slow (drop/recreate)
- Reproducing specific scenarios requires manual setup
- Migration testing needs easy rollback
- A/B testing requires identical starting states

Without snapshots, developers waste time on database setup.

## User Stories

**As a QA engineer**, I want to:
- Save database state before running destructive tests
- Restore to clean state between test runs
- Share snapshots with team members

**As a developer**, I want to:
- Snapshot before running migrations
- Restore quickly if migration fails
- Test code changes against identical data

## Technical Requirements

### Functional Requirements

**FR-1: Snapshot Creation**
- Capture complete database state (schema + data)
- Include CDC configuration in snapshot
- Support incremental snapshots (only changed data)
- Tag snapshots with names and descriptions

**FR-2: Snapshot Storage**
- Store snapshots as Docker volumes
- Compress snapshots to save space
- Support external snapshot storage (S3, local filesystem)

**FR-3: Restore Operations**
- Restore to any snapshot
- Verify snapshot integrity before restore
- Preserve or replace current data (user choice)

**FR-4: Snapshot Management**
- List all snapshots with metadata
- Delete old snapshots
- Export/import snapshots
- Search by tags or date

### Non-Functional Requirements

**NFR-1: Performance**
- Snapshot creation: <10 seconds for 1GB database
- Restore: <5 seconds
- Incremental snapshots: 70% storage reduction

**NFR-2: Reliability**
- Snapshot verification before restore
- Rollback if restore fails
- No data loss during snapshot

## CLI Interface Design

```bash
# Create snapshot
simdb snapshot create --container <name> --tag "before_migration"

# List snapshots
simdb snapshot list --container <name>

# Restore snapshot
simdb snapshot restore --container <name> --snapshot <id>

# Delete snapshot
simdb snapshot delete --snapshot <id>

# Export snapshot
simdb snapshot export --snapshot <id> --output snapshot.tar.gz
```

## Implementation Details

Docker volume snapshots using commit/tag, optional pg_dump/mysqldump for portability, snapshot metadata storage in SQLite.

## Future Enhancements
- Scheduled automatic snapshots
- Snapshot retention policies
- Cloud backup integration
- Differential snapshots
