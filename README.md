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

<p align="center">
  <a href="#installation">English</a> · <a href="README.pt-br.md">Portugues</a>
</p>

---

<h3 align="center">
  <br/>
  MADE WITH CLAUDE, FOR CLAUDE
  <br/><br/>
  <sub>Built entirely by AI to give AI the power to see and control desktop applications.<br/>
  This is what happens when you let Claude build its own tools.</sub>
  <br/><br/>
</h3>

---

## What is this?

**Desktop Automation MCP** is a [Model Context Protocol](https://modelcontextprotocol.io/) server that gives AI agents (like Claude) the ability to **interact with any Windows desktop application** in real time.

It uses the **Windows UI Automation API** to discover windows, inspect control trees, click buttons, type text, read values, capture screenshots, and much more — all through a simple MCP interface.

> Think of it as **"Selenium, but for desktop apps"** — legacy ERP systems, WinForms tools, WPF dashboards, Win32 dialogs, even Electron apps.

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

> **16 tools. Zero configuration. One executable.**

## Installation

### Prerequisites

- **Windows 10/11**
- **[Claude Code](https://docs.anthropic.com/en/docs/claude-code)** (or any MCP-compatible client)

### Step 1 — Download

Go to [Releases](https://github.com/pedrohendp/desktop-automation/releases) and download **`desktop-automation.exe`**.

### Step 2 — Register the MCP

Open your terminal and run:

```bash
claude mcp add desktop-automation -s user -- "C:\Tools\desktop-automation.exe"
```

> Replace `C:\Tools\desktop-automation.exe` with the actual path where you saved the file.

### Step 3 — Restart Claude Code

Close and reopen Claude Code. The MCP will connect automatically.

### Step 4 — Test it

Just ask Claude:

```
"List all open windows on my desktop"
```

That's it!

---

### Linux (via WSL2)

> This MCP uses the Windows UI Automation API. On Linux, it runs through WSL2 on a Windows host.

1. Download `desktop-automation.exe` to your Windows filesystem (e.g., `C:\Tools\`)
2. From WSL2:
   ```bash
   claude mcp add desktop-automation -s user -- "/mnt/c/Tools/desktop-automation.exe"
   ```
3. Restart Claude Code

### macOS (via VM)

> Requires a Windows virtual machine (Parallels, VMware, or UTM).

1. Set up a Windows VM
2. Follow the [Windows installation](#step-1--download) inside the VM
3. Run Claude Code inside the VM

## Quick Start

Once installed, just talk to Claude naturally:

```
"Click the 'Save' button in Notepad"
"Type 'Hello World' into the text field"
"Take a screenshot of the Calculator app"
"Read all data from the grid in my ERP application"
```

Claude will use the MCP tools automatically.

## Architecture

```
desktop-automation.exe
├── MCP Server (rmcp)          — JSON-RPC over stdio
├── COM Thread                 — Windows COM/UI Automation
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

**Built with:** [Rust](https://www.rust-lang.org/) · [rmcp](https://crates.io/crates/rmcp) · [uiautomation](https://crates.io/crates/uiautomation) · [tokio](https://tokio.rs/) · [win_screenshot](https://crates.io/crates/win_screenshot)

## Contributing

```bash
git clone https://github.com/pedrohendp/desktop-automation.git
cd desktop-automation
cargo build --release
```

Requires [Rust 1.75+](https://rustup.rs/) and Windows SDK.

## License

MIT — see [LICENSE](LICENSE).

---

<p align="center">
  <strong>Made with Claude, for Claude.</strong><br/>
  <sub>Give your AI the power to see and touch the desktop.</sub>
</p>
