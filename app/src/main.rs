// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::{borrow::Cow, collections::{HashMap, HashSet}, time::Duration};

use graphics::Graphics;
use logic::{
    field::{Field, GameState},
    hooks::{Cubes, Sounds},
    input::{Input, InputProvider, Inputs}, well::{WELL_COLS, WELL_ROWS},
};
use sdl2::{self as sdl, event::WindowEvent, image::ImageRWops};
use sdl::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    ttf,
};
use sounds::ClientSounds;
use text::Text;

mod graphics;
mod sounds;
mod text;
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
    just_pressed: HashSet<Keycode>,
    current: HashSet<Keycode>,
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

impl SDLInputs {
    fn new() -> SDLInputs {
        SDLInputs {
            just_pressed: HashSet::new(),
            current: HashSet::new(),
        }
    }
    fn push_key(&mut self, keycode: Keycode) {
        self.just_pressed.insert(keycode);
        self.current.insert(keycode);
    }
    fn release_key(&mut self, keycode: Keycode) {
        self.just_pressed.remove(&keycode);
        self.current.remove(&keycode);
    }
}

impl InputProvider for SDLInputs {
    fn peek(&mut self) {}

    fn consume(&mut self) {
        self.just_pressed.clear();
    }

    fn key_just_pressed(&self, input: Input) -> bool {
        self.just_pressed.contains(&input_to_sdl_key(input))
    }

    fn key_down(&self, input: Input) -> bool {
        self.current.contains(&input_to_sdl_key(input))
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
    let text_context = ttf::init().map_err(|e| e.to_string())?;

    let frequency = 44_100;
    let format = sdl::mixer::AUDIO_S16LSB;
    let channels = sdl::mixer::DEFAULT_CHANNELS;
    let chunk_size = 1_024;
    sdl::mixer::open_audio(frequency, format, channels, chunk_size)?;
    sdl::mixer::allocate_channels(4);

    let window = video
        .window("Edrefis", WELL_COLS as u32 * 20, WELL_ROWS as u32 * 20)
        .position_centered()
        .resizable()
        .metal_view()
        .build()
        .map_err(|e| e.to_string())?;

    let (width, height) = window.size();
    let mut gpu_state = pollster::block_on(gpu::State::new(&window))?;
    let mut graphics = graphics_gpu::Graphics::new(&gpu_state)?;

    let mut field = Field::new();
    let mut input_provider = SDLInputs::new();
    let mut inputs = Inputs::new();

    let mut event_pump = ctx.event_pump()?;
    let mut sounds = ClientSounds::new()?;
    let mut cubes = DummyImpl {};
    let mut text = Text::new(&text_context)?;

    let mut ticks = 0u64;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Window {
                    window_id,
                    win_event: WindowEvent::SizeChanged(width, height),
                    ..
                } if window_id == window.id() => {
                    gpu_state.resize(width as u32, height as u32);
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
            GameState::ActivePiece { piece } => {
                graphics.render(&field.well, &piece, &mut gpu_state)?;
            }
            _ => {
                graphics.render(&field.well, &field.next, &mut gpu_state)?;
            }
        }

        // canvas.set_draw_color(Color::RGB(255, 255, 255));
        // canvas.clear();

        // graphics.draw_background(&mut canvas)?;

        // Graphics::well_viewport(&mut canvas, |canvas| {
        //     graphics.draw_well_background(canvas)?;
        //     graphics.draw_well(canvas, &field.well)?;
        //     graphics.draw_outlines(canvas, &field.well)?;

        //     match field.state {
        //         GameState::ActivePiece { piece } => {
        //             graphics.draw_piece(canvas, &piece, lerp(102., 0., piece.ticks_to_lock as f32 / 30.) as u8)?;
        //         }
        //         GameState::PlaceDelay { .. } | GameState::ClearDelay { .. } => {
        //         }
        //         GameState::GameOver { .. } => {
        //             graphics.draw_piece(canvas, &field.next, 50)?;
        //         }
        //     }
        //     Ok(())
        // })?;
        // Graphics::well_side_viewport(&mut canvas, |canvas| {
        //     let mut texture_creator = canvas.texture_creator();
        //     text.draw_text(canvas, &mut texture_creator, "meow", 1, 1, text::Weight::Bold, Color::WHITE)?;
        //     Ok(())
        // })?;

        // canvas.present();

        std::thread::sleep(Duration::from_millis(15));
    }

    Ok(())
}
