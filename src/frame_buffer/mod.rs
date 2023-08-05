use std::{any::Any, io::Stdout};

use tui::{Frame, backend::CrosstermBackend};

trait FrameBuffer: Any {
    fn get_name(&self) -> String;
    fn update(&self);
    fn draw(&self, frame: Frame<CrosstermBackend<Stdout>>);
}

struct OverviewFrameBuffer {}

impl FrameBuffer for OverviewFrameBuffer {
    fn get_name(&self) -> String {
        "self".to_string()
    }

    fn update(&self) {

    }

    fn draw(&self, frame: Frame<CrosstermBackend<Stdout>>) {

    }
}

fn main() {
    let x: Vec<Box<dyn FrameBuffer>> = vec![];
}
