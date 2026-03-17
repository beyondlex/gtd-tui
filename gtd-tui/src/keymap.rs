use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::KeysConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyBinding {
    pub ctrl: bool,
    pub shift: bool,
    pub key: char,
}

impl KeyBinding {
    pub fn matches(&self, event: KeyEvent) -> bool {
        let event_char = match event.code {
            KeyCode::Char(ch) => ch,
            _ => return false,
        };
        let shift_pressed = event.modifiers.contains(KeyModifiers::SHIFT);
        let shift_matches = if self.shift {
            shift_pressed || event_char.is_ascii_uppercase()
        } else {
            !shift_pressed && !event_char.is_ascii_uppercase()
        };

        event_char.to_ascii_lowercase() == self.key
            && event.modifiers.contains(KeyModifiers::CONTROL) == self.ctrl
            && shift_matches
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NavigationKeys {
    pub quit: KeyBinding,
    pub refresh: KeyBinding,
    pub view_inbox: KeyBinding,
    pub view_today: KeyBinding,
    pub view_upcoming: KeyBinding,
    pub view_anytime: KeyBinding,
    pub view_someday: KeyBinding,
    pub select_next: KeyBinding,
    pub select_prev: KeyBinding,
    pub select_first: KeyBinding,
    pub select_last: KeyBinding,
}

impl NavigationKeys {
    pub fn default_keymap() -> Self {
        Self {
            quit: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'q',
            },
            refresh: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'r',
            },
            view_inbox: KeyBinding {
                ctrl: false,
                shift: false,
                key: '1',
            },
            view_today: KeyBinding {
                ctrl: false,
                shift: false,
                key: '2',
            },
            view_upcoming: KeyBinding {
                ctrl: false,
                shift: false,
                key: '3',
            },
            view_anytime: KeyBinding {
                ctrl: false,
                shift: false,
                key: '4',
            },
            view_someday: KeyBinding {
                ctrl: false,
                shift: false,
                key: '5',
            },
            select_next: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'j',
            },
            select_prev: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'k',
            },
            select_first: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'g',
            },
            select_last: KeyBinding {
                ctrl: false,
                shift: true,
                key: 'g',
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TaskKeys {
    pub new_task: KeyBinding,
    pub edit_task: KeyBinding,
    pub toggle_task: KeyBinding,
    pub delete: KeyBinding,
    pub new_item_above: KeyBinding,
    pub new_item_below: KeyBinding,
    pub move_item_up: KeyBinding,
    pub move_item_down: KeyBinding,
    pub save_edit: KeyBinding,
}

impl TaskKeys {
    pub fn default_keymap() -> Self {
        Self {
            new_task: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'n',
            },
            edit_task: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'l',
            },
            toggle_task: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'x',
            },
            delete: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'd',
            },
            new_item_above: KeyBinding {
                ctrl: false,
                shift: true,
                key: 'o',
            },
            new_item_below: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'o',
            },
            move_item_up: KeyBinding {
                ctrl: false,
                shift: true,
                key: 'k',
            },
            move_item_down: KeyBinding {
                ctrl: false,
                shift: true,
                key: 'j',
            },
            save_edit: KeyBinding {
                ctrl: true,
                shift: false,
                key: 's',
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EditorKeys {
    pub nav_up: KeyBinding,
    pub next_focus: KeyBinding,
    pub prev_focus: KeyBinding,
    pub checklist_edit_toggle: KeyBinding,
    pub cancel_edit: KeyBinding,
}

impl EditorKeys {
    pub fn default_keymap() -> Self {
        Self {
            nav_up: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'q',
            },
            next_focus: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'j',
            },
            prev_focus: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'k',
            },
            checklist_edit_toggle: KeyBinding {
                ctrl: true,
                shift: false,
                key: 'e',
            },
            cancel_edit: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'q',
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DateKeys {
    pub prev_day: KeyBinding,
    pub next_day: KeyBinding,
    pub prev_day_in_edit: KeyBinding,
    pub next_day_in_edit: KeyBinding,
    pub prev_week: KeyBinding,
    pub next_week: KeyBinding,
    pub edit_mode: KeyBinding,
    pub prev_month: KeyBinding,
    pub next_month: KeyBinding,
    pub today: KeyBinding,
    pub tomorrow: KeyBinding,
}

impl DateKeys {
    pub fn default_keymap() -> Self {
        Self {
            prev_day: KeyBinding {
                ctrl: true,
                shift: false,
                key: 'h',
            },
            next_day: KeyBinding {
                ctrl: true,
                shift: false,
                key: 'l',
            },
            prev_day_in_edit: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'h',
            },
            next_day_in_edit: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'l',
            },
            prev_week: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'k',
            },
            next_week: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'j',
            },
            edit_mode: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'l',
            },
            prev_month: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'p',
            },
            next_month: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'n',
            },
            today: KeyBinding {
                ctrl: false,
                shift: false,
                key: 't',
            },
            tomorrow: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'm',
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ChecklistKeys {
    pub toggle: KeyBinding,
    pub add: KeyBinding,
    pub next: KeyBinding,
    pub prev: KeyBinding,
}

