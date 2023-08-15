mod ui;
mod command;
mod store;

use std::{io::{self, Stdout}, sync::{Arc, Mutex, mpsc::{Receiver, self}}, thread, time::Duration};
use crossterm::{execute, terminal::{EnterAlternateScreen, enable_raw_mode, disable_raw_mode, LeaveAlternateScreen}, event::{EnableMouseCapture, DisableMouseCapture, KeyEvent, self}, cursor::{SetCursorShape, CursorShape}};
use ratatui::{prelude::{CrosstermBackend, Direction, Constraint, Rect, Layout}, Terminal, Frame};
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

fn main() -> Result<()> {
    let connection = create_database_connection()?;
    let mut task_store = TaskStore::new(connection);
    task_store.fetch_data()?;
    // task_store.create_task(Task { id: None, text: "hallo welt".to_string(), badge_id: Some(1) })?;

    start_ui(task_store)?;

    Ok(())
}

fn start_ui(store: TaskStore) -> Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, SetCursorShape(CursorShape::Block))?;

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

 fn _centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
     let popup_layout = Layout::default()
         .direction(Direction::Vertical)
         .constraints([
             Constraint::Percentage((100 - percent_y) / 2),
             Constraint::Percentage(percent_y),
             Constraint::Percentage((100 - percent_y) / 2),
         ].as_ref())
         .split(r);

     Layout::default()
         .direction(Direction::Horizontal)
         .constraints([
             Constraint::Percentage((100 - percent_x) / 2),
             Constraint::Percentage(percent_x),
             Constraint::Percentage((100 - percent_x) / 2),
         ].as_ref())
         .split(popup_layout[1])[1]
 }

