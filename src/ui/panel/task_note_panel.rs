use crate::{AppState, UpdateResult};

use super::Panel;

struct TaskNotePanel {
}

impl Panel for TaskNotePanel {
    fn get_name(&self) -> String {
        "task-note".to_string()
    }

    fn update(&mut self, app_state: &mut AppState) -> UpdateResult {
        todo!()
    }

    fn draw(&mut self, frame: &mut ratatui::Frame<ratatui::prelude::CrosstermBackend<std::io::Stdout>>, app_state: &AppState) {
        todo!()
    }
}
