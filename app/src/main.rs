// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::{collections::HashSet, time::Duration};

use logic::{
    field::{Field, GameState},
    hooks::{Cubes, Sounds},
    input::{Input, InputProvider, Inputs}, well::WELL_COLS,
};
use sdl2::{self as sdl};
use sdl::{
    event::Event,
    keyboard::Keycode,
    controller::Button,
    event::WindowEvent,
};
use sounds::ClientSounds;

mod sounds;
mod gpu;
mod graphics_gpu;

#[derive(Clone, Copy)]
struct DummyImpl;
impl Cubes for DummyImpl {
    fn spawn_cube(&mut self, _x: i32, _y: i32, _color: logic::well::Block) {}
}
impl Sounds for DummyImpl {
    fn block_spawn(&mut self, _color: logic::well::Block) {}
    fn line_clear(&mut self) {}
    fn lock(&mut self) {}
    fn land(&mut self) {}
}

struct SDLInputs {
    just_pressed_key: HashSet<Keycode>,
    current_key: HashSet<Keycode>,
    just_pressed_btn: HashSet<Button>,
    current_btn: HashSet<Button>,
}

fn input_to_sdl_key(keycode: Input) -> Keycode {
    match keycode {
        Input::Up => Keycode::Up,
        Input::Down => Keycode::Down,
        Input::Left => Keycode::Left,
        Input::Right => Keycode::Right,
        Input::CW => Keycode::X,
        Input::CCW => Keycode::Z,
    }
}
fn input_to_sdl_btn(keycode: Input) -> Button {
    match keycode {
        Input::Up => Button::DPadUp,
        Input::Down => Button::DPadDown,
        Input::Left => Button::DPadLeft,
        Input::Right => Button::DPadRight,
        Input::CW => Button::A,
        Input::CCW => Button::B,
    }
}

impl SDLInputs {
    fn new() -> SDLInputs {
        SDLInputs {
            just_pressed_key: HashSet::new(),
            current_key: HashSet::new(),
            just_pressed_btn: HashSet::new(),
            current_btn: HashSet::new(),
        }
    }
    fn push_key(&mut self, keycode: Keycode) {
        self.just_pressed_key.insert(keycode);
        self.current_key.insert(keycode);
    }
    fn release_key(&mut self, keycode: Keycode) {
        self.just_pressed_key.remove(&keycode);
        self.current_key.remove(&keycode);
    }
    fn push_btn(&mut self, button: Button) {
        self.just_pressed_btn.insert(button);
        self.current_btn.insert(button);
    }
    fn release_btn(&mut self, button: Button) {
        self.just_pressed_btn.remove(&button);
        self.current_btn.remove(&button);
    }
}

impl InputProvider for SDLInputs {
    fn peek(&mut self) {}

    fn consume(&mut self) {
        self.just_pressed_key.clear();
        self.just_pressed_btn.clear();
    }

    fn key_just_pressed(&self, input: Input) -> bool {
        self.just_pressed_key.contains(&input_to_sdl_key(input)) || self.just_pressed_btn.contains(&input_to_sdl_btn(input))
    }

    fn key_down(&self, input: Input) -> bool {
        self.current_key.contains(&input_to_sdl_key(input)) || self.current_btn.contains(&input_to_sdl_btn(input))
    }
}

pub fn lerp(a: f32, b: f32, f: f32) -> f32 {
    a * (1.0 - f) + (b * f)
}

fn main() -> Result<(), String> {
    let ctx = sdl::init()?;
    let video = ctx.video()?;
    // let timer = ctx.timer()?;
    let _audio = ctx.audio()?;
    let controller = ctx.game_controller()?;
    let _c = (0..controller.num_joysticks()?)
        .find_map(|idx| {
            controller.open(idx).ok()
        });

    let frequency = 44_100;
    let format = sdl::mixer::AUDIO_S16LSB;
    let channels = sdl::mixer::DEFAULT_CHANNELS;
    let chunk_size = 1_024;
    sdl::mixer::open_audio(frequency, format, channels, chunk_size)?;
    sdl::mixer::allocate_channels(8);

    let window = video
        .window("Edrefis", WELL_COLS as u32 * 60, WELL_COLS as u32 * 60)
        .position_centered()
        .resizable()
        .metal_view()
        .build()
        .map_err(|e| e.to_string())?;

    let mut gpu_state = pollster::block_on(gpu::State::new(&window))?;
    let mut graphics = graphics_gpu::Graphics::new(&mut gpu_state)?;

    let mut field = Field::new();
    let mut input_provider = SDLInputs::new();
    let mut inputs = Inputs::new();

    let mut event_pump = ctx.event_pump()?;
    let mut sounds = ClientSounds::new()?;
    let mut cubes = DummyImpl {};

    let mut ticks = 0u64;

    let mut stepper = nanotime::StepData::new(Duration::from_secs_f64(1. / 60.));

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Window {
                    window_id,
                    win_event: WindowEvent::SizeChanged(width, height),
                    ..
                } if window_id == window.id() => {
                    gpu_state.resize(width as u32, height as u32)?;
                }
                Event::KeyDown {
                    keycode:
                        Some(
                            x @ (Keycode::X
                            | Keycode::Z
                            | Keycode::Up
                            | Keycode::Down
                            | Keycode::Left
                            | Keycode::Right),
                        ),
                    ..
                } => {
                    input_provider.push_key(x);
                }
                Event::ControllerButtonDown { button, .. } => {
                    input_provider.push_btn(button);
                }
                Event::ControllerButtonUp { button, .. } => {
                    input_provider.release_btn(button);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => {
                    field.level += 50;
                }
                Event::KeyUp {
                    keycode:
                        Some(
                            x @ (Keycode::X
                            | Keycode::Z
                            | Keycode::Up
                            | Keycode::Down
                            | Keycode::Left
                            | Keycode::Right),
                        ),
                    ..
                } => {
                    input_provider.release_key(x);
                }
                Event::Quit { .. } => {
                    break 'running;
                }
                _ => {}
            }
        }

        ticks += 1;
        inputs.tick(ticks, &mut input_provider);
        field.update(&mut inputs, &mut sounds, &mut cubes);

        match field.state {
            GameState::ActivePiece { piece, .. } => {
                graphics.render(&field, &field.well, Some(&piece), &field.next, &mut gpu_state)?;
            }
            _ => {
                graphics.render(&field, &field.well, None, &field.next, &mut gpu_state)?;
            }
        }

        stepper.step();
    }

    Ok(())
}
