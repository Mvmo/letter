use std::{io::Stdout, sync::{mpsc::Receiver, Mutex, Arc}};
use crossterm::event::{KeyCode, KeyEvent};
use log::error;
use ratatui::{Frame, backend::CrosstermBackend, widgets::{ListItem, List, Paragraph}, prelude::{Layout, Direction, Constraint, Rect}, style::{Style, Color}};
use crate::{ui::textarea::TextArea, command::KeyCommandComposer, store::Task, app::{Letter, LetterCommand, EditorMode}};

use super::{Panel, badge_select_panel::BadgeSelectPanel, task_note_panel::TaskNotePanel};

#[derive(Clone, Copy)]
pub enum CursorMovement{
    Up,
    Down,
    Left,
    Right,
    WordForward,
    WordBackward,
}

#[derive(Clone, Copy)]
pub enum NormalCommand {
    Quit,
    Sort,
    Insert,
    InsertAfter,
    InsertAtEndOfLine,
    InsertAtBeginningOfLine,
    InsertNewLineBelow,
    InsertNewLineAbove,
    DeleteLine,
    ToggleTaskState,
    OpenTaskNotes,
    DeleteChar,
    MoveCursor(CursorMovement),
    StartSearch,
    Debug
}

pub struct OverviewPanel {
    rx: Arc<Mutex<Receiver<KeyEvent>>>,
    text_area: TextArea<Letter, LetterCommand>,
    command_composer: KeyCommandComposer<NormalCommand>,
    command_rx: Receiver<NormalCommand>,
    badge_select_panel: Option<BadgeSelectPanel>,
    task_note_panel: Option<Box<dyn Panel>>
}

