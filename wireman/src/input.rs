use std::{cell::RefCell, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use edtui::EditorMode;

use crate::{
    app::AppContext,
    model::{
        headers::{HeadersModel, HeadersSelection},
        MessagesModel, SelectionModel,
    },
    AUTOSAVE_HISTORY,
};

/// The input on the select services and methods page
pub struct SelectionInput<'a> {
    pub model: Rc<RefCell<SelectionModel>>,
    pub messages_model: Rc<RefCell<MessagesModel>>,
    pub context: &'a mut AppContext,
}

impl SelectionInput<'_> {
    pub fn handle(&mut self, code: KeyCode) {
        const SUBS: usize = 2;
        match code {
            KeyCode::BackTab if !self.context.disable_root_events => {
                self.context.tab = self.context.tab.prev();
                self.context.sub = 0;
            }
            KeyCode::Tab if !self.context.disable_root_events => {
                self.context.tab = self.context.tab.next();
                self.context.sub = 0;
            }
            KeyCode::Enter if self.context.sub == 0 => {
                self.context.sub = 1;
                // Select a method if there is none selected yet.
                if self.model.borrow().selected_method().is_none() {
                    self.model.borrow_mut().next_method();
                }
                if let Some(method) = self.model.borrow().selected_method() {
                    self.messages_model.borrow_mut().load_method(&method);
                }
            }
            KeyCode::Enter if self.context.sub == 1 => {
                if self.model.borrow().selected_method().is_none() {
                    // Select a method if there is none selected yet.
                    self.model.borrow_mut().next_method();
                } else {
                    // Otherwise go to next tab
                    self.context.tab = self.context.tab.next();
                    self.context.sub = 0;
                }
            }
            KeyCode::Esc if self.context.sub == 1 => {
                self.context.sub = 0;
                self.model.borrow_mut().clear_method();
            }
            KeyCode::Up => {
                self.context.sub = (self.context.sub + SUBS).saturating_sub(1) % SUBS;
            }
            KeyCode::Down => {
                self.context.sub = self.context.sub.saturating_add(1) % SUBS;
            }
            KeyCode::Char('j') => {
                if self.context.sub == 0 {
                    self.model.borrow_mut().next_service();
                    self.model.borrow_mut().clear_method();
                } else {
                    self.model.borrow_mut().next_method();
                }
                if let Some(method) = self.model.borrow().selected_method() {
                    self.messages_model.borrow_mut().load_method(&method);
                }
            }
            KeyCode::Char('k') => {
                if self.context.sub == 0 {
                    self.model.borrow_mut().previous_service();
                    self.model.borrow_mut().clear_method();
                } else {
                    self.model.borrow_mut().previous_method();
                }
                if let Some(method) = self.model.borrow().selected_method() {
                    self.messages_model.borrow_mut().load_method(&method);
                }
            }
            _ => {}
        }
    }
}

/// The input on the messages page.
pub struct MessagesInput<'a> {
    pub model: Rc<RefCell<MessagesModel>>,
    pub context: &'a mut AppContext,
}

