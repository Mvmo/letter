mod ui;
mod command;

use std::{fs::{File, OpenOptions}, collections::HashMap, str::FromStr, io::{self, Stdout}, sync::{Arc, Mutex, mpsc::{Receiver, self}}, thread, time::Duration, path::Path};
use crossterm::{execute, terminal::{EnterAlternateScreen, enable_raw_mode, disable_raw_mode, LeaveAlternateScreen}, event::{EnableMouseCapture, DisableMouseCapture, KeyEvent, self}, cursor::{SetCursorShape, CursorShape}};
use ratatui::{style::Color, prelude::{CrosstermBackend, Direction, Constraint, Rect, Layout}, Terminal, Frame};
use sqlx::{SqliteConnection, Connection, FromRow, sqlite::SqliteRow, Row};
use ui::panel::{overview_panel::OverviewPanel, Panel};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Badge {
    id: i32,
    name: String,
    color: Color
}

impl<'a> FromRow<'a, SqliteRow> for Badge {
    fn from_row(row: &'a SqliteRow) -> std::result::Result<Self, sqlx::Error> {
        let badge_id = row.try_get::<i32, &str>("id")?;
        let badge_name = row.try_get::<String, &str>("name")?;
        let badge_color_str = row.try_get::<String, &str>("color")?;

        let badge_color = Color::from_str(&badge_color_str)
            .map_err(|_| sqlx::Error::Decode("couldn't decode color".into()))?;

        Ok(Badge {
            id: badge_id,
            name: badge_name,
            color: badge_color
        })
    }
}

pub struct Task {
    id: Option<i32>,
    text: String,
    badge_id: Option<i32>
}

impl<'a> FromRow<'a, SqliteRow> for Task {
    fn from_row(row: &'a SqliteRow) -> std::result::Result<Self, sqlx::Error> {
        let task_id = row.try_get::<i32, &str>("id")?;
        let task_text = row.try_get::<String, &str>("text")?;
        let task_badge_id = row.try_get::<Option<i32>, &str>("badge_id")?;

        Ok(Self {
            id: Some(task_id),
            text: task_text,
            badge_id: task_badge_id
        })
    }
}

pub struct TaskStore {
    connection: SqliteConnection,

    pub badges: HashMap<i32, Badge>,
    pub tasks: Vec<Task>
}

impl TaskStore {
    pub fn new(connection: SqliteConnection) -> Self {
        TaskStore {
            connection,
            badges: HashMap::new(),
            tasks: vec![]
        }
    }

    async fn ensure_proper_setup(&mut self) -> Result<()> {
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS badges (
                id    INTEGER PRIMARY KEY NOT NULL,
                name  TEXT                NOT NULL,
                color TEXT                NOT NULL /* ansi color format */
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id       INTEGER PRIMARY KEY NOT NULL,
                text     TEXT                NOT NULL,
                badge_id INTEGER,

                FOREIGN KEY (badge_id) REFERENCES badges (id)
            );

            INSERT INTO badges (name, color)
                SELECT 'TODO', '#e5b562'
                UNION ALL
                SELECT 'In Progress', '#10edbd'
                UNION ALL
                SELECT 'Done', '#11ed15'
            WHERE (SELECT count(*) FROM badges) = 0;
        "#).execute(&mut self.connection)
            .await?;

        Ok(())
    }

    pub async fn fetch_data(&mut self) -> Result<()> {
        self.ensure_proper_setup().await?;

        self.badges = sqlx::query_as::<_, Badge>("SELECT * FROM badges")
            .fetch_all(&mut self.connection)
            .await?
            .drain(..)
            .map(|badge| (badge.id, badge))
            .collect();

        self.tasks = sqlx::query_as::<_, Task>("SELECT * FROM tasks")
            .fetch_all(&mut self.connection)
            .await?;

        Ok(())
    }

    pub async fn create_task(&mut self, task: Task) -> Result<()> {
        sqlx::query("INSERT INTO tasks (text, badge_id) VALUES ($1, $2);")
            .bind(task.text)
            .bind(task.badge_id)
            .execute(&mut self.connection)
            .await?;

        Ok(())
    }

    pub async fn close(self) {
        self.connection.close();
    }
}

async fn create_database_connection() -> Result<SqliteConnection> {
    let database_path_str = "./.letter.db";

    //OpenOptions::new().create(true).truncate(false).open(Path::new(database_path_str))?;

    SqliteConnection::connect(database_path_str)
        .await
        .map_err(|_| "cannot open sqlite database file".into())
}

#[async_std::main]
async fn main() -> Result<()> {
    let connection = create_database_connection().await?;
    let mut task_store = TaskStore::new(connection);
    task_store.fetch_data().await?;
    task_store.create_task(Task { id: None, text: "hallo welt".to_string(), badge_id: None }).await?;

    start_ui(task_store)?;
    //task_store.close().await;

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

