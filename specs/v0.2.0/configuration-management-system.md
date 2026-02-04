# Configuration Management (Env + Init Scripts)

## Feature Overview

dbarena supports TOML/YAML configuration files for **defaults**, **environment profiles**, and **database-specific env overrides**. It also supports **initialization scripts** executed at container startup via CLI flags.

## What’s Implemented

- Config discovery:
  - `./dbarena.toml` or `./dbarena.yaml`
  - `~/.config/dbarena/config.toml|yaml`
  - explicit `--config <path>`
- Defaults: `persistent`, `memory_mb`, `cpu_shares`
- Profiles: named env var sets, with database-specific overrides
- CLI overrides:
  - `--env KEY=VALUE` (repeatable)
  - `--env-file <path>`
- Config utilities:
  - `dbarena config validate`
  - `dbarena config show`
  - `dbarena config init`

## Not Implemented (Historical Spec Items)

This version **does not** implement schema/DDL generation or “config deploy” commands. Any references to schema definitions or DDL generators are out of scope for the current CLI.

## Configuration Schema (Current)

```toml
[defaults]
persistent = false
memory_mb = 512
cpu_shares = 1024

[profiles.dev]
env = { LOG_LEVEL = "debug" }

[databases.postgres]
default_version = "16"

[databases.postgres.env]
POSTGRES_USER = "postgres"
POSTGRES_PASSWORD = "postgres"
```

See `specs/IMPLEMENTATION_TRUTH.md` for complete schema details.

## CLI Examples

```bash
dbarena config init
dbarena config validate --config ./dbarena.toml
dbarena config show --profile dev

dbarena create postgres --config ./dbarena.toml --profile dev
dbarena create postgres --env POSTGRES_DB=myapp --env-file ./.env.local
```
