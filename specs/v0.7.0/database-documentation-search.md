# Database Documentation Search

## Feature Overview

Provide installable, versioned **official database documentation packs** with **ultra-fast local search** for both LLMs and humans. Docs are downloaded on demand, normalized, indexed, and queried entirely locally.

## Goals

- **Version-specific**: Search scoped by database + version.
- **Fast**: p95 query latency < 50ms once the index is loaded.
- **LLM-friendly**: JSON output includes doc IDs, snippets, and canonical source URLs.
- **Offline-capable** after installation.

## Non-Goals

- No automated CDC configuration.
- No long-running hosted search service.
- No bundling of vendor docs in the binary by default.

## CLI Design

```
dbarena docs list [--installed] [--available] [--json]
dbarena docs install <db> --version <ver> [--force] [--keep-source] [--accept-license]
dbarena docs search --db <db> --version <ver> --query <q> [--limit N] [--json]
dbarena docs show --doc-id <id> [--max-chars N] [--json]
dbarena docs remove <db> --version <ver> [--yes]
```

### Output (JSON)

`docs search --json` returns:
```json
{
  "query": "logical replication slot",
  "db": "postgres",
  "version": "16",
  "results": [
    {
      "doc_id": "pg-16-logical-replication-001",
      "title": "Logical Replication",
      "section": "Replication → Logical Replication",
      "score": 12.42,
      "snippet": "Create a logical replication slot with ...",
      "source_url": "https://www.postgresql.org/docs/16/logical-replication.html"
    }
  ]
}
```

## Storage Layout

Base dir: `~/.local/share/dbarena/docs/`

```
packs/<db>/<version>/
  manifest.json
  content/          # normalized text/markdown chunks
  index/            # search index
  source/           # optional raw downloads (kept with --keep-source)
```

`manifest.json` includes: `db`, `version`, `source`, `license`, `installed_at`, `doc_count`, `byte_size`.

## Search Index

Use a local BM25 index with fields:
- `db` (facet)
- `version` (facet)
- `doc_id`
- `title`
- `section_path`
- `body`
- `source_url`

Chunk documents by headings to ~1–4KB for precision. Query always filters by `db` + `version`.

## Sources (Initial)

- **PostgreSQL**: Official HTML docs per version (`https://www.postgresql.org/docs/<major>/`)
- **MySQL**: Official `*.info.gz` manual per version
- **SQL Server**: MicrosoftDocs `sql-docs` repository (normalized Markdown)

Docs are **downloaded at install time** and require explicit license acceptance (`--accept-license` or interactive prompt).

## Success Criteria

- [ ] Install + index PostgreSQL 16 docs in <2 minutes on first run
- [ ] Search results return in <50ms p95 on a warm cache
- [ ] `docs search --json` returns stable doc IDs and canonical URLs
- [ ] `docs show` returns the correct chunk and respects `--max-chars`
- [ ] Installed packs list clearly shows db + version + size

## Risks & Mitigations

- **Large downloads** → On-demand install + clear size estimates.
- **Licensing** → Require explicit acceptance and store license in manifest.
- **Version ambiguity (SQL Server)** → Document coverage scope in manifest.
