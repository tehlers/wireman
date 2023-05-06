#![allow(clippy::module_name_repetitions)]
use crate::{
    commons::{editor::TextEditor, HelpActions},
    model::{AddressModel, CoreClient, MessagesModel, MetadataModel, SelectionModel},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{cell::RefCell, rc::Rc};

/// Translates key input to actions for the models
pub struct Controller<'a> {
    /// The model for the services and methods list
    pub selection: SelectionModel,

    /// The model for the request and response messages
    pub messages: MessagesModel<'a>,

    /// The model for the address field
    pub address: Rc<RefCell<AddressModel<'a>>>,

    /// The model for the metadata field
    pub metadata: Rc<RefCell<MetadataModel<'a>>>,

    /// The active window
    pub window: Window,

    ///  Whether to display the help
    pub show_help: bool,

    /// Whether to display the address
    pub show_address: bool,

    /// Whether to display the metadata
    pub show_metadata: bool,
}

impl<'a> Controller<'a> {
    /// Instantiate the homepage
    pub fn new(core_client: CoreClient) -> Self {
        // The core client is shared
        let core_client_rc = Rc::new(RefCell::new(core_client));

        // The selection model
        let selection = SelectionModel::new(Rc::clone(&core_client_rc));

        // The address model
        let default_address = core_client_rc.borrow().get_default_address();
        let address = Rc::new(RefCell::new(AddressModel::new(&default_address)));

        // The metadata model
        let metadata = Rc::new(RefCell::new(MetadataModel::new()));

        // The messages model
        let messages =
            MessagesModel::new(core_client_rc, Rc::clone(&address), Rc::clone(&metadata));

        Self {
            selection,
            messages,
            address,
            metadata,
            window: Window::Selection,
            show_help: true,
            show_address: false,
            show_metadata: false,
        }
    }
    /// Handles key events. Returns true if the app should quit
    pub fn on_event(&mut self, key: KeyEvent) -> bool {
        // Handle events when in insert mode
        if self.insert_mode() {
            self.on_event_insert_mode(key);
            return false;
        }

        // Always quite on q in normal mode
        if key.code == KeyCode::Char('q') {
            return true;
        }

        // Handle events in normal mode
        match self.window {
            Window::Selection => self.on_event_selection(key),
            Window::Request => self.on_event_request(key),
            Window::Metadata => self.on_event_metadata(key),
            Window::Address => self.on_event_address(key),
        }
        false
    }

    /// Whether we are in insert mode on any window
    fn insert_mode(&self) -> bool {
        self.messages.request.editor.insert_mode()
            || self.address.borrow().editor.insert_mode()
            || self
                .metadata
                .borrow()
                .get_selected()
                .map_or(false, TextEditor::insert_mode)
    }

    // Handle key events in insert mode
    pub fn on_event_insert_mode(&mut self, key: KeyEvent) {
        match self.window {
            Window::Selection => (),
            Window::Request => self.messages.request.editor.on_key_insert_mode(key),
            Window::Metadata => {
                if let Some(editor) = self.metadata.borrow_mut().get_selected_mut() {
                    editor.on_key_insert_mode(key);
                }
            }
            Window::Address => self.address.borrow_mut().editor.on_key_insert_mode(key),
        }
    }

