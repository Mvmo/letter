use std::{sync::{Mutex, Arc, mpsc::Receiver}, io::Stdout};

use crossterm::event::KeyEvent;
use ratatui::{Frame, prelude::{CrosstermBackend, Rect}};

use crate::ui::{textarea::TextArea, AppState, UpdateResult};

use super::Panel;

struct SearchPanel {
    pub note_id: i64,
    rx: Arc<Mutex<Receiver<KeyEvent>>>,
    text_area: TextArea<AppState, UpdateResult>,
}

impl Panel for SearchPanel {
    fn get_name(&self) -> String {
        "search".to_string()
    }

    fn update(&mut self, app_state: &mut AppState) -> UpdateResult {
        UpdateResult::None
    }

    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, app_state: &AppState) {

    }
}
