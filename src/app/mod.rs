use std::{io::{self, Stdout}, sync::{mpsc::{Receiver, self}, Mutex, Arc}, time::Duration, thread};

use crossterm::{execute, terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, event::{KeyEvent, self}};
use ratatui::{prelude::{CrosstermBackend, Layout, Direction}, Terminal};

use crate::{Result, ui::panel::{overview_panel::OverviewPanel, Panel, debug_panel::DebugPanel}, store::TaskStore};

use self::logger::LetterLogger;

pub mod logger;

pub type LBackend = CrosstermBackend<Stdout>;
pub type LTerminal = Terminal<LBackend>;

pub struct LetterEditor {
    pub terminal: LTerminal,
    pub key_event_receiver: Arc<Mutex<Receiver<KeyEvent>>>,
    pub overview_panel: OverviewPanel,
    pub debug_panel: Arc<Mutex<DebugPanel>>
}

impl LetterEditor {
    pub fn init(&mut self, letter: &Letter) -> Result<()> {
        execute!(io::stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;
        self.terminal.clear()?;
        self.overview_panel.init(letter);

        LetterLogger::init(log::LevelFilter::Info, self.debug_panel.clone())?;
        log::info!("Hallo, welt!");
        log::info!("Hallo, welt!");
        log::info!("HALALASLDFHLASDKFK JASDKLFJ KLSDA:J FKLJASDKL: JF");

        Ok(())
    }

    pub fn deinit(&mut self) -> Result<()> {
        execute!(io::stdout(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn draw(&mut self, letter: &Letter) {
        self.terminal.draw(|frame| {
            self.overview_panel.draw(frame, frame.size(), letter);
            let mut right_top_rect = frame.size().clone();
            right_top_rect.width = 20 * 4;
            right_top_rect.y = 0;
            right_top_rect.x = frame.size().width - right_top_rect.width;
            right_top_rect.height = 20;


            self.debug_panel.lock().unwrap()
                .draw(frame, right_top_rect, letter);
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
            overview_panel,
            debug_panel: Arc::new(Mutex::new(DebugPanel::default()))
        }
    }
}

pub enum EditorMode {
    Normal,
    Insert
}

impl ToString for EditorMode {
    fn to_string(&self) -> String {
        match *self {
            EditorMode::Insert => String::from("INSERT"),
            EditorMode::Normal => String::from("NORMAL")
        }
    }
}

pub struct Letter {
    pub task_store: TaskStore,
    pub editor_mode: EditorMode,
}

pub enum LetterCommand {
    Quit,
    Save
}

