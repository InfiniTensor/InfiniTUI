use std::sync::Arc;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, HighlightSpacing, List, ListState},
};

use crate::app::FocusedBlock;

use super::{File, FileExplorer};
use ratatui::widgets::*;

type LineFactory = Arc<dyn Fn(&FileExplorer) -> Line<'static>>;

pub struct Renderer<'a>(pub(crate) &'a FileExplorer, pub(crate) &'a FocusedBlock);

impl WidgetRef for Renderer<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut state = ListState::default().with_selected(Some(self.0.selected_idx()));

        let highlight_style = if self.0.current().is_dir() {
            self.0.theme().highlight_dir_style
        } else {
            self.0.theme().highlight_item_style
        };

        let mut list = List::new(self.0.files().iter().map(|file| file.text(self.0.theme())))
            .style(self.0.theme().style)
            .highlight_spacing(self.0.theme().highlight_spacing.clone())
            .highlight_style(highlight_style);

        if let Some(symbol) = self.0.theme().highlight_symbol.as_deref() {
            list = list.highlight_symbol(symbol);
        }

        if let Some(block) = self.0.theme().block.as_ref() {
            let mut block = block.clone();

            block = block
                .title_style(match self.1 {
                    FocusedBlock::FileExplorer => Style::default().bold(),
                    _ => Style::default(),
                })
                .title_alignment(Alignment::Center)
                .style(Style::default())
                .border_style(match self.1 {
                    FocusedBlock::FileExplorer => Style::default().fg(Color::Green),
                    _ => Style::default(),
                });

            for title_top in self.0.theme().title_top(self.0) {
                block = block.title_top(title_top)
            }
            for title_bottom in self.0.theme().title_bottom(self.0) {
                block = block.title_bottom(title_bottom)
            }

            list = list.block(block);
        }

        ratatui::widgets::StatefulWidgetRef::render_ref(&list, area, buf, &mut state)
    }
}

impl File {
    /// Returns the text with the appropriate style to be displayed for the file.
    fn text(&self, theme: &Theme) -> Text<'_> {
        let style = if self.is_dir() {
            *theme.dir_style()
        } else {
            *theme.item_style()
        };
        Span::styled(self.name().to_owned(), style).into()
    }
}

/// The theme of the file explorer.
///
/// This struct is used to customize the look of the file explorer.
/// It allows to set the style of the widget and the style of the files.
/// You can also wrap the widget in a block with the [Theme::with_block](#method.block)
/// method and add customizable titles to it with [Theme::with_title_top](#method.title_top)
/// and [Theme::with_title_bottom](#method.title_bottom).
#[derive(Clone, derivative::Derivative)]
#[derivative(Debug, PartialEq, Eq, Hash)]
pub struct Theme {
    block: Option<Block<'static>>,
    #[derivative(Debug = "ignore", PartialEq = "ignore", Hash = "ignore")]
    title_top: Vec<LineFactory>,
    #[derivative(Debug = "ignore", PartialEq = "ignore", Hash = "ignore")]
    title_bottom: Vec<LineFactory>,
    style: Style,
    item_style: Style,
    dir_style: Style,
    highlight_spacing: HighlightSpacing,
    highlight_item_style: Style,
    highlight_dir_style: Style,
    highlight_symbol: Option<String>,
}

