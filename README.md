# NordLayer KDE Tray (Rust)

Small KDE-friendly tray app that wraps common NordLayer CLI commands.

## What it does

- Adds a tray icon (`network-vpn`) in Plasma system tray.
- Provides menu actions for:
  - `login`
  - `connect`
  - `disconnect`
  - `gateways`
- Shows command output (or errors) via desktop notifications.

## Prerequisites

- Linux desktop session with DBus (KDE Plasma recommended)
- `nordlayer` CLI installed and available in `PATH`

Optional:

- `NORDLAYER_BIN=/path/to/nordlayer` if binary is not named `nordlayer`

## Run

```bash
cargo run
```

## Test

```bash
cargo test
```

