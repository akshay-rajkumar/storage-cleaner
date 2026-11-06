# Linux Cleaner 🧹

A desktop storage-cleaning app for apt-based Linux distributions (Ubuntu, Kubuntu, Debian).

**Stack:** Rust core · Tauri 2 · Svelte · TypeScript · Tailwind CSS

## Project Structure

```
Cleaner_claude/
├── crates/
│   ├── core/       # Library: scanners, data types (no UI)
│   └── cli/        # Binary `cleaner`: CLI interface using core
├── app/            # Tauri 2 + Svelte + TypeScript + Tailwind
│   └── src-tauri/  # Tauri Rust backend
└── Cargo.toml      # Workspace root
```

## Quick Start

```bash
# Run all tests
cargo test

# CLI scan
cargo run -p cleaner-cli -- scan
cargo run -p cleaner-cli -- scan --json

# Desktop app (requires Tauri CLI)
cd app && npm install && npm run tauri dev
```

## Current Status

**Step 1 — Read-only scanning**
- [x] User cache scanner (~/.cache subfolders)
- [x] Trash scanner (~/.local/share/Trash)
- [x] Disk usage (total/used/free)
- [x] CLI with table and JSON output
- [x] Tauri dashboard with disk bar and category cards

> ⚠️ This step is **read-only**. No files are modified or deleted.

## License

MIT
