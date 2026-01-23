# Benchmark Quick Reference

Quick reference for simDB performance targets and how to validate them.

## Performance Targets Summary

| Category | Metric | Target | Benchmark |
|----------|--------|--------|-----------|
| **Container Operations** |
| Cold start (PostgreSQL) | Time to ready | <30s | `bench_cold_start_postgres` |
| Warm start (PostgreSQL) | Time to ready | <5s | `bench_warm_start_postgres` |
| Warm start (MySQL) | Time to ready | <5s | `bench_warm_start_mysql` |
| Warm start (SQL Server) | Time to ready | <8s | `bench_warm_start_sqlserver` |
| Health check | Detection time | <5s | `bench_health_check_detection` |
| Destruction | Time to destroy | <3s | `bench_container_destruction` |
| Parallel creation | 3 containers | <10s | `bench_parallel_container_creation` |
| **Resource Usage** |
| PostgreSQL memory | Idle usage | <256MB | `bench_memory_footprint_postgres` |
| MySQL memory | Idle usage | <512MB | `bench_memory_footprint_mysql` |
| Under load memory | Max usage | <512MB | `bench_memory_footprint_under_load` |
| Metrics overhead | CPU usage | <5% | `bench_metrics_collection_overhead` |
| PostgreSQL image | Disk size | <300MB | `bench_image_sizes` |
| MySQL image | Disk size | <500MB | `bench_image_sizes` |
| **Configuration** |
| TOML parsing | Parse time | <50ms | `bench_toml_parsing` |
| DDL generation | Generation time | <100ms | `bench_ddl_generation` |
| Config deployment | Deploy time | <5s | `bench_configuration_deployment` |
| **Data Operations** |
| Seeding (1K rows) | Time to seed | <5s | `bench_seeding_small` |
| Seeding (10K rows) | Time to seed | <15s | `bench_seeding_medium` |
| Seeding (100K rows) | Time to seed | <60s | `bench_seeding_large` |
| Workload TPS | Accuracy | ±10% | `bench_workload_tps_accuracy` |
| High throughput | Sustained TPS | >800 @ 1K target | `bench_workload_high_throughput` |
| **CDC Operations** |
| CDC enable | Time to enable | <10s | `bench_cdc_enable_postgres` |
| Event latency | Capture delay | <1s | `bench_change_event_latency` |
| Event throughput | Events/second | >10,000 | `bench_change_event_throughput` |
| **TUI Performance** |
| Rendering | Frame rate | ≥30 FPS | `bench_tui_rendering_fps` |
| Memory usage | TUI overhead | <50MB | `bench_tui_memory_usage` |
| Update latency | Display delay | <1s | `bench_tui_update_latency` |
| **End-to-End** |
| Complete workflow | Total time | <120s | `bench_complete_cdc_workflow` |

## Quick Commands

### Run All Benchmarks
```bash
./scripts/run_benchmarks.sh
```

### Run Specific Category
```bash
# Container operations
cargo test --release bench_cold_start -- --ignored --nocapture
cargo test --release bench_warm_start -- --ignored --nocapture

# Resource usage
cargo test --release bench_memory_footprint -- --ignored --nocapture

# Data operations
cargo test --release bench_seeding -- --ignored --nocapture
cargo test --release bench_workload -- --ignored --nocapture

# CDC operations
cargo test --release bench_cdc -- --ignored --nocapture

# End-to-end
cargo test --release bench_complete_cdc_workflow -- --ignored --nocapture
```

### Run Single Benchmark
```bash
cargo test --release bench_warm_start_postgres -- --ignored --nocapture
```

### Update Baseline
```bash
./scripts/update_baseline.sh
```

### Compare Against Baseline
```bash
cargo run --bin compare-benchmarks \
  --baseline benchmarks/baseline.json \
  --current benchmark_results.json \
  --threshold 10
```

## CI/CD Integration

Benchmarks run automatically on:
- **Pull Requests**: Detect regressions before merge
- **Main Branch**: Update baseline after merge
- **Weekly Schedule**: Track long-term trends

### Viewing Results

