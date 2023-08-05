
use std::{io::Stdout, sync::{mpsc::Receiver, Mutex, Arc}};

use crossterm::event::{KeyEvent, KeyCode};
use ratatui::{Frame, prelude::{CrosstermBackend, Rect}, widgets::Paragraph};

pub struct TextArea {
    pub lines: Vec<String>,
    cursor: (usize, usize)
}

impl TextArea {
    pub fn new(lines: Vec<String>) -> Self {
        TextArea { lines, cursor: (0, 0) }
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor.0 -= 1
    }

    pub fn move_cursor_right(&mut self) {
        self.cursor.0 += 1
    }

    pub fn insert_char_after_cursor(&mut self, c: char) {
        let str_index = self.cursor.1;
        let str = self.lines.get_mut(str_index).unwrap();
        print!("{}", self.cursor.0);
        str.insert_str(self.cursor.0, c.to_string().as_str());
    }

    pub fn update(&mut self, rx: Arc<Mutex<Receiver<KeyEvent>>>) {
        let rx = rx.lock().unwrap();
        if let Ok(key) = rx.recv() {
            match key.code {
                KeyCode::Char(c) => {
                    self.insert_char_after_cursor(c)
                },
                _ => {}
            }
        }

    }

    pub fn draw(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, rect: Rect) {
        self.lines.iter()
            .map(|line| Paragraph::new(line.to_string()))
            .enumerate()
            .for_each(|(index, p)| frame.render_widget(p, Rect::new(0, index as u16, frame.size().width, 1)));

        frame.set_cursor(self.cursor.0 as u16, self.cursor.1 as u16);
    }
}
