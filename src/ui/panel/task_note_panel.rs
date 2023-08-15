use std::sync::{Arc, Mutex, mpsc::Receiver};

use crossterm::event::{KeyEvent, KeyCode};
use ratatui::{prelude::Rect, widgets::{Block, Borders, BorderType}};

use crate::{AppState, UpdateResult, ui::textarea::TextArea, AppMode};

use super::Panel;

pub struct TaskNotePanel {
    pub task_note_id: i64,
    rx: Arc<Mutex<Receiver<KeyEvent>>>,
    text_area: TextArea<AppState, UpdateResult>,
}

impl Panel for TaskNotePanel {
    fn get_name(&self) -> String {
        "task-note".to_string()
    }

    fn update(&mut self, app_state: &mut AppState) -> UpdateResult {
        match app_state.mode {
            AppMode::INPUT => {
                if let Some(update_result) = self.text_area.update(self.rx.clone(), app_state) {
                    return update_result
                }
            },
            AppMode::NORMAL => {
                let rx = self.rx.lock().unwrap();
                if let Ok(key_event) = rx.recv() {
                    match key_event.code {
                        KeyCode::Char('i') => return UpdateResult::UpdateMode(AppMode::INPUT),
                        KeyCode::Esc => return UpdateResult::Quit,
                        _ => return UpdateResult::None
                    }
                }
            }
        }
        return UpdateResult::None
    }

    // TODO use rect
    fn draw(&mut self, frame: &mut ratatui::Frame<ratatui::prelude::CrosstermBackend<std::io::Stdout>>, _: Rect, _: &AppState) {
        let mut r = frame.size();
        r.x = r.width / 2;
        r.width = r.width / 2;

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .title("Notes");

        let inner = block.inner(r);
        frame.render_widget(block, r);
        self.text_area.draw(frame, inner);
    }
}

impl TaskNotePanel {
    pub fn new(app_state: &AppState, rx: Arc<Mutex<Receiver<KeyEvent>>>, task_note_id: i64) -> Self {
        let lines: Vec<String> = app_state.task_store.get_note_by_id(task_note_id).unwrap()
            .text.lines()
            .map(|s| String::from(s))
            .collect();

        let mut text_area = TextArea::new(lines);
        let esc_callback = |_: &mut TextArea<AppState, UpdateResult>, _: &mut AppState| {
            return (true, UpdateResult::UpdateMode(AppMode::NORMAL));
        };

        text_area.on_key(KeyCode::Esc, Box::new(esc_callback));

        Self {
            rx,
            task_note_id,
            text_area
        }
    }
}
