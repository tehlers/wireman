use super::root::layout;
use crate::{
    model::headers::{AuthSelection, HeadersModel, HeadersSelection},
    widgets::editor::{view_single_selected, view_single_unselected},
    widgets::kv::KV,
};
use edtui::{EditorState, StatusLine};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, StatefulWidget, Tabs, Widget},
};
use tui_widget_list::{List, ListState};

use super::theme::THEME;

/// The request and response tab
pub struct HeadersPage<'a> {
    model: &'a HeadersModel,
}

impl<'a> HeadersPage<'a> {
    pub fn new(model: &'a HeadersModel) -> Self {
        Self { model }
    }

    pub fn footer_keys(&self) -> Vec<(&'static str, &'static str)> {
        match self.model.selected {
            HeadersSelection::Addr => {
                vec![
                    ("q", "Quit"),
                    ("Esc", "Unselect"),
                    ("↑/k", "Up"),
                    ("↓/j", "Down"),
                ]
            }
            HeadersSelection::Auth => {
                vec![
                    ("q", "Quit"),
                    ("Esc", "Unselect"),
                    ("Tab", "Switch"),
                    ("↑/k", "Up"),
                    ("↓/j", "Down"),
                ]
            }
            HeadersSelection::Meta => {
                vec![
                    ("q", "Quit"),
                    ("Esc", "Unselect"),
                    ("Tab", "Switch"),
                    ("↑/k", "Up"),
                    ("↓/j", "Down"),
                    ("^h/^a", "Add Header"),
                    ("^d", "Remove Header"),
                ]
            }
            HeadersSelection::None => {
                vec![
                    ("q", "Quit"),
                    ("Tab", "Next Tab"),
                    ("↑/k", "Up"),
                    ("↓/j", "Down"),
                    ("^h/^a", "Add Header"),
                ]
            }
        }
    }
}

impl Widget for HeadersPage<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        // let area = layout(area, Direction::Vertical, &[1, 4, 1, 5, 1, 0, 1]);
        let [addr_title, addr_content, _, auth_title, auth_content, _, meta_title, meta_content, status] =
            layout(area, Direction::Vertical, &[1, 3, 1, 1, 4, 1, 1, 0, 1]);

        // Address
        ListElements::VDivider(String::from(" Address ")).render(addr_title, buf);
        Address {
            state: self.model.addr.state.clone(),
            title: String::new(),
            selected: self.model.selected == HeadersSelection::Addr,
        }
        .render(addr_content, buf);

        // Authentication
        ListElements::VDivider(String::from(" Authentication ")).render(auth_title, buf);
        let body = match self.model.auth.selected {
            AuthSelection::Bearer => Authentication {
                state: self.model.auth.bearer.state.clone(),
                title: String::new(),
                selected: self.model.selected == HeadersSelection::Auth,
                selected_tag: 0,
            },
            AuthSelection::Basic => Authentication {
                state: self.model.auth.basic.state.clone(),
                title: String::new(),
                selected: self.model.selected == HeadersSelection::Auth,
                selected_tag: 1,
            },
        };
        body.render(auth_content, buf);

        // Metadata
        if !self.model.meta.is_hidden() {
            ListElements::VDivider(String::from(" Headers ")).render(meta_title, buf);
            let headers = &self.model.meta.headers;
            let index = self.model.meta.selected;
            Metadata {
                content: headers
                    .iter()
                    .enumerate()
                    .map(|(i, x)| KV {
                        key: x.0.state.clone(),
                        val: x.1.state.clone(),
                        key_selected: (self.model.selected == HeadersSelection::Meta)
                            && index.map_or(false, |x| x.row == i && x.col == 0),
                        val_selected: (self.model.selected == HeadersSelection::Meta)
                            && index.map_or(false, |x| x.row == i && x.col == 1),
                    })
                    .collect(),
                selected_row: index.map(|x| x.row),
            }
            .render(meta_content, buf);
        }

        // Show a combined status line for all editors
        StatusLine::default()
            .style_text(THEME.status_line.0)
            .style_line(THEME.status_line.1)
            .mode(self.model.mode().name())
            .render(status, buf);
    }
}

#[allow(clippy::large_enum_variant)]
enum ListElements {
    VSpace(usize),
    VDivider(String),
}

impl Widget for ListElements {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            Self::VSpace(_) => {}
            Self::VDivider(title) => {
                Block::default()
                    .title(title)
                    .borders(Borders::TOP)
                    .title_alignment(Alignment::Center)
                    .title_style(THEME.divider.title)
                    .border_style(THEME.divider.border_style)
                    .border_type(THEME.divider.border_type)
                    .render(area, buf);
            }
        };
    }
}

#[derive(Clone)]
struct Address {
    state: EditorState,
    title: String,
    selected: bool,
}

impl Widget for Address {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        if self.selected {
            view_single_selected(&mut self.state, &self.title).render(area, buf);
        } else {
            view_single_unselected(&mut self.state, &self.title).render(area, buf);
        }
    }
}

#[derive(Clone)]
struct Authentication {
    state: EditorState,
    title: String,
    selected: bool,
    selected_tag: usize,
}

impl Widget for Authentication {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let [title, content] = layout(area, Direction::Vertical, &[1, 0]);

        let titles = vec![" Bearer ", " Basic "];
        Tabs::new(titles)
            .style(THEME.tabs)
            .highlight_style(THEME.tabs_selected)
            .select(self.selected_tag)
            .divider("")
            .render(title.inner(&Margin::new(0, 0)), buf);

        if self.selected {
            view_single_selected(&mut self.state, self.title).render(content, buf);
        } else {
            view_single_unselected(&mut self.state, self.title).render(content, buf);
        }
    }
}

#[derive(Clone)]
struct Metadata {
    content: Vec<KV>,
    selected_row: Option<usize>,
}

impl Widget for Metadata {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = ListState::default();
        state.selected = self.selected_row;
        let list = List::new(self.content);
        list.render(area, buf, &mut state);
    }
}
