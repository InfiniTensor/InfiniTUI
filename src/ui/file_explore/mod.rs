use anyhow::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::*,
    Frame,
};
use std::path::{Path, PathBuf};
use tracing::error;

use crate::app::FocusedBlock;

use self::{
    input::Input,
    widget::{Renderer, Theme},
};

use super::formatter;

pub mod input;
pub mod widget;

fn get_file_content(path: &Path) -> Result<String> {
    let mut content = String::new();

    // If the path is a file, read its content.
    if path.is_file() {
        content = std::fs::read_to_string(path)?;
    }

    Ok(content)
}

#[derive(Debug, Default, Clone)]
pub struct FileExplorerPreview {
    pub scroll: u16,
}

#[derive(Clone, Debug)]
pub struct FileExplorer {
    root_dir: PathBuf,
    cwd: PathBuf,
    files: Vec<File>,
    selected: usize,
    theme: Theme,
    pub preview: FileExplorerPreview,
}

impl FileExplorer {
    pub fn new(root_dir: &str) -> FileExplorer {
        let root_dir: PathBuf = root_dir.into();
        let cwd = root_dir.clone(); // Modify this line;
        error!("cwd {:?}", cwd);
        let theme = Theme::default().with_title_top(|_fe| "File Explore".into());

        let mut file_explorer = Self {
            root_dir,
            cwd,
            files: vec![],
            selected: 0,
            theme: theme,
            preview: FileExplorerPreview::default(),
        };

        if let Err(e) = file_explorer.get_and_set_files() {
            error!("Error getting and setting files: {}", e)
        }

        file_explorer
    }

    #[inline]
    pub fn with_theme(root_dir: &str, theme: Theme) -> Result<FileExplorer> {
        let mut file_explorer = Self::new(root_dir);

        file_explorer.theme = theme;

        Ok(file_explorer)
    }

    #[inline]
    pub const fn widget<'a>(&'a self, focus: &'a FocusedBlock) -> impl WidgetRef + 'a {
        Renderer(self, focus)
    }

    pub fn handle<I: Into<Input>>(&mut self, input: I) -> Result<()> {
        let input = input.into();

        match input {
            Input::Up => {
                if self.selected == 0 {
                    self.selected = self.files.len() - 1;
                } else {
                    self.selected -= 1;
                }
            }
            Input::Down => {
                if self.selected == self.files.len() - 1 {
                    self.selected = 0;
                } else {
                    self.selected += 1;
                }
            }
            Input::Left => {
                let parent = self.cwd.parent();

                if let Some(parent) = parent {
                    if parent == self.root_dir {
                        self.cwd = parent.to_path_buf();
                        self.get_and_set_files()?;
                        self.selected = 0
                    }
                }
            }
            Input::Right => {
                if self.files[self.selected].path.is_dir() {
                    self.cwd = self.files.swap_remove(self.selected).path;
                    self.get_and_set_files()?;
                    self.selected = 0
                }
            }
            Input::None => (),
        }

        Ok(())
    }

    #[inline]
    pub fn set_cwd<P: Into<PathBuf>>(&mut self, cwd: P) -> Result<()> {
        self.cwd = cwd.into();
        self.get_and_set_files()?;
        self.selected = 0;

        Ok(())
    }

    #[inline]
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    #[inline]
    pub fn set_selected_idx(&mut self, selected: usize) {
        assert!(selected < self.files.len());
        self.selected = selected;
    }

    /// Returns the current file or directory selected.
    #[inline]
    pub fn current(&self) -> &File {
        &self.files[self.selected]
    }

    /// Returns the current working directory of the file explorer.
    #[inline]
    pub const fn cwd(&self) -> &PathBuf {
        &self.cwd
    }

    #[inline]
    pub const fn files(&self) -> &Vec<File> {
        &self.files
    }

    /// Returns the index of the selected file or directory in the current [`Vec`](https://doc.rust-lang.org/stable/std/vec/struct.Vec.html) of files
    /// ```
    #[inline]
    pub const fn selected_idx(&self) -> usize {
        self.selected
    }

    #[inline]
    pub const fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Get the files and directories in the current working directory and set them in the file explorer.
    /// It add the parent directory at the beginning of the [`Vec`](https://doc.rust-lang.org/stable/std/vec/struct.Vec.html) of files if it exist.
    fn get_and_set_files(&mut self) -> Result<()> {
        let (mut dirs, mut none_dirs): (Vec<_>, Vec<_>) = std::fs::read_dir(&self.cwd)?
            .filter_map(|entry| {
                entry.ok().map(|e| {
                    let path = e.path();
                    let is_dir = path.is_dir();
                    let name = if is_dir {
                        format!("{}/", e.file_name().to_string_lossy())
                    } else {
                        e.file_name().to_string_lossy().into_owned()
                    };

                    File { name, path, is_dir }
                })
            })
            .partition(|file| file.is_dir);

        dirs.sort_unstable_by(|f1, f2| f1.name.cmp(&f2.name));
        none_dirs.sort_unstable_by(|f1, f2| f1.name.cmp(&f2.name));

        if let Some(parent) = self.cwd.parent() {
            let mut files = Vec::with_capacity(1 + dirs.len() + none_dirs.len());

            files.push(File {
                name: "./".to_owned(),
                path: parent.to_path_buf(),
                is_dir: true,
            });

            files.extend(dirs);
            files.extend(none_dirs);

            self.files = files
        } else {
            let mut files = Vec::with_capacity(dirs.len() + none_dirs.len());

            files.extend(dirs);
            files.extend(none_dirs);

            self.files = files;
        };

        Ok(())
    }

    pub(crate) fn render(
        &self,
        frame: &mut Frame,
        area: ratatui::prelude::Rect,
        focus_block: crate::app::FocusedBlock,
        formatter: &formatter::Formatter,
    ) {
        let file_content = get_file_content(self.current().path()).unwrap_or("".into());
        let file_content = formatter.format(&file_content);

        let (file_block, preview_block) = {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(area);
            (chunks[0], chunks[1])
        };

        frame.render_widget(Clear, area);
        frame.render_widget(&self.widget(&focus_block), file_block);

        let preview = Paragraph::new(file_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("File Preview ")
                    .title_style(match focus_block {
                        crate::app::FocusedBlock::FileExplorerPreview => Style::default().bold(),
                        _ => Style::default(),
                    })
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Double)
                    .border_style(match focus_block {
                        crate::app::FocusedBlock::FileExplorerPreview => {
                            Style::default().fg(Color::Green)
                        }
                        _ => Style::default(),
                    }),
            )
            .wrap(Wrap { trim: false })
            .scroll((self.preview.scroll, 0));

        frame.render_widget(preview, preview_block);
    }
}

/// A file or directory in the file explorer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct File {
    name: String,
    path: PathBuf,
    is_dir: bool,
}

impl File {
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub const fn path(&self) -> &PathBuf {
        &self.path
    }

    #[inline]
    pub const fn is_dir(&self) -> bool {
        self.is_dir
    }
}
