# Version 0.7.0 - Database Documentation Search

## Release Summary

This release introduces installable, versioned **official database documentation packs** with **ultra-fast local search**. Docs are downloaded on demand, normalized, indexed, and searchable for both LLMs and humans.

## Status

**Planned**

## Key Features

- **Doc Packs**: On-demand install for PostgreSQL, MySQL, and SQL Server docs by version
- **Fast Search**: Local BM25 index with sub-50ms p95 query latency (warm cache)
- **Version Scoping**: Search explicitly by db + version for deterministic results
- **LLM-Friendly Output**: JSON results include doc IDs, snippets, and canonical URLs

## Value Proposition

Users can:
- Find authoritative answers without leaving the CLI
- Search the exact version they are running
- Provide reliable citations/snippets to LLMs or humans

## Target Users

- **Database Engineers**: Need quick access to official docs
- **CDC Developers**: Look up version-specific replication/CDC details
- **QA Engineers**: Verify behavior against documentation

## Dependencies

**Previous Versions:**
- v0.1.0 (Docker Container Management)
- v0.2.0 (Configuration + Init Scripts)
- v0.4.0 (Database Metrics + TUI)

**System Requirements:**
- Docker Engine 20.10+
- Rust 1.92+
- Disk space for doc packs (varies by DB/version)

## Success Criteria

- [ ] Install + index PostgreSQL 16 docs in <2 minutes on first run
- [ ] Search results return in <50ms p95 on a warm cache
- [ ] `docs search --json` returns stable doc IDs and canonical URLs
- [ ] `docs show` returns the correct chunk and respects `--max-chars`
- [ ] Installed packs list clearly shows db + version + size

## Next Steps

**v0.8.0 - Change Event Monitoring** will introduce:
- Real-time change event inspection (requires external CDC setup)
- Event rate visualization and filtering
- TUI integration for change events
