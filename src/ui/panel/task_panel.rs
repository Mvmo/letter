use std::io::Stdout;

use ratatui::{prelude::CrosstermBackend, Frame};

use crate::{AppState, UpdateResult};
use crate::ui::textarea::TextArea;

use super::Panel;

pub struct TaskPanel {
    pub task_index: usize,
    text_area: TextArea
}

impl TaskPanel {
    fn new(task_index: usize) -> Self {
        TaskPanel {
            task_index,
            text_area: TextArea { lines: , cursor:  }
        }
    }
}

impl Panel for TaskPanel {
    fn get_name(&self) -> String {
        format!("Task {}", self.task_index)
    }

    fn update(&mut self, app_state: &mut AppState) -> UpdateResult {
        UpdateResult::None
    }

    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, app_state: &AppState) {
    }
}
