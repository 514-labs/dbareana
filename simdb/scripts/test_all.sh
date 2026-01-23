#!/bin/bash
set -e

echo "======================================"
echo "  simDB Test Suite"
echo "======================================"
echo ""

# Run unit tests
echo "📋 Running unit tests..."
cargo test --lib
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "⚠️  Docker is not running - skipping integration tests"
    exit 0
fi

# Run integration tests
echo "🔗 Running integration tests..."
cargo test --test integration -- --ignored
echo ""

echo "======================================"
echo "  All Tests Passed ✓"
echo "======================================"
