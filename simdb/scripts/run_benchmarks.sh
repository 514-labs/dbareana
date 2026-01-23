#!/bin/bash
set -e

echo "======================================"
echo "  simDB Benchmark Suite"
echo "======================================"
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Error: Docker is not running"
    echo "Please start Docker and try again"
    exit 1
fi

echo "✓ Docker is running"
echo ""

# Pre-pull images for accurate warm start benchmarks
echo "📦 Pre-pulling images for accurate benchmarks..."
docker pull postgres:16 2>&1 | grep -E "(Pulling|Downloaded|Already|Status)" || true
docker pull mysql:8.0 2>&1 | grep -E "(Pulling|Downloaded|Already|Status)" || true
echo ""

# Run benchmarks
echo "🚀 Running benchmarks..."
echo ""

cargo test --test benchmarks -- --ignored --nocapture

echo ""
echo "======================================"
echo "  Benchmark Results Complete"
echo "======================================"
