# NordLayer KDE Tray (Rust)

Small KDE-friendly system tray app that wraps NordLayer CLI commands.

## AI Note

This project was developed with significant AI assistance (code generation, refactoring,
and documentation). Please review and test changes before using in production.

## Features

- **Colored shield icon** in Plasma system tray:
  - 🟢 Green when connected
  - 🔴 Red when disconnected
  - 🟡 Amber when not logged in
  - 🟠 Orange when connecting/reconnecting
  - ⚫ Grey on error
- **Status display** in menu: shows connection state + currently connected gateway (if any)
- **Private Gateways** submenu: lists your private gateways with click-to-connect
- **Shared Gateways** submenu: lists public shared gateways with click-to-connect
- **Checkmark (✓)** next to the currently connected gateway
- **Quick actions**: Disconnect, Login, Refresh Status, Refresh Gateways, Quit
- **Desktop notifications** for all command results and errors
  - Summary always includes action + status (useful on shells that hide body text)
  - Body includes command output (up to 8 lines) and the latest status

## Prerequisites

- Linux desktop session with DBus (KDE Plasma recommended)
- `nordlayer` CLI installed and available in `PATH`

Optional:
- `NORDLAYER_BIN=/path/to/nordlayer` if binary is not named `nordlayer`

## Build & Run

```bash
cargo run
```

The tray icon will appear in your Plasma system tray (usually bottom-right).

## Install

### Option 1: Download from GitHub Releases

Each tagged release (`v*`) publishes:

- `nordlayer-kde-linux-x86_64.tar.gz` (standalone binary)
- `*.rpm` package(s)

### Option 2: Build RPM locally

```bash
cargo install cargo-generate-rpm
cargo build --release
cargo generate-rpm
```

RPM output is written to:

```bash
target/generate-rpm/
```

Install (openSUSE example):

```bash
sudo zypper install target/generate-rpm/*.rpm
```

## Autostart (KDE)

For tray apps, desktop autostart is preferred over a systemd service.

If installed from RPM, launcher file is installed to:

```bash
/usr/share/applications/nordlayer-kde.desktop
```

Enable user autostart:

```bash
mkdir -p ~/.config/autostart
cp /usr/share/applications/nordlayer-kde.desktop ~/.config/autostart/
```

You can also manage this via KDE System Settings -> Autostart.

## Test

```bash
cargo test
```

## How it works

- Queries `nordlayer status` (plain-text) to detect connection state + active gateway
- Queries `nordlayer gateways -f` (template) to list private and shared gateways by ID and name
- Separates gateways into two submenus for easy navigation
- Shows a checkmark next to whichever gateway you're connected to
- All menu items refresh status after running actions, so you immediately see the result

## Parser Assumptions

- `status` parsing is plain-text and looks for keywords like `Login:`, `Connected`, `Not Connected`, `not logged in`.
- `gateways` parsing expects template markers in output: `PRIVATE|<id>|<name>` and `SHARED|<id>|<name>`.
- Gateway parser accepts three stream styles:
  - normal newline-separated rows
  - escaped `\n` rows
  - glued streams where markers appear back-to-back
- If NordLayer CLI output format changes, update `GATEWAYS_TEMPLATE` and parser functions in `src/parser.rs`.

## CI / Release Automation

- GitHub Actions workflow: `.github/workflows/release.yml`
- Trigger: push a tag like `v0.1.0`
- Artifacts uploaded to the GitHub Release:
  - release tarball
  - RPM package(s)
  - `SHA256SUMS.txt` checksums for verification

## Security Hardening

- Dependency automation:
  - Dependabot for Cargo and GitHub Actions (`.github/dependabot.yml`)
- Supply-chain checks in CI:
  - `cargo audit`
  - `cargo deny` with policy in `deny.toml`
- Release/build integrity:
  - `--locked` builds in CI workflows
  - Least-privilege workflow permissions
  - Release checksums (`SHA256SUMS.txt`)
- Governance:
  - `CODEOWNERS` for workflow/dependency manifest review
  - `SECURITY.md` for private vulnerability reporting

