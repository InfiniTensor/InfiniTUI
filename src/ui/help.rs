use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Clear, Padding, Row, Table, TableState},
    Frame,
};

use rust_i18n::t;
use std::borrow::Cow;

pub struct Help {
    block_height: usize,
    state: TableState,
    keys: Vec<(&'static str, Cow<'static, str>)>,
}

impl Default for Help {
    fn default() -> Self {
        let mut state = TableState::new().with_offset(0);
        state.select(Some(0));

        Self {
            block_height: 0,
            state,
            keys: vec![
                ("Esc", t!("help_esc")),
                ("Tab", t!("help_tab")),
                (
                    "ctrl + n",
                    t!("help_ctrl_n"),
                ),
                (
                    "ctrl + s",
                    t!("help_ctrl_s"),
                ),
                ("ctrl + f", t!("help_ctrl_f")),
                ("ctrl + p", t!("help_ctrl_p")),
                ("ctrl + h", t!("help_ctrl_h")),
                ("ctrl + t", t!("help_ctrl_t")),
                ("j or Down", t!("help_j_or_down")),
                ("k or Up", t!("help_k_or_up")),
                ("G", t!("help_g")),
                ("gg", t!("help_gg")),
                ("?", t!("help_?")),
            ],
        }
    }
}

impl Help {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scroll_down(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.keys.len().saturating_sub(self.block_height - 6) {
                    i
                } else {
                    i + 1
                }
            }
            None => 1,
        };
        *self.state.offset_mut() = i;
        self.state.select(Some(i));
    }
    pub fn scroll_up(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i > 1 {
                    i - 1
                } else {
                    0
                }
            }
            None => 1,
        };
        *self.state.offset_mut() = i;
        self.state.select(Some(i));
    }

    pub fn render(&mut self, frame: &mut Frame, block: Rect) {
        self.block_height = block.height as usize;
        let widths = [Constraint::Length(15), Constraint::Min(60)];
        let rows: Vec<Row> = self
            .keys
            .iter()
            .map(|key| Row::new(vec![key.0, &key.1]))
            .collect();

        let table = Table::new(rows, widths).block(
            Block::default()
                .padding(Padding::uniform(2))
                .title(" Help ")
                .title_style(Style::default().bold())
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .style(Style::default())
                .border_style(Style::default()),
        );

        frame.render_widget(Clear, block);
        frame.render_stateful_widget(table, block, &mut self.state);
    }
}
