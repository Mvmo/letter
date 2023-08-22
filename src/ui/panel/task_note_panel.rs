use std::sync::{Arc, Mutex, mpsc::Receiver};

use crossterm::event::{KeyEvent, KeyCode};
use log::info;
use ratatui::{prelude::Rect, widgets::{Block, Borders, BorderType, Clear}};

use crate::{ui::textarea::TextArea, Letter, LetterCommand, EditorMode};

use super::Panel;

pub struct TaskNotePanel {
    pub note_id: i64,
    rx: Arc<Mutex<Receiver<KeyEvent>>>,
    text_area: TextArea<Letter, LetterCommand>,
}

impl Panel for TaskNotePanel {
    fn get_name(&self) -> String {
        "task-note".to_string()
    }

    fn update(&mut self, letter: &mut Letter) -> Option<LetterCommand> {
        match letter.editor_mode {
            EditorMode::Insert => {
                return self.text_area.update(self.rx.clone(), letter);
            }
            EditorMode::Normal => {
                let rx = self.rx.lock().unwrap();
                if let Ok(key_event) = rx.recv() {
                    match key_event.code {
                        KeyCode::Char('i') => letter.editor_mode = EditorMode::Insert,
                        KeyCode::Esc => {
                            if let Err(_) = letter.task_store.update_note_text(self.note_id, &self.text_area.lines.join("\n")) {
                                info!("Error on update note text")
                            }
                            return Some(LetterCommand::Quit);
                        },
                        _ => return None
                    }
                }
            }
        }
        return None
    }

    // TODO use rect
    fn draw(&mut self, frame: &mut ratatui::Frame<ratatui::prelude::CrosstermBackend<std::io::Stdout>>, _: Rect, _: &Letter) {
        let mut r = frame.size();
        r.x = r.width / 2;
        r.width = r.width / 2;

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .title("Notes");

        let inner = block.inner(r);
        frame.render_widget(Clear, r);
        frame.render_widget(block, r);
        self.text_area.draw(frame, inner);
    }
}

impl TaskNotePanel {
    pub fn new(app_state: &Letter, rx: Arc<Mutex<Receiver<KeyEvent>>>, note_id: i64) -> Self {
        let lines: Vec<String> = app_state.task_store.get_note_by_id(note_id).unwrap()
            .text.lines()
            .map(|s| String::from(s))
            .collect();

        let mut text_area = TextArea::new(lines);
        let esc_callback = |_: &mut TextArea<Letter, LetterCommand>, _: &mut Letter| {
            (true, Some(LetterCommand::Quit))
        };

        text_area.on_key(KeyCode::Esc, Box::new(esc_callback));

        Self {
            rx,
            note_id,
            text_area
        }
    }
 }
