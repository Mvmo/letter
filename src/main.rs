use std::{path::PathBuf, fs::File, io::BufReader};
use std::io::{Lines, Read};

static DEFAULT_LOCATION: &str = "tasks";

fn main() {
    let task_store = TaskStore::from(PathBuf::from(DEFAULT_LOCATION));
    let tasks = task_store.load_all_tasks();
}

struct TaskStore {
    path: PathBuf
}

enum TaskState {
    Done = 0,
    Working,
    Waiting
}

struct Task {
    state: TaskState,
    text: String,
}

impl Task {
    fn from_line(line: String) -> Self {
        let mut splitted = line.splitn(2, " ");
        let state_str = splitted.next();
        let text_str = splitted.next();

        println!("State: {} and Text: {}", state_str.unwrap(), text_str.unwrap());

        Task { state: TaskState::Working, text: String::new() }
    }
}

impl TaskStore {
    fn load_all_tasks(&self) -> Vec<Task> {
        match File::open(self.path.clone()) {
            Ok(file) => {
                let mut reader = BufReader::new(file);
                let mut str = String::new();
                reader.read_to_string(&mut str).expect("BITTE");

                let tasks: Vec<Task> = str.split("\n")
                    .into_iter()
                    .map(|line| Task::from_line(String::from(line)))
                    .collect();

                tasks
            },
            Err(err) => {
                eprintln!("Error opening file: {}", err);
                vec![]
            }
        }
    }
}

impl From<PathBuf> for TaskStore {
    fn from(path: PathBuf) -> Self {
        TaskStore { path }
    }
}

fn create_task() {}
