use std::{io::Stdout, sync::{Arc, Mutex, mpsc::Receiver}};

use crossterm::event::{KeyEvent, KeyCode};
use log::error;
use ratatui::{Frame, prelude::{CrosstermBackend, Rect}, widgets::{ListItem, List, Clear}, style::{Style, Color}};

use crate::{Letter, LetterCommand};

use super::Panel;

pub struct BadgeSelectPanel {
    rx: Arc<Mutex<Receiver<KeyEvent>>>,
    pub position: (usize, usize),
    cursor: usize,
    task_idx_sort_order: usize,
    values: Vec<(Option<i64>, String)>
}

impl Panel for BadgeSelectPanel {
    fn get_name(&self) -> String {
        "badge-select".to_string()
    }

    fn update(&mut self, app_state: &mut Letter) -> Option<LetterCommand> {
        let rx = self.rx.lock().unwrap();
        if let Ok(key_event) = rx.try_recv() {
            match key_event.code {
                KeyCode::Char('j') => {
                    if self.cursor == self.values.len() - 1 {
                        self.cursor = 0;
                    } else {
                        self.cursor += 1;
                    }
                },
                KeyCode::Char('k') => {
                    if self.cursor == 0 {
                        self.cursor = self.values.len() - 1;
                    } else {
                        self.cursor -= 1;
                    }
                }
                KeyCode::Enter => {
                    let (badge_idx, _) = self.values.get(self.cursor as usize).expect("something is really wrong :(");
                    if let Some(badge_idx) = badge_idx {
                        let badge = app_state.task_store.badges.get(badge_idx).expect("even more wrong");
                        let badge_id = badge.id;
                        let idx = self.task_idx_sort_order;

                        if let Err(_) = app_state.task_store.update_task_badge(idx as i64, badge_id) {
                            error!("couldn't update task badge for {idx} using badge {badge_id}")
                        }
                    } else {
                        if let Err(_) = app_state.task_store.unset_task_badge(self.task_idx_sort_order as i64) {
                            error!("couldn't update task badge")
                        }
                    }

                    return Some(LetterCommand::Quit)
                }
                KeyCode::Char('t') => return Some(LetterCommand::Quit),
                KeyCode::Esc => return Some(LetterCommand::Quit),
                _ => {}
            }
        }
        None
    }

    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, _: Rect, _: &Letter) {
        let list_items: Vec<ListItem> = self.values
            .iter()
            .enumerate()
            .map(|(idx, (_, badge_name))| {
                if idx == self.cursor {
                    return ListItem::new(format!("> {}", badge_name.clone())).style(Style::default().bg(Color::Rgb(20, 0, 20)));
                }
                return ListItem::new(badge_name.clone());
            })
            .collect();

        let width = list_items.iter()
            .map(|li| li.width())
            .max()
            .unwrap_or(0);

        let list = List::new(list_items).style(Style::default().bg(Color::Black));
        let (x, y) = self.position;
        let rect = Rect::new(x as u16, y as u16 + 1, width as u16, list.len() as u16);

        frame.render_widget(Clear, rect);
        frame.render_widget(list, rect);
    }
}

impl BadgeSelectPanel {
    pub fn new(letter: &Letter, task_idx_sort_order: usize, position: (usize, usize), rx: Arc<Mutex<Receiver<KeyEvent>>>) -> Self {
        let task = &letter.task_store.tasks.get(task_idx_sort_order).unwrap();
        let badges = &letter.task_store.badges;

        let mut values: Vec<(Option<i64>, String)> = badges.iter()
            .filter(|(_, badge)| Some(badge.id) != task.badge_id)
            .map(|(idx, badge)| (Some(*idx), badge.name.clone()))
            .collect();

        values.push((None, "None".to_string()));

        Self {
            cursor: 0,
            values,
            task_idx_sort_order,
            rx,
            position,
        }
    }
}