impl ChecklistKeys {
    pub fn default_keymap() -> Self {
        Self {
            toggle: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'x',
            },
            add: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'l',
            },
            next: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'j',
            },
            prev: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'k',
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Keymap {
    pub navigation: NavigationKeys,
    pub task: TaskKeys,
    pub editor: EditorKeys,
    pub date: DateKeys,
    pub checklist: ChecklistKeys,
}

impl Keymap {
    pub fn default_keymap() -> Self {
        Self {
            navigation: NavigationKeys::default_keymap(),
            task: TaskKeys::default_keymap(),
            editor: EditorKeys::default_keymap(),
            date: DateKeys::default_keymap(),
            checklist: ChecklistKeys::default_keymap(),
        }
    }

    pub fn from_config(config: &KeysConfig) -> Self {
        let default = Self::default_keymap();
        Self {
            navigation: NavigationKeys {
                quit: config
                    .quit
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.navigation.quit),
                refresh: config
                    .refresh
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.navigation.refresh),
                view_inbox: config
                    .view_inbox
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.navigation.view_inbox),
                view_today: config
                    .view_today
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.navigation.view_today),
                view_upcoming: config
                    .view_upcoming
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.navigation.view_upcoming),
                view_anytime: config
                    .view_anytime
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.navigation.view_anytime),
                view_someday: config
                    .view_someday
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.navigation.view_someday),
                select_next: config
                    .select_next
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.navigation.select_next),
                select_prev: config
                    .select_prev
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.navigation.select_prev),
                select_first: config
                    .select_first
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.navigation.select_first),
                select_last: config
                    .select_last
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.navigation.select_last),
            },
            task: TaskKeys {
                new_task: config
                    .new_task
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.task.new_task),
                edit_task: config
                    .edit_task
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.task.edit_task),
                toggle_task: config
                    .toggle_task
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.task.toggle_task),
                delete: default.task.delete,
                new_item_above: config
                    .new_item_above
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.task.new_item_above),
                new_item_below: config
                    .new_item_below
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.task.new_item_below),
                move_item_up: config
                    .move_item_up
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.task.move_item_up),
                move_item_down: config
                    .move_item_down
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.task.move_item_down),
                save_edit: config
                    .save_edit
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.task.save_edit),
            },
            editor: EditorKeys {
                nav_up: config
                    .cancel_edit
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.editor.nav_up),
                next_focus: config
                    .next_focus
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.editor.next_focus),
                prev_focus: config
                    .prev_focus
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.editor.prev_focus),
                checklist_edit_toggle: config
                    .checklist_edit_toggle
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.editor.checklist_edit_toggle),
                cancel_edit: config
                    .cancel_edit
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.editor.cancel_edit),
            },
            date: DateKeys {
                prev_day: config
                    .date_prev_day
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.date.prev_day),
                next_day: config
                    .date_next_day
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.date.next_day),
                prev_day_in_edit: config
                    .date_prev_day
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.date.prev_day_in_edit),
                next_day_in_edit: config
                    .date_next_day
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.date.next_day_in_edit),
                prev_week: config
                    .date_prev_week
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.date.prev_week),
                next_week: config
                    .date_next_week
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.date.next_week),
                edit_mode: config
                    .date_edit_mode
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.date.edit_mode),
                prev_month: config
                    .date_prev_month
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.date.prev_month),
                next_month: config
                    .date_next_month
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.date.next_month),
                today: config
                    .date_today
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.date.today),
                tomorrow: config
                    .date_tomorrow
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.date.tomorrow),
            },
            checklist: ChecklistKeys {
                toggle: config
                    .checklist_toggle
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.checklist.toggle),
                add: config
                    .checklist_add
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.checklist.add),
                next: config
                    .checklist_next
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.checklist.next),
                prev: config
                    .checklist_prev
                    .as_deref()
                    .and_then(parse_key_binding)
                    .unwrap_or(default.checklist.prev),
            },
        }
    }
}

pub fn parse_key_binding(value: &str) -> Option<KeyBinding> {
    let mut ctrl = false;
    let mut shift = false;
    let mut key: Option<char> = None;
    for part in value.split('+') {
        let token = part.trim();
        let lowered = token.to_lowercase();
        if lowered == "ctrl" || lowered == "control" {
            ctrl = true;
            continue;
        }
        if lowered == "shift" {
            shift = true;
            continue;
        }
        if token.chars().count() == 1 {
            let ch = token.chars().next()?;
            if ch.is_ascii_uppercase() {
                shift = true;
            }
            key = Some(ch.to_ascii_lowercase());
        }
    }
    key.map(|key| KeyBinding { ctrl, shift, key })
}
