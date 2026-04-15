# CLAUDE.md

This file provides guidance for AI assistants (and humans) working on this repository.

## Project Overview

- Rust KDE tray application for controlling NordLayer CLI.
- Primary UI surface is a system tray menu (StatusNotifier/KSNI).
- Key functionality:
  - Read NordLayer status
  - List private/shared gateways
  - Connect/disconnect/login actions
  - Show desktop notifications

## Tech Stack

- Rust (cargo)
- `ksni` for tray integration
- `notify-rust` for notifications

## Local Commands

```bash
cargo fmt
cargo test
cargo run
```

## Code Conventions

- Keep behavior-focused parsing tests in `src/parser.rs` and public-API checks in `tests/parser.rs`.
- Prefer small helper functions over duplicated menu/action code.
- Keep notification text informative even when body text is hidden by desktop shell.
- Preserve fallback parsing paths where CLI output can vary between versions.

## AI Collaboration Notes

- This repository is AI-assisted; human review is required for all changes.
- Avoid introducing secrets or personal/private data in source, tests, or docs.
- Prefer safe, incremental refactors with tests passing after each change.
- If parser assumptions change, update:
  - `README.md` (Parser Assumptions section)
  - parser tests in `src/parser.rs` and `tests/parser.rs`

## Out-of-Scope by Default

- No forced migrations to async/runtime frameworks unless explicitly requested.
- No breaking CLI behavior changes without updating docs/tests.

