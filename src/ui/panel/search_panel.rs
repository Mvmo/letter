use std::{sync::{Mutex, Arc, mpsc::Receiver}, io::Stdout};

use crossterm::event::{KeyEvent, KeyCode};
use ratatui::{Frame, prelude::{CrosstermBackend, Rect, Layout, Direction, Constraint}, widgets::{Block, Borders, ListItem, List}};

use crate::{ui::textarea::TextArea, Letter, LetterCommand};

use super::Panel;

pub struct SearchPanel {
    rx: Arc<Mutex<Receiver<KeyEvent>>>,
    text_area: TextArea<Letter, LetterCommand>,
    items: Vec<String>
}

impl SearchPanel {
    pub fn new(rx: Arc<Mutex<Receiver<KeyEvent>>>, letter: &mut Letter) -> Self {
        let mut text_area = TextArea::new(vec![]);
        text_area.on_key(KeyCode::Esc, Box::new(|_, _: &mut Letter| {
            return (true, Some(LetterCommand::Quit));
        }));
        text_area.disallow_line_breaks();

        let items = letter.task_store.tasks.iter()
            .map(|task| task.text.clone())
            .collect();

        Self {
            rx,
            text_area,
            items
        }
    }
}

impl Panel for SearchPanel {
    fn get_name(&self) -> String {
        "search".to_string()
    }

    fn update(&mut self, letter: &mut Letter) -> Option<LetterCommand> {
        rustic_fuzz::fuzzy_sort_in_place(&mut self.items, &self.text_area.lines.join(" ").to_string());
        return self.text_area.update(self.rx.clone(), letter);
    }

    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, _area: Rect, _letter: &Letter) {
        let rect = centered_rect(70, 80, frame.size());
        let search_block = Block::default()
            .title("Search")
            .borders(Borders::ALL);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(99),
                Constraint::Percentage(1)
            ]).split(search_block.inner(rect));

        let list_items: Vec<ListItem> = self.items.iter()
            .map(|item| ListItem::new(item.clone()))
            .collect();

        let list = List::new(list_items);

        frame.render_widget(search_block, rect);
        frame.render_widget(list, layout[0]);
        self.text_area.draw(frame, layout[1]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
