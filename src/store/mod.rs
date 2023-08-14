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
    pub badge_id: Option<i64>
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

    // TODO make private
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

