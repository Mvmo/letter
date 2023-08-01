use core::fmt;
use std::{path::PathBuf, fs::File, io::BufReader};
use std::io::{Read, BufWriter, Write};

static DEFAULT_LOCATION: &str = "tasks";

fn main() {
    let mut task_store = TaskStore::new(PathBuf::from(DEFAULT_LOCATION));
    task_store.add_task(Task { state: TaskState::Waiting, text: String::from("hallo, welt") });
    task_store.save();
}

struct TaskStore {
    path: PathBuf,
    tasks: Vec<Task>
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

fn start_task_editor() {

}
