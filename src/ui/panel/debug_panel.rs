use std::io::Stdout;

use ratatui::{Frame, prelude::{CrosstermBackend, Rect}, widgets::{Paragraph, Block, Borders, BorderType}, style::{Style, Color}};

use crate::app::{Letter, LetterCommand, logger::Writeable};

use super::Panel;

#[derive(Default)]
pub struct DebugPanel {
    lines: Vec<String>
}

impl Writeable for DebugPanel {
    fn write_line(&mut self, message: &str) {
        self.lines.push(message.to_string());
    }

    fn flush(&mut self) {
        self.lines.clear()
    }
}

impl Panel for DebugPanel {
    fn get_name(&self) -> String {
        "debug".to_string()
    }

    fn update(&mut self, _: &mut Letter) -> Option<LetterCommand> {
        None
    }

    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, _: &Letter) {
        let block = Block::default()
            .title("Debug")
            .style(Style::default().bg(Color::DarkGray))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let inner = block.inner(area);
        frame.render_widget(block, area);
        let area = inner;

        self.lines.iter()
            .map(|str| Paragraph::new(str.clone()))
            .enumerate()
            .for_each(|(i, c)| frame.render_widget(c, Rect { x: area.x, y: area.y + i as u16, width: area.width, height: 1 }));
    }
}
