use std::io::Stdout;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use crossterm::event::KeyEvent;
use ratatui::{prelude::CrosstermBackend, Frame};

use crate::{AppState, UpdateResult};
use crate::ui::textarea::TextArea;

use super::Panel;

pub struct TaskPanel {
    task_index: usize,
    text_area: TextArea<Option<u16>, bool>,
    rx: Arc<Mutex<Receiver<KeyEvent>>>
}

impl TaskPanel {
    pub fn new(task_index: usize, task_str: String, rx: Arc<Mutex<Receiver<KeyEvent>>>) -> Self {
        TaskPanel {
            task_index,
            text_area: TextArea::new(vec![task_str]),
            rx
        }
    }
}

impl Panel for TaskPanel {
    fn get_name(&self) -> String {
        format!("Task #{}", self.task_index)
    }

    fn update(&mut self, app_state: &mut AppState) -> UpdateResult {
        self.text_area.update(self.rx.clone(), &mut None);
        UpdateResult::None
    }

    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, app_state: &AppState) {
        self.text_area.draw(frame, frame.size());
    }
}
