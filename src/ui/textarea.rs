use std::{io::Stdout, sync::{mpsc::Receiver, Mutex, Arc}, collections::HashMap};

use crossterm::event::{KeyEvent, KeyCode};
use ratatui::{Frame, prelude::{CrosstermBackend, Rect}, widgets::Paragraph, style::{Style, Color}};

pub struct TextArea<S, R> {
    pub lines: Vec<String>,
    cursor: (usize, usize),
    allow_line_breaks: bool,
    callbacks: HashMap<KeyCode, Box<dyn FnMut(&mut Self, &mut S) -> (bool, R)>>
}

impl<S, R> TextArea<S, R> {
    pub fn new(lines: Vec<String>) -> Self {
        let mut lines = lines;
        if lines.len() == 0 {
            lines = vec!["".to_string()];
        }

        TextArea { lines, cursor: (0, 0), allow_line_breaks: true, callbacks: HashMap::new() }
    }

    pub fn disallow_line_breaks(&mut self) {
        self.allow_line_breaks = false
    }

    pub fn set_lines(&mut self, lines: Vec<String>) {
        self.cursor = (0, 0);
        self.lines = lines;
        if self.lines.len() == 0 {
            self.lines = vec!["".to_string()]
        }
    }

    pub fn on_key(&mut self, key_code: KeyCode, callback: Box<dyn FnMut(&mut Self, &mut S) -> (bool, R)>) {
        self.callbacks.insert(key_code, callback);
    }

    pub fn get_cursor(&self) -> (usize, usize) {
        self.cursor
    }

    pub fn set_cursor(&mut self, cursor: (usize, usize)) {
        self.cursor = cursor
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

    pub fn move_cursor_one_word_forward(&mut self) {
        let (x, y) = self.cursor;
        let line = self.lines.get(y).unwrap();
        if line.len() == 0 {
            self.move_cursor_down();
            // TODO edgecase: last line last word
            self.move_cursor_to_line_start();
            return;
        }

        let next_word_index = line.char_indices()
            .skip(x)
            .skip_while(|(_, c)| *c != ' ')
            .skip_while(|(_, c)| *c == ' ')
            .map(|(i, _)| i)
            .find(|_| true);

        if let Some(index) = next_word_index {
            self.cursor = (index, y);
        } else {
            self.move_cursor_down();
            self.move_cursor_to_line_start();
        }
    }

    pub fn move_cursor_one_word_backward(&mut self) {
        let (x, y) = self.cursor;
        let line = self.lines.get(y).unwrap();
        if line.len() == 0 || x == 0 {
            self.move_cursor_up();
            // TODO edgecase: last line last word
            self.move_cursor_to_line_end();
            return;
        }

        let (start, _) = line.split_at(x);
        let prev_index = start.char_indices()
            .rev()
            .skip_while(|(_, c)| *c != ' ')
            .skip_while(|(_, c)| *c == ' ')
            .map(|(i, _)| i)
            .find(|_| true);

        if let Some(index) = prev_index {
            self.cursor = (index + 1, y);
        } else {
            self.move_cursor_up();
            self.move_cursor_to_line_end();
        }
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
        if !self.allow_line_breaks {
            return;
        }

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

    pub fn insert_line(&mut self, index: usize, line: String) {
        self.lines.insert(index, line);
    }

    pub fn delete_char_at_cursor(&mut self) {
        let (x, y) = self.cursor;
        let (start, end) = self.lines.split_at_mut(y);
        let str = end.get_mut(0).unwrap();

        if x == 0 || str.len() == 0 {
            if !self.allow_line_breaks {
                return;
            }

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

    pub fn delete_current_line(&mut self) {
        let (x, y) = self.cursor;

        if y == 0 {
            self.lines.remove(y);
            if self.lines.len() == 0 {
                self.lines.insert(0, "".to_string());
                self.move_cursor_to_line_start();
            }

            return;
        }

        if y == self.lines.len() - 1 {
            self.move_cursor_up();
        }

        self.lines.remove(y);
        self.move_cursor_to_line_end();
    }

    pub fn update(&mut self, rx: Arc<Mutex<Receiver<KeyEvent>>>, state: &mut S) -> Option<R> {
        let rx = rx.lock().unwrap();
        if let Ok(key) = rx.recv() {
            let key_code = key.code;
            if self.callbacks.contains_key(&key_code) {
                let mut callback = self.callbacks.remove(&key_code).unwrap();
                let (should_cancel, result) = callback(self, state);
                self.callbacks.insert(key_code, callback);
                if should_cancel {
                    return Some(result);
                }
            }

            match key_code {
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

        None
    }

    pub fn draw(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, rect: Rect) {
        let (x, y) = self.cursor;
        self.lines.iter()
            .map(|line| Paragraph::new(line.to_string()))
            .enumerate()
            .map(|(idx, p)| {
                if idx == y {
                    (idx, p.style(Style::default().bg(Color::Rgb(100, 100, 100))))
                } else {
                    (idx, p)
                }
            })
            .for_each(|(index, p)| frame.render_widget(p, Rect::new(rect.x, rect.y + index as u16, rect.width, 1)));

        frame.set_cursor(rect.x + x as u16, rect.y + y as u16);
    }
}
