use super::{headers::HeadersPage, messages::MessagesPage, selection::SelectionPage, theme::THEME};
use crate::context::{AppContext, Tab};
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph, Tabs, Widget},
};
use theme::{self, Theme};

pub struct Root<'a> {
    ctx: &'a AppContext,
}

impl<'a> Root<'a> {
    pub fn new(ctx: &'a AppContext) -> Self {
        Root { ctx }
    }
}
impl Root<'_> {
    fn render_navbar(&self, area: Rect, buf: &mut Buffer) {
        let [left, right] = layout(area, Direction::Horizontal, &[0, 50]);

        Paragraph::new(Span::styled("WireMan", THEME.app_title)).render(left, buf);
        let titles = vec![" Selection ", " Messages ", " Address & Headers "];
        Tabs::new(titles)
            .style(THEME.tabs)
            .highlight_style(THEME.tabs_selected)
            .select(self.ctx.tab.index())
            .divider("")
            .render(right, buf);
    }

    fn render_content(&self, area: Rect, buf: &mut Buffer) {
        match self.ctx.tab {
            Tab::Selection => SelectionPage {
                model: &mut self.ctx.selection.borrow_mut(),
                tab: self.ctx.selection_tab,
            }
            .render(area, buf),
            Tab::Messages => MessagesPage {
                model: &mut self.ctx.messages.borrow_mut(),
                tab: self.ctx.messages_tab,
            }
            .render(area, buf),
            Tab::Headers => HeadersPage::new(&self.ctx.headers.borrow()).render(area, buf),
        };
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let keys = match self.ctx.tab {
            Tab::Selection => SelectionPage::footer_keys(self.ctx.selection_tab),
            Tab::Messages => MessagesPage::footer_keys(),
            Tab::Headers => HeadersPage::new(&self.ctx.headers.borrow()).footer_keys(),
        };
        let spans: Vec<Span> = keys
            .iter()
            .flat_map(|(key, desc)| {
                let key = Span::styled(format!(" {key} "), THEME.key_binding.key);
                let desc = Span::styled(format!(" {desc} "), THEME.key_binding.description);
                [key, desc]
            })
            .collect();
        Paragraph::new(Line::from(spans))
            .alignment(Alignment::Center)
            .fg(Color::Indexed(236))
            .bg(Color::Indexed(232))
            .render(area, buf);
    }
}

impl Widget for Root<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = Theme::global();
        Block::new().style(THEME.root).render(area, buf);
        if theme.root.hide_footer_help {
            let [header, content] = layout(area, Direction::Vertical, &[1, 0]);
            self.render_navbar(header, buf);
            self.render_content(content, buf);
        } else {
            let [header, content, footer] = layout(area, Direction::Vertical, &[1, 0, 1]);
            self.render_navbar(header, buf);
            self.render_content(content, buf);
            self.render_footer(footer, buf);
        }
    }
}

/// simple helper method to split an area into multiple sub-areas
pub fn layout<const N: usize>(area: Rect, direction: Direction, heights: &[u16]) -> [Rect; N] {
    use ratatui::layout::Constraint::{Length, Min};
    let constraints = heights
        .iter()
        .map(|&h| if h > 0 { Length(h) } else { Min(0) });
    if direction == Direction::Vertical {
        Layout::vertical(constraints).areas(area)
    } else {
        Layout::horizontal(constraints).areas(area)
    }
}

// /// simple helper method to split an area into multiple sub-areas
// pub fn layout_margin<const N: usize>(
//     area: Rect,
//     direction: Direction,
//     heights: &[u16],
//     margin: u16,
// ) -> [Rect; N] {
//     use ratatui::layout::Constraint::{Length, Min};
//     let constraints = heights
//         .iter()
//         .map(|&h| if h > 0 { Length(h) } else { Min(0) });
//     if direction == Direction::Vertical {
//         Layout::vertical(constraints)
//             .margin(10)
//             .areas(area)
//     } else {
//         Layout::horizontal(constraints)
//             .horizontal_margin(margin)
//             .areas(area)
//     }
// }
