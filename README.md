<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust"/>
  <img src="https://img.shields.io/badge/MCP-Protocol-blue?style=for-the-badge" alt="MCP"/>
  <img src="https://img.shields.io/badge/Windows-UI_Automation-0078D6?style=for-the-badge&logo=windows&logoColor=white" alt="Windows"/>
  <img src="https://img.shields.io/badge/License-MIT-yellow?style=for-the-badge" alt="MIT License"/>
</p>

<h1 align="center">Desktop Automation MCP</h1>

<p align="center">
  <strong>Control any Windows desktop application through AI.</strong><br/>
  WinForms, WPF, Win32, Electron — if it has a UI, this MCP can drive it.
</p>

<p align="center">
  <code>click buttons</code> · <code>read values</code> · <code>fill forms</code> · <code>take screenshots</code> · <code>navigate trees</code> · <code>read grids</code>
</p>

---

<h3 align="center">
  <br/>
  ⚡ MADE WITH CLAUDE, FOR CLAUDE ⚡
  <br/><br/>
  <sub>Built entirely by AI to give AI the power to see and control desktop applications.<br/>
  This is what happens when you let Claude build its own tools.</sub>
  <br/><br/>
</h3>

---

## What is this?

**Desktop Automation MCP** is a [Model Context Protocol](https://modelcontextprotocol.io/) server that gives AI agents (like Claude) the ability to **interact with any Windows desktop application** in real time.

It uses the **Windows UI Automation API** to discover windows, inspect control trees, click buttons, type text, read values, capture screenshots, and much more — all through a simple MCP interface.

### Think of it as "Selenium, but for desktop apps"

Instead of automating browsers, this automates **any native Windows application**: legacy ERP systems, WinForms tools, WPF dashboards, Win32 dialogs, even Electron apps.

## Features

| Category | Tools | Description |
|---|---|---|
| **Window Discovery** | `list_windows` | Find all visible top-level windows with handles, titles, and process info |
| **UI Tree Inspection** | `get_window_tree` | Get the full hierarchical control tree of any window |
| **Element Search** | `find_element`, `wait_for_element` | Find controls by name, automation ID, or type — with wait/polling support |
| **Element Properties** | `get_element_properties` | Get all properties: name, type, bounding rect, patterns, and state |
| **Click & Invoke** | `click_element` | Click or invoke any UI element (buttons, links, menu items) |
| **Text Input** | `set_value`, `send_keys` | Type into fields, send special keys (`{Enter}`, `{Tab}`, `{Ctrl}c`) |
| **Read Values** | `get_value` | Read current value/text from any control |
| **Selection** | `select_item` | Select items in lists, combo boxes, tabs, and tree views |
| **Expand/Collapse** | `expand_collapse` | Expand or collapse tree nodes, combo boxes, expanders |
| **Grid/Table Data** | `read_grid` | Read headers and cell data from DataGridView and Table controls |
| **Screenshots** | `screenshot_window` | Capture window screenshots as base64-encoded PNG images |
| **Window Management** | `set_window_state` | Minimize, maximize, restore, or bring windows to foreground |
| **Focus Tracking** | `get_focused_element` | Get the currently focused element across all windows |

### 16 tools. Zero configuration. One executable.

## Use Cases

- **Automated Testing** — Drive UI tests on legacy desktop applications
- **Data Extraction** — Scrape data from ERP systems, accounting software, or any desktop tool
- **Process Automation** — Automate repetitive workflows across multiple desktop apps
- **Accessibility Auditing** — Inspect UI Automation trees and element properties
- **AI-Powered Assistants** — Let Claude interact with your desktop apps on your behalf
- **Legacy System Integration** — Bridge old desktop applications with modern AI workflows

## Installation

### Prerequisites

- **Windows 10/11** (UI Automation is a Windows-native API)
- **Claude Code** (or any MCP-compatible client)

### Option 1: Download Pre-built Binary (Recommended)

1. Go to the [Releases](https://github.com/pedrohendp/desktop-automation/releases) page
2. Download `desktop-automation.exe`
3. Place it anywhere on your machine (e.g., `C:\Tools\desktop-automation.exe`)
4. Add it to Claude Code:

```bash
claude mcp add desktop-automation -- "C:\Tools\desktop-automation.exe"
```

5. Restart Claude Code. Done!

### Option 2: Build from Source

#### Requirements

- [Rust](https://rustup.rs/) (1.75+)
- Windows SDK (comes with Visual Studio Build Tools)

```bash
# Clone the repository
git clone https://github.com/pedrohendp/desktop-automation.git
cd desktop-automation

# Build in release mode
cargo build --release

# The binary will be at target/release/desktop-automation.exe
```

Then add it to Claude Code:

```bash
claude mcp add desktop-automation -- "C:\path\to\target\release\desktop-automation.exe"
```

## Setup Guide

### Windows

<details>
<summary><strong>Step-by-step (click to expand)</strong></summary>

1. **Download** the latest `desktop-automation.exe` from [Releases](https://github.com/pedrohendp/desktop-automation/releases)

2. **Create a folder** for the executable:
   ```powershell
   mkdir C:\Tools
   ```

3. **Move the executable** to that folder:
   ```powershell
   move desktop-automation.exe C:\Tools\
   ```

4. **Register with Claude Code** (run in your terminal):
   ```bash
   claude mcp add desktop-automation -- "C:\Tools\desktop-automation.exe"
   ```

5. **Restart Claude Code** and verify the MCP is connected:
   ```
   Just ask Claude: "List all open windows on my desktop"
   ```

</details>

### Linux

<details>
<summary><strong>Step-by-step (click to expand)</strong></summary>

> **Note:** This MCP relies on the Windows UI Automation API. On Linux, it can only run inside a Windows environment.

**Option A: WSL2 (Windows Subsystem for Linux)**

If you're using Claude Code from WSL2 on a Windows host:

1. Download `desktop-automation.exe` to your Windows filesystem (e.g., `C:\Tools\`)

2. From WSL2, register the MCP pointing to the Windows executable:
   ```bash
   claude mcp add desktop-automation -- "/mnt/c/Tools/desktop-automation.exe"
   ```

3. Restart Claude Code

**Option B: Wine (experimental)**

```bash
# Install Wine
sudo apt install wine64

# Download the .exe and run via Wine
claude mcp add desktop-automation -- wine "/path/to/desktop-automation.exe"
```

> Wine support is experimental. UI Automation may not work correctly in all scenarios.

</details>

### macOS

<details>
<summary><strong>Step-by-step (click to expand)</strong></summary>

> **Note:** This MCP uses the Windows UI Automation API and requires a Windows environment.

**Option A: Parallels / VMware / UTM (Recommended)**

1. Run Windows in a virtual machine (Parallels, VMware Fusion, or UTM)
2. Inside the Windows VM, follow the [Windows setup guide](#windows)
3. Run Claude Code inside the VM, or connect to it remotely

**Option B: Remote Windows Machine**

If you have access to a Windows machine (remote desktop, cloud VM, etc.):

1. Install the MCP on the Windows machine
2. Use an MCP proxy/tunnel to connect your local Claude Code to the remote MCP server

> Native macOS desktop automation would require a separate implementation using the macOS Accessibility API. Contributions welcome!

</details>

## Quick Start

Once installed, just talk to Claude naturally:

```
You: "List all open windows"
You: "Click the 'Save' button in Notepad"
You: "Type 'Hello World' into the text field"
You: "Take a screenshot of the Calculator app"
You: "Read all data from the grid in my ERP application"
You: "Expand the 'Settings' tree node"
```

Claude will use the MCP tools automatically to interact with your desktop.

## Configuration

### Claude Code (CLI)

```bash
# Add (global — available in all projects)
claude mcp add desktop-automation -- "/path/to/desktop-automation.exe"

# Add (project-scoped)
claude mcp add desktop-automation --scope project -- "/path/to/desktop-automation.exe"

# Remove
claude mcp remove desktop-automation

# Verify
claude mcp list
```

### Claude Desktop App

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "desktop-automation": {
      "command": "C:\\Tools\\desktop-automation.exe",
      "args": []
    }
  }
}
```

### Any MCP Client

This is a standard **stdio MCP server**. Just point your client to the executable:

```json
{
  "type": "stdio",
  "command": "/path/to/desktop-automation.exe",
  "args": []
}
```

## Architecture

```
desktop-automation.exe
├── MCP Server (rmcp)          — Handles JSON-RPC communication over stdio
├── COM Thread                 — Dedicated thread for Windows COM/UI Automation
├── Automation Core            — UI Automation API wrapper
│   ├── Element discovery      — Find and traverse UI elements
│   ├── Pattern support        — Invoke, Value, Selection, ExpandCollapse, Grid
│   └── Property access        — Name, Type, AutomationId, BoundingRect, State
└── Tools
    ├── Window tools           — list_windows, get_window_tree, set_window_state
    ├── Find tools             — find_element, wait_for_element
    ├── Property tools         — get_element_properties, get_value, get_focused_element
    ├── Interaction tools      — click_element, set_value, send_keys, select_item, expand_collapse
    ├── Screenshot tools       — screenshot_window
    └── Advanced tools         — read_grid
```

**Built with:**
- [Rust](https://www.rust-lang.org/) — Performance and safety
- [rmcp](https://crates.io/crates/rmcp) — MCP protocol implementation
- [uiautomation](https://crates.io/crates/uiautomation) — Windows UI Automation bindings
- [tokio](https://tokio.rs/) — Async runtime
- [win_screenshot](https://crates.io/crates/win_screenshot) — Window capture

## Contributing

Contributions are welcome! Whether it's bug fixes, new tools, or platform support.

```bash
git clone https://github.com/pedrohendp/desktop-automation.git
cd desktop-automation
cargo build
cargo run
```

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  <strong>Made with Claude, for Claude.</strong><br/>
  <sub>Give your AI the power to see and touch the desktop.</sub>
</p>