**GitHub Actions:**
```
Actions → Performance Benchmarks → View artifact
```

**Local:**
```bash
cat benchmark_report.md
```

## Regression Thresholds

| Severity | Threshold | Action |
|----------|-----------|--------|
| Critical | >20% slower | Build fails, blocks merge |
| Warning | 10-20% slower | Warning in PR, review required |
| Info | <10% | Informational only |

## Pre-Release Checklist

Before releasing a new version:
- [ ] Run full benchmark suite: `./scripts/run_benchmarks.sh`
- [ ] Verify all benchmarks pass
- [ ] Review any regressions (even if <10%)
- [ ] Update baseline if performance improved: `./scripts/update_baseline.sh`
- [ ] Document performance changes in release notes

## Benchmark Environment

For consistent results:
- **OS**: Ubuntu 22.04 LTS (CI) or similar
- **Docker**: 20.10+
- **CPU**: 4+ cores
- **RAM**: 16GB recommended
- **Disk**: SSD recommended
- **Network**: Stable connection for image pulls

## Troubleshooting

### Benchmarks Failing

**Symptom**: Benchmarks consistently fail on local machine but pass in CI

**Solutions:**
- Ensure Docker has adequate resources allocated (Settings → Resources)
- Close other applications consuming CPU/memory
- Check disk space (need 10GB+ free)
- Verify Docker daemon is not rate-limited

### Inconsistent Results

**Symptom**: Benchmark times vary significantly between runs

**Solutions:**
- Run benchmarks multiple times and average results
- Close resource-intensive applications
- Use `--test-threads=1` flag for serial execution
- Check for background Docker containers

### Image Pull Timeouts

**Symptom**: Cold start benchmarks timeout

**Solutions:**
- Pre-pull images before benchmarking
- Increase timeout in benchmark code
- Check network connection
- Use local Docker registry mirror

## Performance Tracking

### View Historical Trends

```bash
# Generate trend report from last 30 days
cargo run --bin benchmark-trends --days 30
```

### Grafana Dashboard

Import dashboard: `grafana/simdb-benchmarks.json`

Tracks:
- Startup times over time
- Memory usage trends
- TPS accuracy
- CDC latency

### Export for Analysis

```bash
# Export to CSV
cargo run --bin export-benchmarks --format csv > benchmarks.csv

# Export to JSON
cargo run --bin export-benchmarks --format json > benchmarks.json
```

## Custom Benchmarks

### Add New Benchmark

1. Create test in `tests/benchmarks.rs`:
```rust
#[tokio::test]
async fn bench_my_feature() {
    let start = Instant::now();
    // ... test code ...
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < TARGET);
    report_metric("my_feature_time", elapsed);
}
```

2. Add to benchmark suite:
```bash
cargo test --release bench_my_feature -- --ignored --nocapture
```

3. Update baseline:
```bash
./scripts/update_baseline.sh
```

### Custom Thresholds

Edit `.github/workflows/benchmarks.yml`:
```yaml
- name: Compare with baseline
  run: |
    cargo run --bin compare-benchmarks \
      --baseline benchmarks/baseline.json \
      --current benchmark_results.json \
      --threshold 15  # Custom 15% threshold
```

## FAQ

**Q: How long do benchmarks take?**
A: Full suite: ~30 minutes. Individual categories: 2-5 minutes each.

**Q: Can I run benchmarks in parallel?**
A: No, benchmarks must run serially to avoid resource contention. Use `--test-threads=1`.

**Q: Why are my local results different from CI?**
A: Different hardware, background processes, Docker configuration. CI results are canonical.

**Q: Should I update baseline after every PR?**
A: No, only update when intentional performance improvements are made.

**Q: What if a benchmark is flaky?**
A: Increase sample size, add warmup period, or relax threshold slightly. Report as issue if persistent.

## Resources

- **Full Documentation**: [BENCHMARKS.md](./BENCHMARKS.md)
- **Optimization Guide**: [DOCKER_OPTIMIZATION.md](./DOCKER_OPTIMIZATION.md)
- **Implementation Guide**: [v0.1.0/docker-container-management.md](./v0.1.0/docker-container-management.md)
