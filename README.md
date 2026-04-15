# NordLayer KDE Tray (Rust)

Small KDE-friendly system tray app that wraps NordLayer CLI commands.

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

