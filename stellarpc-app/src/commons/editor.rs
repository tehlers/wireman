#![allow(clippy::module_name_repetitions)]
use arboard::Clipboard;
use crossterm::event::KeyEvent;
use lazy_static::lazy_static;
use std::sync::Mutex;
use tui_vim_editor::{buffer::mode::Mode, Buffer, Input};

lazy_static! {
    pub static ref CLIPBOARD: Mutex<Option<Clipboard>> = Mutex::new(Clipboard::new().ok());
}

/// Basic editor. Supports different modes, json formatting
/// and specifies commonly used key bindings.
#[derive(Clone)]
pub struct TextEditor {
    /// Buffer contains all the core functionality
    pub buffer: Buffer,

    /// The input register
    input: Input,

    /// Error buffer
    error: Option<ErrorKind>,

    // /// The block
    // block: Option<Block<'a>>,
    // style: Style,
    /// Whether the editor is focused.
    focus: bool,
}

impl Default for TextEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl TextEditor {
    /// Returns an empty editor
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            input: Input::default(),
            error: None,
            focus: false,
        }
    }

    pub fn focus(&mut self) {
        self.focus = true;
    }

    pub fn unfocus(&mut self) {
        self.focus = false;
    }

    /// Returns an empty editor
    pub fn from_str(text: &str) -> Self {
        let mut editor = Self::new();
        editor.set_text_raw(text);
        editor
    }

    /// Gets the editors content as raw text
    pub fn get_text_raw(&self) -> String {
        self.buffer
            .lines()
            .iter()
            .map(|x| x.into())
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Set the editors content from raw text
    pub fn set_text_raw(&mut self, text: &str) {
        self.buffer.clear();
        for line in text.lines() {
            self.buffer.push(line)
        }
    }

    /// Return the error
    pub fn get_error(&self) -> Option<ErrorKind> {
        self.error.clone()
    }

    /// Set the error
    pub fn set_error(&mut self, error: Option<ErrorKind>) {
        self.error = error;
    }

    /// Clear all text
    pub fn clear(&mut self) {
        self.buffer.clear()
    }

    /// Whether the editor is in insert mode
    pub fn insert_mode(&self) -> bool {
        self.buffer.mode == Mode::Insert
    }

    /// Paste text from clipboard to editor
    pub fn paste_from_clipboard(&mut self) {
        if let Ok(mut clipboard) = CLIPBOARD.lock() {
            if let Some(clipboard) = &mut *clipboard {
                if let Ok(text) = clipboard.get_text() {
                    for c in text.chars() {
                        self.insert_char(c);
                    }
                }
            }
        }
    }

    /// Yanks the full text
    pub fn yank(&self) {
        Self::yank_to_clipboard(&self.get_text_raw());
    }

    /// Yank text to clipboard
    pub fn yank_to_clipboard(text: &str) {
        if let Ok(mut clipboard) = CLIPBOARD.lock() {
            if let Some(clipboard) = &mut *clipboard {
                let _res = clipboard.set_text(text);
            }
        }
    }

    /// Insert a str at the current cursor position. Handles newlines.
    fn insert_char(&mut self, c: char) {
        self.buffer.insert_char(c)
    }

    /// Pretty formats the editors text. The error is stored
    /// internall in the error buffer.
    pub fn format_json(&mut self) {
        match pretty_format_json(&self.get_text_raw()) {
            Ok(pretty) => {
                self.set_text_raw(&pretty);
                self.error = None;
            }
            Err(err) => self.error = Some(err),
        }
    }

    /// Returns if the editor is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.lines().is_empty()
    }

    /// Key bindings in normal mode
    pub fn on_key(&mut self, key: KeyEvent) {
        match key.code {
            _ => {
                self.input.on_key(key, &mut self.buffer);
            } // KeyCode::Char('i') => self.mode = EditorMode::Insert,
              // KeyCode::Char('a') => {
              //     self.mode = EditorMode::Insert;
              //     self.buffer.move_cursor(CursorMove::Forward);
              // }
              // // Cursor movement
              // KeyCode::Down | KeyCode::Char('j') => self.buffer.move_cursor(CursorMove::Down),
              // KeyCode::Up | KeyCode::Char('k') => self.buffer.move_cursor(CursorMove::Up),
              // KeyCode::Left | KeyCode::Char('h') => self.buffer.move_cursor(CursorMove::Back),
              // KeyCode::Right | KeyCode::Char('l') => self.buffer.move_cursor(CursorMove::Forward),
              // KeyCode::Char('w') => self.buffer.move_cursor(CursorMove::WordForward),
              // KeyCode::Char('b') => self.buffer.move_cursor(CursorMove::WordBack),
              // KeyCode::Char('J') => self.buffer.move_cursor(CursorMove::End),
              // KeyCode::Char('H') => self.buffer.move_cursor(CursorMove::Head),
              // // Delete
              // KeyCode::Char('x') => {
              //     self.buffer.delete_next_char();
              // }
              // KeyCode::Char('d') => {
              //     self.buffer.delete_line_by_end();
              // }
              // KeyCode::Char('D') => {
              //     self.buffer.delete_line_by_head();
              // }
              // // Undo
              // KeyCode::Char('u') => {
              //     self.buffer.undo();
              // }
              // KeyCode::Char('r') => {
              //     self.buffer.redo();
              // }
              // // Yank & Paste
              // KeyCode::Char('p') => self.paste_from_clipboard(),
              // KeyCode::Char('y') => self.yank(),
              // // Format json
              // KeyCode::Char('f') => self.format_json(),
              // _ => {}
        }
    }
}

/// The editor mode, i.e. Normal or Insert.
#[derive(Clone, PartialEq, Eq, Default)]
pub enum EditorMode {
    #[default]
    None,
    Normal,
    Insert,
}

/// The error of the request. Can hold a kind value
/// to distinguish between format and grpc errors.
#[derive(Clone)]
pub struct ErrorKind {
    pub kind: String,
    pub msg: String,
}

impl ErrorKind {
    pub fn format_error(msg: String) -> Self {
        Self {
            kind: "Format Error".to_owned(),
            msg,
        }
    }

    pub fn default_error<T: Into<String>>(msg: T) -> Self {
        Self {
            kind: "Error".to_string(),
            msg: msg.into(),
        }
    }
}

impl From<serde_json::Error> for ErrorKind {
    fn from(err: serde_json::Error) -> Self {
        Self::format_error(err.to_string())
    }
}

impl From<core::error::Error> for ErrorKind {
    fn from(err: core::error::Error) -> Self {
        Self::default_error(err.to_string())
    }
}

/// Pretty formats a string assuming it is in json format.
/// Returns an error if formatting fails.
pub fn pretty_format_json(input: &str) -> Result<String, ErrorKind> {
    let parsed = serde_json::from_str::<serde_json::Value>(input)?;
    let pretty = serde_json::to_string_pretty(&parsed)?;
    Ok(pretty)
}
