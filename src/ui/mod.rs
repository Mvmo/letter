use std::{io::{self, Stdout}, sync::{Arc, Mutex, mpsc::{Receiver, self}}, thread, time::Duration};

use ratatui::{prelude::CrosstermBackend, Terminal, Frame};
use crossterm::{execute, terminal::{EnterAlternateScreen, enable_raw_mode, LeaveAlternateScreen, disable_raw_mode}, event::{EnableMouseCapture, DisableMouseCapture, KeyEvent, self}, cursor::{SetCursorShape, CursorShape}};

use crate::{store::TaskStore, Result};

use self::panel::{overview_panel::OverviewPanel, Panel};

pub mod panel;
pub mod textarea;

pub enum UpdateResult {
    Quit,
    UpdateMode(AppMode),
    Save,
    None
}

#[derive(Clone, Copy)]
pub enum AppMode {
    NORMAL,
    INPUT,
}

impl Into<String> for AppMode {
    fn into(self) -> String {
        match self {
            Self::NORMAL => String::from("NORMAL"),
            Self::INPUT => String::from("INPUT"),
        }
    }
}