impl MessagesInput<'_> {
    pub fn handle(&mut self, event: KeyEvent) {
        fn is_control(event: KeyEvent) -> bool {
            event.modifiers == KeyModifiers::CONTROL
        }

        const SUBS: usize = 2;
        match event.code {
            KeyCode::BackTab if !self.context.disable_root_events => {
                self.context.tab = self.context.tab.prev();
                self.context.sub = 0;
            }
            KeyCode::Tab if !self.context.disable_root_events => {
                self.context.tab = self.context.tab.next();
                self.context.sub = 0;
            }
            KeyCode::Down if !self.context.disable_root_events => {
                self.context.sub = self.context.sub.saturating_add(1) % SUBS;
            }
            KeyCode::Up if !self.context.disable_root_events => {
                self.context.sub = (self.context.sub + SUBS).saturating_sub(1) % SUBS;
            }
            KeyCode::Enter if self.context.sub == 0 && !self.context.disable_root_events => {
                self.model.borrow_mut().start_request();
            }
            KeyCode::Char('y') if is_control(event) && !self.context.disable_root_events => {
                self.model.borrow_mut().yank_grpcurl();
            }
            KeyCode::Char('f')
                if is_control(event)
                    && self.context.sub == 0
                    && !self.context.disable_root_events =>
            {
                let request = &mut self.model.borrow_mut().request.editor;
                request.format_json();
            }
            KeyCode::Char('d') if is_control(event) && !self.context.disable_root_events => {
                let method = self.model.borrow().selected_method.clone();
                if let Some(method) = method {
                    self.model.borrow().history_model.delete(&method);
                    self.model.borrow_mut().request.load_template(&method);
                    self.model.borrow_mut().headers_model.borrow_mut().clear();
                }
            }
            KeyCode::Char('s') if is_control(event) && !self.context.disable_root_events => {
                self.model.borrow().history_model.save(&self.model.borrow());
            }
            KeyCode::Char('l') if is_control(event) && !self.context.disable_root_events => {
                let history_model = self.model.borrow().history_model.clone();
                history_model.load(&mut self.model.borrow_mut());
            }
            KeyCode::Char('1') if !self.context.disable_root_events => {
                self.handle_history_reload(1);
            }
            KeyCode::Char('2') if !self.context.disable_root_events => {
                self.handle_history_reload(2);
            }
            KeyCode::Char('3') if !self.context.disable_root_events => {
                self.handle_history_reload(3);
            }
            KeyCode::Char('4') if !self.context.disable_root_events => {
                self.handle_history_reload(4);
            }
            KeyCode::Char('5') if !self.context.disable_root_events => {
                self.handle_history_reload(5);
            }
            _ => {
                let mut disable_root_events = false;
                if self.context.sub == 0 {
                    let request = &mut self.model.borrow_mut().request.editor;
                    request.on_key(event, false);
                    disable_root_events = request.insert_mode();
                }
                if self.context.sub == 1 {
                    let response = &mut self.model.borrow_mut().response.editor;
                    response.on_key(event, false);
                    disable_root_events = response.insert_mode();
                }
                // Disable all root key events if one of the editors went into insert mode
                // to not overwrite keys such as 'q' for quitting.
                self.context.disable_root_events = disable_root_events;
            }
        }
    }

    fn handle_history_reload(&mut self, index: usize) {
        if AUTOSAVE_HISTORY {
            self.model.borrow().history_model.save(&self.model.borrow());
        }

        let mut model = self.model.borrow_mut();
        model.history_model.select(index);

        let history_model = model.history_model.clone();
        let _ = history_model.load(&mut model);
    }
}

/// The input on the headers page.
pub struct HeadersInput<'a> {
    pub model: Rc<RefCell<HeadersModel>>,
    pub context: &'a mut AppContext,
}

impl HeadersInput<'_> {
    pub fn handle(&mut self, event: KeyEvent) {
        const SUBS: usize = 2;
        let mut model = self.model.borrow_mut();
        match event.code {
            KeyCode::Esc if !self.context.disable_root_events => {
                model.selected = HeadersSelection::None;
            }
            KeyCode::Char('k') | KeyCode::Up
                if !self.context.disable_root_events && !model.meta.block_prev() =>
            {
                model.selected = model.prev();
            }
            KeyCode::Char('j') | KeyCode::Down
                if !self.context.disable_root_events && !model.meta.block_next() =>
            {
                model.selected = model.next();
            }
            _ => {
                let selected = model.selected.clone();
                match selected {
                    HeadersSelection::Addr => match event.code {
                        KeyCode::Tab => {
                            self.context.tab = self.context.tab.next();
                            self.context.sub = 0;
                        }
                        KeyCode::BackTab => {
                            self.context.tab = self.context.tab.prev();
                            self.context.sub = 0;
                        }
                        _ => model.addr.on_key(event, true),
                    },
                    HeadersSelection::Auth => model.auth.on_key(event),
                    HeadersSelection::Meta => model.meta.on_key(event),
                    HeadersSelection::None => match event.code {
                        KeyCode::Tab => {
                            self.context.tab = self.context.tab.next();
                            self.context.sub = 0;
                        }
                        KeyCode::BackTab => {
                            self.context.tab = self.context.tab.prev();
                            self.context.sub = 0;
                        }
                        KeyCode::Enter => {
                            model.selected = HeadersSelection::Addr;
                        }
                        KeyCode::Char('h') if event.modifiers == KeyModifiers::CONTROL => {
                            model.meta.add();
                            model.selected = HeadersSelection::Meta;
                        }
                        _ => {}
                    },
                }
                // Disable all root key events unless all editors are in normal mode.
                self.context.disable_root_events = model.mode() != EditorMode::Normal;
                // Make sure that a valid field is selected
                if selected == HeadersSelection::Meta && model.meta.headers.is_empty() {
                    model.selected = HeadersSelection::None;
                }
            }
        }
    }
}