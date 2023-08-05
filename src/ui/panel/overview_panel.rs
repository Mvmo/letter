use std::{io::Stdout, sync::{mpsc::Receiver, Mutex, Arc}};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, backend::CrosstermBackend, widgets::{ListItem, List, ListState}, style::{Style, Color, Modifier}};

use crate::{UpdateResult, AppState, AppMode};

use super::Panel;

pub struct OverviewPanel {
    rx: Arc<Mutex<Receiver<KeyEvent>>>,
    list_state: ListState
}

impl Panel for OverviewPanel {
    fn get_name(&self) -> String {
        "Overview".to_string()
    }

    fn update(&mut self, app_state: &mut AppState) -> UpdateResult {
        let state = app_state;
        let rx = &self.rx;
        let list_state = &mut self.list_state;
        let task_store = &mut state.task_store;
        let tasks = &mut task_store.tasks;

        if let None = list_state.selected() {
            list_state.select(Some(0));
        }

        let rx = self.rx.lock().unwrap();
        if let Ok(key_event) = rx.try_recv() {
            match key_event.code {
                KeyCode::Char('i') => {
                    return UpdateResult::UpdateMode(AppMode::INPUT(String::from("")))
                }
                KeyCode::Char('q') => {
                    return UpdateResult::Quit;
                },
                KeyCode::Char('s') => {
                    tasks.sort_by_key(|task| (*task).state);
                    return UpdateResult::None;
                },
                KeyCode::Char('j') => {
                    let new_index = match list_state.selected() {
                        Some(index) => {
                            if index >= tasks.len() - 1 {
                                0
                            } else {
                                index + 1
                            }
                        },
                        None => 0
                    };

                    list_state.select(Some(new_index));
                    return UpdateResult::None;
                },
                KeyCode::Char('k') => {
                    let new_index = match list_state.selected() {
                        Some(index) => {
                            if index == 0 {
                                tasks.len() - 1
                            } else {
                                index - 1
                            }
                        },
                        None => 0
                    };

                    list_state.select(Some(new_index));
                    return UpdateResult::None;
                },
                KeyCode::Char(' ') => {
                    let mut task = &mut tasks[list_state.selected().unwrap()];
                    task.state = task.state.next();
                    return UpdateResult::Save
                },
                KeyCode::Enter => {
                    let task_index = list_state.selected().unwrap();
                    let task_text = task_store.tasks[task_index].text.clone();
                    return UpdateResult::UpdateMode(AppMode::EDIT(task_index, task_text))
                }
                _ => {}
            }
        }
        UpdateResult::None
    }

    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, app_state: &AppState) {
        let tasks = &app_state.task_store.tasks;
        let list_state = &mut self.list_state;

        let items: Vec<ListItem> = tasks.iter()
            .map(|task| {
                ListItem::new(format!("{}", *task))
            }).collect();

        let my_list = List::new(items).highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::Gray));

        let mut rect = frame.size().clone();
        rect.height = rect.height - 2;
        rect.y = 0;

        frame.render_stateful_widget(my_list, rect, list_state);
    }
}

impl OverviewPanel {
    pub fn new(rx: Arc<Mutex<Receiver<KeyEvent>>>) -> Self {
        OverviewPanel { rx, list_state: ListState::default() }
    }
}
