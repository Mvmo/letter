use std::{io::Stdout, sync::{mpsc::Receiver, Mutex, Arc}};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, backend::CrosstermBackend, widgets::{ListItem, List, Paragraph}};
use crate::{UpdateResult, AppState, AppMode, ui::textarea::TextArea, Task, TaskState, command::KeyCommandComposer};
use super::Panel;

#[derive(Clone, Copy)]
pub enum CursorMovement {
    Up,
    Down,
    Left,
    Right,
    WordForward,
    WordBackward,
    StartOfLine,
    EndOfLine,
}

#[derive(Clone, Copy)]
pub enum NormalCommand {
    Quit,
    Sort,
    SwitchToInsertMode,
    InsertAtEndOfLine,
    InsertAtBeginningOfLine,
    InsertNewLineBelow,
    InsertNewLineAbove,
    DeleteLine,
    ToggleTaskState,
    DeleteChar,
    MoveCursor(CursorMovement),
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
        self.command_composer.register_keycommand(vec![KeyCode::Char('I')], NormalCommand::InsertAtBeginningOfLine);
        self.command_composer.register_keycommand(vec![KeyCode::Char('A')], NormalCommand::InsertAtEndOfLine);
        self.command_composer.register_keycommand(vec![KeyCode::Char('d'), KeyCode::Char('d')], NormalCommand::DeleteLine);
        self.command_composer.register_keycommand(vec![KeyCode::Char('t')], NormalCommand::ToggleTaskState);
        self.command_composer.register_keycommand(vec![KeyCode::Char(' '), KeyCode::Char('q')], NormalCommand::Quit);
        self.command_composer.register_keycommand(vec![KeyCode::Char(' '), KeyCode::Char('s')], NormalCommand::Sort);
        self.command_composer.register_keycommand(vec![KeyCode::Char('h')], NormalCommand::MoveCursor(CursorMovement::Left));
        self.command_composer.register_keycommand(vec![KeyCode::Char('j')], NormalCommand::MoveCursor(CursorMovement::Down));
        self.command_composer.register_keycommand(vec![KeyCode::Char('k')], NormalCommand::MoveCursor(CursorMovement::Up));
        self.command_composer.register_keycommand(vec![KeyCode::Char('l')], NormalCommand::MoveCursor(CursorMovement::Right));
        self.command_composer.register_keycommand(vec![KeyCode::Char('w')], NormalCommand::MoveCursor(CursorMovement::WordForward));
        self.command_composer.register_keycommand(vec![KeyCode::Char('b')], NormalCommand::MoveCursor(CursorMovement::WordBackward));
        self.command_composer.register_keycommand(vec![KeyCode::Char('O')], NormalCommand::InsertNewLineAbove);
        self.command_composer.register_keycommand(vec![KeyCode::Char('o')], NormalCommand::InsertNewLineBelow);
        self.command_composer.register_keycommand(vec![KeyCode::Char('x')], NormalCommand::DeleteChar);

        let enter_callback = |text_area: &mut TextArea<AppState, UpdateResult>, app_state: &mut AppState| {
            let (_, y) = text_area.get_cursor();
            app_state.task_store.tasks.insert(y + 1, Task { state: TaskState::Todo, text: String::new() });
            text_area.insert_line(y + 1, String::new());
            text_area.move_cursor_down();
            return (true, UpdateResult::Save);
        };

        let esc_callback = |text_area: &mut TextArea<AppState, UpdateResult>, app_state: &mut AppState| {
            if text_area.lines.len() > app_state.task_store.tasks.len() {
                let diff = text_area.lines.len() - app_state.task_store.tasks.len();
                for _ in 0..diff {
                    app_state.task_store.tasks.push(Task { state: TaskState::Todo, text: String::new() });
                }
            }

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
                        self.text_area.delete_current_line();
                        if task_store.tasks.len() > y {
                            task_store.tasks.remove(y);
                        }
                        return UpdateResult::Save;
                    }
                    NormalCommand::DeleteChar => {
                        // TODO last char could break everything
                        self.text_area.delete_char_at_cursor();
                        return UpdateResult::Save;
                    }
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
                            CursorMovement::WordForward => self.text_area.move_cursor_one_word_forward(),
                            CursorMovement::WordBackward => self.text_area.move_cursor_one_word_backward(),
                        }

                        return UpdateResult::None;
                    },
                    NormalCommand::InsertAtEndOfLine => {
                        self.text_area.move_cursor_to_line_end();
                        return UpdateResult::UpdateMode(AppMode::INPUT);
                    }
                    NormalCommand::InsertAtBeginningOfLine => {
                        self.text_area.move_cursor_to_line_start();
                        return UpdateResult::UpdateMode(AppMode::INPUT);
                    }
                    NormalCommand::InsertNewLineAbove => {
                        let index = y.max(0);
                        task_store.tasks.insert(index, Task { state: TaskState::Todo, text: "".to_string() });
                        self.text_area.insert_line(index, String::new());
                        self.text_area.move_cursor_to_line_start();
                        return UpdateResult::UpdateMode(AppMode::INPUT);
                    }
                    NormalCommand::InsertNewLineBelow => {
                        let index = y + 1;
                        task_store.tasks.insert(index, Task { state: TaskState::Todo, text: "".to_string() });
                        self.text_area.insert_line(index, String::new());
                        self.text_area.move_cursor_down();
                        return UpdateResult::UpdateMode(AppMode::INPUT);
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

        let (x, y) = self.text_area.get_cursor();
        let p = Paragraph::new(format!(
            "cl -> {} | {} | {},{}",
            self.command_composer.len(),
            self.command_composer.get_combo_string(),
            x, y
        ));
        frame.render_widget(p, rr);
    }
}

impl OverviewPanel {
    pub fn new(rx: Arc<Mutex<Receiver<KeyEvent>>>) -> Self {
        let (command_composer, command_rx) = KeyCommandComposer::new();
        OverviewPanel { rx, text_area: TextArea::new(vec![]), command_composer, command_rx }
    }
}
