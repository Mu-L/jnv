use std::collections::HashSet;

use promkit_widgets::{
    core::crossterm::{
        event::{KeyCode, KeyModifiers},
        style::{Attribute, Attributes, Color, ContentStyle},
    },
    text_editor::Mode,
};
use serde::{Deserialize, Serialize};
use tokio::time::Duration;

mod content_style;
use content_style::content_style_serde;
mod duration;
use duration::duration_serde;
pub mod event;
use event::{EventDef, EventDefSet, KeyEventDef};
mod text_editor;
use text_editor::text_editor_mode_serde;

#[derive(Serialize, Deserialize)]
pub struct EditorConfig {
    pub theme_on_focus: EditorTheme,
    pub theme_on_defocus: EditorTheme,
    #[serde(with = "text_editor_mode_serde")]
    pub mode: Mode,
    pub word_break_chars: HashSet<char>,
}

#[derive(Serialize, Deserialize)]
pub struct EditorTheme {
    pub prefix: String,

    #[serde(with = "content_style_serde")]
    pub prefix_style: ContentStyle,

    #[serde(with = "content_style_serde")]
    pub active_char_style: ContentStyle,

    #[serde(with = "content_style_serde")]
    pub inactive_char_style: ContentStyle,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            theme_on_focus: EditorTheme {
                prefix: String::from("❯❯ "),
                prefix_style: ContentStyle {
                    foreground_color: Some(Color::Blue),
                    ..Default::default()
                },
                active_char_style: ContentStyle {
                    background_color: Some(Color::Magenta),
                    ..Default::default()
                },
                inactive_char_style: ContentStyle::default(),
            },
            theme_on_defocus: EditorTheme {
                prefix: String::from("▼ "),
                prefix_style: ContentStyle {
                    foreground_color: Some(Color::Blue),
                    attributes: Attributes::from(Attribute::Dim),
                    ..Default::default()
                },
                active_char_style: ContentStyle {
                    attributes: Attributes::from(Attribute::Dim),
                    ..Default::default()
                },
                inactive_char_style: ContentStyle {
                    attributes: Attributes::from(Attribute::Dim),
                    ..Default::default()
                },
            },
            mode: Mode::Insert,
            word_break_chars: HashSet::from(['.', '|', '(', ')', '[', ']']),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct JsonConfig {
    pub max_streams: Option<usize>,
    pub theme: JsonTheme,
}

#[derive(Serialize, Deserialize)]
pub struct JsonTheme {
    pub indent: usize,

    #[serde(with = "content_style_serde")]
    pub curly_brackets_style: ContentStyle,

    #[serde(with = "content_style_serde")]
    pub square_brackets_style: ContentStyle,

    #[serde(with = "content_style_serde")]
    pub key_style: ContentStyle,

    #[serde(with = "content_style_serde")]
    pub string_value_style: ContentStyle,

    #[serde(with = "content_style_serde")]
    pub number_value_style: ContentStyle,

    #[serde(with = "content_style_serde")]
    pub boolean_value_style: ContentStyle,

