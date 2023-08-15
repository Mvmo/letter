use std::sync::{Arc, Mutex, mpsc::Receiver};

use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;

use crate::{AppState, UpdateResult, ui::textarea::TextArea, AppMode};

use super::Panel;

struct TaskNotePanel {
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
                self.text_area.update(self.rx.clone(), app_state);
            },
            AppMode::NORMAL => {

            }
        }
        return UpdateResult::None
    }

    fn draw(&mut self, frame: &mut ratatui::Frame<ratatui::prelude::CrosstermBackend<std::io::Stdout>>, area: Rect, app_state: &AppState) {
        let mut r = frame.size();
        r.x = r.width / 2;
        r.width = r.width / 2;

        self.text_area.draw(frame, r);
    }
}

impl TaskNotePanel {
    pub fn new(app_state: &AppState, rx: Arc<Mutex<Receiver<KeyEvent>>>, task_note_id: i64) -> Self {
        let lines: Vec<String> = app_state.task_store.get_note_by_id(task_note_id).unwrap()
            .text.lines()
            .map(|s| String::from(s))
            .collect();

        Self {
            rx,
            task_note_id,
            text_area: TextArea::new(lines),
        }
    }
}
