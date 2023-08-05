mod ui;

use core::fmt;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;
use std::{path::PathBuf, fs::File, io::BufReader};
use std::io::{Read, BufWriter, Write, self};

use crossterm::event::{EnableMouseCapture, DisableMouseCapture, self, KeyEvent};
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen};
use crossterm::execute;

use tui::layout::{Rect, Layout, Direction, Constraint};
use tui::Terminal;
use tui::backend::CrosstermBackend;
use ui::panel::Panel;
use ui::panel::overview_panel::OverviewPanel;

static DEFAULT_LOCATION: &str = "tasks";

struct TaskStore {
    path: PathBuf,
    tasks: Arc<Mutex<Vec<Arc<Mutex<Task>>>>>,
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

                let tasks: Arc<Mutex<Vec<Arc<Mutex<Task>>>>> = Arc::new(Mutex::new(str.to_owned().split("\n")
                    .into_iter()
                    .filter_map(|line| Task::from_line(line))
                    .map(|t| Arc::new(Mutex::new(t)))
                    .collect()));

                TaskStore { tasks, path }
            },
            Err(err) => {
                eprintln!("Error opening file: {}", err);
                TaskStore { tasks: Arc::new(Mutex::new(vec![])), path }
            }
        }
    }

    fn add_task(&mut self, task: Task) {
        self.tasks.lock().unwrap().push(Arc::new(Mutex::new(task)))
    }

    fn save(&self) {
        let file = File::create(self.path.clone());
        if let Ok(file) = file {
            let mut writer = BufWriter::new(file);
            self.tasks.lock().unwrap().iter()
                .for_each(|task| {
                    let task = task.clone();
                    let task_str: String = task.lock().unwrap().clone().into();
                    writer.write(format!("{}\n", task_str).as_bytes()).expect("couldn't write task to file :(");
                });

            writer.flush().expect("couldn't flush file");
        }
    }

}

impl<'a> From<PathBuf> for TaskStore {
    fn from(path: PathBuf) -> Self {
        TaskStore { path, tasks: Arc::new(Mutex::new(vec![])) }
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut task_store = TaskStore::new(PathBuf::from(DEFAULT_LOCATION));
    task_store.add_task(Task { state: TaskState::Todo, text: String::from("hallo, welt") });
    task_store.save();

    start_ui(&mut task_store)?;

    Ok(())
}

fn start_ui(store: &mut TaskStore) -> Result<(), Box<dyn std::error::Error>>{

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    enable_raw_mode()?;

    let rx = Arc::new(Mutex::new(spawn_key_listener()?));
    let app_state = Arc::new(Mutex::new(AppState { task_store: store, mode: AppMode::NORMAL}));

    let mut frame_buffers: Vec<Box<dyn Panel>> = vec![Box::new(OverviewPanel::new(app_state.clone(), rx))];

    loop {
        let top_frame: &mut Box<dyn Panel> = frame_buffers.last_mut().unwrap();
        let update_result = top_frame.update();
        match update_result {
            UpdateResult::Quit => break,
            UpdateResult::UpdateMode(mode) => app_state.lock().unwrap().mode = mode,
            UpdateResult::Save => app_state.lock().unwrap().task_store.save(),
            UpdateResult::None => {}
        }
        terminal.draw(|f| top_frame.draw(f))?;
    }

    disable_raw_mode()?;
    terminal.show_cursor()?;

    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

pub enum UpdateResult {
    Quit,
    UpdateMode(AppMode),
    Save,
    None
}

#[derive(Clone)]
pub enum AppMode {
    NORMAL,
    INPUT(String),
    EDIT(Arc<Mutex<Task>>, String)
}

impl Into<String> for AppMode {
    fn into(self) -> String {
        match self {
            Self::NORMAL => String::from("NORMAL"),
            Self::INPUT(_) => String::from("INPUT"),
            Self::EDIT(_, _) => String::from("EDIT")
        }
    }
}

pub struct AppState<'a> {
    task_store: &'a TaskStore,
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

