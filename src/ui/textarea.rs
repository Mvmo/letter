use std::{io::Stdout, sync::{mpsc::Receiver, Mutex, Arc}};

use crossterm::{event::{KeyEvent, KeyCode}, cursor};
use ratatui::{Frame, prelude::{CrosstermBackend, Rect}, widgets::Paragraph};

pub struct TextArea {
    pub lines: Vec<String>,
    cursor: (usize, usize),
    allow_line_breaks: bool,
}

impl TextArea {
    pub fn new(lines: Vec<String>) -> Self {
        TextArea { lines, cursor: (0, 0), allow_line_breaks: true }
    }

    pub fn move_cursor_left(&mut self) {
        let (x, y) = self.cursor;
        if x == 0 {
            if y > 0 {
                self.move_cursor_up();
                self.move_cursor_to_line_end();
            }
            return
        }

        self.cursor = (x - 1, y);
    }

    pub fn move_cursor_left_times(&mut self, times: u16) {
        for _ in 0..times {
            self.move_cursor_left();
        }
    }

    pub fn move_cursor_right(&mut self) {
        let (x, y) = self.cursor;
        let current_str = self.lines.get(y).unwrap();
        if current_str.len() == 0 || x == current_str.len() {
            if y < self.lines.len() - 1 {
                self.move_cursor_down();
                self.move_cursor_to_line_start();
            }
            return
        }

        self.cursor = (x + 1, y);
    }

    pub fn move_cursor_right_times(&mut self, times: u16) {
        for _ in 0..times {
            self.move_cursor_right();
        }
    }

    pub fn move_cursor_down(&mut self) {
        let (x, y) = self.cursor;
        if y == self.lines.len() - 1 {
            return
        }

        self.cursor = (x, y + 1);

        let line_below = self.lines.get(y + 1).unwrap();
        if x > line_below.len() {
            self.move_cursor_to_line_end();
        }
    }

    pub fn move_cursor_up(&mut self) {
        let (x, y) = self.cursor;
        if y == 0 {
            return
        }

        self.cursor = (x, y - 1);

        let line_above = self.lines.get(y - 1).unwrap();
        if x > line_above.len() {
            self.move_cursor_to_line_end();
        }
    }

    pub fn move_cursor_to_line_start(&mut self) {
        let (_, y) = self.cursor;
        self.cursor = (0, y);
    }

    pub fn move_cursor_to_line_end(&mut self) {
        let (_, y) = self.cursor;
        let line = self.lines.get(y).unwrap();
        if line.len() == 0 {
            self.cursor = (0, y);
            return;
        }

        self.cursor = (line.len(), y);
    }

    pub fn insert_char_at_cursor(&mut self, c: char) {
        let (x, y) = self.cursor;
        let str = self.lines.get_mut(y).unwrap();
        if str.len() == 0 {
            str.push(c);
            self.move_cursor_right();
            return;
        }

        str.insert_str(x, c.to_string().as_str());
        self.move_cursor_right()
    }

    pub fn insert_line_break_at_cursor(&mut self) {
        let (x, y) = self.cursor;
        let str = self.lines.get_mut(y).unwrap();

        if str.len() == 0 || x == str.len() - 1 {
            self.lines.insert(y, "".to_string());
            self.move_cursor_down();
            self.move_cursor_to_line_start();
            return;
        }

        let right = str.drain(x..str.len()).as_str().to_string();
        self.lines.insert(y + 1, right.as_str().to_string());

        self.move_cursor_down();
        self.move_cursor_to_line_start();
    }

    pub fn delete_char_at_cursor(&mut self) {
        let (x, y) = self.cursor;
        let (start, end) = self.lines.split_at_mut(y);
        let str = end.get_mut(0).unwrap();

        if x == 0 || str.len() == 0 {
            let line_above = start.get_mut(start.len() - 1).unwrap();
            let line_len = line_above.len();
            line_above.push_str(str.as_str());
            self.lines.remove(y);
            self.move_cursor_up();
            self.move_cursor_right_times(line_len as u16);
            return;
        }

        str.remove(x - 1);
        self.move_cursor_left();
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
                    self.insert_char_at_cursor(c)
                },
                KeyCode::Backspace => {
                    self.delete_char_at_cursor()
                }
                KeyCode::Enter => {
                    self.insert_line_break_at_cursor()
                }
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
        frame.set_cursor(x as u16, y as u16);
    }
}
