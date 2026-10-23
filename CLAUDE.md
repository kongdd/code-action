# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a uTools plugin that provides VSCode history search and Everything file search functionality. It allows users to quickly search and open recent VSCode projects and search files using Everything on Windows.

**Key Features:**
- `vsc_action`: Search VSCode recently opened projects/workspaces
- `es`: Everything file search integration
- `vsc-setting`: Configuration interface for the plugin
- Right-click context menu for multiple opening methods (Terminal, Directory, VSCode, Zed)

## Build Commands

```bash
# Compile TypeScript to JavaScript
tsc

# The compiled output goes to the dist/ directory
# Entry point: dist/preload.js (referenced in plugin.json)
```

## Architecture

### Plugin System

The codebase uses the `utools-helper` library to structure plugins. Entry point is `src/preload.ts`:

```typescript
InitPlugins([
  new VSCode(),      // Handles VSCode history search
  new Everything(),  // Handles Everything file search
  Setting.Init("vsc-setting", config)  // Configuration UI
])
```

Each plugin implements the `Plugin` interface with:
- `code`: Feature code matching plugin.json
- `enter()`: Called when feature is activated
- `search(word)`: Called on user input
- `select(item)`: Called when user selects an item

### Core Components

**src/vscode.ts** - VSCode history plugin
- Reads VSCode's SQLite database (`state.vscdb`) to extract recently opened projects
- Supports both local paths and remote workspaces (WSL, SSH, containers)
- Implements "collect" feature (Ctrl+Enter to pin/unpin favorites)
- Executes `code` command with `--folder-uri` or `--file-uri` flags

**src/everything.ts** - Everything search plugin
- Calls `es.exe` (Everything CLI) to search files
- Filters out certain extensions (.lnk, .log)
- Opens files/folders with default application or explorer

**src/files.ts** - VSCode database reader
- Uses sql.js (WebAssembly build in third_party/sqljs/) to query VSCode's state.vscdb
- Queries `ItemTable` for `history.recentlyOpenedPathsList`
- Validates file existence for the first 20 entries

**src/contextmenu.ts** - Right-click menu
- Dynamically creates context menu with actions:
  - `wt.exe -d`: Open in Windows Terminal
  - `explorer.exe`: Open in File Explorer
  - `code.exe`: Open in VSCode
  - `zed`: Open in Zed
  - Copy Linux Path (converts backslashes to forward slashes)

**src/config.ts** - Configuration schema
- Platform-specific defaults (shell commands differ on Windows/Mac/Linux)
- Configurable paths: `code` executable, VSCode db location, storage.json
- All settings are machine-specific (`only_current_machine: true`)

**src/plugin.ts** - Utility functions
- `getIcon()`: Maps file extensions to icon files
- `check_path()`: Normalizes paths for Windows (handles file:/// URIs)
- `execPromise()`: Promisified child_process.exec

### Data Flow

1. **VSCode Search**: User types in uTools → `VSCode.search()` → `GetFiles()` reads SQLite → filters by keyword → returns ListItems → user selects → executes `code` command
2. **Everything Search**: User types pattern → `Everything.search()` → calls `es.exe` with pattern → parses output → returns ListItems → user selects → opens with default app

### Platform-Specific Behavior

- **Windows**: No shell wrapper by default, uses backslashes for paths
- **macOS**: Uses `zsh -l -c` as shell wrapper
- **Linux**: Uses `bash -l -c` as shell wrapper

### Context Menu Implementation

The context menu is event-delegation based:
- Created once on DOMContentLoaded
- Positioned at mouse coordinates on right-click
- Hidden on click outside or mouse move away
- Reads path from `.list-item-description` element

## Development Notes

- TypeScript compiled to ES2018 CommonJS modules
- Uses utools-api-types for uTools API type definitions
- sql.js bundled locally in third_party/sqljs/ to reduce package size
- Remote paths (vscode-remote://) are NOT URI-encoded to preserve scheme
- The plugin must be loaded in uTools environment (uses global `utools` API)

## Configuration Locations

Default VSCode database path (Windows):
```
%APPDATA%\Code\User\globalStorage\state.vscdb
```

Everything CLI expected at:
```
C:\Program Files\Everything\es.exe
```

Users must have `code` command available in PATH before using the plugin.
