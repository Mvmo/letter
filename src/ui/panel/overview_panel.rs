use std::{io::Stdout, sync::{mpsc::Receiver, Mutex, Arc}};

use crossterm::event::{KeyCode, KeyEvent};
use tui::{Frame, backend::CrosstermBackend, widgets::{ListItem, List, ListState}};

use crate::{UpdateResult, AppState, AppMode};

use super::Panel;

pub struct OverviewPanel<'a> {
    state: Arc<Mutex<AppState<'a>>>,
    rx: Arc<Mutex<Receiver<KeyEvent>>>,
    list_state: Arc<Mutex<ListState>>
}

impl <'a> Panel for OverviewPanel<'a> {
    fn get_name(&self) -> String {
        "Overview".to_string()
    }

    fn update(&mut self) -> UpdateResult {
        let mut state = self.state.lock().unwrap();
        let rx = self.rx.lock().unwrap();
        let mut list_state = self.list_state.lock().unwrap();

        if let Ok(key_event) = rx.try_recv() {
            match key_event.code {
                KeyCode::Char('i') => {
                    return UpdateResult::UpdateMode(AppMode::INPUT(String::from("")))
                }
                KeyCode::Char('q') => {
                    return UpdateResult::Quit;
                },
                KeyCode::Char('s') => {
                    state.task_store.tasks.lock().unwrap().sort_by_key(|task| (*task.lock().unwrap()).state);
                    return UpdateResult::None;
                },
                KeyCode::Char('j') => {
                    let new_index = match list_state.selected() {
                        Some(index) => {
                            if index >= state.task_store.tasks.lock().unwrap().len() - 1 {
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
                                state.task_store.tasks.lock().unwrap().len() - 1
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
                    let tasks = state.task_store.tasks.lock().unwrap();
                    let mut task = tasks[list_state.selected().unwrap()].lock().unwrap();
                    task.state = task.state.next();
                    return UpdateResult::Save
                },
                KeyCode::Enter => {
                    let tasks = state.task_store.tasks.lock().unwrap();
                    let task_ref = tasks[list_state.selected().unwrap()].clone();
                    let task = task_ref.lock().unwrap();

                    return UpdateResult::UpdateMode(AppMode::EDIT(task_ref.clone(), task.text.clone()))
                }
                _ => {}
            }
        }
        UpdateResult::None
    }

    fn draw(&self, frame: &mut Frame<CrosstermBackend<Stdout>>) {
        let state = self.state.lock().unwrap();
        let items: Vec<ListItem> = state.task_store.tasks.lock().unwrap().iter()
            .map(|task| {
                ListItem::new(format!("{}", *task.lock().unwrap()))
            }).collect();

        let my_list = List::new(items).highlight_symbol("-> ");

        let mut rect = frame.size().clone();
        rect.height = rect.height - 2;
        rect.y = 0;

        let mut ls = self.list_state.lock().unwrap();
        frame.render_stateful_widget(my_list, rect, &mut ls);
    }
}

impl<'a> OverviewPanel<'a> {
    pub fn new(state: Arc<Mutex<AppState<'a>>>, rx: Arc<Mutex<Receiver<KeyEvent>>>) -> Self {
        OverviewPanel { state, rx, list_state: Arc::new(Mutex::new(ListState::default())) }
    }
}