    /// Handle key events on the selection window
    fn on_event_selection(&mut self, key: KeyEvent) {
        let model = &mut self.selection;
        match key.code {
            KeyCode::Tab => {
                self.window = Window::Request;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                model.next();
                if let Some(method) = model.selected_method() {
                    self.messages.load_method(&method);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                model.previous();
                if let Some(method) = model.selected_method() {
                    self.messages.load_method(&method);
                }
            }
            KeyCode::Enter if model.is_parent_selected() => {
                model.expand();
                if let Some(method) = model.selected_method() {
                    self.messages.load_method(&method);
                }
            }
            KeyCode::Esc if !model.is_parent_selected() => {
                model.collapse();
                model.clear_method();
                self.messages.clear_method();
            }
            KeyCode::Char('H') => self.toggle_help(),
            KeyCode::Char('A') => self.toggle_address(),
            KeyCode::Char('M') => self.toggle_metadata(),
            _ => {}
        }
    }

    /// Handle key events on the request window
    fn on_event_request(&mut self, key: KeyEvent) {
        let model = &mut self.messages;
        match key.code {
            KeyCode::Tab => {
                self.window = Window::Selection;
            }
            KeyCode::Char('y') if key.modifiers == KeyModifiers::CONTROL => {
                model.yank_grpcurl();
            }
            KeyCode::Enter => model.call_grpc(),
            KeyCode::Char('H') => self.toggle_help(),
            KeyCode::Char('A') => self.toggle_address(),
            KeyCode::Char('M') => self.toggle_metadata(),
            _ => model.request.editor.on_key_normal_mode(key),
        }
    }

    /// handle key events on the metadata window
    fn on_event_metadata(&mut self, key: KeyEvent) {
        let model = &mut self.metadata;
        match key.code {
            KeyCode::Tab => {
                if model.borrow().is_key_selected() {
                    model.borrow_mut().select_val();
                } else {
                    model.borrow_mut().select_key();
                }
            }
            KeyCode::Char('H') => self.toggle_help(),
            KeyCode::Char('M') => self.toggle_metadata(),
            _ => {
                if let Some(editor) = model.borrow_mut().get_selected_mut() {
                    editor.on_key_normal_mode(key);
                }
            }
        }
    }

    /// handle key events on the address window
    fn on_event_address(&mut self, key: KeyEvent) {
        let model = &mut self.address;
        match key.code {
            KeyCode::Char('H') => self.toggle_help(),
            KeyCode::Char('A') => self.toggle_address(),
            _ => {
                model.borrow_mut().editor.on_key_normal_mode(key);
            }
        }
    }

    /// Returns the help actions of the current active window
    pub fn help_hint(&self) -> Option<HelpActions> {
        if !self.show_help {
            return None;
        }

        // help in insert mode
        if self.insert_mode() {
            return Some(HelpActions::from_items(vec![("Esc", "Normal mode")]));
        }

        // help in normal mode
        match &self.window {
            Window::Selection => {
                let action = if self.selection.is_parent_selected() {
                    ("Enter", "Select")
                } else {
                    ("Esc", "Collapse")
                };
                Some(HelpActions::from_items(vec![
                    ("q", "Quit"),
                    ("Tab", "Go to Request"),
                    ("j/↓", "down"),
                    ("k/↑", "up"),
                    action,
                ]))
            }
            Window::Request => Some(HelpActions::from_items(vec![
                ("q", "Quit"),
                ("Tab", "Go to Selection"),
                ("i", "Insert mode"),
                ("p", "Paste"),
                ("u", "Undo"),
                ("y", "Yank"),
                ("<C-y>", "Yank as grpcurl"),
                ("Enter", "gRPC request"),
            ])),
            Window::Metadata => {
                let action = if self.metadata.borrow().is_key_selected() {
                    ("Tab", "Select Value")
                } else {
                    ("Tab", "Select Key")
                };
                Some(HelpActions::from_items(vec![
                    ("q", "Quit"),
                    ("M", "Untoggle metadata"),
                    action,
                    ("i", "Insert mode"),
                    ("p", "Paste"),
                    ("u", "Undo"),
                    ("y", "Yank"),
                ]))
            }
            Window::Address => Some(HelpActions::from_items(vec![
                ("q", "Quit"),
                ("A", "Untoggle address"),
                ("i", "Insert mode"),
            ])),
        }
    }

    /// Toggle the help window on or off
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Toggle the address window on or off
    pub fn toggle_address(&mut self) {
        self.show_address = !self.show_address;
        if self.show_address {
            self.window = Window::Address;
        } else {
            self.window = Window::Request;
        };
    }

    /// Toggle the meetadata window on or off
    pub fn toggle_metadata(&mut self) {
        self.show_metadata = !self.show_metadata;
        if self.show_metadata {
            self.window = Window::Metadata;
        } else {
            self.window = Window::Request;
        };
    }
}
#[derive(PartialEq, Eq)]
pub enum Window {
    Selection,
    Request,
    Address,
    Metadata,
}