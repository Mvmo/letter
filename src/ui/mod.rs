use std::{io::{self, Stdout}, sync::{Arc, Mutex, mpsc::{Receiver, self}}, thread, time::Duration};

use ratatui::{prelude::CrosstermBackend, Terminal, Frame};
use crossterm::{execute, terminal::{EnterAlternateScreen, enable_raw_mode, LeaveAlternateScreen, disable_raw_mode}, event::{EnableMouseCapture, DisableMouseCapture, KeyEvent, self}, cursor::{SetCursorShape, CursorShape}};

use crate::{store::TaskStore, Result};

use self::panel::{overview_panel::OverviewPanel, Panel};

pub mod panel;
pub mod textarea;

pub fn start_ui(store: TaskStore) -> Result<()> {
    let mut stdout = io::stdout(); execute!(stdout, EnterAlternateScreen, EnableMouseCapture, SetCursorShape(CursorShape::Block))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    enable_raw_mode()?;

    let rx = spawn_key_listener()?;
    let rx_arc_mutex = Arc::new(Mutex::new(rx));
    let mut app_state = AppState { task_store: store, mode: AppMode::NORMAL};

    let mut overview_panel = Box::new(OverviewPanel::new(rx_arc_mutex.clone()));
    overview_panel.init(&app_state);
    let mut panel_stack: Vec<Box<dyn Panel>> = vec![overview_panel];

    loop {
        let top_frame: &mut Box<dyn Panel> = panel_stack.last_mut().unwrap();
        let update_result = top_frame.update(&mut app_state);
        match update_result {
            UpdateResult::Quit => break,
            UpdateResult::UpdateMode(mode) => {
                app_state.mode = mode;
            },
            UpdateResult::Save => {},// app_state.task_store.save(),
            UpdateResult::None => {}
        }
        terminal.draw(|f| draw_ui(f, &mut panel_stack, &app_state))?;
    }

    disable_raw_mode()?;
    terminal.show_cursor()?;

    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn draw_ui(frame: &mut Frame<CrosstermBackend<Stdout>>, panel_stack: &mut [Box<dyn Panel>], state: &AppState) {
    panel_stack.iter_mut()
        .rev()
        .for_each(|panel| {
            panel.draw(frame, frame.size(), state)
        });
}

pub enum UpdateResult {
    Quit,
    UpdateMode(AppMode),
    Save,
    None
}

#[derive(Clone, Copy)]
pub enum AppMode {
    NORMAL,
    INPUT,
}

impl Into<String> for AppMode {
    fn into(self) -> String {
        match self {
            Self::NORMAL => String::from("NORMAL"),
            Self::INPUT => String::from("INPUT"),
        }
    }
}

pub struct AppState {
    task_store: TaskStore,
    mode: AppMode,
}

fn spawn_key_listener() -> Result<Receiver<KeyEvent>> {
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

    Ok(rx)
}



