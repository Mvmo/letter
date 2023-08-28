mod ui;
mod command;
mod store;
mod app;

use std::{path::PathBuf, fs::File, io::Stdout};

use app::{Letter, LetterEditor, EditorMode, LetterCommand};
use ratatui::{Frame, prelude::{CrosstermBackend, Rect}};
use rusqlite::Connection;
use store::TaskStore;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

type DrawFrame<'a> = (Frame<'a, CrosstermBackend<Stdout>>, Rect);

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
    fn update(&mut self, state: &mut LetterState);
    fn draw(&self, state: &LetterState, draw_frame: DrawFrame);
}

struct TaskListWindow {}

impl Window for TaskListWindow {
    fn update(&mut self, state: &mut LetterState) {
    }

    fn draw(&self, state: &LetterState, draw_frame: DrawFrame) {
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

