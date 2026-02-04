# Implementation Truth (Code is Source of Truth)

This document reflects the **current Rust implementation** in `cli/` and is the reference for spec alignment. If any spec conflicts with this document, update the spec to match the code (or mark the feature as Planned/Not Implemented).

---

## CLI Surface Area (from `cli/src/cli/mod.rs`, `cli/src/main.rs`)

### Top-Level Commands
- `create` — create and start containers
- `start`, `stop`, `restart`, `destroy`
- `list`, `inspect`, `logs`
- `stats` — resource + database metrics (`--tui`, `--multipane`, `--follow`, `--json`)
- `exec` — execute arbitrary commands in containers
- `query` — execute SQL script text or file in a container
- `config` — `validate`, `show`, `init`
- `init` — `test`, `validate` for SQL scripts
- `snapshot` — `create`, `list`, `restore`, `delete`, `inspect`
- `volume` — `create`, `list`, `delete`, `inspect`
- `network` — `create`, `list`, `inspect`, `delete`, `connect`, `disconnect`
- `template` — `save`, `list`, `delete`, `export`, `import`, `inspect`
- `seed` — seed data into a container
- `workload` — run a workload against a container (no subcommand)

### Interactive Mode
Running `dbarena` with no command launches the interactive main menu.

### Supported Databases (from `cli/src/container/config.rs`)
- PostgreSQL (`postgres`)
- MySQL (`mysql`)
- SQL Server (`sqlserver`)

Default versions:
- PostgreSQL: `16`
- MySQL: `8.0`
- SQL Server: `2022-latest`

---

## Container Behavior (from `cli/src/container/*`)

- Container names default to `dbarena-<db>-<random>`.
- Ports: database default port + randomized host port if not specified.
- Labels:
  - `dbarena.managed=true`
  - `dbarena.database=<db>`
  - `dbarena.version=<version>`
  - `dbarena.init_scripts=<json array>` (only when init scripts are provided)
  - `dbarena.init_scripts.continue_on_error=<true|false>` (only when init scripts are provided)
- Default container env vars:
  - Postgres: `POSTGRES_USER=postgres`, `POSTGRES_PASSWORD=postgres`, `POSTGRES_DB=testdb`
  - MySQL: `MYSQL_ROOT_PASSWORD=mysql`, `MYSQL_DATABASE=testdb`
  - SQL Server: `ACCEPT_EULA=Y`, `SA_PASSWORD=YourStrong@Passw0rd`

**Known gaps / partial wiring:**
- `persistent` is stored in `ContainerConfig` but no automatic volume creation/mounting is performed during `create`.
- Config schema supports volumes/bind mounts, but `create` does not currently apply them.

---

## Configuration File Schema (from `cli/src/config/schema.rs`)

Supported formats: **TOML** / **YAML**. Load precedence:
1) explicit `--config`
2) `./dbarena.toml` / `./dbarena.yaml`
3) `~/.config/dbarena/config.toml|yaml`

### Top-level keys
- `version` (optional)
- `defaults`:
  - `persistent` (bool)
  - `memory_mb` (u64)
  - `cpu_shares` (u64)
- `profiles` (map of env var maps)
- `databases` (map keyed by db type):
  - `default_version` (string)
  - `env` (map)
  - `profiles` (map of env var maps)
  - `init_scripts` (string or detailed `{ path, continue_on_error }`)
  - `auto_volume`, `volume_path`, `volumes[]`, `bind_mounts[]`
- `monitoring`:
  - `enabled`, `interval_ms`, `cpu_warning_threshold`, `memory_warning_threshold`
- `snapshots`:
  - `auto_pause`, `storage_path`, `max_snapshots_per_container`

**Runtime usage today:**
- `create` uses config only for **env vars** and **profiles**.
- `init_scripts` in config are **validated** by `config validate` but are **not executed** by `create`.

---

## Init Scripts (from `cli/src/init/*`)
`create` supports `--init-script` (repeatable) plus:
- `--continue-on-error` (per create)
- logs written to `~/.local/share/dbarena/logs/`

## Utility Command Notes
- `query` accepts either a positional container name/ID or `--container <name>`.
- `snapshot` accepts either positional snapshot identifiers or `--snapshot <id|name>`.
- `snapshot create` accepts positional container name or `--container <name>`.

## Templates
- `template save` captures env vars, init script labels (if present), resource limits, network info, and volume mounts from Docker inspect.
- Templates are stored as TOML under `~/.local/share/dbarena/templates/`.

---

## Seed Configuration (from `cli/src/seed/config.rs`)

```toml
global_seed = 42
batch_size = 1000

[[seed_rules]]
table = "users"
count = 100

[[seed_rules.columns]]
name = "id"
generator = "sequential"
```

Supports `seed_rules` in either flat array or nested `seed_rules.tables`.

---

## Workload Configuration (from `cli/src/workload/config.rs`)

Key fields:
- `name`
- `pattern` (optional; built-in)
- `custom_operations` (optional)
- `custom_queries` (optional)
- `tables` (required)
- `connections` (default 10)
- `target_tps` (default 100)
- `duration_seconds` or `transaction_count`

Built-in patterns: `oltp`, `ecommerce`, `olap`, `reporting`, `time_series`, `social_media`, `iot`, `read_heavy`, `write_heavy`, `balanced`.

---

## Metrics & TUI (from `cli/src/monitoring/*`, `cli/src/database_metrics/*`)
- `dbarena stats --tui` and `--multipane` are implemented.
- Database metrics collected for Postgres/MySQL/SQL Server via Docker exec.

---

## CDC
No CDC configuration automation is implemented.

Use `dbarena exec` / `dbarena query` for CDC setup commands executed inside containers.
