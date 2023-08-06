use std::{io::Stdout, sync::{mpsc::Receiver, Mutex, Arc}};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, backend::CrosstermBackend, widgets::{ListItem, List}, style::{Style, Color, Modifier}};
use crate::{UpdateResult, AppState, AppMode, ui::textarea::TextArea, Task, TaskState};
use super::Panel;

pub struct OverviewPanel {
    rx: Arc<Mutex<Receiver<KeyEvent>>>,
    text_area: TextArea<AppState, UpdateResult>,
}

impl OverviewPanel {
    pub fn init(&mut self, app_state: &AppState) {
        let enter_callback = |text_area: &mut TextArea<AppState, UpdateResult>, app_state: &mut AppState| {
            let (_, y) = text_area.get_cursor();
            app_state.task_store.tasks.insert(y + 1, Task { state: TaskState::Todo, text: String::new() });
            text_area.insert_line(y + 1, String::new());
            text_area.move_cursor_down();
            return (true, UpdateResult::Save);
        };

        let esc_callback = |text_area: &mut TextArea<AppState, UpdateResult>, app_state: &mut AppState| {
            text_area.lines.iter()
                .enumerate()
                .for_each(|(idx, line)| {
                    app_state.task_store.tasks.get_mut(idx).unwrap().text = line.clone();
                });

            app_state.task_store.save();
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
        let task_store = &mut state.task_store;
        let tasks = &mut task_store.tasks;

        if let AppMode::INPUT(_) = state.mode {
            return self.text_area.update(rx.clone(), state).unwrap_or(UpdateResult::None);
        }

        let rx = self.rx.lock().unwrap();
        if let Ok(key_event) = rx.try_recv() {
            let (x, y) = self.text_area.get_cursor();
            match key_event.code {
                KeyCode::Char('i') => {
                    self.text_area.move_cursor_to_line_end();
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
                    self.text_area.move_cursor_down();
                    return UpdateResult::None;
                },
                KeyCode::Char('k') => {
                    self.text_area.move_cursor_up();
                    return UpdateResult::None;
                },
                KeyCode::Char(' ') => {
                    let mut task = &mut tasks[y];
                    task.state = task.state.next();
                    return UpdateResult::Save
                },
                KeyCode::Enter => {
                    // let task_index = list_state.selected().unwrap();
                    // let task_text = task_store.tasks[task_index].text.clone();
                    //return UpdateResult::UpdateMode(AppMode::EDIT(task_index, task_text))
                }
                _ => {}
            }
        }
        UpdateResult::None
    }

    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, app_state: &AppState) {
        let tasks = &app_state.task_store.tasks;
        let items: Vec<ListItem> = tasks.iter()
            .map(|task| {
                ListItem::new(format!("{}", task.state))
            }).collect();

        let my_list = List::new(items).highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::Gray));

        let mut rect = frame.size().clone();
        rect.width = 2;
        rect.height = rect.height - 2;
        rect.y = 0;

        frame.render_widget(my_list, rect);

        let mut text_area_rect = frame.size().clone();
        text_area_rect.x = 2;
        text_area_rect.width = text_area_rect.width - 2;
        self.text_area.draw(frame, text_area_rect);
    }
}

impl OverviewPanel {
    pub fn new(rx: Arc<Mutex<Receiver<KeyEvent>>>) -> Self {
        OverviewPanel { rx, text_area: TextArea::new(vec![]) }
    }
}
