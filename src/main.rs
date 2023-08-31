mod ui;
mod command;
mod store;
// mod app;

use std::{path::PathBuf, fs::File, io::{Stdout, stdout}, fmt::Display, process::exit, sync::mpsc::{self, Receiver}, thread, time::Duration};

use command::KeyCommandComposer;
use crossterm::{terminal::enable_raw_mode, event::{self, KeyCode}};
use log::error;
use ratatui::{prelude::{CrosstermBackend, Rect, Layout, Direction, Constraint}, Terminal, widgets::{Block, Borders, Paragraph, ListItem, List}, style::{Color, Style}};
use rusqlite::Connection;
use store::TaskStore;
use ui::textarea::TextArea;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<Stdout>>;

#[derive(Clone, Copy)]
enum LetterMode {
    Normal,
    Insert
}

impl Display for LetterMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            LetterMode::Normal => f.write_str("NORMAL"),
            LetterMode::Insert => f.write_str("INSERT")
        }
    }
}

struct LetterState {
    store: TaskStore,
    mode: LetterMode
}

impl LetterState {
    fn new(store: TaskStore) -> Self {
        Self {
            store,
            mode: LetterMode::Normal
        }
    }
}

trait Window {
    fn handle_event(&mut self, state: &mut LetterState, event: LetterEvent) -> WindowCommand;
    fn update(&mut self, state: &mut LetterState) -> WindowCommand;
    fn draw(&self, state: &LetterState, frame: &mut Frame, rect: Rect);
}

struct TaskListWindow {
    text_area: TextArea<LetterState, LetterCommand>,
}

impl TaskListWindow {
    fn new(store: &TaskStore) -> Self {
        let text_area = TextArea::new(store.tasks.iter().map(|task| task.text.clone()).collect());
        return TaskListWindow { text_area }
    }
}

impl Window for TaskListWindow {
    fn update(&mut self, _state: &mut LetterState) -> WindowCommand {
        None
    }

    fn draw(&self, state: &LetterState, frame: &mut Frame, rect: Rect) {
        let block = Block::default()
            .title("Tasks")
            .borders(Borders::ALL);

        let block_rect = block.inner(rect);
        frame.render_widget(block, rect);
        let rect = block_rect;

        let widest_badge_used = state.store.tasks.iter()
            .filter_map(|task| state.store.get_badge(&task))
            .map(|badge| badge.name.len())
            .max()
            .unwrap_or(0) as u16;

        let editor_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(widest_badge_used),
                Constraint::Length(1),
                Constraint::Length(rect.width - widest_badge_used)
            ]).split(rect);

        let task_status_list: Vec<ListItem> = state.store.tasks.iter()
            .map(|task| {
                let badge = state.store.get_badge(task);
                let color = badge.map(|badge| badge.color).unwrap_or_else(|| Color::Black);
                let name = badge.map(|badge| badge.name.clone()).unwrap_or_else(|| String::new());
                ListItem::new(format!("{}", name))
                    .style(Style::default().bg(color))
            }).collect();

        frame.render_widget(List::new(task_status_list), editor_layout[0]);
        self.text_area.draw(frame, editor_layout[2]);
    }

    fn handle_event(&mut self, state: &mut LetterState, event: LetterEvent) -> WindowCommand {

        let (_, y) = self.text_area.get_cursor();
        if let LetterEvent::CommandEvent(LetterCommand::Delete(DeleteCommand::DeleteLine)) = event {
            if state.store.tasks.len() > y {
                if let Err(_) = state.store.delete_task(y as i64) {
                    error!("couldn't delete task at {y}")
                }
            }
        }

        self.text_area.handle_letter_event(event)
    }
}

enum _WindowCommand {
    Quit,
    SwitchMode(LetterMode),
}

type WindowCommand = Option<_WindowCommand>;

struct WindowManager {
    windows: Vec<Box<dyn Window>>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    state: LetterState,

    keycommand_composer: KeyCommandComposer<LetterCommand>,
    letter_command_receiver: Receiver<LetterCommand>
}

impl WindowManager {
    fn new(store: TaskStore) -> Self {
        let windows = vec![];

        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend).expect("couldn't initiate terminal");

        let state = LetterState::new(store);

        let (mut keycommand_composer, rx) = KeyCommandComposer::new();
        keycommand_composer.register_keycommand(vec![KeyCode::Char('h')], LetterCommand::MoveCursor(CursorDirection::Left));
        keycommand_composer.register_keycommand(vec![KeyCode::Char('j')], LetterCommand::MoveCursor(CursorDirection::Down));
        keycommand_composer.register_keycommand(vec![KeyCode::Char('k')], LetterCommand::MoveCursor(CursorDirection::Up));
        keycommand_composer.register_keycommand(vec![KeyCode::Char('l')], LetterCommand::MoveCursor(CursorDirection::Right));
        keycommand_composer.register_keycommand(vec![KeyCode::Char('w')], LetterCommand::MoveCursor(CursorDirection::OneWordForward));
        keycommand_composer.register_keycommand(vec![KeyCode::Char('b')], LetterCommand::MoveCursor(CursorDirection::OneWordBackward));
        keycommand_composer.register_keycommand(vec![KeyCode::Char('i')], LetterCommand::SwitchMode(LetterMode::Insert));
        keycommand_composer.register_keycommand(vec![KeyCode::Char(' '), KeyCode::Char('q')], LetterCommand::Quit);
        keycommand_composer.register_keycommand(vec![KeyCode::Char('d'), KeyCode::Char('d')], LetterCommand::Delete(DeleteCommand::DeleteLine));

