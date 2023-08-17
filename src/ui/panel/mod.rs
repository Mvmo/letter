use std::io::Stdout;

use ratatui::{Frame, backend::CrosstermBackend, prelude::Rect};

use super::{AppState, UpdateResult};

pub mod overview_panel;
pub mod task_note_panel;
pub mod badge_select_panel;

pub trait Panel {
    fn get_name(&self) -> String;
    fn update(&mut self, app_state: &mut AppState) -> UpdateResult;
    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, app_state: &AppState);
}
