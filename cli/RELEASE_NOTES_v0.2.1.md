# dbarena v0.2.1 Release Notes

**Release Date:** 2026-01-24
**Type:** Patch Release

## 🐛 Bug Fixes

### Fixed Ctrl+C (SIGINT) Handling

**Issue:** When pressing Ctrl+C to interrupt dbarena, the application did not exit cleanly. This could leave the terminal in an inconsistent state or prevent proper cleanup.

**Fix:** Added proper signal handling for Ctrl+C (SIGINT) using tokio's signal utilities.

**Changes:**
- Added `tokio::signal::ctrl_c()` handler in main function
- Uses `tokio::select!` to gracefully handle interruption
- Exits with standard code 130 for SIGINT
- Displays user-friendly message: "Interrupted by user (Ctrl+C)"

**Example:**
```bash
$ dbarena create postgres
Creating containers...
^C

Interrupted by user (Ctrl+C)
$ # Clean exit, terminal works normally
```

## 📦 Installation

### Quick Install

```bash
curl -sSL https://raw.githubusercontent.com/514-labs/dbareana/main/cli/install.sh | bash
```

### Manual Download

Download from [GitHub Releases](https://github.com/514-labs/dbareana/releases/tag/v0.2.1):

```bash
curl -LO https://github.com/514-labs/dbareana/releases/download/v0.2.1/dbarena
chmod +x dbarena
sudo mv dbarena /usr/local/bin/
```

### Upgrade from v0.2.0

If you installed v0.2.0 using the install script, simply run the install script again:

```bash
curl -sSL https://raw.githubusercontent.com/514-labs/dbareana/main/cli/install.sh | bash
```

Or manually:
```bash
curl -LO https://github.com/514-labs/dbareana/releases/download/v0.2.1/dbarena
chmod +x dbarena
sudo mv dbarena /usr/local/bin/dbarena
```

## 🔍 Technical Details

### Code Changes

**File:** `src/main.rs`

**Before:**
```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    // ... rest of code
}
```

**After:**
```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let result = tokio::select! {
        result = run() => result,
        _ = tokio::signal::ctrl_c() => {
            eprintln!("\n\nInterrupted by user (Ctrl+C)");
            std::process::exit(130);
        }
    };
    result
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    // ... rest of code
}
```

### Exit Codes

- **Normal exit:** 0
- **Error exit:** 1
- **Ctrl+C (SIGINT):** 130 (standard Unix convention: 128 + signal number 2)

## 🧪 Testing

Tested scenarios:
- ✅ Ctrl+C during interactive menu
- ✅ Ctrl+C during container creation
- ✅ Ctrl+C during long-running operations
- ✅ Normal exit still works (exit code 0)
- ✅ Error exit still works (exit code 1)

## 📊 What's Changed

- **Files modified:** 2 (Cargo.toml, src/main.rs)
- **Lines changed:** +14, -1
- **New dependencies:** None (uses existing tokio features)

## 🔄 Backwards Compatibility

**100% compatible with v0.2.0**

All v0.2.0 features continue to work exactly as before:
- Configuration management
- Initialization scripts
- SQL execution
- All CLI commands and flags

This is purely a bug fix release with no breaking changes.

## 📝 Changelog

### v0.2.1 (2026-01-24)
- **Fixed:** Ctrl+C (SIGINT) now exits cleanly with proper signal handling

### v0.2.0 (2026-01-24)
- Initial release with configuration management, init scripts, and SQL execution

## 🙏 Credits

Thanks to users who reported the Ctrl+C issue!

**Contributors:**
- Claude Sonnet 4.5 (Bug fix implementation)

## 📄 License

MIT OR Apache-2.0

## 🐛 Found a Bug?

Report issues at: https://github.com/514-labs/dbareana/issues

---

**Full Changelog**: [v0.2.0...v0.2.1](https://github.com/514-labs/dbareana/compare/v0.2.0...v0.2.1)
