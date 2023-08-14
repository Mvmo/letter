use std::{io::Stdout, sync::{mpsc::Receiver, Mutex, Arc}};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, backend::CrosstermBackend, widgets::{ListItem, List, Paragraph}, prelude::{Layout, Direction, Constraint}, style::{Style, Color}};
use crate::{UpdateResult, AppState, AppMode, ui::textarea::TextArea, command::KeyCommandComposer, store::Task};
use super::{Panel, badge_select_panel::BadgeSelectPanel};

// TODO: Bug when first line is just text line and then press enter
//

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
    command_rx: Receiver<NormalCommand>,
    context_frame: Option<Box<dyn Panel>>
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
            app_state.task_store.create_task_at(y as i64 + 1, Task::default());
            text_area.move_cursor_down();
            return (true, UpdateResult::Save);
        };

        let esc_callback = |text_area: &mut TextArea<AppState, UpdateResult>, app_state: &mut AppState| {
            if text_area.lines.len() > app_state.task_store.tasks.len() {
                let diff = text_area.lines.len() - app_state.task_store.tasks.len();
                for _ in 0..diff {
                    app_state.task_store.create_task(Task::default());
                }
            }

            text_area.lines.iter()
                .enumerate()
                .for_each(|(idx, line)| {
                    app_state.task_store.update_task_text(idx as i64, line);
                });

            return (true, UpdateResult::UpdateMode(AppMode::NORMAL));
        };

        self.text_area.disallow_line_breaks();

        self.text_area.on_key(KeyCode::Enter, Box::new(enter_callback));
        self.text_area.on_key(KeyCode::Esc, Box::new(esc_callback));

        let lines: Vec<String> = app_state.task_store.tasks
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

        if let Some(context_frame) = &mut self.context_frame {
            let update_result = context_frame.update(state);
            if let UpdateResult::Quit = update_result {
                self.context_frame = None;
                return UpdateResult::None;
            }

            return update_result;
        }

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
                        // TODO tasks.sort_by_key(|task| (*task).state);
                        return UpdateResult::None;
                    },
                    NormalCommand::DeleteLine => {
                        self.text_area.delete_current_line();
                        if task_store.tasks.len() > y {
                            task_store.delete_task(y as i64);
                            //task_store.tasks.remove(y);
                        }
                        return UpdateResult::Save;
                    }
                    NormalCommand::DeleteChar => {
                        // TODO last char could break everything | implement save as well
                        self.text_area.delete_char_at_cursor();
                        return UpdateResult::Save;
                    }
                    NormalCommand::ToggleTaskState => {
                        // let mut task = tasks.get_mut(y).unwrap();
                        // TODO task.state = task.state.next();
                        // TODO task_store.save();
                        match self.context_frame {
                            Some(_) => self.context_frame = None,
                            None => self.context_frame = Some(Box::new(BadgeSelectPanel::new(state, y, (x, y), self.rx.clone())))
                        }
                        //self.context_frame = Some(Box::new(BadgeSelectPanel { position: (x, y) }));
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
                        task_store.create_task_at(index as i64, Task::default());
                        self.text_area.insert_line(index, String::new());
                        self.text_area.move_cursor_to_line_start();
                        return UpdateResult::UpdateMode(AppMode::INPUT);
                    }
                    NormalCommand::InsertNewLineBelow => {
                        let index = y + 1;
                        if task_store.tasks.len() == 0 {
                            task_store.create_task_at(index as i64 - 1, Task::default());
                            return UpdateResult::UpdateMode(AppMode::INPUT);
                        }

                        task_store.create_task_at(index as i64, Task::default());
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
        let full_width = frame.size().width;
        let full_height = frame.size().height;

        let overview_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(full_height - 3),
                Constraint::Length(3)
            ]).split(frame.size());

        // TODO - move this calc somewhere else
        let widest_badge = app_state.task_store.badges.iter()
            .map(|(_, badge)| badge.name.len())
            .max()
            .unwrap_or(0) as u16;

        let editor_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(widest_badge),
                Constraint::Length(1),
                Constraint::Length(full_width - widest_badge)
            ]).split(overview_layout[0]);

        let task_status_list: Vec<ListItem> = app_state.task_store.tasks.iter()
            .map(|task| {
                let badge = app_state.task_store.get_badge(task);
                let color = badge.map(|badge| badge.color).unwrap_or_else(|| Color::Black);
                let name = badge.map(|badge| badge.name.clone()).unwrap_or_else(|| String::new());
                ListItem::new(format!("{}", name))
                    .style(Style::default().bg(color))
            }).collect();

        frame.render_widget(List::new(task_status_list), editor_layout[0]);
        self.text_area.draw(frame, editor_layout[2]); // TODO: Create custom widget for text area

        let (x, y) = self.text_area.get_cursor();
        let status_bar_v_layout = Layout::default()
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1)
            ]).split(overview_layout[1]);

        let status_bar_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(40),
                Constraint::Percentage(20),
            ]).split(status_bar_v_layout[1]);

        let mode_str: String = app_state.mode.into();
        let mode_paragraph = Paragraph::new(format!("-- {mode_str} --")).style(Style::default().fg(Color::LightYellow));
        frame.render_widget(mode_paragraph, status_bar_layout[0]);

        let combo_str = self.command_composer.get_combo_string();
        let combo_paragraph = Paragraph::new(format!("{combo_str}"));
        frame.render_widget(combo_paragraph, status_bar_layout[1]);

        let coordinates_paragraph = Paragraph::new(format!("{y},{x}"));
        frame.render_widget(coordinates_paragraph, status_bar_layout[2]);

        if let Some(context_frame) = &mut self.context_frame {
            context_frame.draw(frame, app_state)
        }
    }
}

impl OverviewPanel {
    pub fn new(rx: Arc<Mutex<Receiver<KeyEvent>>>) -> Self {
        let (command_composer, command_rx) = KeyCommandComposer::new();
        OverviewPanel { rx, text_area: TextArea::new(vec![]), command_composer, command_rx, context_frame: None }
    }
}
