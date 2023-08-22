use std::{collections::HashMap, sync::mpsc::{Sender, Receiver, self}};

use crossterm::event::KeyCode;

pub struct KeyCommandComposer<C: Copy> {
    command_registry: HashMap<Vec<KeyCode>, C>,
    current_composition: Vec<KeyCode>,
    tx: Sender<C>,
}

fn key_code_to_string(key_code: &KeyCode) -> String {
    match key_code {
        KeyCode::Char(' ') => "<space>".to_string(),
        KeyCode::Char(c) => c.to_string(),
        _ => "{unknown}".to_string()
    }
}

impl<C: Copy> KeyCommandComposer<C> {
    pub fn new() -> (Self, Receiver<C>) {
        let (tx, rx) = mpsc::channel();
        (KeyCommandComposer { command_registry: HashMap::new(), current_composition: Vec::new(), tx }, rx)
    }

    pub fn _len(&self) -> usize {
        return self.current_composition.len();
    }

    pub fn get_combo_string(&self) -> String {
        self.current_composition.iter()
            .map(|key_code| key_code_to_string(key_code))
            .collect()
    }

    pub fn push_key(&mut self, key_code: KeyCode) {
        let key_code = key_code.clone();
        if key_code == KeyCode::Esc {
            self.clear_composition();
            return;
        }

        self.current_composition.push(key_code);
        if !self.validate() {
            self.clear_composition();
            return;
        }

        if self.command_registry.contains_key(&self.current_composition) {
            let command = self.command_registry.get(&self.current_composition).unwrap();
            self.tx.send(command.clone()).unwrap();
            self.clear_composition();
            return;
        }
    }

    pub fn validate(&mut self) -> bool {
        let current_composition = &self.current_composition;
        let reg = &self.command_registry;

        reg.iter()
            .map(|(c, _)| c.clone())
            .filter(|c| c.len() >= current_composition.len())
            .map(|c| c.split_at(current_composition.len()).0.to_vec())
            .any(|c| *c == self.current_composition)
    }

    pub fn clear_composition(&mut self) {
        self.current_composition.clear();
    }

    pub fn register_keycommand(&mut self, key_chain: Vec<KeyCode>, cmd: C) {
        self.command_registry.insert(key_chain, cmd);
    }
}

