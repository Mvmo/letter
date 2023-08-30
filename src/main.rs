mod ui;
mod command;
mod store;
mod app;

use std::{path::PathBuf, fs::File, io::{Stdout, stdout}, fmt::Display};

use app::{Letter, EditorMode, LetterCommand};
use crossterm::terminal::enable_raw_mode;
use ratatui::{prelude::{CrosstermBackend, Rect, Layout, Direction, Constraint}, Terminal, widgets::{Block, Borders, Paragraph}};
use rusqlite::Connection;
use store::TaskStore;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<Stdout>>;

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
    fn update(&mut self, state: &mut LetterState) -> WindowCommand;
    fn draw(&self, state: &LetterState, frame: &mut Frame, rect: Rect);
}

struct TestWindow {}

impl Default for TestWindow {
    fn default() -> Self {
        return TestWindow {  }
    }
}

impl Window for TestWindow {
    fn update(&mut self, _state: &mut LetterState) -> WindowCommand {
        None
    }

    fn draw(&self, _state: &LetterState, frame: &mut Frame, rect: Rect) {
        let block = Block::default()
            .title("LKJKLJLK")
            .borders(Borders::ALL);

        frame.render_widget(block, rect)
    }
}


struct TaskListWindow {}

impl Default for TaskListWindow {
    fn default() -> Self {
        return TaskListWindow {  }
    }
}

impl Window for TaskListWindow {
    fn update(&mut self, _state: &mut LetterState) -> WindowCommand {
        None
    }

    fn draw(&self, _state: &LetterState, frame: &mut Frame, rect: Rect) {
        let block = Block::default()
            .title("Hallo")
            .borders(Borders::ALL);

        frame.render_widget(block, rect)
    }
}

enum _WindowCommand {
    Quit
}

type WindowCommand = Option<_WindowCommand>;

struct WindowManager {
    windows: Vec<Box<dyn Window>>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    state: LetterState
}

impl WindowManager {
    fn new(store: TaskStore) -> Self {
        let windows = vec![];

        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend).expect("couldn't initiate terminal");

        let state = LetterState::new(store);

        WindowManager { windows, terminal, state }
    }

    fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        self.terminal.clear()?;

        loop {
            self.windows.iter_mut()
                .for_each(|window| {
                    window.update(&mut self.state);
                });

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

fn main() -> Result<()> {
    let connection = create_database_connection()?;

    let mut task_store = TaskStore::new(connection);
    task_store.fetch_data()?;

    //task_store.fetch_data()?;

    let mut window_manager = WindowManager::new(task_store);
    window_manager.push_window(Box::new(TaskListWindow::default()));
    window_manager.push_window(Box::new(TestWindow::default()));
    window_manager.push_window(Box::new(TestWindow::default()));
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

