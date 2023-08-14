use std::{io::Stdout, sync::{Arc, Mutex, mpsc::Receiver}, collections::HashMap};

use crossterm::event::{KeyEvent, KeyCode};
use ratatui::{Frame, prelude::{CrosstermBackend, Rect}, widgets::{ListItem, List, Clear}, style::{Style, Color}};

use crate::{AppState, UpdateResult};

use super::Panel;

pub struct BadgeSelectPanel {
    rx: Arc<Mutex<Receiver<KeyEvent>>>,
    position: (usize, usize),
    cursor: usize,
    task_idx_sort_order: usize,
    values: Vec<(i64, String)>
}

impl Panel for BadgeSelectPanel {
    fn get_name(&self) -> String {
        "badge-select".to_string()
    }

    fn update(&mut self, app_state: &mut AppState) -> UpdateResult {
        let rx = self.rx.lock().unwrap();
        if let Ok(key_event) = rx.try_recv() {
            match key_event.code {
                KeyCode::Char('j') => self.cursor += 1,
                KeyCode::Char('k') => self.cursor -= 1,
                KeyCode::Enter => {
                    let (badge_idx, _) = self.values.get(self.cursor as usize).expect("something is really wrong :(");
                    let badge = app_state.task_store.badges.get(badge_idx).expect("even more wrong");
                    app_state.task_store.update_task_badge(self.task_idx_sort_order as i64, badge.id);
                    return UpdateResult::Quit;
                }
                KeyCode::Esc => return UpdateResult::Quit,
                _ => {}
            }
        }
        UpdateResult::None
    }

    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, app_state: &AppState) {
        let list_items: Vec<ListItem> = self.values
            .iter()
            .enumerate()
            .map(|(idx, (_, badge_name))| {
                let mut list_item = ListItem::new(badge_name.clone());
                if idx == self.cursor {
                    list_item = list_item.style(Style::default().bg(Color::DarkGray));
                }

                list_item
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
    pub fn new(app_state: &AppState, task_idx_sort_order: usize, position: (usize, usize), rx: Arc<Mutex<Receiver<KeyEvent>>>) -> Self {
        let task = &app_state.task_store.tasks.get(task_idx_sort_order).unwrap();
        let badges = &app_state.task_store.badges;

        let values = badges.iter()
            .filter(|(_, badge)| Some(badge.id) != task.badge_id)
            .map(|(idx, badge)| (*idx, badge.name.clone()))
            .collect();

        Self {
            cursor: 0,
            values,
            task_idx_sort_order,
            rx,
            position,
        }
    }
}
