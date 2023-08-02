use core::fmt;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;
use std::{path::PathBuf, fs::File, io::BufReader};
use std::io::{Read, BufWriter, Write, self, Stdout};

use crossterm::event::{EnableMouseCapture, DisableMouseCapture, self, KeyEvent, KeyCode};
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen};
use crossterm::execute;

use tui::style::Color;
use tui::text::Span;
use tui::{Terminal, Frame};
use tui::backend::CrosstermBackend;
use tui::widgets::{Block, ListState, List, ListItem};

static DEFAULT_LOCATION: &str = "tasks";

struct TaskStore {
    path: PathBuf,
    tasks: Vec<Task>,
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

                TaskStore { tasks, path }
            },
            Err(err) => {
                eprintln!("Error opening file: {}", err);
                TaskStore { tasks: vec![], path }
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

}

impl<'a> From<PathBuf> for TaskStore {
    fn from(path: PathBuf) -> Self {
        TaskStore { path, tasks: vec![] }
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

    let mut app_state = AppState { task_store: store, mode: AppMode::NORMAL, list_state: &mut ListState::default() };

    loop {
        let update_result = update(&mut app_state, &rx)?;
        match update_result {
            UpdateResult::Quit => break,
            UpdateResult::UpdateMode(mode) => app_state.mode = mode,
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
    None
}

#[derive(Clone)]
enum AppMode {
    NORMAL,
    INPUT(String)
}

impl Into<String> for AppMode {
    fn into(self) -> String {
        match self {
            Self::NORMAL => String::from("NORMAL"),
            Self::INPUT(_) => String::from("INPUT")
        }
    }
}

struct AppState<'a> {
    task_store: &'a TaskStore,
    mode: AppMode,
    list_state: &'a mut ListState
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
    }
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
            KeyCode::Down => {
                let new_index = match state.list_state.selected() {
                    Some(index) => {
                        if index >= state.task_store.tasks.len() {
                            0
                        } else {
                            index + 1
                        }
                    },
                    None => 0
                };

                state.list_state.select(Some(new_index));
                return Ok(UpdateResult::None);
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
    let items: Vec<ListItem> = state.task_store.tasks.iter()
        .map(|task| {
            let task_str: String = (*task).clone().into();
            ListItem::new(task_str)
        }).collect();

    let my_list = List::new(items).highlight_symbol("***");

    let mut rect = frame.size().clone();
    rect.height = rect.height - 2;
    rect.y = 0;

    frame.render_stateful_widget(my_list, rect, state.list_state);
}

fn draw_status_bar(frame: &mut Frame<CrosstermBackend<Stdout>>, state: &AppState) {
    let state_str: String = state.mode.clone().into();
    let mut my_box = Block::default()
        .title(state_str)
        .title_style(tui::style::Style::default().bg(Color::LightGreen).fg(Color::Black));

    if let AppMode::INPUT(input_mode) = state.mode.clone() {
        my_box = my_box.title(format!("Input > {}", input_mode));
    }

    let mut rect = frame.size().clone();
    rect.height = 1;
    rect.y = frame.size().bottom() - rect.height;
    frame.render_widget(my_box, rect);
}
