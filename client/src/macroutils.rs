// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use macroquad::prelude::*;

use logic::input::{Input, InputProvider, Inputs};

fn input_to_macroquad_key(input: Input) -> KeyCode {
    match input {
    Input::Up => KeyCode::Up,
    Input::Down => KeyCode::Down,
    Input::Left => KeyCode::Left,
    Input::Right => KeyCode::Right,
    Input::CW => KeyCode::X,
    Input::CCW => KeyCode::Z,
    }
}

pub struct MacroquadInputProvider;
impl InputProvider for MacroquadInputProvider {
    fn peek(&mut self) {
    }

    fn consume(&mut self) {
    }

    fn key_just_pressed(&self, input: Input) -> bool {
        is_key_pressed(input_to_macroquad_key(input))
    }

    fn key_down(&self, input: Input) -> bool {
        is_key_down(input_to_macroquad_key(input))
    }
}

impl MacroquadInputProvider {
    pub fn new() -> MacroquadInputProvider {
        MacroquadInputProvider {}
    }
}

pub trait Updater {
    fn update(&mut self, inputs: &Inputs, ticks: u64);
    fn draw(&mut self);
}

pub struct Ticker<U: Updater> {
    update_calls_count: u64,
    updater: U,
    inputs: Inputs,
    macroquad_inputs: MacroquadInputProvider,
}

impl<U: Updater> Ticker<U> {
    pub fn new(updater: U) -> Ticker<U> {
        Ticker {
            update_calls_count: 0,
            updater,
            inputs: Inputs::new(),
            macroquad_inputs: MacroquadInputProvider::new(),
        }
    }
    pub async fn run(&mut self) {
        loop {
            let expected_update_calls_count = (get_time() * 60.) as u64;
            for _ in 0..expected_update_calls_count - self.update_calls_count {
                self.inputs.tick(self.update_calls_count, &mut self.macroquad_inputs);
                self.update_calls_count += 1;
                self.updater.update(&self.inputs, self.update_calls_count);
            }
            self.updater.draw();
            next_frame().await;
        }
    }
}
