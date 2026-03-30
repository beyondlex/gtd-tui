# Configuration Guide

## Principle: All Styles and Keymaps Must Be Configurable

This document serves as a guideline for future development. **All style and keymap settings must be configurable by users** through the configuration file.

---

## Configuration Architecture

### File Location

- **Primary**: `~/.config/gtd-tui/config.toml`
- **Alternative**: `$XDG_CONFIG_HOME/gtd-tui/config.toml`
- **Database Override**: `GTD_TUI_DB_PATH` environment variable

### Configuration Structure

```toml
[theme.calendar]
# Calendar widget styles

[theme.editor]
# Editor component styles

[keys]
# All keyboard shortcuts
```

---

## Implementation Checklist

### ✅ Completed

- [x] Editor cursor style (`theme.editor.cursor`)
- [x] Basic navigation keys
- [x] Task operation keys
- [x] Editor navigation keys
- [x] Date picker keys
- [x] Checklist keys

### ⏳ TODO - Styles to Make Configurable

#### Calendar Theme (`theme.calendar`)

| Property | Current Default | Config Key | Status |
|----------|----------------|------------|--------|
| Weekday style | `bold` | `weekday` | ✅ |
| Weekend style | `red bold` | `weekend` | ✅ |
| Today style | `green bold` | `today` | ✅ |
| Selected day style | `blue bold` | `selected` | ✅ |
| Bracket style | `magenta` | `bracket` | ✅ |

#### Editor Theme (`theme.editor`)

| Property | Current Default | Config Key | Status |
|----------|----------------|------------|--------|
| Edit mode text style | `lightyellow bold` | `checklist_edit` | ✅ |
| Selected task style | `blue bold` | `task_selected` | ✅ |
| Date label style | `darkgray bold` | `date_label` | ✅ |
| Selected checklist item | `blue bold` | `checklist_item_selected` | ✅ |
| Title field label | `cyan bold` | `field_title` | ✅ |
| Notes field label | `green bold` | `field_notes` | ✅ |
| Due field label | `yellow bold` | `field_due` | ✅ |
| Checklist field label | `magenta bold` | `field_checklist` | ✅ |
| Completed task style | `darkgray` + crossed | `completed` | ✅ |
| **Cursor style** | `yellow` | `cursor` | ✅ |

#### Additional Styles to Add

When implementing new UI components, always add corresponding style configurations:

```rust
// Example: Adding a new style
#[derive(Debug, Clone, Deserialize, Default)]
pub struct EditorThemeConfig {
    // ... existing fields ...
    pub new_component: Option<String>,  // Add config field
}

#[derive(Debug, Clone, Copy)]
pub struct EditorTheme {
    // ... existing fields ...
    pub new_component: Style,  // Add theme field
}

impl EditorTheme {
    pub fn from_config(config: &EditorThemeConfig) -> Self {
        // ... existing code ...
        let new_component = parse_style(config.new_component.as_deref())
            .unwrap_or_else(|| Style::default().fg(Color::White));  // Default style
        Self {
            // ... existing fields ...
            new_component,
        }
    }
}
```

### ⏳ TODO - Keymaps to Review/Expand

#### Current Keymap Categories

| Category | Struct | Config Prefix | Status |
|----------|--------|---------------|--------|
| Navigation | `NavigationKeys` | `view_*`, `select_*` | ✅ |
| Task Operations | `TaskKeys` | `new_task`, `edit_task`, etc. | ✅ |
| Editor Navigation | `EditorKeys` | `next_focus`, `prev_focus`, etc. | ✅ |
| Date Picker | `DateKeys` | `date_*` | ✅ |
| Checklist | `ChecklistKeys` | `checklist_*` | ✅ |

#### Keymap Implementation Pattern

When adding new keymaps, follow this pattern:

1. **Add to `KeysConfig`** (`config.rs`):
```rust
pub struct KeysConfig {
    // ... existing fields ...
    pub new_action: Option<String>,  // Add config field
}
```

