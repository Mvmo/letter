
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
        let (x, y) = self.cursor;
        if x == 0 {
            if y > 0 {
                let str_above = self.lines.get(y - 1).unwrap();
                self.cursor = (str_above.len() - 1, y - 1)
            }
            return
        }

        self.cursor = (x - 1, y);
    }

    pub fn move_cursor_right(&mut self) {
        let (x, y) = self.cursor;
        let current_str = self.lines.get(y).unwrap();
        if x == current_str.len() - 1 {
            if y < self.lines.len() - 1 {
                self.cursor = (0, y + 1)
            }
            return
        }

        self.cursor = (x + 1, y);
    }

    pub fn move_cursor_down(&mut self) {
        let (x, y) = self.cursor;
        if y == self.lines.len() - 1 {
            return
        }

        self.cursor = (x, y + 1);
    }

    pub fn move_cursor_up(&mut self) {
        let (x, y) = self.cursor;
        if y == 0 {
            return
        }

        self.cursor = (x, y - 1);
    }

    pub fn insert_char_after_cursor(&mut self, c: char) {
        let (x, y) = self.cursor;
        let str = self.lines.get_mut(y).unwrap();
        str.insert_str(x + 1, c.to_string().as_str());
        self.move_cursor_right()
    }

    pub fn update(&mut self, rx: Arc<Mutex<Receiver<KeyEvent>>>) {
        let rx = rx.lock().unwrap();
        if let Ok(key) = rx.recv() {
            match key.code {
                KeyCode::Left => {
                    self.move_cursor_left();
                },
                KeyCode::Up => {
                    self.move_cursor_up();
                },
                KeyCode::Right => {
                    self.move_cursor_right();
                },
                KeyCode::Down => {
                    self.move_cursor_down();
                },
                KeyCode::Char(c) => {
                    self.insert_char_after_cursor(c)
                },
                KeyCode::Esc => {
                    panic!("HAHL");
                }
                _ => {}
            }
        }

    }

    pub fn draw(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, rect: Rect) {
        self.lines.iter()
            .map(|line| Paragraph::new(line.to_string()))
            .enumerate()
            .for_each(|(index, p)| frame.render_widget(p, Rect::new(0, index as u16, frame.size().width, 1)));

        let (x, y) = self.cursor;
        frame.set_cursor(rect.x + x as u16, rect.y + y as u16);

        frame.set_cursor(self.cursor.0 as u16, self.cursor.1 as u16);
    }
}
