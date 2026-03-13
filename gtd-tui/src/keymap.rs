use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::KeysConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy)]
pub struct Keymap {
    pub quit: KeyBinding,
    pub view_inbox: KeyBinding,
    pub view_today: KeyBinding,
    pub view_upcoming: KeyBinding,
    pub view_anytime: KeyBinding,
    pub view_someday: KeyBinding,
    pub select_next: KeyBinding,
    pub select_prev: KeyBinding,
    pub new_task: KeyBinding,
    pub edit_task: KeyBinding,
    pub toggle_task: KeyBinding,
    pub refresh: KeyBinding,
    pub save_edit: KeyBinding,
    pub nav_up: KeyBinding,
    pub next_focus: KeyBinding,
    pub prev_focus: KeyBinding,
    pub checklist_edit_toggle: KeyBinding,
    pub date_prev_day: KeyBinding,
    pub date_next_day: KeyBinding,
    pub date_prev_day_in_edit_mode: KeyBinding,
    pub date_next_day_in_edit_mode: KeyBinding,
    pub date_prev_week: KeyBinding,
    pub date_next_week: KeyBinding,
    pub date_edit_mode: KeyBinding,
    pub date_prev_month: KeyBinding,
    pub date_next_month: KeyBinding,
    pub date_today: KeyBinding,
    pub date_tomorrow: KeyBinding,
    pub new_item_above: KeyBinding,
    pub new_item_below: KeyBinding,
    pub move_item_up: KeyBinding,
    pub move_item_down: KeyBinding,
    pub checklist_toggle: KeyBinding,
    pub checklist_add: KeyBinding,
    pub checklist_next: KeyBinding,
    pub checklist_prev: KeyBinding,
}

impl Default for Keymap {
    fn default() -> Self {
        Self::default_keymap()
    }
}

impl Keymap {
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
            save_edit: KeyBinding {
                ctrl: true,
                shift: false,
                key: 's',
            },
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
            date_prev_day: KeyBinding {
                ctrl: true,
                shift: false,
                key: 'h',
            },
            date_next_day: KeyBinding {
                ctrl: true,
                shift: false,
                key: 'l',
            },
            date_prev_day_in_edit_mode: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'h',
            },
            date_next_day_in_edit_mode: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'l',
            },
            date_prev_week: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'k',
            },
            date_next_week: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'j',
            },
            date_edit_mode: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'l',
            },
            date_prev_month: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'p',
            },
            date_next_month: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'n',
            },
            date_today: KeyBinding {
                ctrl: false,
                shift: false,
                key: 't',
            },
            date_tomorrow: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'm',
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
            checklist_toggle: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'x',
            },
            checklist_add: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'l',
            },
            checklist_next: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'j',
            },
            checklist_prev: KeyBinding {
                ctrl: false,
                shift: false,
                key: 'k',
            },
        }
    }

    pub fn from_config(config: &KeysConfig) -> Self {
        let default = Self::default_keymap();
        Self {
            quit: config
                .quit
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.quit),
            view_inbox: config
                .view_inbox
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.view_inbox),
            view_today: config
                .view_today
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.view_today),
            view_upcoming: config
                .view_upcoming
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.view_upcoming),
            view_anytime: config
                .view_anytime
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.view_anytime),
            view_someday: config
                .view_someday
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.view_someday),
            select_next: config
                .select_next
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.select_next),
            select_prev: config
                .select_prev
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.select_prev),
            new_task: config
                .new_task
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.new_task),
            edit_task: config
                .edit_task
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.edit_task),
            toggle_task: config
                .toggle_task
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.toggle_task),
            refresh: config
                .refresh
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.refresh),
            save_edit: config
                .save_edit
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.save_edit),
            nav_up: config
                .cancel_edit
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.nav_up),
            next_focus: config
                .next_focus
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.next_focus),
            new_item_above: config
                .new_item_above
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.new_item_above),
            new_item_below: config
                .new_item_below
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.new_item_below),
            move_item_up: config
                .move_item_up
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.move_item_up),
            move_item_down: config
                .move_item_down
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.move_item_down),
            prev_focus: config
                .prev_focus
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.prev_focus),
            checklist_edit_toggle: config
                .checklist_edit_toggle
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.checklist_edit_toggle),
            date_prev_day: config
                .date_prev_day
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_prev_day),
            date_next_day: config
                .date_next_day
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_next_day),
            date_prev_day_in_edit_mode: config
                .date_prev_day
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_prev_day_in_edit_mode),
            date_next_day_in_edit_mode: config
                .date_next_day
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_next_day_in_edit_mode),
            date_prev_week: config
                .date_prev_week
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_prev_week),
            date_next_week: config
                .date_next_week
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_next_week),
            date_edit_mode: config
                .date_edit_mode
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_edit_mode),
            date_prev_month: config
                .date_prev_month
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_prev_month),
            date_next_month: config
                .date_next_month
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_next_month),
            date_today: config
                .date_today
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_today),
            date_tomorrow: config
                .date_tomorrow
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.date_tomorrow),
            checklist_toggle: config
                .checklist_toggle
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.checklist_toggle),
            checklist_add: config
                .checklist_add
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.checklist_add),
            checklist_next: config
                .checklist_next
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.checklist_next),
            checklist_prev: config
                .checklist_prev
                .as_deref()
                .and_then(parse_key_binding)
                .unwrap_or(default.checklist_prev),
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
