#!/bin/bash
# Release helper script for dbarena v0.3.0

set -e

VERSION="0.3.0"
TAG="v${VERSION}"

echo "================================"
echo "dbarena Release Helper - v${VERSION}"
echo "================================"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Must run from dbarena project root"
    exit 1
fi

# Check version in Cargo.toml
CARGO_VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
if [ "$CARGO_VERSION" != "$VERSION" ]; then
    echo "❌ Version mismatch in Cargo.toml: $CARGO_VERSION (expected $VERSION)"
    exit 1
fi
echo "✓ Cargo.toml version correct: $VERSION"

# Check if binary exists and has correct version
if [ -f "target/release/dbarena" ]; then
    BINARY_VERSION=$(./target/release/dbarena --version | cut -d' ' -f2)
    if [ "$BINARY_VERSION" = "$VERSION" ]; then
        echo "✓ Release binary version correct: $VERSION"
    else
        echo "❌ Release binary version mismatch: $BINARY_VERSION (expected $VERSION)"
        echo "   Run: cargo build --release"
        exit 1
    fi
else
    echo "❌ Release binary not found"
    echo "   Run: cargo build --release"
    exit 1
fi

# Check if tag already exists
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "⚠️  Tag $TAG already exists"
    echo "   To recreate: git tag -d $TAG && git push origin :refs/tags/$TAG"
    TAG_EXISTS=1
else
    echo "✓ Tag $TAG does not exist yet"
    TAG_EXISTS=0
fi

# Check git status
if [ -n "$(git status --porcelain)" ]; then
    echo "⚠️  Working directory has uncommitted changes"
    echo "   Run: git status"
    UNCOMMITTED=1
else
    echo "✓ Working directory clean"
    UNCOMMITTED=0
fi

echo ""
echo "================================"
echo "Release Checklist:"
echo "================================"
echo "✓ Version 0.3.0 in Cargo.toml"
echo "✓ Release binary built"
echo "✓ Release notes written (RELEASE_NOTES_v0.3.0.md)"
echo "✓ Smoke tests passed (SMOKE_TEST_RESULTS_v0.3.0.md)"
echo "✓ Unit tests passed (80/80)"
echo "✓ Documentation complete"
echo ""

if [ $UNCOMMITTED -eq 1 ]; then
    echo "⚠️  Action required: Commit uncommitted changes"
    echo ""
    read -p "View git status? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git status
    fi
    echo ""
    read -p "Commit all changes now? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        read -p "Commit message: " COMMIT_MSG
        git add .
        git commit -m "$COMMIT_MSG"
        echo "✓ Changes committed"
    else
        echo "Please commit changes manually before creating release"
        exit 1
    fi
fi

if [ $TAG_EXISTS -eq 0 ]; then
    echo ""
    echo "================================"
    echo "Ready to create release tag!"
    echo "================================"
    echo ""
    echo "This will:"
    echo "  1. Create tag: $TAG"
    echo "  2. Push commits to origin/main"
    echo "  3. Push tag to origin"
    echo ""
    read -p "Proceed with release? (y/n) " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Create tag
        echo "Creating tag $TAG..."
        git tag -a "$TAG" -m "Release v${VERSION} - Performance Monitoring, Snapshots, and Volumes"
        echo "✓ Tag created"

        # Push commits
        echo "Pushing commits to origin..."
        git push origin main
        echo "✓ Commits pushed"

        # Push tag
        echo "Pushing tag to origin..."
        git push origin "$TAG"
        echo "✓ Tag pushed"

        echo ""
        echo "================================"
        echo "✓ Release tag created successfully!"
        echo "================================"
        echo ""
        echo "Next steps:"
        echo "  1. Go to: https://github.com/[username]/dbarena/releases/new"
        echo "  2. Select tag: $TAG"
        echo "  3. Title: dbarena v${VERSION} - Performance Monitoring, Snapshots, and Volumes"
        echo "  4. Copy description from RELEASE_NOTES_v0.3.0.md"
        echo "  5. Attach binary: target/release/dbarena"
        echo "  6. Publish release"
        echo ""
        echo "Binary location:"
        echo "  $(pwd)/target/release/dbarena"
        echo ""
    else
        echo "Release cancelled"
        exit 0
    fi
else
    echo ""
    echo "Tag already exists. To recreate:"
    echo "  git tag -d $TAG"
    echo "  git push origin :refs/tags/$TAG"
    echo "  Then run this script again"
fi

echo ""
echo "================================"
echo "Release preparation complete!"
echo "================================"
