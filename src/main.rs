mod ui;
mod command;
mod store;

use std::{io::{Stdout, stdout}, sync::{mpsc::{Receiver, self}, Arc, Mutex}, thread, time::Duration};

use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode}, event::{KeyEvent, self}};
use ratatui::{Terminal, prelude::CrosstermBackend, widgets::Paragraph};
use rusqlite::Connection;
use store::TaskStore;
use ui::panel::{overview_panel::OverviewPanel, Panel};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn create_database_connection() -> Result<Connection> {
    let database_path_str = "./.letter.db";
    // TODO - create file if it doesn't exist // OpenOptions::new().create(true).truncate(false).open(Path::new(database_path_str))?;

    Connection::open(database_path_str)
        .map_err(|_| "cannot open sqlite database file".into())
}

pub type LBackend = CrosstermBackend<Stdout>;
pub type LTerminal = Terminal<LBackend>;

struct LetterEditor {
    pub terminal: LTerminal,
    pub key_event_receiver: Arc<Mutex<Receiver<KeyEvent>>>,
    pub overview_panel: OverviewPanel
}

impl LetterEditor {
    fn init(&mut self, letter: &Letter) -> Result<()> {
        execute!(stdout(), EnterAlternateScreen)?;
        self.terminal.clear()?;
        self.overview_panel.init(letter);
        enable_raw_mode()?;
        Ok(())
    }

    fn deinit(&mut self) -> Result<()> {
        execute!(stdout(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn draw(&mut self, letter: &Letter) {
        self.terminal.draw(|frame| {
            self.overview_panel.draw(frame, frame.size(), letter)
        }).expect(":(");
    }

    fn update(&mut self, letter: &mut Letter) -> Option<LetterCommand> {
        self.overview_panel.update(letter)
    }
}

impl Default for LetterEditor {
    fn default() -> Self {
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend).unwrap();

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

        let rx = Arc::new(Mutex::new(rx));
        let overview_panel = OverviewPanel::new(rx.clone());

        Self {
            terminal,
            key_event_receiver: rx,
            overview_panel
        }
    }
}

pub enum EditorMode {
    Normal,
    Insert
}

pub struct Letter {
    task_store: TaskStore,
    editor_mode: EditorMode,
}

pub enum LetterCommand {
    Quit,
    Save
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

    editor.deinit();

    Ok(())
}

