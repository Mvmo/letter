mod ui;
mod command;
mod store;
mod app;

use std::{path::PathBuf, fs::File, io::Stdout};

use app::{Letter, LetterEditor, EditorMode, LetterCommand};
use ratatui::{prelude::{CrosstermBackend, Rect, Layout, Direction, Constraint}, Terminal};
use rusqlite::Connection;
use store::TaskStore;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<Stdout>>;

enum LetterMode {
    Normal,
    Insert
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
    fn update(&mut self, state: &mut LetterState) -> WindowCommand;
    fn draw(&self, state: &LetterState, frame: &mut Frame, rect: Rect);
}

struct TaskListWindow {}

enum _WindowCommand {
    Quit
}

type WindowCommand = Option<_WindowCommand>;

impl Window for TaskListWindow {
    fn update(&mut self, _state: &mut LetterState) -> WindowCommand {
        None
    }

    fn draw(&self, _state: &LetterState, _frame: &mut Frame, _rect: Rect) {
    }
}

struct WindowManager {
    windows: Vec<Box<dyn Window>>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl WindowManager {
    fn run(&mut self, store: TaskStore) -> Result<()> {
        let mut state = LetterState::new(store);

        loop {
            self.windows.iter_mut()
                .for_each(|window| {
                    window.update(&mut state);
                });

            self.terminal.draw(|frame| {
                // take up equal space for every window horizontally
                let layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([]);

                self.windows.iter()
                    .for_each(|window| {
                        window.draw(&state, frame, Rect::new(32, 32, 80, 80));
                    });
            })?;
        }
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

fn main() -> Result<()> {
    let connection = create_database_connection()?;
    let mut task_store = TaskStore::new(connection);
    task_store.fetch_data()?;

    let mut letter = Letter::new(task_store);
    let mut editor = LetterEditor::default();

    editor.init(&letter)?;

    loop {
        let cmd_out = editor.update(&mut letter);
        if let Some(cmd) = cmd_out {
            match cmd {
                LetterCommand::Quit => break,
                LetterCommand::Debug => editor.debug_panel_shown = !editor.debug_panel_shown,
                _ => {}
            }
        }

        editor.draw(&mut letter);
    }

    editor.deinit()?;

    Ok(())
}