    #[serde(with = "content_style_serde")]
    pub null_value_style: ContentStyle,
}

impl Default for JsonConfig {
    fn default() -> Self {
        Self {
            max_streams: None,
            theme: JsonTheme {
                indent: 2,
                curly_brackets_style: ContentStyle {
                    attributes: Attributes::from(Attribute::Bold),
                    ..Default::default()
                },
                square_brackets_style: ContentStyle {
                    attributes: Attributes::from(Attribute::Bold),
                    ..Default::default()
                },
                key_style: ContentStyle {
                    foreground_color: Some(Color::Cyan),
                    ..Default::default()
                },
                string_value_style: ContentStyle {
                    foreground_color: Some(Color::Green),
                    ..Default::default()
                },
                number_value_style: ContentStyle::default(),
                boolean_value_style: ContentStyle::default(),
                null_value_style: ContentStyle {
                    foreground_color: Some(Color::Grey),
                    ..Default::default()
                },
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CompletionConfig {
    pub lines: Option<usize>,
    pub cursor: String,
    pub search_result_chunk_size: usize,
    pub search_load_chunk_size: usize,

    #[serde(with = "content_style_serde")]
    pub active_item_style: ContentStyle,

    #[serde(with = "content_style_serde")]
    pub inactive_item_style: ContentStyle,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            lines: Some(3),
            cursor: String::from("❯ "),
            search_result_chunk_size: 100,
            search_load_chunk_size: 50000,
            active_item_style: ContentStyle {
                foreground_color: Some(Color::Grey),
                background_color: Some(Color::Yellow),
                ..Default::default()
            },
            inactive_item_style: ContentStyle {
                foreground_color: Some(Color::Grey),
                ..Default::default()
            },
        }
    }
}

// TODO: remove Clone derive
#[derive(Clone, Serialize, Deserialize)]
pub struct Keybinds {
    pub exit: EventDefSet,
    pub copy_query: EventDefSet,
    pub copy_result: EventDefSet,
    pub switch_mode: EventDefSet,
    pub on_editor: EditorKeybinds,
    pub on_json_viewer: JsonViewerKeybinds,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EditorKeybinds {
    pub backward: EventDefSet,
    pub forward: EventDefSet,
    pub move_to_head: EventDefSet,
    pub move_to_tail: EventDefSet,
    pub move_to_previous_nearest: EventDefSet,
    pub move_to_next_nearest: EventDefSet,
    pub erase: EventDefSet,
    pub erase_all: EventDefSet,
    pub erase_to_previous_nearest: EventDefSet,
    pub erase_to_next_nearest: EventDefSet,
    pub completion: EventDefSet,
    pub on_completion: CompletionKeybinds,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CompletionKeybinds {
    pub up: EventDefSet,
    pub down: EventDefSet,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonViewerKeybinds {
    pub up: EventDefSet,
    pub down: EventDefSet,
    pub move_to_head: EventDefSet,
    pub move_to_tail: EventDefSet,
    pub toggle: EventDefSet,
    pub expand: EventDefSet,
    pub collapse: EventDefSet,
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            exit: EventDefSet::from(KeyEventDef::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            copy_query: EventDefSet::from(KeyEventDef::new(
                KeyCode::Char('q'),
                KeyModifiers::CONTROL,
            )),
            copy_result: EventDefSet::from(KeyEventDef::new(
                KeyCode::Char('o'),
                KeyModifiers::CONTROL,
            )),
            switch_mode: EventDefSet::from_iter([
                EventDef::Key(KeyEventDef::new(KeyCode::Down, KeyModifiers::SHIFT)),
                EventDef::Key(KeyEventDef::new(KeyCode::Up, KeyModifiers::SHIFT)),
            ]),
            on_editor: EditorKeybinds {
                backward: EventDefSet::from(KeyEventDef::new(KeyCode::Left, KeyModifiers::NONE)),
                forward: EventDefSet::from(KeyEventDef::new(KeyCode::Right, KeyModifiers::NONE)),
                move_to_head: EventDefSet::from(KeyEventDef::new(
                    KeyCode::Char('a'),
                    KeyModifiers::CONTROL,
                )),
                move_to_tail: EventDefSet::from(KeyEventDef::new(
                    KeyCode::Char('e'),
                    KeyModifiers::CONTROL,
                )),
                move_to_next_nearest: EventDefSet::from(KeyEventDef::new(
                    KeyCode::Char('f'),
                    KeyModifiers::ALT,
                )),
                move_to_previous_nearest: EventDefSet::from(KeyEventDef::new(
                    KeyCode::Char('b'),
                    KeyModifiers::ALT,
                )),
                erase: EventDefSet::from(KeyEventDef::new(KeyCode::Backspace, KeyModifiers::NONE)),
                erase_all: EventDefSet::from(KeyEventDef::new(
                    KeyCode::Char('u'),
                    KeyModifiers::CONTROL,
                )),
                erase_to_previous_nearest: EventDefSet::from(KeyEventDef::new(
                    KeyCode::Char('w'),
                    KeyModifiers::CONTROL,
                )),
                erase_to_next_nearest: EventDefSet::from(KeyEventDef::new(
                    KeyCode::Char('d'),
                    KeyModifiers::ALT,
                )),
                completion: EventDefSet::from(KeyEventDef::new(KeyCode::Tab, KeyModifiers::NONE)),
                on_completion: CompletionKeybinds {
                    up: EventDefSet::from(KeyEventDef::new(KeyCode::Up, KeyModifiers::NONE)),
                    down: EventDefSet::from_iter([
                        EventDef::Key(KeyEventDef::new(KeyCode::Tab, KeyModifiers::NONE)),
                        EventDef::Key(KeyEventDef::new(KeyCode::Down, KeyModifiers::NONE)),
                    ]),
                },
            },
            on_json_viewer: JsonViewerKeybinds {
                up: EventDefSet::from_iter([
                    EventDef::Key(KeyEventDef::new(KeyCode::Char('k'), KeyModifiers::CONTROL)),
                    EventDef::Key(KeyEventDef::new(KeyCode::Up, KeyModifiers::NONE)),
                ]),
                down: EventDefSet::from_iter([
                    EventDef::Key(KeyEventDef::new(KeyCode::Char('j'), KeyModifiers::CONTROL)),
                    EventDef::Key(KeyEventDef::new(KeyCode::Down, KeyModifiers::NONE)),
                ]),
                move_to_head: EventDefSet::from(KeyEventDef::new(
                    KeyCode::Char('l'),
                    KeyModifiers::CONTROL,
                )),
                move_to_tail: EventDefSet::from(KeyEventDef::new(
                    KeyCode::Char('h'),
                    KeyModifiers::CONTROL,
                )),
                toggle: EventDefSet::from(KeyEventDef::new(KeyCode::Enter, KeyModifiers::NONE)),
                expand: EventDefSet::from(KeyEventDef::new(
                    KeyCode::Char('p'),
                    KeyModifiers::CONTROL,
                )),
                collapse: EventDefSet::from(KeyEventDef::new(
                    KeyCode::Char('n'),
                    KeyModifiers::CONTROL,
                )),
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ReactivityControl {
    #[serde(with = "duration_serde")]
    pub query_debounce_duration: Duration,

    #[serde(with = "duration_serde")]
    pub resize_debounce_duration: Duration,

    #[serde(with = "duration_serde")]
    pub spin_duration: Duration,
}

impl Default for ReactivityControl {
    fn default() -> Self {
        Self {
            query_debounce_duration: Duration::from_millis(600),
            resize_debounce_duration: Duration::from_millis(200),
            spin_duration: Duration::from_millis(300),
        }
    }
}

/// Note that the config struct and the `.toml` configuration file are
/// managed separately because the current toml crate
/// does not readily support the following features:
///
/// - Preserve docstrings as comments in the `.toml` file
///   - https://github.com/toml-rs/toml/issues/376
/// - Output inline tables
///   - https://github.com/toml-rs/toml/issues/592
///
/// Also difficult to patch `Config` using only the items specified in the configuration file
/// (Premise: To address the complexity of configurations,
/// it assumes using a macro to avoid managing Option-wrapped structures on our side).s
///
/// The main challenge is that, for nested structs,
/// it is not able to wrap every leaf field with Option<>.
/// https://github.com/colin-kiegel/rust-derive-builder/issues/254
#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub no_hint: bool,
    pub reactivity_control: ReactivityControl,
    pub editor: EditorConfig,
    pub json: JsonConfig,
    pub completion: CompletionConfig,
    pub keybinds: Keybinds,
}

impl Config {
    pub fn load_from(content: &str) -> anyhow::Result<Self> {
        toml::from_str(content).map_err(Into::into)
    }
}
