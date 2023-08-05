use std::io::Stdout;

use tui::{Frame, backend::CrosstermBackend};

use crate::UpdateResult;

pub mod overview_panel;

pub trait Panel {
    fn get_name(&self) -> String;
    fn update(&mut self) -> UpdateResult;
    fn draw(&self, frame: &mut Frame<CrosstermBackend<Stdout>>);
}
