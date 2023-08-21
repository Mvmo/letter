mod ui;
mod command;
mod store;
mod app;

use app::{Letter, LetterEditor, EditorMode, LetterCommand};
use rusqlite::Connection;
use store::TaskStore;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn create_database_connection() -> Result<Connection> {
    let database_path_str = "./.letter.db";
    // TODO - create file if it doesn't exist // OpenOptions::new().create(true).truncate(false).open(Path::new(database_path_str))?;

    Connection::open(database_path_str)
        .map_err(|_| "cannot open sqlite database file".into())
}

fn main() -> Result<()> {
    let connection = create_database_connection()?;
    let mut task_store = TaskStore::new(connection);
    task_store.fetch_data()?;

    let mut letter = Letter { task_store, editor_mode: EditorMode::Normal };
    let mut editor = LetterEditor::default();

    editor.init(&letter)?;

    loop {
        if let Some(command) = editor.update(&mut letter) {
            match command {
                LetterCommand::Quit => break,
                _ => {}
            }
        }

        editor.draw(&mut letter);
    }

    editor.deinit()?;

    Ok(())
}

