# Utilities & State Management

## Feature Overview

Consolidated utility commands for operating on running containers and managing persistent state. Includes command execution, snapshots, volumes, networks, and reusable templates.

## Scope

Covered commands:
- `exec`, `query`
- `snapshot` (create/list/restore/delete/inspect)
- `volume` (create/list/delete/inspect)
- `network` (create/list/inspect/delete/connect/disconnect)
- `template` (save/list/delete/export/import/inspect)

## Technical Notes (as implemented)

### Exec / Query
- Uses Docker exec to run commands inside containers.
- Supports parallel execution across multiple containers.

### Snapshots
- Snapshots stored as Docker images with metadata labels.
- Restore creates a container directly from snapshot image.

### Volumes / Networks
- Thin wrappers over Docker primitives with dbarena-friendly output.

### Templates
- Templates are stored locally and can be exported/imported as files.

## CLI Examples

```bash
dbarena exec my-postgres -- psql -U postgres -c "SELECT 1"
dbarena query --container my-postgres --file ./query.sql

dbarena snapshot create --container my-postgres --name baseline
dbarena snapshot restore --snapshot baseline --name restored-pg

dbarena volume create pgdata --mount-path /var/lib/postgresql/data
dbarena network create dbnet --subnet 172.20.0.0/16

dbarena template save my-postgres --name baseline-pg
dbarena template export baseline-pg --path ./baseline-pg.json
```
