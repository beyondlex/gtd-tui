# gtd-tui Project Overview

## Executive Summary

**gtd-tui** is a terminal-based task management application inspired by Things 3, built with Rust. It implements GTD (Getting Things Done) methodology with a focus on keyboard-first interaction and customizable hotkeys. The project is designed with a modular architecture that separates core business logic from the UI layer, enabling future expansion to native macOS/iOS applications.

**Current Status**: MVP (Minimum Viable Product) - Basic task management with TUI interface, SQLite persistence, and configurable keyboard shortcuts.

---

## Table of Contents

1. [Project Structure](#project-structure)
2. [Architecture](#architecture)
3. [Core Features](#core-features)
4. [Technical Stack](#technical-stack)
5. [Data Models](#data-models)
6. [User Interface](#user-interface)
7. [Configuration](#configuration)
8. [Development Guide](#development-guide)
9. [Future Roadmap](#future-roadmap)

---

## Project Structure

```
gtd-tui/
├── Cargo.toml              # Workspace configuration
├── SPEC.md                 # Product and architecture specification
├── USAGE.md                # User operation guide
├── AGENTS.md               # Repository guidelines
├── qwen/                   # AI-generated documentation
│   └── PROJECT_OVERVIEW.md # This file
│
├── gtd-core/               # Core library (shared across platforms)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── models/         # Data models
│       │   ├── mod.rs
│       │   ├── area.rs
│       │   ├── project.rs
│       │   ├── task.rs
│       │   ├── tag.rs
│       │   ├── heading.rs
│       │   ├── checklist_item.rs
│       │   ├── recurrence_rule.rs
│       │   └── hotkey_config.rs
│       ├── storage/        # Storage abstraction
│       ├── services/       # Business logic
│       └── hotkey/         # Hotkey management
│
├── gtd-tui/                # TUI application
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # Entry point
│       ├── app.rs          # Application state
│       ├── state.rs        # UI state types (View, Mode, Focus)
│       ├── config.rs       # Configuration loading
│       ├── keymap.rs       # Key binding system
│       ├── editor.rs       # Inline editor state machine
│       ├── commands.rs     # Command implementations
│       └── ui/
│           ├── mod.rs      # UI rendering
│           ├── layout.rs   # Layout components
│           ├── theme.rs    # Theme configuration
│           └── views/
│               ├── inbox.rs
│               ├── today.rs
│               ├── upcoming.rs
│               ├── anytime.rs
│               └── somedays.rs
│
└── skills/                 # Agent skill definitions
```

---

## Architecture

### Layered Design

```
┌─────────────────────────────────────────┐
│         UI Layer (Multiple Frontends)   │
│  ┌─────────────┐  ┌─────────────────┐   │
│  │  TUI (Rust) │  │  macOS (Swift)  │   │
│  │  (current)  │  │  (future)       │   │
│  └──────┬──────┘  └────────┬────────┘   │
└─────────┼──────────────────┼────────────┘
          │                  │
          └──────────┬───────┘
                     ▼
┌─────────────────────────────────────────┐
│         gtd-core (Shared Library)       │
│  - Data Models                          │
│  - Business Logic                       │
│  - Storage Abstraction                  │
│  - Hotkey Management                    │
└───────────────────┬─────────────────────┘
                    ▼
┌─────────────────────────────────────────┐
│         Storage Layer                   │
│  ┌─────────────┐  ┌─────────────────┐   │
│  │  SQLite     │  │  JSON Export    │   │
│  │  (Local)    │  │  (Cloud Sync)   │   │
│  └─────────────┘  └─────────────────┘   │
└─────────────────────────────────────────┘
```

### Key Design Principles

1. **Data First**: Complete separation between data layer and UI layer
2. **Keyboard First**: All operations accessible via keyboard shortcuts
3. **GTD Friendly**: Follows Getting Things Done methodology
4. **Cloud Sync Ready**: JSON export/import for cloud synchronization

---

## Core Features

### Views (GTD Organization)

| View | Description | Shortcut |
|------|-------------|----------|
| **Inbox** | Quick capture for unsorted tasks | `1` |
| **Today** | Tasks planned for today | `2` |
| **Upcoming** | Future tasks by date | `3` |
| **Anytime** | Tasks organized by Area/Project | `4` |
| **Someday** | Future possibilities | `5` |

### Task Attributes

- **Title** (required) - Task name
- **Notes** - Detailed description
- **Due Date** - Target completion date
- **Status** - Pending / Completed / Cancelled
- **Checklist** - Sub-tasks with individual completion state
- **Tags** - Multi-label categorization (planned)
- **Project/Area** - Organizational hierarchy (planned)

### Operations

| Action | Default Key | Description |
|--------|-------------|-------------|
| Navigate | `j`/`k` | Move selection |
| New Task | `n` | Create new task |
| Edit | `l` | Enter edit mode |
| Toggle Complete | `x` | Mark done/pending |
| Delete | `d` | Remove task |
| Save | `Ctrl+s` | Persist changes |
| Quit | `q` | Exit application |

---

## Technical Stack

### Dependencies

#### gtd-core
```toml
chrono = "0.4"        # Date/time handling
serde = "1"           # Serialization
serde_json = "1"      # JSON support
rusqlite = "0.31"     # SQLite database
uuid = "1"            # Unique identifiers
```

#### gtd-tui
```toml
ratatui = "0.26"      # TUI framework
crossterm = "0.27"    # Terminal manipulation
anyhow = "1"          # Error handling
toml = "0.8"          # Configuration parsing
```

### Rust Edition

- **Edition**: 2024
- **Minimum Rust Version**: 1.85
- **Lints**: `unsafe_code = warn`, `clippy::all = warn`, `clippy::pedantic = warn`

---

## Data Models

### Core Entities

#### Task
```rust
pub struct Task {
    pub id: Uuid,
    pub project_id: Option<Uuid>,
    pub heading_id: Option<Uuid>,
    pub area_id: Option<Uuid>,
    pub title: String,
    pub notes: Option<String>,
    pub status: TaskStatus,
    pub start_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub is_today: bool,
    pub is_someday: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### TaskStatus
```rust
pub enum TaskStatus {
    Pending,
    Completed,
    Cancelled,
}
```

#### Other Models (Planned/Partial)
- `Area` - Domain/category classification
- `Project` - Goal-oriented task grouping
- `Heading` - Section within projects
- `Tag` - Multi-label tags
- `ChecklistItem` - Sub-task items
- `RecurrenceRule` - Repeating task rules
- `HotkeyConfig` - Keyboard shortcut configuration

---

## User Interface

### Application States

#### View Modes
```rust
pub enum View {
    Inbox,
    Today,
    Upcoming,
    Anytime,
    Someday,
}
```

#### Operation Modes
```rust
pub enum Mode {
    Normal,        // Navigation and selection
    Editing,       // Inline editing
    ConfirmDelete, // Delete confirmation
}
```

#### Editor Focus Areas
```rust
pub enum Focus {
    Title,
    Notes,
    DueDate,
    Checklist,
}
```

#### Editing Layers
```rust
pub enum Layer {
    TaskItem,      // Task-level editing
    ChecklistItem, // Checklist item editing
}
```

### UI Layout

```
┌────────────────────────────────────────────────────────┐
│  [Logo] gtd-tui              🔍 Search    ⚙️ Settings  │
├─────────────┬──────────────────────────────────────────┤
│             │                                          │
│  INBOX      │   ┌──────────────────────────────────┐   │
│  ─────────  │   │ Task Title                   ☐   │   │
│  Today (3)  │   │   Tags: @work @urgent          │   │
│  Upcoming   │   │   Due: tomorrow                │   │
│  Anytime    │   └──────────────────────────────────┘   │
│  Someday    │                                          │
│             │   ┌──────────────────────────────────┐   │
│  AREAS      │   │ Another Task                 ☐   │   │
│  ─────────  │   └──────────────────────────────────┘   │
│  > Work     │                                          │
│  > Life     │                                          │
│  > Learning │                                          │
│             │                                          │
│  + New View │                                          │
├─────────────┴──────────────────────────────────────────┤
│  ↑↓ Navigate  Enter Select  n New  e Edit  d Delete   │
└────────────────────────────────────────────────────────┘
```

---

## Configuration

### Configuration File

**Location**: `~/.config/gtd-tui/config.toml` or `$XDG_CONFIG_HOME/gtd-tui/config.toml`

### Example Configuration

```toml
[theme.calendar]
weekday = "bold"
weekend = "red bold"
today = "green bold"
selected = "blue bold"
bracket = "magenta"

[theme.editor]
checklist_edit = "lightyellow bold"

[keys]
quit = "q"
view_inbox = "1"
view_today = "2"
view_upcoming = "3"
view_anytime = "4"
view_someday = "5"
select_next = "j"
select_prev = "k"
new_task = "n"
edit_task = "l"
toggle_task = "x"
save_edit = "ctrl+s"
```

### Available Key Bindings

| Configuration Key | Default | Description |
|-------------------|---------|-------------|
| `quit` | `q` | Exit application |
| `view_inbox` | `1` | Switch to Inbox view |
| `view_today` | `2` | Switch to Today view |
| `view_upcoming` | `3` | Switch to Upcoming view |
| `view_anytime` | `4` | Switch to Anytime view |
| `view_someday` | `5` | Switch to Someday view |
| `select_next` | `j` | Select next item |
| `select_prev` | `k` | Select previous item |
| `new_task` | `n` | Create new task |
| `edit_task` | `l` | Edit selected task |
| `toggle_task` | `x` | Toggle task completion |
| `save_edit` | `ctrl+s` | Save edits |
| `cancel_edit` | `q` | Cancel editing |
| `next_focus` | `j` | Next field in editor |
| `prev_focus` | `k` | Previous field in editor |
| `date_prev_day` | `h` | Previous day |
| `date_next_day` | `l` | Next day |
| `date_prev_week` | `k` | Previous week |
| `date_next_week` | `j` | Next week |
| `date_today` | `t` | Set to today |
| `date_tomorrow` | `m` | Set to tomorrow |

### Theme Colors

Available colors: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `gray`, `darkgray`, `lightred`, `lightgreen`, `lightyellow`, `lightblue`, `lightmagenta`, `lightcyan`, `white`, `reset`

---

## Development Guide

### Build Commands

```bash
# Build workspace
cargo build

# Build specific package
cargo build -p gtd-tui
cargo build -p gtd-core

# Run TUI application
cargo run -p gtd-tui

# Run tests
cargo test

# Format code
cargo fmt

# Lint with clippy
cargo clippy -- -W clippy::pedantic
```

### Database Location

**Default**: `~/.local/share/gtd-tui/gtd.db`

**Override**: Set `GTD_TUI_DB_PATH` environment variable

```bash
GTD_TUI_DB_PATH=/custom/path/gtd.db cargo run -p gtd-tui
```

### Keymap System

The keymap system allows full customization of keyboard shortcuts:

1. **KeyBinding Structure**:
```rust
pub struct KeyBinding {
    pub ctrl: bool,
    pub shift: bool,
    pub key: char,
}
```

2. **Parsing Format**:
   - Single char: `"j"`, `"n"`, `"A"` (uppercase = shift)
   - With modifier: `"ctrl+s"`, `"ctrl+e"`
   - Case insensitive for key portion

3. **Keymap Groups**:
   - `NavigationKeys` - View switching and navigation
   - `TaskKeys` - Task operations
   - `EditorKeys` - Editor navigation
   - `DateKeys` - Date picker controls
   - `ChecklistKeys` - Checklist operations

### Editor State Machine

The inline editor uses a state machine with multiple layers:

```
Normal Mode
    │
    ├─► TaskItem Layer (Normal)
    │       │
    │       ├─► Title Editing
    │       ├─► Notes Editing
    │       ├─► Due Date Editing
    │       └─► Checklist Layer
    │               │
    │               ├─► ChecklistItem Editing
    │               └─► ...
    │
    └─► ConfirmDelete Mode
```

### Adding New Features

1. **New Model**: Add to `gtd-core/src/models/`
2. **New View**: Add to `gtd-tui/src/ui/views/`
3. **New Command**: Add to `gtd-tui/src/commands.rs`
4. **New Key Binding**: Add to `gtd-tui/src/keymap.rs` and config

---

## Future Roadmap

### Phase 1: MVP (Current)
- [x] Basic CRUD operations (Task)
- [x] View switching (Inbox, Today, Upcoming, Anytime, Someday)
- [x] Default hotkey system
- [x] Task completion/deletion
- [x] Data persistence (SQLite)
- [ ] Search functionality

### Phase 2: Enhanced Features
- [ ] Tag system
- [ ] Recurring tasks
- [ ] Full hotkey customization
- [ ] Checklist improvements
- [ ] Task notes editing
- [ ] Date/reminder settings

### Phase 3: Advanced Features
- [ ] Command palette
- [ ] JSON export/import
- [ ] Cloud sync support
- [ ] Theme customization
- [ ] Plugin system

### Future Platforms
- [ ] macOS native app (SwiftUI)
- [ ] iOS app (SwiftUI)
- [ ] Shared data layer across platforms

---

## File Locations Summary

| Purpose | Path |
|---------|------|
| Database | `~/.local/share/gtd-tui/gtd.db` |
| Config | `~/.config/gtd-tui/config.toml` |
| Config (alt) | `$XDG_CONFIG_HOME/gtd-tui/config.toml` |

---

## Related Documentation

- **[SPEC.md](../SPEC.md)**: Detailed product and architecture specification
- **[USAGE.md](../USAGE.md)**: User operation guide
- **[AGENTS.md](../AGENTS.md)**: Repository guidelines and coding standards

---

*Document generated: March 30, 2026*
*Based on project analysis of gtd-tui workspace*