        WindowManager { windows, terminal, state, keycommand_composer, letter_command_receiver: rx }
    }

    fn handle_window_command(&mut self, window_idx: usize, cmd: &WindowCommand) {
        if let Some(cmd) = cmd {
            match cmd {
                _WindowCommand::Quit => {
                    self.windows.remove(window_idx);
                    if self.windows.len() == 0 {
                        exit(0)
                    }
                },
                _WindowCommand::SwitchMode(mode) => {
                    self.keycommand_composer.clear_composition();
                    self.state.mode = *mode;
                }
            }
        }
    }

    fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        self.terminal.clear()?;

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            loop {
                if event::poll(Duration::from_millis(50)).unwrap() {
                    if let event::Event::Key(key_event) = event::read().unwrap() {
                        tx.send(key_event).unwrap();
                    }
                }
            }
        });

        loop {
            let cmds: Vec<(usize, WindowCommand)> = self.windows.iter_mut()
                .enumerate()
                .map(|(idx, window)| (idx, window.update(&mut self.state)))
                .collect();

            cmds.iter().for_each(|(window_idx, cmd)| {
                self.handle_window_command(*window_idx, cmd)
            });

            if let Ok(key_event) = rx.try_recv() {
                match self.state.mode {
                    LetterMode::Normal => {
                        self.keycommand_composer.push_key(key_event.code);
                        if let Ok(cmd) = self.letter_command_receiver.try_recv() {
                            let last_idx = self.windows.len() - 1;
                            let wcmd = self.windows.get_mut(last_idx).unwrap()
                                .handle_event(&mut self.state, LetterEvent::CommandEvent(cmd));

                            self.handle_window_command(last_idx, &wcmd);
                        }
                    },
                    LetterMode::Insert => {
                        let last_idx = self.windows.len() - 1;
                        let cmd = self.windows.get_mut(last_idx).unwrap()
                            .handle_event(&mut self.state, LetterEvent::RawKeyInputEvent(key_event.code));

                        self.handle_window_command(last_idx, &cmd);
                    }
                }
            }

            self.terminal.draw(|frame| {
                let percentage_per_window = 100 / self.windows.len() as u16;

                let constraints: Vec<Constraint> = self.windows.iter()
                    .map(|_| Constraint::Percentage(percentage_per_window))
                    .collect();

                let panel_grid = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(99),
                        Constraint::Percentage(1)
                    ])
                    .split(frame.size());

                let window_grid = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(constraints)
                    .split(panel_grid[0]);

                self.windows.iter()
                    .enumerate()
                    .for_each(|(idx, window)| {
                        window.draw(&self.state, frame, window_grid[idx]);
                    });

                Self::draw_status_bar(&self.state, frame, panel_grid[1]);
            })?;
        }

    }

    fn draw_status_bar(state: &LetterState, frame: &mut Frame, rect: Rect) {
        let status_paragraph = Paragraph::new(format!("-- {} --", state.mode));
        frame.render_widget(status_paragraph, rect);
    }

    fn push_window(&mut self, window: Box<dyn Window>) {
        self.windows.push(window);
    }
}

fn create_database_connection() -> Result<Connection> {
    let db_path_str = "./.letter.db";

    let db_path = PathBuf::from(db_path_str);
    if !db_path.exists() {
        File::create(db_path)?;
    }

    Connection::open(db_path_str)
        .map_err(|_| "cannot open sqlite database file".into())
}

#[derive(Clone, Copy)]
enum CursorDirection {
    Left,
    Up,
    Right,
    Down,
    OneWordForward,
    OneWordBackward
}

#[derive(Clone, Copy)]
enum DeleteCommand {
    DeleteLine,
    DeleteChar,
}

#[derive(Clone, Copy)]
enum LetterCommand {
    MoveCursor(CursorDirection),
    Delete(DeleteCommand),
    Quit,
    SwitchMode(LetterMode),
}

#[derive(Clone, Copy)]
enum LetterEvent {
    CommandEvent(LetterCommand),
    RawKeyInputEvent(KeyCode)
}

fn main() -> Result<()> {
    let connection = create_database_connection()?;

    let mut task_store = TaskStore::new(connection);
    task_store.fetch_data()?;

    let mut window_manager = WindowManager::new(task_store);
    let task_list_window = TaskListWindow::new(&window_manager.state.store);
    window_manager.push_window(Box::new(task_list_window));
    window_manager.run()?;

    Ok(())
}

//fn main() -> Result<()> {
//    let connection = create_database_connection()?;
//    let mut task_store = TaskStore::new(connection);
//    task_store.fetch_data()?;
//
//    let mut letter = Letter::new(task_store);
//    let mut editor = LetterEditor::default();
//
//    editor.init(&letter)?;
//
//    loop {
//        let cmd_out = editor.update(&mut letter);
//        if let Some(cmd) = cmd_out {
//            match cmd {
//                LetterCommand::Quit => break,
//                LetterCommand::Debug => editor.debug_panel_shown = !editor.debug_panel_shown,
//                _ => {}
//            }
//        }
//
//        editor.draw(&mut letter);
//    }
//
//    editor.deinit()?;
//
//    Ok(())
//}

