mod ui;
mod command;

use std::{collections::HashMap, str::FromStr, io::{self, Stdout}, sync::{Arc, Mutex, mpsc::{Receiver, self}}, thread, time::Duration};
use crossterm::{execute, terminal::{EnterAlternateScreen, enable_raw_mode, disable_raw_mode, LeaveAlternateScreen}, event::{EnableMouseCapture, DisableMouseCapture, KeyEvent, self}, cursor::{SetCursorShape, CursorShape}};
use ratatui::{style::Color, prelude::{CrosstermBackend, Direction, Constraint, Rect, Layout}, Terminal, Frame};
use rusqlite::{Connection, Row};
use ui::panel::{overview_panel::OverviewPanel, Panel};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Badge {
    id: i64,
    pub name: String,
    pub color: Color
}

impl Badge {
    fn from_row(row: &Row) -> Result<Self> {
        let badge_id = row.get("id")?;
        let badge_name = row.get("name")?;
        let badge_color_str: String = row.get("color")?;

        let badge_color: Color = Color::from_str(&badge_color_str)?;

        Ok(Badge {
            id: badge_id,
            name: badge_name,
            color: badge_color
        })
    }
}

#[derive(PartialEq, Eq)]
pub struct Task {
    id: Option<i64>,
    text: String,
    badge_id: Option<i64>
}

impl Task {
    fn from_row(row: &Row) -> Result<Self> {
        let task_id = row.get("id")?;
        let task_text = row.get("text")?;
        let task_badge_id = row.get("badge_id")?;

        Ok(Self {
            id: Some(task_id),
            text: task_text,
            badge_id: task_badge_id
        })
    }
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: None,
            text: String::new(),
            badge_id: None
        }
    }
}

pub struct TaskStore {
    connection: Connection,

    pub badges: HashMap<i64, Badge>,
    pub tasks: Vec<Task>
}

impl TaskStore {
    pub fn new(connection: Connection) -> Self {
        TaskStore {
            connection,
            badges: HashMap::new(),
            tasks: vec![]
        }
    }

    fn ensure_proper_setup(&mut self) -> Result<()> {
        self.connection.execute(r#"
            CREATE TABLE IF NOT EXISTS badges (
                id    INTEGER PRIMARY KEY NOT NULL,
                name  TEXT                NOT NULL,
                color TEXT                NOT NULL /* ansi color format */
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id         INTEGER PRIMARY KEY NOT NULL,
                text       TEXT                NOT NULL,
                badge_id   INTEGER,
                sort_order INTEGER NOT NULL,

                FOREIGN KEY (badge_id) REFERENCES badges (id)
            );

            INSERT INTO badges (name, color)
                SELECT 'TODO', '#e5b562'
                UNION ALL
                SELECT 'In Progress', '#10edbd'
                UNION ALL
                SELECT 'Done', '#11ed15'
            WHERE (SELECT count(*) FROM badges) = 0;
        "#, ())?;
        Ok(())
    }

    pub fn fetch_data(&mut self) -> Result<()> {
        self.ensure_proper_setup()?;

        self.badges = self.connection.prepare("SELECT * FROM badges")?
            .query_map([], |row| {
                Badge::from_row(row)
                    .map_err(|_| rusqlite::Error::ExecuteReturnedResults)
            })?
            .filter_map(|badge| badge.ok())
            .map(|badge| (badge.id, badge))
            .collect();

        self.tasks = self.connection.prepare("SELECT * FROM tasks ORDER BY sort_order")?
            .query_map([], |row| {
                Task::from_row(row)
                    .map_err(|_| rusqlite::Error::ExecuteReturnedResults)
            })?
            .filter_map(|task| task.ok())
            .collect();

        Ok(())
    }

    fn insert_task(&mut self, sort_index: i64, task: &Task) -> Result<()> {
        self.connection.execute("UPDATE tasks SET sort_order = sort_order + 1 WHERE sort_order >= ?1", (sort_index,))?;
        self.connection.execute("INSERT INTO tasks (text, badge_id, sort_order) VALUES (?1, ?2, ?3)", (&task.text, task.badge_id, sort_index))?;

        Ok(())
    }

    pub fn create_task_at(&mut self, index: i64, task: Task) -> Result<()> {
        self.insert_task(index, &task)?;
        self.tasks.insert(index as usize, task);

        Ok(())
    }

    pub fn create_task(&mut self, task: Task) -> Result<()> {
        let index = self.tasks.len() as i64;
        self.create_task_at(index, task)?;

        Ok(())
    }

    pub fn delete_task(&mut self, task: &Task) -> Result<()> {
        self.connection.execute("DELETE FROM tasks WHERE id = ?1", (task.id,))?;
        self.tasks.retain(|t| t != task);

        Ok(())
    }

    pub fn update_task_text(&mut self, idx_sort_order: i64, text: &str) -> Result<()> {
        self.connection.execute(r#"
            UPDATE tasks
                SET text = ?1
            WHERE sort_order = ?2
        "#, (text, idx_sort_order))?;

        // TODO update list

        Ok(())
    }

    pub fn update_task_order(&mut self, task: &Task, sort_order: i64) -> Result<()> {
        self.connection.execute(r#"
            UPDATE tasks
                SET sort_order = ?1
            WHERE id = ?2
        "#, (sort_order, task.id))?;

        // TODO update list

        Ok(())
    }

    pub fn get_badge(&self, task: &Task) -> Option<&Badge> {
        let badge_id = &task.badge_id?;
        self.badges.get(badge_id)
    }

}

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
    task_store.create_task(Task { id: None, text: "hallo welt".to_string(), badge_id: Some(1) })?;

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
            panel.draw(frame, state)
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

