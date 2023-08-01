use core::fmt;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;
use std::{path::PathBuf, fs::File, io::BufReader};
use std::io::{Read, BufWriter, Write, self, Stdout};

use crossterm::event::{EnableMouseCapture, DisableMouseCapture, self, KeyEvent, KeyCode};
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen};
use crossterm::execute;

use tui::{Terminal, Frame};
use tui::backend::{CrosstermBackend, Backend};
use tui::widgets::{Block, Borders, ListState, List, ListItem};

static DEFAULT_LOCATION: &str = "tasks";

struct TaskStore {
    path: PathBuf,
    tasks: Vec<Task>,

    list_state: ListState
}

#[derive(Clone, Copy)]
enum TaskState {
    Done,
    Working,
    Waiting,
    Unkown
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Done => "Done",
            Self::Working => "Working",
            Self::Waiting => "Waiting",
            Self::Unkown => "?"
        })
    }
}

impl<'a> fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.state, self.text)
    }
}

impl Into<String> for TaskState {
    fn into(self) -> String {
        match self {
            TaskState::Done => String::from("D"),
            TaskState::Working => String::from("W"),
            TaskState::Waiting => String::from("X"),
            TaskState::Unkown => String::from("?")
        }
    }
}

impl From<&str> for TaskState {
    fn from(value: &str) -> Self {
        match value {
            "D" => TaskState::Done,
            "W" => TaskState::Working,
            "X" => TaskState::Waiting,
            _ => TaskState::Unkown
        }
    }
}

#[derive(Clone)]
struct Task {
    state: TaskState,
    text: String,
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

impl TaskStore {
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

                TaskStore { tasks, path, list_state: ListState::default() }
            },
            Err(err) => {
                eprintln!("Error opening file: {}", err);
                TaskStore { tasks: vec![], path, list_state: ListState::default() }
            }
        }
    }

    fn add_task(&mut self, task: Task) {
        self.tasks.push(task)
    }

    fn save(&self) {
        let file = File::create(self.path.clone());
        if let Ok(file) = file {
            let mut writer = BufWriter::new(file);
            self.tasks.iter()
                .for_each(|task| {
                    let task_str: String = (*task).clone().into();
                    writer.write(format!("{}\n", task_str).as_bytes()).expect("couldn't write task to file :(");
                });

            writer.flush().expect("couldn't flush file");
        }
    }

    pub fn next(&mut self) {
        println!("next");
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.tasks.len() - 1 {
                    0
                } else {
                    i + 1
                }
            },
            None => 0
        };
        println!("next has {}", i);

        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tasks.len() - 1
                } else {
                    i - 1
                }
            },
            None => 0
        };

        self.list_state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.list_state.select(None)
    }
}

impl<'a> From<PathBuf> for TaskStore {
    fn from(path: PathBuf) -> Self {
        TaskStore { path, tasks: vec![], list_state: ListState::default() }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut task_store = TaskStore::new(PathBuf::from(DEFAULT_LOCATION));
    task_store.add_task(Task { state: TaskState::Waiting, text: String::from("hallo, welt") });
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

    let rx = spawn_key_listener()?;

    loop {
        let update_result = update(store, &rx)?;
        match update_result {
            UpdateResult::Quit => break,
            _ => {}
        }
        terminal.draw(|f| draw_ui(f, store))?;
    }

    disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}

enum UpdateResult {
    Quit,
    None
}

enum AppMode {
    NORMAL
}

struct AppState<'a> {
    task_store: &'a TaskStore,
    mode: AppMode
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

fn update(store: &TaskStore, rx: &Receiver<KeyEvent>) -> Result<UpdateResult, Box<dyn std::error::Error>> {
    if let Ok(key_event) = rx.try_recv() {
        match key_event.code {
            KeyCode::Char('q') => {
                return Ok(UpdateResult::Quit);
            },
            _ => {}
        }
    }
    Ok(UpdateResult::None)
}

fn update_normal_mode() {

}

fn draw_ui(frame: &mut Frame<CrosstermBackend<Stdout>>, store: &TaskStore) {
    let my_list = List::new(vec![ListItem::new("hallo"), ListItem::new("hello")]);
    frame.render_widget(my_list, frame.size());
}
