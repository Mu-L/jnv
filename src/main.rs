use std::{
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};

use anyhow::anyhow;
use clap::Parser;
use config::{
    Config, ConfigFromFile, EditorConfig, EditorConfigFromFile, JsonTheme, JsonThemeFromFile,
    Keybinds, KeybindsFromFile,
};
use crossterm::style::Attribute;
use promkit::{
    jsonz::format::RowFormatter,
    listbox::{self, Listbox},
    text_editor,
};

mod editor;
use editor::Editor;
mod config;
mod json;
use json::JsonStreamProvider;
mod processor;
use processor::{
    init::ViewInitializer, monitor::ContextMonitor, spinner::SpinnerSpawner, Context, Processor,
    ViewProvider, Visualizer,
};
mod prompt;
mod render;
use render::{PaneIndex, Renderer, EMPTY_PANE};
mod search;
use search::{IncrementalSearcher, SearchProvider};

/// JSON navigator and interactive filter leveraging jq
#[derive(Parser)]
#[command(
    name = "jnv",
    version,
    help_template = "
{about}

Usage: {usage}

Examples:
- Read from a file:
        {bin} data.json

- Read from standard input:
        cat data.json | {bin}

Arguments:
{positionals}

Options:
{options}
"
)]
pub struct Args {
    /// Optional path to a JSON file.
    /// If not provided or if "-" is specified,
    /// reads from standard input.
    pub input: Option<PathBuf>,

    #[arg(
        short = 'e',
        long = "edit-mode",
        default_value = "insert",
        value_parser = edit_mode_validator,
        help = "Edit mode for the interface ('insert' or 'overwrite').",
        long_help = r#"
        Specifies the edit mode for the interface.
        Acceptable values are "insert" or "overwrite".
        - "insert" inserts a new input at the cursor's position.
        - "overwrite" mode replaces existing characters with new input at the cursor's position.
        "#,
    )]
    pub edit_mode: text_editor::Mode,

    #[arg(
        short = 'n',
        long = "no-hint",
        help = "Disables the display of hints.",
        long_help = "
        When this option is enabled, it prevents the display of
        hints that typically guide or offer suggestions to the user.
        "
    )]
    pub no_hint: bool,

    #[arg(
        short = 'c',
        long = "config",
        help = "Path to the configuration file.",
        long_help = "
        Specifies the path to the configuration file.
        "
    )]
    pub config_file: Option<PathBuf>,

    #[arg(
        long = "max-streams",
        help = "Maximum number of JSON streams to display",
        long_help = "
        Sets the maximum number of JSON streams to load and display.
        Limiting this value improves performance for large datasets.
        If not set, all streams will be displayed.
        "
    )]
    pub max_streams: Option<usize>,

    #[arg(
        long = "suggestions",
        default_value = "3",
        help = "Number of autocomplete suggestions to show",
        long_help = "
        Sets the number of autocomplete suggestions displayed during incremental search.
        Higher values show more suggestions but may occupy more screen space.
        Adjust this value based on your screen size and preference.
        "
    )]
    pub suggestions: usize,
}

fn edit_mode_validator(val: &str) -> anyhow::Result<text_editor::Mode> {
    match val {
        "insert" | "" => Ok(text_editor::Mode::Insert),
        "overwrite" => Ok(text_editor::Mode::Overwrite),
        _ => Err(anyhow!("edit-mode must be 'insert' or 'overwrite'")),
    }
}

/// Parses the input based on the provided arguments.
///
/// This function reads input data from either a specified file or standard input.
/// If the `input` argument is `None`, or if it is a path
/// that equals "-", data is read from standard input.
/// Otherwise, the function attempts to open and
/// read from the file specified in the `input` argument.
fn parse_input(args: &Args) -> anyhow::Result<String> {
    let mut ret = String::new();

    match &args.input {
        None => {
            io::stdin().read_to_string(&mut ret)?;
        }
        Some(path) => {
            if path == &PathBuf::from("-") {
                io::stdin().read_to_string(&mut ret)?;
            } else {
                File::open(path)?.read_to_string(&mut ret)?;
            }
        }
    }

    Ok(ret)
}

