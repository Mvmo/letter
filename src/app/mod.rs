use std::{io::{self, Stdout}, sync::{mpsc::{Receiver, self}, Mutex, Arc}, time::Duration, thread};

use crossterm::{execute, terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, event::{KeyEvent, self}};
use ratatui::{prelude::CrosstermBackend, Terminal};

use crate::{Result, ui::panel::{overview_panel::OverviewPanel, Panel}, store::TaskStore};

pub type LBackend = CrosstermBackend<Stdout>;
pub type LTerminal = Terminal<LBackend>;

pub struct LetterEditor {
    pub terminal: LTerminal,
    pub key_event_receiver: Arc<Mutex<Receiver<KeyEvent>>>,
    pub overview_panel: OverviewPanel
}

impl LetterEditor {
    pub fn init(&mut self, letter: &Letter) -> Result<()> {
        execute!(io::stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;
        self.terminal.clear()?;
        self.overview_panel.init(letter);
        Ok(())
    }

    pub fn deinit(&mut self) -> Result<()> {
        execute!(io::stdout(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn draw(&mut self, letter: &Letter) {
        self.terminal.draw(|frame| {
            self.overview_panel.draw(frame, frame.size(), letter)
        }).expect(":(");
    }

    pub fn update(&mut self, letter: &mut Letter) -> Option<LetterCommand> {
        self.overview_panel.update(letter)
    }
}

impl Default for LetterEditor {
    fn default() -> Self {
        let backend = CrosstermBackend::new(io::stdout());
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
    pub task_store: TaskStore,
    pub editor_mode: EditorMode,
}

pub enum LetterCommand {
    Quit,
    Save
}

