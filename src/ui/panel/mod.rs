use std::io::Stdout;

use ratatui::{Frame, backend::CrosstermBackend};

use crate::{UpdateResult, AppState};

pub mod overview_panel;

pub trait Panel {
    fn get_name(&self) -> String;
    fn update(&mut self, app_state: &mut AppState) -> UpdateResult;
    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, app_state: &AppState);
}
