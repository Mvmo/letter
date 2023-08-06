use std::{io::Stdout, sync::{mpsc::Receiver, Mutex, Arc}};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, backend::CrosstermBackend, widgets::{ListItem, List, ListState}, style::{Style, Color, Modifier}};
use crate::{UpdateResult, AppState, AppMode, ui::textarea::TextArea};
use super::Panel;

pub struct OverviewPanel {
    rx: Arc<Mutex<Receiver<KeyEvent>>>,
    text_area: TextArea<AppState, UpdateResult>,
    list_state: ListState
}

impl OverviewPanel {
    pub fn init(&mut self, app_state: &AppState) {
        let enter_callback = |text_area: &mut TextArea<AppState, UpdateResult>, app_state: &mut AppState| {
            return (true, UpdateResult::None);
        };

        let esc_callback = |text_area: &mut TextArea<AppState, UpdateResult>, app_state: &mut AppState| {
            return (true, UpdateResult::UpdateMode(AppMode::NORMAL));
        };

        self.text_area.disallow_line_breaks();

        self.text_area.on_key(KeyCode::Enter, Box::new(enter_callback));
        self.text_area.on_key(KeyCode::Esc, Box::new(esc_callback));

        let lines: Vec<String> = app_state.task_store.tasks.clone()
            .iter()
            .map(|task| task.text.clone())
            .collect();

        self.text_area.set_lines(lines);
    }
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

        if let AppMode::INPUT(_) = state.mode {
            return self.text_area.update(rx.clone(), state).unwrap_or(UpdateResult::None);
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

                    self.text_area.set_cursor((0, new_index));
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

                    self.text_area.set_cursor((0, new_index));
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
                ListItem::new(format!("{}", task.text))
            }).collect();

        let my_list = List::new(items).highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::Gray));

        let mut rect = frame.size().clone();
        rect.height = rect.height - 2;
        rect.y = 0;

        frame.render_stateful_widget(my_list, rect, list_state);
        self.text_area.draw(frame, frame.size());
    }
}

impl OverviewPanel {
    pub fn new(rx: Arc<Mutex<Receiver<KeyEvent>>>) -> Self {
        OverviewPanel { rx, list_state: ListState::default(), text_area: TextArea::new(vec![]) }
    }
}
