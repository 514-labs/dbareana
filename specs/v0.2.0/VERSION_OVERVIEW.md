# Version 0.2.0 - Configuration + Init Scripts

## Release Summary

This release adds configuration files, environment profiles, and initialization scripts. Configuration files are used to set defaults and environment variables; init scripts can be run at container startup via CLI flags.

## Status

**Implemented** (see `specs/IMPLEMENTATION_TRUTH.md`)

## Key Features

- TOML/YAML configuration discovery and loading
- Global defaults (persistent/memory/cpu)
- Environment profiles (global + database-specific)
- CLI env overrides (`--env`, `--env-file`)
- Init scripts (`--init-script`, `--continue-on-error`)
- Config utilities: `config validate`, `config show`, `config init`

## Dependencies

**Previous Version:**
- v0.1.0 (Docker Container Management + Rust CLI Foundation)

## Success Criteria

- [x] Config file can be discovered or passed explicitly
- [x] Environment profiles resolve with correct precedence
- [x] Init scripts execute after container startup
- [x] Config validation reports actionable errors

## Next Steps

**v0.3.0 - Resource Monitoring** introduces container resource metrics.