/// Ensures the configuration file exists, creating it with default settings if it doesn't
///
/// If the file already exists, returns Ok.
/// If the file doesn't exist, writes the default configuration in TOML format.
/// Returns an error if file creation fails.
fn ensure_file_exists(path: &PathBuf, default_config: &Config) -> anyhow::Result<()> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow!("Failed to create directory: {}", e))?;
    }

    std::fs::File::create(path)?.write_all(toml::to_string_pretty(default_config)?.as_bytes())?;

    Ok(())
}

/// Determines the configuration file path with the following precedence:
/// 1. The provided `config_path` argument, if it exists.
/// 2. The default configuration file path in the user's configuration directory.
///
/// If the configuration file does not exist, it will be created.
/// Returns an error if the file creation fails.
fn determine_config_file(
    config_path: Option<PathBuf>,
    default_config: &Config,
) -> anyhow::Result<PathBuf> {
    // If a custom path is provided
    if let Some(path) = config_path {
        ensure_file_exists(&path, default_config)?;
        return Ok(path);
    }

    // Use the default path
    let default_path = dirs::config_dir()
        .ok_or_else(|| anyhow!("Failed to determine the configuration directory"))?
        .join("jnv")
        .join("config.toml");

    ensure_file_exists(&default_path, default_config)?;
    Ok(default_path)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let input = parse_input(&args)?;

    let (mut config, mut keybinds, mut editor_config, mut json_theme) = (
        Config::default(),
        Keybinds::default(),
        EditorConfig::default(),
        JsonTheme::default(),
    );
    if let Ok(config_file) = determine_config_file(args.config_file, &config) {
        // Note that the configuration file absolutely exists.
        let content = std::fs::read_to_string(&config_file)?;
        let config_from_file = ConfigFromFile::load_from(&content)?;
        let keybinds_from_file = KeybindsFromFile::load_from(&content)?;
        let editor_config_from_file = EditorConfigFromFile::load_from(&content)?;
        let json_theme_from_file = JsonThemeFromFile::load_from(&content)?;
        config.patch_with(config_from_file);
        keybinds.patch_with(keybinds_from_file);
        editor_config.patch_with(editor_config_from_file);
        json_theme.patch_with(json_theme_from_file);
    }

    let config::Config {
        search_result_chunk_size,
        query_debounce_duration,
        resize_debounce_duration,
        search_load_chunk_size,
        active_item_style,
        inactive_item_style,
        spin_duration,
    } = config;

    let listbox_state = listbox::State {
        listbox: Listbox::default(),
        cursor: String::from("❯ "),
        active_item_style: Some(active_item_style),
        inactive_item_style: Some(inactive_item_style),
        lines: Some(args.suggestions),
    };

    let searcher = IncrementalSearcher::new(listbox_state, search_result_chunk_size);

    let text_editor_state = text_editor::State {
        texteditor: Default::default(),
        history: Default::default(),
        prefix: editor_config.theme_on_focus.prefix.clone(),
        mask: Default::default(),
        prefix_style: editor_config.theme_on_focus.prefix_style,
        active_char_style: editor_config.theme_on_focus.active_char_style,
        inactive_char_style: editor_config.theme_on_focus.inactive_char_style,
        edit_mode: args.edit_mode,
        word_break_chars: editor_config.word_break_chars,
        lines: Default::default(),
    };

    let provider = &mut JsonStreamProvider::new(
        RowFormatter {
            curly_brackets_style: json_theme.curly_brackets_style,
            square_brackets_style: json_theme.square_brackets_style,
            key_style: json_theme.key_style,
            string_value_style: json_theme.string_value_style,
            number_value_style: json_theme.number_value_style,
            boolean_value_style: json_theme.boolean_value_style,
            null_value_style: json_theme.null_value_style,
            active_item_attribute: Attribute::Bold,
            inactive_item_attribute: Attribute::Dim,
            indent: json_theme.indent,
        },
        args.max_streams,
    );

    let item = Box::leak(input.into_boxed_str());

    let loading_suggestions_task = searcher.spawn_load_task(provider, item, search_load_chunk_size);

    let editor = Editor::new(
        text_editor_state,
        searcher,
        editor_config.theme_on_focus,
        editor_config.theme_on_defocus,
        keybinds,
    );

    prompt::run(
        item,
        spin_duration,
        query_debounce_duration,
        resize_debounce_duration,
        provider,
        editor,
        loading_suggestions_task,
        args.no_hint,
    )
    .await?;

    Ok(())
}
