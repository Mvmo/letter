mod ui;
mod command;

use core::fmt;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;
use std::{path::PathBuf, fs::File, io::BufReader};
use std::io::{Read, BufWriter, Write, self, Stdout};

use crossterm::cursor::{SetCursorShape, CursorShape};
use crossterm::event::{EnableMouseCapture, DisableMouseCapture, self, KeyEvent};
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen};
use crossterm::execute;

use ratatui::layout::{Rect, Layout, Direction, Constraint};
use ratatui::widgets::Paragraph;
use ratatui::{Terminal, Frame};
use ratatui::backend::CrosstermBackend;
use ui::panel::Panel;
use ui::panel::overview_panel::OverviewPanel;

static DEFAULT_LOCATION: &str = "tasks";

struct TaskStore {
    path: PathBuf,
    tasks: Vec<Task>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TaskState {
    Todo,
    InProgress,
    Done,
    Unkown
}

#[derive(Clone)]
pub struct Task {
    state: TaskState,
    text: String,
}

impl TaskState {
    fn next(&self) -> Self {
        match self {
            Self::Todo => Self::InProgress,
            Self::InProgress => Self::Done,
            Self::Done => Self::Todo,
            Self::Unkown => Self::Todo,
        }
    }
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Todo => "◻️",
            Self::InProgress => "🟨",
            Self::Done => "✅",
            Self::Unkown => "🚫"
        })
    }
}

impl<'a> fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.state, self.text)
    }
}

impl Into<String> for TaskState {
    fn into(self) -> String {
        match self {
            TaskState::Todo => String::from("T"),
            TaskState::InProgress => String::from("P"),
            TaskState::Done => String::from("D"),
            TaskState::Unkown => String::from("?")
        }
    }
}

impl From<&str> for TaskState {
    fn from(value: &str) -> Self {
        match value {
            "T" => TaskState::Todo,
            "P" => TaskState::InProgress,
            "D" => TaskState::Done,
            _ => TaskState::Unkown
        }
    }
}



impl Into<String> for Task {
    fn into(self) -> String {
        let state_str: String = self.state.into();
        return format!("{} {}", state_str, self.text);
    }
}

impl Task {
    fn from_line(line: impl Into<String>) -> Option<Task> {
        let line: String = line.into();
        let mut splitted = line.splitn(2, " ");
        let state = TaskState::from(splitted.next()?.clone());
        let text = splitted.next()?.clone();

        let task = Task { state: state.clone(), text: text.to_string() };
        Some(task)
    }
}

impl<'a> TaskStore {
    fn new(path: PathBuf) -> Self {
        match File::open(path.clone()) {
            Ok(file) => {
                let mut reader = BufReader::new(file);
                let mut str = String::new();
                reader.read_to_string(&mut str).expect("BITTE");

                let tasks: Vec<Task> = str.to_owned().split("\n")
                    .into_iter()
                    .filter_map(|line| Task::from_line(line))
                    .collect();

                TaskStore { tasks, path }
            },
            Err(err) => {
                eprintln!("Error opening file: {}", err);
                TaskStore { tasks: vec![], path }
            }
        }
    }

    fn _add_task(&mut self, task: Task) {
        self.tasks.push(task)
    }

    fn save(&self) {
        let file = File::create(self.path.clone());
        if let Ok(file) = file {
            let mut writer = BufWriter::new(file);
            self.tasks.iter()
                .for_each(|task| {
                    let task = task.clone();
                    let task_str: String = task.clone().into();
                    writer.write(format!("{}\n", task_str).as_bytes()).expect("couldn't write task to file :(");
                });

            writer.flush().expect("couldn't flush file");
        }
    }

}

impl<'a> From<PathBuf> for TaskStore {
    fn from(path: PathBuf) -> Self {
        TaskStore { path, tasks: vec![] }
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let task_store = TaskStore::new(PathBuf::from(DEFAULT_LOCATION));
    start_ui(task_store)?;

    Ok(())
}

fn start_ui(store: TaskStore) -> Result<(), Box<dyn std::error::Error>>{
    let mut stdout = io::stdout();
    let cursor_style = CursorShape::Line;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, SetCursorShape(cursor_style))?;

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
            UpdateResult::UpdateMode(mode) => app_state.mode = mode,
            UpdateResult::Save => app_state.task_store.save(),
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

    draw_status_bar(frame, state);
}

fn draw_status_bar(frame: &mut Frame<CrosstermBackend<Stdout>>, state: &AppState) {
    let status_bar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
        .split(frame.size());

    let status_bar_paragraphs_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(33)])
        .split(status_bar_chunks[1]);

    let state = state;

    let mode_str: String = state.mode.clone().into();
    let mode_p = Paragraph::new(mode_str);

    frame.render_widget(mode_p, status_bar_paragraphs_chunks[0]);
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

fn spawn_key_listener() -> Result<Receiver<KeyEvent>, Box<dyn std::error::Error>> {
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

