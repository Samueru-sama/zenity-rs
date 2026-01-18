//! rask - Display simple GUI dialogs from the command line.

use std::process::ExitCode;

use lexopt::prelude::*;

use rask::{ButtonPreset, EntryResult, Icon, entry, message, password};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code as u8),
        Err(e) => {
            eprintln!("rask: {e}");
            ExitCode::from(100)
        }
    }
}

fn run() -> Result<i32, Box<dyn std::error::Error>> {
    let mut parser = lexopt::Parser::from_env();

    // Global options
    let mut title = String::new();
    let mut text = String::new();
    let mut entry_text = String::new();

    // Dialog type
    let mut dialog_type: Option<DialogType> = None;

    while let Some(arg) = parser.next()? {
        match arg {
            Long("help") | Short('h') => {
                print_help();
                return Ok(0);
            }
            Long("version") => {
                println!("rask {VERSION}");
                return Ok(0);
            }

            // Dialog types
            Long("info") => dialog_type = Some(DialogType::Info),
            Long("warning") => dialog_type = Some(DialogType::Warning),
            Long("error") => dialog_type = Some(DialogType::Error),
            Long("question") => dialog_type = Some(DialogType::Question),
            Long("entry") => dialog_type = Some(DialogType::Entry),
            Long("password") => dialog_type = Some(DialogType::Password),

            // Common options
            Long("title") => title = parser.value()?.string()?,
            Long("text") => text = parser.value()?.string()?,
            Long("entry-text") => entry_text = parser.value()?.string()?,
            Long("hide-text") => {
                // If --hide-text is specified with --entry, treat as password mode
                if dialog_type == Some(DialogType::Entry) {
                    dialog_type = Some(DialogType::Password);
                }
            }

            // TODO: Add more dialog types
            Long("progress") | Long("file-selection") | Long("list") | Long("calendar") => {
                return Err(format!("dialog type not yet implemented: {:?}", arg).into());
            }

            Value(val) => {
                // Positional argument - treat as text if text is empty
                if text.is_empty() {
                    text = val.string()?;
                }
            }

            _ => return Err(arg.unexpected().into()),
        }
    }

    // Default to info if no dialog type specified
    let dialog_type = dialog_type.unwrap_or(DialogType::Info);

    // Build and show the dialog
    match dialog_type {
        DialogType::Info => {
            let result = message()
                .title(if title.is_empty() { "Information" } else { &title })
                .text(&text)
                .icon(Icon::Info)
                .buttons(ButtonPreset::Ok)
                .show()?;
            Ok(result.exit_code())
        }
        DialogType::Warning => {
            let result = message()
                .title(if title.is_empty() { "Warning" } else { &title })
                .text(&text)
                .icon(Icon::Warning)
                .buttons(ButtonPreset::Ok)
                .show()?;
            Ok(result.exit_code())
        }
        DialogType::Error => {
            let result = message()
                .title(if title.is_empty() { "Error" } else { &title })
                .text(&text)
                .icon(Icon::Error)
                .buttons(ButtonPreset::Ok)
                .show()?;
            Ok(result.exit_code())
        }
        DialogType::Question => {
            let result = message()
                .title(if title.is_empty() { "Question" } else { &title })
                .text(&text)
                .icon(Icon::Question)
                .buttons(ButtonPreset::YesNo)
                .show()?;
            Ok(result.exit_code())
        }
        DialogType::Entry => {
            let result = entry()
                .title(if title.is_empty() { "Entry" } else { &title })
                .text(&text)
                .entry_text(&entry_text)
                .show()?;
            handle_entry_result(result)
        }
        DialogType::Password => {
            let result = password()
                .title(if title.is_empty() { "Password" } else { &title })
                .text(&text)
                .show()?;
            handle_entry_result(result)
        }
    }
}

fn handle_entry_result(result: EntryResult) -> Result<i32, Box<dyn std::error::Error>> {
    match result {
        EntryResult::Text(text) => {
            println!("{text}");
            Ok(0)
        }
        EntryResult::Cancelled => Ok(1),
        EntryResult::Closed => Ok(255),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DialogType {
    Info,
    Warning,
    Error,
    Question,
    Entry,
    Password,
}

fn print_help() {
    println!(
        r#"rask {VERSION} - Display simple GUI dialogs from the command line

USAGE:
    rask [OPTIONS] --<dialog-type> [TEXT]

DIALOG TYPES:
    --info              Display an information dialog
    --warning           Display a warning dialog
    --error             Display an error dialog
    --question          Display a question dialog (Yes/No)
    --entry             Display a text entry dialog
    --password          Display a password entry dialog
    --progress          Display a progress dialog (not yet implemented)
    --file-selection    Display a file selection dialog (not yet implemented)
    --list              Display a list dialog (not yet implemented)
    --calendar          Display a calendar dialog (not yet implemented)

OPTIONS:
    --title=TEXT        Set the dialog title
    --text=TEXT         Set the dialog text/prompt
    --entry-text=TEXT   Set default text for entry dialog
    --hide-text         Hide entered text (password mode)
    -h, --help          Print this help message
    --version           Print version information

EXAMPLES:
    rask --info --text="Operation completed"
    rask --question --text="Do you want to continue?"
    rask --entry --text="Enter your name:" --entry-text="John"
    rask --password --text="Enter password:"

EXIT CODES:
    0   OK/Yes button clicked, or text entered
    1   Cancel/No button clicked
    255 Dialog was closed
    100 Error occurred
"#
    );
}
