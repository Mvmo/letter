mod ui;
mod command;
mod store;

use rusqlite::Connection;
use store::TaskStore;
use ui::start_ui;

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

    start_ui(task_store)?;

    Ok(())
}

