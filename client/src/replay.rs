// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::{HashSet, VecDeque};

use macroquad::prelude::*;
use nanoserde::{DeJson, SerJson};
use logic::input::{Input, InputProvider, Inputs, RECORDABLE_INPUTS};

pub struct ReplayInputProvider {
    ticks: VecDeque<InputTick>,
    keys: HashSet<Input>,
}

impl ReplayInputProvider {
    pub fn new(replay: Replay) -> ReplayInputProvider {
        ReplayInputProvider {
            ticks: replay.ticks.into(),
            keys: HashSet::new(),
        }
    }
}

impl InputProvider for ReplayInputProvider {
    fn peek(&mut self) {
        match self.ticks.front() {
        Some(ref tick) => {
            for key in &tick.down {
                self.keys.insert(*key);
            }
            for key in &tick.up {
                self.keys.remove(key);
            }
        }
        None => {
            self.keys.clear();
        }
        }
    }
    fn consume(&mut self) {
        self.ticks.pop_front();
    }

    fn key_just_pressed(&self, input: Input) -> bool {
        self.ticks.get(0).map(|it| it.down.contains(&input)).unwrap_or(false)
    }

    fn key_down(&self, input: Input) -> bool {
        self.keys.contains(&input)
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(SerJson, DeJson)]
pub struct InputTick {
    down: Vec<Input>,
    up: Vec<Input>,
}

#[derive(SerJson, DeJson)]
pub struct Replay {
    seed: u32,
    ticks: Vec<InputTick>,
}

impl Replay {
    pub fn new(seed: u32) -> Replay {
        Replay {
            seed,
            ticks: vec![],
        }
    }
    pub fn replay_tick(&mut self, inputs: &Inputs) {
        let mut replay_tick = InputTick { down: vec![], up: vec![] };
        for input in RECORDABLE_INPUTS {
            if inputs.key_just_pressed(*input) {
                replay_tick.down.push((*input).into());
            } else if inputs.key_just_released(*input) {
                replay_tick.up.push((*input).into());
            }
        }
        self.ticks.push(replay_tick);
    }
}