2. **Add to appropriate `*Keys` struct** (`keymap.rs`):
```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct NewActionKeys {
    pub new_action: KeyBinding,
}

impl NewActionKeys {
    pub fn default_keymap() -> Self {
        Self {
            new_action: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'x',  // Default key
            },
        }
    }
}
```

3. **Load from config** (`keymap.rs`):
```rust
impl Keymap {
    pub fn from_config(config: &KeysConfig) -> Self {
        let default = Self::default_keymap();
        Self {
            // ... existing fields ...
            new_action: config
                .new_action
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.new_action),
        }
    }
}
```

4. **Use in code** (`commands.rs` or UI code):
```rust
let keymap = get_keymap();
if keymap.new_action.matches(key) {
    // Handle action
}
```

---

## Available Colors

All color configurations support these values:

- `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `gray`
- `darkgray`, `lightred`, `lightgreen`, `lightyellow`, `lightblue`
- `lightmagenta`, `lightcyan`, `white`, `reset`

## Available Modifiers

Style modifiers can be combined with colors:

- `bold` - Bold text
- `dim` - Dimmed text
- `italic` - Italic text
- `underlined` - Underlined text
- `slow_blink` - Slow blinking
- `rapid_blink` - Rapid blinking
- `hidden` - Hidden text
- `crossed_out` - Strikethrough

### Format Examples

```toml
# Simple color
cursor = "yellow"

# Color with modifier
weekday = "bold"
weekend = "red bold"

# Multiple modifiers (if supported)
selected = "blue bold underlined"
```

---

## Key Binding Format

### Supported Formats

| Format | Example | Description |
|--------|---------|-------------|
| Single key | `"a"`, `"j"`, `"1"` | Basic key |
| Uppercase (shift) | `"A"`, `"N"` | Shift + key |
| Control modifier | `"ctrl+s"`, `"ctrl+c"` | Control + key |
| Combined | `"ctrl+shift+n"` | Multiple modifiers |

### Parsing Rules

- Case-insensitive for key portion
- Modifiers: `ctrl`, `shift`, `control`
- Separator: `+` (plus sign)
- Single character keys only

---

## Implementation Guidelines

### For New Features

When implementing any new feature that involves:

1. **Visual Styling** → Add to `theme.editor` or `theme.calendar`
2. **User Interaction** → Add to appropriate keymap category
3. **UI Components** → Consider both style and keymap needs

### Code Review Checklist

Before merging new features, verify:

- [ ] All hardcoded styles are moved to theme config
- [ ] All keyboard shortcuts are configurable
- [ ] Default values are documented in USAGE.md
- [ ] Config structure follows existing patterns
- [ ] `parse_style()` and `parse_key_binding()` are used consistently

### Testing Configuration

1. Test with default config (no config file)
2. Test with partial config (override only some values)
3. Test with full config (override all values)
4. Test with invalid config (should fall back to defaults)

---

## Future Enhancements

### Potential Additions

- [ ] **View-specific themes**: Different colors for different views
- [ ] **Conditional styling**: Style based on task state (overdue, high priority)
- [ ] **Keymap profiles**: Predefined keymap sets (vim, emacs, default)
- [ ] **Mouse support**: Configurable mouse bindings
- [ ] **Layout customization**: Configurable UI layout ratios

### Migration Strategy

When adding new config options:

1. Make all new fields `Option<T>` with `#[serde(default)]`
2. Provide sensible defaults in `Default` impl
3. Document in USAGE.md with default values
4. Maintain backward compatibility

---

## Related Documentation

- **[USAGE.md](../USAGE.md)**: User-facing configuration guide
- **[SPEC.md](../SPEC.md)**: Architecture specification
- **[PROJECT_OVERVIEW.md](./PROJECT_OVERVIEW.md)**: Project structure overview

---

*Last updated: March 30, 2026*
*Keep this document updated as new configurable options are added*
