use std::{collections::HashMap, str::FromStr};
use ratatui::style::Color;
use rusqlite::{Connection, Row};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Badge {
    pub id: i64,
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
    pub id: Option<i64>,
    pub text: String,
    pub badge_id: Option<i64>,
    pub note_id: Option<i64>
}

impl Task {
    fn from_row(row: &Row) -> Result<Self> {
        let task_id = row.get("id")?;
        let task_text = row.get("text")?;
        let task_badge_id = row.get("badge_id")?;
        let task_note_id = row.get("note_id")?;

        Ok(Self {
            id: Some(task_id),
            text: task_text,
            badge_id: task_badge_id,
            note_id: task_note_id,
        })
    }
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: None,
            text: String::new(),
            badge_id: None,
            note_id: None
        }
    }
}

pub struct Note {
    pub id: Option<i64>,
    pub text: String,
}

impl Note {
    fn new(id: Option<i64>, text: String) -> Self {
        Self { id, text }
    }

    fn from_row(row: &Row) -> Result<Self> {
        let note_id = row.get("id")?;
        let note_text = row.get("text")?;

        Ok(Self {
            id: Some(note_id),
            text: note_text,
        })
    }
}

impl Default for Note {
    fn default() -> Self {
        Self {
            id: None,
            text: String::new(),
        }
    }
}

pub struct TaskStore {
    connection: Connection,

    // TODO make private
    pub badges: HashMap<i64, Badge>,
    pub notes: HashMap<i64, Note>,
    pub tasks: Vec<Task>
}

impl TaskStore {
    pub fn new(connection: Connection) -> Self {
        TaskStore {
            connection,
            badges: HashMap::new(),
            notes: HashMap::new(),
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
        "#, ())?;

        self.connection.execute(r#"
            CREATE TABLE IF NOT EXISTS notes (
                id   INTEGER PRIMARY KEY NOT NULL,
                text TEXT                NOT NULL
            );
        "#, ())?;

        self.connection.execute(r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id         INTEGER PRIMARY KEY NOT NULL,
                text       TEXT                NOT NULL,
                badge_id   INTEGER,
                note_id    INTEGER,
                sort_order INTEGER NOT NULL,

                FOREIGN KEY (badge_id) REFERENCES badges (id),
                FOREIGN KEY (note_id) REFERENCES notes (id)
            );
        "#, ())?;

        self.connection.execute(r#"
            INSERT INTO badges (name, color)
                SELECT 'TODO', '#FF9B9B'
                UNION ALL
                SELECT 'In Progress', '#FFD6A5'
                UNION ALL
                SELECT 'Done', '#CBFFA9'
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

        self.notes = self.connection.prepare("SELECT * FROM notes")?
            .query_map([], |row| {
                Note::from_row(row)
                    .map_err(|_| rusqlite::Error::ExecuteReturnedResults)
            })?
            .filter_map(|note| note.ok())
            .map(|note| (note.id.unwrap(), note))
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

    pub fn get_or_create_note_id(&mut self, idx_sort_order: i64) -> Result<i64> {
        let task = self.tasks.get(idx_sort_order as usize).unwrap();
        match task.note_id {
            Some(note_id) => return Ok(note_id),
            None => {
                let id: i64 = self.connection.prepare("INSERT INTO notes (text) VALUES ('') RETURNING ID")?
                    .query_map([], |row| {
                        row.get::<&str, i64>("id")
                    })?
                    .filter_map(|id| id.ok())
                    .sum(); // TODO works but maybe there's a better way

                self.connection.execute("UPDATE tasks SET note_id = ?1 WHERE sort_order = ?2", (id, idx_sort_order))?;

                let task = self.tasks.get_mut(idx_sort_order as usize).unwrap();
                task.note_id = Some(id);

                let note = Note::new(Some(id), String::new());
                self.notes.insert(id, note);

                Ok(id)
            }
        }
    }

    pub fn delete_task(&mut self, idx_sort_order: i64) -> Result<()> {
        self.connection.execute("DELETE FROM tasks WHERE sort_order = ?1", (idx_sort_order,))?;
        self.connection.execute("UPDATE tasks SET sort_order = sort_order - 1 WHERE sort_order >= ?1", (idx_sort_order,))?;
        self.tasks.remove(idx_sort_order as usize);

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

    pub fn update_task_badge(&mut self, idx_sort_order: i64, badge_id: i64) -> Result<()> {
        self.connection.execute(r#"
            UPDATE tasks
                SET badge_id = ?1
            WHERE sort_order = ?2
        "#, (badge_id, idx_sort_order))?;

        //self.fetch_data();
        let task = self.tasks.get_mut(idx_sort_order as usize).expect("couldn't find task");
        task.badge_id = Some(badge_id);

        Ok(())
    }

    pub fn _update_task_order(&mut self, task: &Task, sort_order: i64) -> Result<()> {
        self.connection.execute(r#"
            UPDATE tasks
                SET sort_order = ?1
            WHERE id = ?2
        "#, (sort_order, task.id))?;

        // TODO update list

        Ok(())
    }

    pub fn update_note_text(&mut self, note_idx: i64, text: &str) -> Result<()> {
        self.connection.execute(r#"
            UPDATE notes
                SET text = ?1
            WHERE id = ?2
        "#, (text, note_idx))?;

        let note = self.notes.get_mut(&note_idx).unwrap();
        note.text = String::from(text);

        Ok(())
    }

    pub fn get_badge(&self, task: &Task) -> Option<&Badge> {
        let badge_id = &task.badge_id?;
        self.badges.get(badge_id)
    }

    pub fn get_note_by_id(&self, note_id: i64) -> Option<&Note> {
        self.notes.get(&note_id)
    }

}

