use core::fmt;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;
use std::{path::PathBuf, fs::File, io::BufReader};
use std::io::{Read, BufWriter, Write, self, Stdout};

use crossterm::event::{EnableMouseCapture, DisableMouseCapture, self, KeyEvent, KeyCode};
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen};
use crossterm::execute;

use tui::style::{Color, Style};
use tui::{Terminal, Frame, text};
use tui::backend::CrosstermBackend;
use tui::widgets::{Block, ListState, List, ListItem};

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
struct Task {
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

    let rx = spawn_key_listener()?;

    let mut app_state = AppState { task_store: store, mode: AppMode::NORMAL, list_state: &mut ListState::default()};
    app_state.list_state.select(Some(0));

    loop {
        let update_result = update(&mut app_state, &rx)?;
        match update_result {
            UpdateResult::Quit => break,
            UpdateResult::UpdateMode(mode) => app_state.mode = mode,
            UpdateResult::Save => app_state.task_store.save(),
            UpdateResult::None => {}
        }
        terminal.draw(|f| draw_ui(f, &mut app_state))?;
    }

    disable_raw_mode()?;
    terminal.show_cursor()?;

    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

enum UpdateResult {
    Quit,
    UpdateMode(AppMode),
    Save,
    None
}

#[derive(Clone)]
enum AppMode {
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

struct AppState<'a> {
    task_store: &'a TaskStore,
    mode: AppMode,
    list_state: &'a mut ListState,
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

fn update(state: &mut AppState, rx: &Receiver<KeyEvent>) -> Result<UpdateResult, Box<dyn std::error::Error>> {
    match state.mode {
        AppMode::NORMAL => update_normal_mode(state, rx),
        AppMode::INPUT(_) => update_input_mode(state, rx),
        AppMode::EDIT(_, _) => update_edit_mode(state, rx),
    }
}

fn update_edit_mode(state: &mut AppState<'_>, rx: &Receiver<KeyEvent>) -> Result<UpdateResult, Box<dyn std::error::Error>> {
    let edit_mode_task_and_string = match state.mode.clone() {
        AppMode::EDIT(task, str) => (task, str),
        _ => return Err(format!("update input mode got called with normal mode!!").into())
    };

    let task_ref: Arc<Mutex<Task>> = edit_mode_task_and_string.0;
    let task_ref_clone = task_ref.clone();

    let mut task = task_ref.lock().unwrap();

    let str: String = edit_mode_task_and_string.1;

    if let Ok(key_event) = rx.recv() {
        match key_event.code {
            KeyCode::Char(c) => return Ok(UpdateResult::UpdateMode(AppMode::EDIT(task_ref_clone, str + c.to_string().as_str()))),
            KeyCode::Esc => return Ok(UpdateResult::UpdateMode(AppMode::NORMAL)),
            KeyCode::Enter => {
                task.text = str;
                return Ok(UpdateResult::UpdateMode(AppMode::NORMAL))
            },
            _ => return Ok(UpdateResult::None)
        }
    }

    Ok(UpdateResult::Save)
}

fn update_normal_mode(state: &mut AppState, rx: &Receiver<KeyEvent>) -> Result<UpdateResult, Box<dyn std::error::Error>> {
    if let Ok(key_event) = rx.try_recv() {
        match key_event.code {
            KeyCode::Char('i') => {
                return Ok(UpdateResult::UpdateMode(AppMode::INPUT(String::from(""))))
            }
            KeyCode::Char('q') => {
                return Ok(UpdateResult::Quit);
            },
            KeyCode::Char('s') => {
                state.task_store.tasks.lock().unwrap().sort_by_key(|task| (*task.lock().unwrap()).state);
                return Ok(UpdateResult::None);
            },
            KeyCode::Char('j') => {
                let new_index = match state.list_state.selected() {
                    Some(index) => {
                        if index >= state.task_store.tasks.lock().unwrap().len() - 1 {
                            0
                        } else {
                            index + 1
                        }
                    },
                    None => 0
                };

                state.list_state.select(Some(new_index));
                return Ok(UpdateResult::None);
            },
            KeyCode::Char('k') => {
                let new_index = match state.list_state.selected() {
                    Some(index) => {
                        if index == 0 {
                            state.task_store.tasks.lock().unwrap().len() - 1
                        } else {
                            index - 1
                        }
                    },
                    None => 0
                };

                state.list_state.select(Some(new_index));
                return Ok(UpdateResult::None);
            },
            KeyCode::Char(' ') => {
                let tasks = state.task_store.tasks.lock().unwrap();
                let mut task = tasks[state.list_state.selected().unwrap()].lock().unwrap();
                task.state = task.state.next();
                return Ok(UpdateResult::Save)
            },
            KeyCode::Enter => {
                let tasks = state.task_store.tasks.lock().unwrap();
                let task_ref = tasks[state.list_state.selected().unwrap()].clone();
                let task = task_ref.lock().unwrap();

                return Ok(UpdateResult::UpdateMode(AppMode::EDIT(task_ref.clone(), task.text.clone())))
            }
            _ => {}
        }
    }
    Ok(UpdateResult::None)
}

fn update_input_mode(state: &mut AppState, rx: &Receiver<KeyEvent>) -> Result<UpdateResult, Box<dyn std::error::Error>> {
    let current_input = match state.mode.clone() {
        AppMode::INPUT(str) => str,
        _ => return Err(format!("update input mode got called with normal mode!!").into())
    };

    if let Ok(key_event) = rx.try_recv() {
        match key_event.code {
            KeyCode::Esc => {
                return Ok(UpdateResult::UpdateMode(AppMode::NORMAL))
            }
            KeyCode::Char(c) => {
                return Ok(UpdateResult::UpdateMode(AppMode::INPUT(current_input + c.to_string().as_str())))
            }
            _ => return Ok(UpdateResult::None)
        }
    };

    Ok(UpdateResult::None)
}

fn draw_ui(frame: &mut Frame<CrosstermBackend<Stdout>>, state: &mut AppState) {
    let my_list = List::new(vec![ListItem::new("hallo"), ListItem::new("hello")]);
    frame.render_widget(my_list, frame.size());
    draw_task_list(frame, state);
    draw_status_bar(frame, state)
}

fn draw_task_list(frame: &mut Frame<CrosstermBackend<Stdout>>, state: &mut AppState) {
    let items: Vec<ListItem> = state.task_store.tasks.lock().unwrap().iter()
        .map(|task| {
            ListItem::new(format!("{}", *task.lock().unwrap()))
        }).collect();

    let my_list = List::new(items).highlight_symbol("-> ");

    let mut rect = frame.size().clone();
    rect.height = rect.height - 2;
    rect.y = 0;

    frame.render_stateful_widget(my_list, rect, state.list_state);
}

fn draw_status_bar(frame: &mut Frame<CrosstermBackend<Stdout>>, state: &AppState) {
    let state_str: String = state.mode.clone().into();
    let mut my_box = Block::default()
        .title(text::Span::styled(state_str, Style::default().fg(Color::Black).bg(Color::LightGreen)));

    if let AppMode::INPUT(input_mode) = state.mode.clone() {
        my_box = my_box.title(format!("Input > {}", input_mode));
    }

    if let AppMode::EDIT(task_ref, str) = state.mode.clone() {
        let task = task_ref.lock().unwrap();
        // my_box = my_box.title(format!("Original > {}", task.text));
        my_box = my_box.title(format!("Updated > {}", str));
    }

    let mut rect = frame.size().clone();
    rect.height = 1;
    rect.y = frame.size().bottom() - rect.height;
    frame.render_widget(my_box, rect);
}
