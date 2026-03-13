# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Getting Things Done (GTD) TUI application inspired by Things 3, written in Rust with fully customizable keyboard shortcuts. Designed as a workspace with a shared core library (`gtd-core`) for potential multi-platform support (TUI, macOS, iOS in the future).

## Build and Run

```bash
# Run the TUI application
cargo run -p gtd-tui

# Build the workspace
cargo build

# Format and lint (workspace-level lints are configured)
cargo fmt
cargo clippy

# Run tests (when added)
cargo test
cargo test -p gtd-core
cargo test -p gtd-tui
```

## Configuration

- **Config file**: `~/.config/gtd-tui/config.toml` (or `$XDG_CONFIG_HOME/gtd-tui/config.toml`)
- **Database**: `~/.local/share/gtd-tui/gtd.db` (override via `GTD_TUI_DB_PATH` env var)
- See `USAGE.md` for available configuration options and keybindings

## Architecture

### Workspace Structure

```
gtd-tui/
├── gtd-core/          # Core library - data models, storage, business logic, hotkey system
│   ├── src/models/    # Area, Project, Task, ChecklistItem, Tag, Heading, RecurrenceRule, HotkeyConfig
│   ├── src/storage/   # Storage trait + SqliteStorage implementation
│   ├── src/services/  # Business logic layer
│   └── src/hotkey/    # Action enum, HotkeyBinding, KeyModifiers
└── gtd-tui/           # TUI application
    ├── src/app.rs     # Main app state machine: View, Mode, Layer, Focus handling
    ├── src/config.rs  # TOML config loading with Keymap struct
    └── src/ui/        # ratatui rendering: layout, theme, views/
```

### Key Architectural Patterns

1. **Data-UI Separation**: `gtd-core` contains all data models and the `Storage` trait for multi-platform reuse. The TUI app is purely a view layer.

2. **Inline Editor**: Dual-layer structure for task editing:
   - `Layer::TaskItem`: Navigates between fields (Title, Notes, DueDate, Checklist)
   - `Layer::ChecklistItem`: Navigates within checklist items
   - Press `l` to enter editing mode or descend into the ChecklistItem layer

3. **Hotkey System**: Fully customizable via config.toml. Keybindings are parsed from strings like `"ctrl+c"`, `"j"`, `"Shift+k"`. The `Keymap` struct in `app.rs` converts config strings to actual key handlers.

4. **App State Machine** (`app.rs`):
   - `View`: Inbox, Today, Upcoming, Anytime, Someday
   - `Mode`: Normal, Editing, ConfirmDelete
   - `Layer`: TaskItem (editing fields), ChecklistItem (editing items)
   - `Focus`: Title, Notes, DueDate, Checklist (which field is selected)

5. **Storage Trait**: Abstract storage interface defined in `gtd-core/src/storage/mod.rs`. Currently only `SqliteStorage` is implemented, but the trait enables future alternative backends.

### Data Models

All models use `uuid::Uuid` as primary keys and include `created_at`/`updated_at` timestamps. Core entities:
- `Area`: Top-level category
- `Project`: Belongs to Area, contains Tasks
- `Heading`: Grouping within Projects
- `Task`: Main entity, can belong to Area, Project, or Heading; has title, notes, due_date, status, checklist_items
- `ChecklistItem`: Subtasks within a Task
- `Tag`: Categorization for tasks

### View Routing

The `app.rs` `on_key()` method is the central event handler that routes keyboard events based on current `View`, `Mode`, and `Layer`. Views are rendered via `ui/views/*.rs` modules.

## Development Notes

- Rust edition 2024, requires Rust 1.85+
- Workspace-level linting: `unsafe_code = "warn"`, clippy `all` and `pedantic` as warnings
- When adding tests, prefer unit tests co-located in `src/` modules and integration tests under `tests/`
- Refer to `SPEC.md` for the planned complete architecture and `USAGE.md` for user-facing keybindings