impl OverviewPanel {
    pub fn init(&mut self, letter: &Letter) {
        self.command_composer.register_keycommand(vec![KeyCode::Char('i')], NormalCommand::Insert);
        self.command_composer.register_keycommand(vec![KeyCode::Char('I')], NormalCommand::InsertAtBeginningOfLine);
        self.command_composer.register_keycommand(vec![KeyCode::Char('A')], NormalCommand::InsertAtEndOfLine);
        self.command_composer.register_keycommand(vec![KeyCode::Char('a')], NormalCommand::InsertAfter);
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
        self.command_composer.register_keycommand(vec![KeyCode::Enter], NormalCommand::OpenTaskNotes);
        self.command_composer.register_keycommand(vec![KeyCode::Char(' '), KeyCode::Char('f'), KeyCode::Char('f')], NormalCommand::StartSearch);
        self.command_composer.register_keycommand(vec![KeyCode::Char(' '), KeyCode::Char('~')], NormalCommand::Debug);

        let enter_callback = |text_area: &mut TextArea<Letter, LetterCommand>, letter: &mut Letter| {
            let (_, y) = text_area.get_cursor();
            if y >= letter.task_store.tasks.len() {
                return (true, None)
            }

            if let Err(_) = letter.task_store.create_task_at(y as i64 + 1, Task::default()) {
                error!("couldn't create task at {y}+1 using Task::default");
            }

            text_area.insert_line_break_at_cursor();
            text_area.move_cursor_down();
            return (true, Some(LetterCommand::Save));
        };

        let esc_callback = |text_area: &mut TextArea<Letter, LetterCommand>, letter: &mut Letter| {
            if text_area.lines.len() > letter.task_store.tasks.len() {
                let diff = text_area.lines.len() - letter.task_store.tasks.len();
                for _ in 0..diff {
                    if let Err(_) = letter.task_store.create_task(Task::default()) {
                        error!("couldn't create task using Task::default()")
                    }
                }
            }

            text_area.lines.iter()
                .enumerate()
                .for_each(|(idx, line)| {
                    if let Err(_) = letter.task_store.update_task_text(idx as i64, line) {
                        error!("couldn't update task text for task {idx} -> {line}")
                    }
                });

            letter.editor_mode = EditorMode::Normal;
            (true, None)
        };

        self.text_area.disallow_line_breaks();

        self.text_area.on_key(KeyCode::Enter, Box::new(enter_callback));
        self.text_area.on_key(KeyCode::Esc, Box::new(esc_callback));

        let lines: Vec<String> = letter.task_store.tasks
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

    fn update(&mut self, letter: &mut Letter) -> Option<LetterCommand> {
        if let Some(badge_select_panel) = &mut self.badge_select_panel {
            let update_cmd = badge_select_panel.update(letter);
            if let Some(LetterCommand::Quit) = update_cmd {
                self.badge_select_panel = None;
                return None;
            }

            return update_cmd;
        }

        if let Some(note_panel) = &mut self.task_note_panel {
            let letter_command_option = note_panel.update(letter);
            if let Some(letter_command) = letter_command_option {
                if let LetterCommand::Quit = letter_command {
                    self.task_note_panel = None;
                    return None
                }
                else {
                    return Some(letter_command)
                }
            }

            return None;
        }

        if let EditorMode::Insert = letter.editor_mode {
            return self.text_area.update(self.rx.clone(), letter);
        }

        let (x, y) = self.text_area.get_cursor();
        if let EditorMode::Normal = letter.editor_mode {
            if let Ok(command) = self.command_rx.try_recv() {
                match command {
                    NormalCommand::Insert => {
                        letter.editor_mode = EditorMode::Insert;
                        return None
                    },
                    NormalCommand::InsertAfter => {
                        self.text_area.move_cursor_right();
                        letter.editor_mode = EditorMode::Insert;
                        return None
                    },
                    NormalCommand::Quit => {
                        return Some(LetterCommand::Quit)
                    },
                    NormalCommand::Sort => {
                        // TODO tasks.sort_by_key(|task| (*task).state);
                        return None
                    },
                    NormalCommand::DeleteLine => {
                        self.text_area.delete_current_line();
                        if letter.task_store.tasks.len() > y {
                            if let Err(_) = letter.task_store.delete_task(y as i64) {
                                error!("couldn't delete task at {y}")
                            }
                        }
                        return Some(LetterCommand::Save);
                    }
                    NormalCommand::DeleteChar => {
                        // TODO last char could break everything | implement save as well
                        self.text_area.delete_char_at_cursor();
                        return Some(LetterCommand::Save);
                    }
                    NormalCommand::ToggleTaskState => {
                        match self.badge_select_panel {
                            Some(_) => self.badge_select_panel = None,
                            None => self.badge_select_panel = Some(BadgeSelectPanel::new(letter, y, (x, y), self.rx.clone()))
                        }

                        return None;
                    }
                    NormalCommand::MoveCursor(movement) => {
                        match movement {
                            CursorMovement::Up => self.text_area.move_cursor_up(),
                            CursorMovement::Down => self.text_area.move_cursor_down(),
                            CursorMovement::Left => self.text_area.move_cursor_left(),
                            CursorMovement::Right => self.text_area.move_cursor_right(),
                            CursorMovement::WordForward => self.text_area.move_cursor_one_word_forward(),
                            CursorMovement::WordBackward => self.text_area.move_cursor_one_word_backward(),
                        }

                        return None;
                    },
                    NormalCommand::InsertAtEndOfLine => {
                        self.text_area.move_cursor_to_line_end();
                        letter.editor_mode = EditorMode::Insert;
                        return None
                    }
                    NormalCommand::InsertAtBeginningOfLine => {
                        self.text_area.move_cursor_to_line_start();
                        letter.editor_mode = EditorMode::Insert;
                        return None
                    }
                    NormalCommand::InsertNewLineAbove => {
                        let index = y.max(0);

                        if let Err(_) = letter.task_store.create_task_at(index as i64, Task::default()) {
                            error!("couldn't create task at {index} using Task::default")
                        }

                        self.text_area.insert_line(index, String::new());
                        self.text_area.move_cursor_to_line_start();
                        letter.editor_mode = EditorMode::Insert
                    }
                    NormalCommand::InsertNewLineBelow => {
                        let index = y + 1;
                        if letter.task_store.tasks.len() == 0 {
                            if let Err(_) = letter.task_store.create_task_at(index as i64 - 1, Task::default()) {
                                error!("couldn't create task at {index} - 1")
                            }
                            letter.editor_mode = EditorMode::Insert;
                        }

                        if let Err(_) = letter.task_store.create_task_at(index as i64, Task::default()) {
                            error!("couldn't create task at {index}")
                        }

                        self.text_area.insert_line(index, String::new());
                        self.text_area.move_cursor_down();
                        letter.editor_mode = EditorMode::Insert;
                    },
                    NormalCommand::OpenTaskNotes => {
                        if let Ok(note_id) = letter.task_store.get_or_create_note_id(y as i64) {
                            let note_panel = TaskNotePanel::new(letter, self.rx.clone(), note_id);
                            self.task_note_panel = Some(Box::new(note_panel));
                        }
                    }
                    NormalCommand::StartSearch => {
                        // self.badge_select_panel = Some(Box::new(SearchPanel::new(self.rx.clone(), letter)))
                    }
                    NormalCommand::Debug => {
                        return Some(LetterCommand::Debug);
                    }
                }
            }
        }

        let rx = self.rx.lock().unwrap();

        if let Ok(key_event) = rx.try_recv() {
            if let EditorMode::Normal = letter.editor_mode {
                let keycode = key_event.code;
                self.command_composer.push_key(keycode);
            }
        }
        None
    }

    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, letter: &Letter) {
        let full_width = frame.size().width;
        let full_height = frame.size().height;

        let overview_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(full_height - 3),
                Constraint::Length(3)
            ]).split(frame.size());

        let widest_badge = letter.task_store.badges.iter()
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

        let task_status_list: Vec<ListItem> = letter.task_store.tasks.iter()
            .map(|task| {
                let badge = letter.task_store.get_badge(task);
                let color = badge.map(|badge| badge.color).unwrap_or_else(|| Color::Black);
                let name = badge.map(|badge| badge.name.clone()).unwrap_or_else(|| String::new());
                ListItem::new(format!("{}", name))
                    .style(Style::default().bg(color))
            }).collect();

        frame.render_widget(List::new(task_status_list), editor_layout[0]);
        self.text_area.draw(frame, editor_layout[2]);

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

        let mode_str = letter.editor_mode.to_string();
        let mode_paragraph = Paragraph::new(format!("-- {mode_str} --")).style(Style::default().fg(Color::LightYellow));
        frame.render_widget(mode_paragraph, status_bar_layout[0]);

        let combo_str = self.command_composer.get_combo_string();
        let combo_paragraph = Paragraph::new(format!("{combo_str}"));
        frame.render_widget(combo_paragraph, status_bar_layout[1]);

        let coordinates_paragraph = Paragraph::new(format!("{y},{x}"));
        frame.render_widget(coordinates_paragraph, status_bar_layout[2]);

        if let Some(badge_select_panel) = &mut self.badge_select_panel {
            badge_select_panel.position.0 = editor_layout[2].x as usize;
            badge_select_panel.draw(frame, area, letter);
        }

        if let Some(note_panel) = &mut self.task_note_panel {
            note_panel.draw(frame, area, letter);
        }
    }
}

impl OverviewPanel {
    pub fn new(rx: Arc<Mutex<Receiver<KeyEvent>>>) -> Self {
        let (command_composer, command_rx) = KeyCommandComposer::new();
        let text_area: TextArea<Letter, LetterCommand> = TextArea::new(vec![]);
        OverviewPanel { rx, text_area, command_composer, command_rx, badge_select_panel: None, task_note_panel: None }
    }
}