impl Theme {
    /// Create a new empty theme.
    ///
    /// The theme will not have any style set. To get a theme with the default style, use [Theme::default](#method.default).
    ///
    /// # Example
    /// ```no_run
    /// # use ratatui_explorer::Theme;
    /// let theme = Theme::new();
    /// ```
    pub const fn new() -> Self {
        Self {
            block: None,
            title_top: Vec::new(),
            title_bottom: Vec::new(),
            style: Style::new(),
            item_style: Style::new(),
            dir_style: Style::new(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            highlight_item_style: Style::new(),
            highlight_dir_style: Style::new(),
            highlight_symbol: None,
        }
    }

    #[inline]
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn add_default_title(self) -> Self {
        self.with_title_top(|file_explorer: &FileExplorer| {
            Line::from(file_explorer.cwd().display().to_string())
        })
    }

    #[inline]
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn with_block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    #[inline]
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn with_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    #[inline]
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn with_item_style<S: Into<Style>>(mut self, item_style: S) -> Self {
        self.item_style = item_style.into();
        self
    }

    #[inline]
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn with_dir_style<S: Into<Style>>(mut self, dir_style: S) -> Self {
        self.dir_style = dir_style.into();
        self
    }

    #[inline]
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn with_highlight_item_style<S: Into<Style>>(mut self, highlight_item_style: S) -> Self {
        self.highlight_item_style = highlight_item_style.into();
        self
    }

    #[inline]
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn with_highlight_dir_style<S: Into<Style>>(mut self, highlight_dir_style: S) -> Self {
        self.highlight_dir_style = highlight_dir_style.into();
        self
    }

    #[inline]
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn with_highlight_symbol(mut self, highlight_symbol: &str) -> Self {
        self.highlight_symbol = Some(highlight_symbol.to_owned());
        self
    }

    #[inline]
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn with_highlight_spacing(mut self, highlight_spacing: HighlightSpacing) -> Self {
        self.highlight_spacing = highlight_spacing;
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn with_title_top(
        mut self,
        title_top: impl Fn(&FileExplorer) -> Line<'static> + 'static,
    ) -> Self {
        self.title_top.push(Arc::new(title_top));
        self
    }

    #[inline]
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn with_title_bottom(
        mut self,
        title_bottom: impl Fn(&FileExplorer) -> Line<'static> + 'static,
    ) -> Self {
        self.title_bottom.push(Arc::new(title_bottom));
        self
    }

    /// Returns the wrapping block (if it exist) of the file explorer of the theme.
    #[inline]
    pub const fn block(&self) -> Option<&Block<'static>> {
        self.block.as_ref()
    }

    /// Returns the style of the widget of the theme.
    #[inline]
    pub const fn style(&self) -> &Style {
        &self.style
    }

    /// Returns the style of the non directories items of the theme.
    #[inline]
    pub const fn item_style(&self) -> &Style {
        &self.item_style
    }

    /// Returns the style of the directories items of the theme.
    #[inline]
    pub const fn dir_style(&self) -> &Style {
        &self.dir_style
    }

    /// Returns the style of the highlighted non directories items of the theme.
    #[inline]
    pub const fn highlight_item_style(&self) -> &Style {
        &self.highlight_item_style
    }

    /// Returns the style of the highlighted directories items of the theme.
    #[inline]
    pub const fn highlight_dir_style(&self) -> &Style {
        &self.highlight_dir_style
    }

    /// Returns the symbol used to highlight the selected item of the theme.
    #[inline]
    pub fn highlight_symbol(&self) -> Option<&str> {
        self.highlight_symbol.as_deref()
    }

    /// Returns the spacing between the highlighted item and the other items of the theme.
    #[inline]
    pub const fn highlight_spacing(&self) -> &HighlightSpacing {
        &self.highlight_spacing
    }

    /// Returns the generated top titles of the theme.
    #[inline]
    pub fn title_top(&self, file_explorer: &FileExplorer) -> Vec<Line> {
        self.title_top
            .iter()
            .map(|title_top| title_top(file_explorer))
            .collect()
    }

    /// Returns the generated bottom titles of the theme.
    #[inline]
    pub fn title_bottom(&self, file_explorer: &FileExplorer) -> Vec<Line> {
        self.title_bottom
            .iter()
            .map(|title_bottom| title_bottom(file_explorer))
            .collect()
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            block: Some(Block::default().borders(Borders::ALL)),
            title_top: Vec::new(),
            title_bottom: Vec::new(),
            style: Style::default(),
            item_style: Style::default().fg(Color::White),
            dir_style: Style::default().fg(Color::LightBlue),
            highlight_spacing: HighlightSpacing::Always,
            highlight_item_style: Style::default().fg(Color::White).bg(Color::DarkGray),
            highlight_dir_style: Style::default().fg(Color::LightBlue).bg(Color::DarkGray),
            highlight_symbol: None,
        }
    }
}
