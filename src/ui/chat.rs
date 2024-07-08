use super::*;

use super::spinner::Spinner;
use crate::llm::LLMAnswer;
use futures::executor::block_on;
use std::{
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};
use tokio::time::{self, Duration}; // 引用 Spinner 模块

use super::formatter::Formatter;

#[derive(Debug, Clone, Default)]
pub struct Answer<'a> {
    pub plain_answer: String,
    pub formatted_answer: Text<'a>,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub content: String,
    pub style: Style,
}

impl ChatMessage {
    pub fn new(content: String, is_user: bool) -> Self {
        let style = if is_user {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(Color::Green)
        };
        Self { content, style }
    }
}

#[derive(Debug, Clone)]
pub struct Chat<'a> {
    pub plain_chat: Vec<String>,
    pub formatted_chat: Text<'a>,
    pub answer: Answer<'a>,
    pub scroll: u16,
    area_height: u16,
    area_width: u16,
    pub automatic_scroll: Rc<AtomicBool>,
    pub ai_typing: bool,
    pub spinner: Spinner, // 使用 Spinner
}

impl Default for Chat<'_> {
    fn default() -> Self {
        Self {
            plain_chat: Vec::new(),
            formatted_chat: Text::raw(""),
            answer: Answer::default(),
            scroll: 0,
            area_height: 0,
            area_width: 0,
            automatic_scroll: Rc::new(AtomicBool::new(true)),
            ai_typing: false,
            spinner: Spinner::default(),
        }
    }
}

impl Chat<'_> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_answer(&mut self, event: LLMAnswer, formatter: &Formatter) {
        match event {
            LLMAnswer::StartAnswer => {
                self.formatted_chat.lines.pop();
            }

            LLMAnswer::Answer(answer) => {
                self.answer.plain_answer.push_str(answer.as_str());
                self.answer.formatted_answer =
                    formatter.format(format!("🤖: {}", &self.answer.plain_answer).as_str());
            }

            LLMAnswer::EndAnswer => {
                self.formatted_chat
                    .extend(self.answer.formatted_answer.clone());

                self.formatted_chat.extend(Text::raw("\n"));

                self.plain_chat
                    .push(format!("🤖: {}", self.answer.plain_answer));

                self.answer = Answer::default();
            }
        }
    }

    pub fn height(&self) -> usize {
        let mut chat = self.formatted_chat.clone();
        chat.extend(self.answer.formatted_answer.clone());
        let nb_lines = chat.lines.len() + 3;
        chat.lines.iter().fold(nb_lines, |acc, line| {
            acc + line.width() / self.area_width as usize
        })
    }

    pub fn move_to_bottom(&mut self) {
        self.scroll = (self.formatted_chat.height() + self.answer.formatted_answer.height())
            .saturating_sub((self.area_height - 2).into()) as u16;
    }

    pub fn move_to_top(&mut self) {
        self.scroll = 0;
    }

    pub async fn update_spinner(&mut self) {
        if self.ai_typing {
            self.spinner.update();
            time::sleep(Duration::from_millis(100)).await;
        }
    }

    // pub fn render(&mut self, frame: &mut Frame, area: Rect) {
    //     let mut text = self.formatted_chat.clone();
    //     text.extend(self.answer.formatted_answer.clone());

    //     self.area_height = area.height;
    //     self.area_width = area.width;

    //     let scroll: u16 = if self
    //         .automatic_scroll
    //         .load(std::sync::atomic::Ordering::Relaxed)
    //     {
    //         let scroll = self.height().saturating_sub(self.area_height.into()) as u16;
    //         self.scroll = scroll;
    //         scroll
    //     } else {
    //         self.scroll
    //     };

    //     let chat = Paragraph::new(text)
    //         .scroll((scroll, 0))
    //         .wrap(Wrap { trim: false })
    //         .block(
    //             Block::default()
    //                 .title("InfiniLM AI Chat")
    //                 .borders(Borders::ALL)
    //                 .border_style(Style::default().fg(Color::Green))
    //                 .border_type(BorderType::Rounded)
    //                 .style(Style::default()),
    //         )
    //         .style(Style::default().fg(Color::White)) // 设置默认样式为白色字体，黑色背景
    //         .alignment(Alignment::Left);

    //     frame.render_widget(chat, area);
    // }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let mut text = self.formatted_chat.clone();
        text.extend(self.answer.formatted_answer.clone());

        // 设置每个 Line 的 spans 中每个 Span 的样式为白色字体
        let styled_lines: Vec<Line> = text
            .lines
            .iter()
            .map(|line| Line {
                spans: line
                    .spans
                    .iter()
                    .map(|span| Span::styled(span.content.clone(), Style::default()))
                    .collect(),
                style: Style::default(),
                alignment: line.alignment,
            })
            .collect();

        let styled_text = Text {
            lines: styled_lines,
            alignment: Some(Alignment::Left), // 根据需要设置对齐方式
            style: Style::default(),
        };

        self.area_height = area.height;
        self.area_width = area.width;

        let scroll: u16 = if self
            .automatic_scroll
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let scroll = self.height().saturating_sub(self.area_height.into()) as u16;
            self.scroll = scroll;
            scroll
        } else {
            self.scroll
        };

        let chat = Paragraph::new(styled_text)
            .scroll((scroll, 0))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .title("InfiniLM AI Chat")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .border_type(BorderType::Rounded)
                    .style(Style::default()),
            )
            // .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Left);

        frame.render_widget(chat, area);
    }
}
