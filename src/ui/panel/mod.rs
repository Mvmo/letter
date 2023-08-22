use std::io::Stdout;

use ratatui::{Frame, backend::CrosstermBackend, prelude::Rect};

use crate::{Letter, LetterCommand};

pub mod overview_panel;
pub mod task_note_panel;
pub mod badge_select_panel;
pub mod search_panel;

pub mod debug_panel;

pub trait Panel {
    fn get_name(&self) -> String;
    fn update(&mut self, letter: &mut Letter) -> Option<LetterCommand>;
    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, letter: &Letter);
}
