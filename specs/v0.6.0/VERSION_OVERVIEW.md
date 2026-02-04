# Version 0.6.0 - Utilities & State Management

## Release Summary

This release consolidates the utility and state-management commands that already exist in the CLI: `exec`, `query`, `snapshot`, `volume`, `network`, and `template`. These tools provide repeatable workflows for running commands, managing snapshots, and organizing container state.

## Status

**Implemented** (see `specs/IMPLEMENTATION_TRUTH.md`)

## Key Features

- **Exec / Query Utilities**: Run arbitrary commands or SQL in containers
- **Snapshots**: Create, list, restore, delete, and inspect snapshots
- **Volumes**: Create/list/delete/inspect volumes
- **Networks**: Create/list/inspect/delete/connect/disconnect networks
- **Templates**: Save, import/export, and reuse container configurations

## Value Proposition

Users can:
- Automate repeatable database experiments and recovery workflows
- Share and reuse environment configurations via templates
- Manage storage and network primitives without leaving the CLI

## Dependencies

**Previous Versions:**
- v0.1.0 (Docker Container Management)
- v0.2.0 (Configuration + Init Scripts)
- v0.3.0 (Resource Monitoring)

## Commands (Implemented)

```
dbarena exec [--all] [--filter <pattern>] [--user <user>] [--workdir <dir>] [--parallel] <containers...> -- <command>
dbarena query [--container <name>] [-i] --script <sql> | --file <path>

dbarena snapshot create --container <name> --name <snap>
dbarena snapshot list [--json]
dbarena snapshot restore --snapshot <id|name> [--name <name>] [--port <port>]
dbarena snapshot delete --snapshot <id|name> [--yes]
dbarena snapshot inspect --snapshot <id|name> [--json]

dbarena volume create <name> [--mount-path /data]
dbarena volume list [--all] [--json]
dbarena volume delete <name> [--force] [--yes]
dbarena volume inspect <name> [--json]

dbarena network create <name> [--driver <driver>] [--subnet <cidr>] [--gateway <ip>] [--internal]
dbarena network list [--all] [--json]
dbarena network inspect <name> [--json]
dbarena network delete <name> [--yes]
dbarena network connect <network> <container> [--alias <alias>...]
dbarena network disconnect <network> <container>

dbarena template save <container> --name <name> [--description <text>]
dbarena template list [--json]
dbarena template delete <name> [--yes]
dbarena template export <name> --path <file>
dbarena template import --path <file>
dbarena template inspect <name> [--json]
```

## Success Criteria

- [x] All utility commands available in CLI help
- [x] Snapshot lifecycle commands work end-to-end
- [x] Volume/network operations map to Docker primitives
- [x] Templates can be saved and exported/imported

## Notes

- `query` and `snapshot` accept positional identifiers for backward compatibility, but the spec uses `--container` and `--snapshot` for clarity.

Detailed command behavior and flags are captured in `specs/IMPLEMENTATION_TRUTH.md`.
