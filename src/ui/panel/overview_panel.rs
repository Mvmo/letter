use std::{io::Stdout, sync::{mpsc::Receiver, Mutex, Arc}};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, backend::CrosstermBackend, widgets::{ListItem, List, Paragraph}, text::Span};
use crate::{UpdateResult, AppState, AppMode, ui::textarea::TextArea, Task, TaskState, command::KeyCommandComposer};
use super::Panel;

#[derive(Clone, Copy)]
pub enum CursorMovement {
    Up,
    Down,
    Left,
    Right,
    StartOfLine,
    EndOfLine,
}

#[derive(Clone, Copy)]
pub enum NormalCommand {
    Quit,
    Sort,
    SwitchToInsertMode,
    DeleteLine,
    ToggleTaskState,
    MoveCursor(CursorMovement)
}

pub struct OverviewPanel {
    rx: Arc<Mutex<Receiver<KeyEvent>>>,
    text_area: TextArea<AppState, UpdateResult>,
    command_composer: KeyCommandComposer<NormalCommand>,
    command_rx: Receiver<NormalCommand>
}

impl OverviewPanel {
    pub fn init(&mut self, app_state: &AppState) {
        self.command_composer.register_keycommand(vec![KeyCode::Char('i')], NormalCommand::SwitchToInsertMode);
        self.command_composer.register_keycommand(vec![KeyCode::Char('d'), KeyCode::Char('d')], NormalCommand::DeleteLine);
        self.command_composer.register_keycommand(vec![KeyCode::Char('t')], NormalCommand::ToggleTaskState);
        self.command_composer.register_keycommand(vec![KeyCode::Char(' '), KeyCode::Char('q')], NormalCommand::Quit);
        self.command_composer.register_keycommand(vec![KeyCode::Char(' '), KeyCode::Char('s')], NormalCommand::Sort);
        self.command_composer.register_keycommand(vec![KeyCode::Char('h')], NormalCommand::MoveCursor(CursorMovement::Left));
        self.command_composer.register_keycommand(vec![KeyCode::Char('j')], NormalCommand::MoveCursor(CursorMovement::Down));
        self.command_composer.register_keycommand(vec![KeyCode::Char('k')], NormalCommand::MoveCursor(CursorMovement::Up));
        self.command_composer.register_keycommand(vec![KeyCode::Char('l')], NormalCommand::MoveCursor(CursorMovement::Right));

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
        let task_store = &mut state.task_store;
        let tasks = &mut task_store.tasks;

        if let AppMode::INPUT = state.mode {
            return self.text_area.update(self.rx.clone(), state).unwrap_or(UpdateResult::None);
        }

        let (x, y) = self.text_area.get_cursor();
        if let AppMode::NORMAL = state.mode {
            if let Ok(command) = self.command_rx.try_recv() {
                match command {
                    NormalCommand::SwitchToInsertMode => {
                        return UpdateResult::UpdateMode(AppMode::INPUT);
                    },
                    NormalCommand::Quit => {
                        return UpdateResult::Quit;
                    },
                    NormalCommand::Sort => {
                        tasks.sort_by_key(|task| (*task).state);
                        return UpdateResult::None;
                    },
                    NormalCommand::DeleteLine => {
                        task_store.tasks.remove(y);
                        self.text_area.delete_current_line();
                        // todo: remove line from task_store
                        return UpdateResult::Save;
                    },
                    NormalCommand::ToggleTaskState => {
                        let mut task = tasks.get_mut(y).unwrap();
                        task.state = task.state.next();
                        task_store.save();
                        return UpdateResult::None;
                    }
                    NormalCommand::MoveCursor(movement) => {
                        match movement {
                            CursorMovement::Up => self.text_area.move_cursor_up(),
                            CursorMovement::Down => self.text_area.move_cursor_down(),
                            CursorMovement::Left => self.text_area.move_cursor_left(),
                            CursorMovement::Right => self.text_area.move_cursor_right(),
                            CursorMovement::StartOfLine => self.text_area.move_cursor_to_line_start(),
                            CursorMovement::EndOfLine => self.text_area.move_cursor_to_line_end(),
                        }

                        return UpdateResult::None;
                    }
                }
            }
        }

        let rx = self.rx.lock().unwrap();

        if let Ok(key_event) = rx.try_recv() {
            let (_, y) = self.text_area.get_cursor();
            if let AppMode::NORMAL = state.mode {
                let keycode = key_event.code;
                self.command_composer.push_key(keycode);
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

        let my_list = List::new(items);
        let mut rect = frame.size().clone();
        rect.width = 2;
        rect.height = rect.height - 4;
        rect.y = 0;

        frame.render_widget(my_list, rect);

        let mut text_area_rect = frame.size().clone();
        text_area_rect.x = 2;
        text_area_rect.width = text_area_rect.width - 2;
        text_area_rect.height = text_area_rect.height - 4;
        self.text_area.draw(frame, text_area_rect);

        let mut rr = frame.size();
        rr.y = rr.height - 4;
        rr.height = 2;

        let p = Paragraph::new(format!("cl -> {}", self.command_composer.len()));
        frame.render_widget(p, rr);
    }
}

impl OverviewPanel {
    pub fn new(rx: Arc<Mutex<Receiver<KeyEvent>>>) -> Self {
        let (command_composer, command_rx) = KeyCommandComposer::new();
        OverviewPanel { rx, text_area: TextArea::new(vec![]), command_composer, command_rx }
    }
}
