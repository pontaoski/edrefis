// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use core::str;
use std::collections::{HashSet, VecDeque};

use cubes::{lerp, ClientCubes};
use logic::field::{level_to_gravity, Field, GameState};
use gfx::{color, Graphics, DST_BLOCK_SIZE};
use macroquad::prelude::*;
use logic::input::{Input, InputProvider, Inputs, INPUTS};
use macroutils::{MacroquadInputProvider, Ticker, Updater};
use nanoserde::{DeJson, SerJson};
use logic::proto::{ClientToServer, ServerToClient};
use quad_net::quad_socket::client::QuadSocket;
use replay::Replay;
use sound::ClientSounds;
use text::{Text, Weight};
use logic::well::{WELL_COLS, WELL_ROWS};

mod cubes;
mod gfx;
mod macroutils;
mod sound;
mod text;
mod replay;

struct FieldAndGraphics {
    render_target: RenderTarget,
    render_target_cam: Camera2D,

    next_target: RenderTarget,
    next_target_cam: Camera2D,

    left_ui_target: RenderTarget,
    left_ui_cam: Camera2D,

    field: Field,
    cubes: ClientCubes,

    client_id: u32,

    inputs_override: Option<(Inputs, NetworkInputProvider)>,
}

fn make(w: f32, h: f32) -> (RenderTarget, Camera2D) {
    let target = render_target(w as u32, h as u32);
    let mut cam = Camera2D::from_display_rect(Rect::new(0., 0., w, h));
    cam.zoom.x *= -1.;
    cam.render_target = Some(target.clone());

    (target, cam)
}

impl FieldAndGraphics {
    fn new(inputs_override: Option<(Inputs, NetworkInputProvider)>, field: Field, client_id: u32) -> FieldAndGraphics {
        let (well_render_target, render_target_cam) = make(
            DST_BLOCK_SIZE * WELL_COLS as f32,
            DST_BLOCK_SIZE * WELL_ROWS as f32,
        );
        well_render_target.texture.set_filter(FilterMode::Linear);

        let (next_target, next_target_cam) = make(DST_BLOCK_SIZE * 4., DST_BLOCK_SIZE * 2.);

        let (left_ui_target, left_ui_cam) =
            make(DST_BLOCK_SIZE * 5 as f32, DST_BLOCK_SIZE * WELL_ROWS as f32);

        FieldAndGraphics {
            render_target: well_render_target,
            render_target_cam,
            next_target,
            next_target_cam,
            left_ui_target,
            left_ui_cam,
            field,
            cubes: ClientCubes::new(),
            client_id,
            inputs_override,
        }
    }
}

struct Game {
    graphics: Graphics,
    sounds: ClientSounds,
    text: Text,
    replay: Replay,
    my_id: u32,
    network: QuadSocket,
    last_tick: f64,

    fields: Vec<FieldAndGraphics>,
    fps: VecDeque<i32>,
    differences: VecDeque<f64>,
}

impl Game {
    fn draw_well_bg() {
        let well_width = WELL_COLS as f32;
        let well_height = WELL_ROWS as f32;
        let line_thickness = 1. / 8.;
        let outline = Color::new(0.77625, 0.96804, 1.00513, 0.5);
        let wall = Color::new(0.77625, 0.96804, 1.00513, 0.2);

        // well bg
        draw_affine_parallelogram(
            Vec3::new(1., WELL_ROWS as f32 / 2., WELL_COLS as f32 / -2.),
            -(WELL_ROWS as f32) * Vec3::Y,
            WELL_COLS as f32 * Vec3::Z,
            None,
            Color::new(0., 0., 0., 0.4),
        );

        // bottom
        draw_affine_parallelogram(
            Vec3::new(0., well_height / 2. + line_thickness, well_width / -2.),
            -line_thickness * Vec3::Y,
            well_width * Vec3::Z,
            None,
            outline,
        );
        draw_affine_parallelogram(
            Vec3::new(0., well_height / 2. + line_thickness, well_width / -2.),
            Vec3::new(1., -line_thickness, 0.),
            Vec3::new(0., 0., well_width),
            None,
            wall,
        );

        // left
        draw_affine_parallelogram(
            Vec3::new(0., well_height / 2. + line_thickness, well_width / 2.),
            -(well_height + line_thickness) * Vec3::Y,
            line_thickness * Vec3::Z,
            None,
            outline,
        );
        draw_affine_parallelogram(
            Vec3::new(0., well_height / 2. + line_thickness, well_width / 2.),
            Vec3::new(0., -(well_height + line_thickness), 0.),
            Vec3::new(1., 0., line_thickness),
            None,
            wall,
        );

        // right
        draw_affine_parallelogram(
            Vec3::new(
                0.,
                well_height / 2. + line_thickness,
                well_width / -2. - line_thickness,
            ),
            -(well_height + line_thickness) * Vec3::Y,
            line_thickness * Vec3::Z,
            None,
            outline,
        );
        draw_affine_parallelogram(
            Vec3::new(
                0.,
                well_height / 2. + line_thickness,
                well_width / -2. - line_thickness,
            ),
            Vec3::new(0., -(well_height + line_thickness), 0.),
            Vec3::new(1., 0., line_thickness),
            None,
            wall,
        );
    }
    fn draw_field_well(&self, field: &FieldAndGraphics) {
        set_camera(&field.render_target_cam);
        clear_background(Color::new(0., 0., 0., 0.));

        if let GameState::GameOver { .. } = field.field.state {
            self.graphics.draw_well(&field.field.well, true);
        } else {
            self.graphics.draw_well(&field.field.well, false);
        }
        self.graphics.draw_outlines(&field.field.well);
        if let GameState::ActivePiece { ref piece } = field.field.state {
            self.graphics.draw_piece(piece, lerp(0.4, 0.0, piece.ticks_to_lock as f32 / 30.));
        }
    }
    fn draw_field_next(&self, field: &FieldAndGraphics) {
        set_camera(&field.next_target_cam);
        clear_background(Color::new(0., 0., 0., 0.));
        self.graphics.draw_piece_at(&field.field.next, 0, -1, 0.);
    }
    fn draw_left_ui(&self, field: &FieldAndGraphics) {
        set_camera(&field.left_ui_cam);
        clear_background(Color::new(0., 0., 0., 0.));

        let line_spacing = 8.;

        let sidebar_x = 10.;
        let gravity_y = 100.;

        let gravity_header =
            self.text
                .draw_text("Gravity", sidebar_x, gravity_y, Weight::Medium, WHITE, 24.);
        let gravity = level_to_gravity(field.field.level);
        let gravity_label = self.text.draw_text(
            &format!(
                "{}",
                if gravity < 256 {
                    gravity / 2
                } else {
                    gravity / 256
                }
            ),
            sidebar_x,
            gravity_y + line_spacing + gravity_header.height,
            Weight::Bold,
            WHITE,
            32.,
        );

        if gravity < 256 {
            self.text.draw_text(
                " /128",
                sidebar_x + gravity_label.width,
                gravity_y + line_spacing + gravity_header.height,
                Weight::Medium,
                WHITE,
                24.,
            );
        } else {
            self.text.draw_text(
                "G",
                sidebar_x + gravity_label.width,
                gravity_y + line_spacing + gravity_header.height,
                Weight::Medium,
                WHITE,
                32.,
            );
        }

        let level_y = 200.;

        let level_header =
            self.text
                .draw_text("Level", sidebar_x, level_y, Weight::Medium, WHITE, 24.);
        let level = field.field.level;
        let level_label = self.text.draw_text(
            &format!("{}", level),
            sidebar_x,
            level_y + line_spacing*2. + level_header.height,
            Weight::Bold,
            WHITE,
            32.,
        );
        self.text.draw_text(
            &format!(" /{}", ((level / 100) + 1) * 100),
            sidebar_x + level_label.width,
            level_y + line_spacing*2. + level_header.height,
            Weight::Medium,
            WHITE,
            24.,
        );
    }
    fn draw_field(&self, field: &FieldAndGraphics, offset: Mat4) {
        self.draw_field_well(field);
        self.draw_field_next(field);
        self.draw_left_ui(field);

        set_default_camera();
        set_camera(&Camera3D {
            position: vec3(-35., 0., 0.),
            up: vec3(0., -1., 0.),
            target: vec3(0., 0., 0.),
            ..Default::default()
        });
        let gl = unsafe { get_internal_gl() }.quad_gl;

        gl.push_model_matrix(offset);

        draw_affine_parallelogram(
            Vec3::new(0., 2. - (WELL_ROWS + 4) as f32 / 2., -2.),
            -2. * Vec3::Y,
            4. * Vec3::Z,
            Some(&field.next_target.texture),
            WHITE,
        );

        Game::draw_well_bg();

        draw_affine_parallelogram(
            Vec3::new(0., WELL_ROWS as f32 / 2., 5.),
            -(WELL_ROWS as f32) * Vec3::Y,
            5. * Vec3::Z,
            Some(&field.left_ui_target.texture),
            WHITE,
        );

        draw_affine_parallelogram(
            Vec3::new(0., WELL_ROWS as f32 / 2., WELL_COLS as f32 / -2.),
            -(WELL_ROWS as f32) * Vec3::Y,
            WELL_COLS as f32 * Vec3::Z,
            Some(&field.render_target.texture),
            WHITE,
        );

        gl.pop_model_matrix();
    }
    fn draw_perf(&mut self) {
        self.text.draw_text(&format!("Average FPS: {:.2}", self.fps.iter().sum::<i32>() as f32 / self.fps.len() as f32), 10., 10., Weight::Medium, WHITE, 12.);
        let mut sorted_fps = self.fps.iter().collect::<Vec<_>>();
        sorted_fps.sort();
        let amt = (sorted_fps.len() / 4) as f32;
        let upper = sorted_fps.iter().rev().take(sorted_fps.len() / 4).cloned().sum::<i32>() as f32;
        let lower = sorted_fps.iter().take(sorted_fps.len() / 4).cloned().sum::<i32>() as f32;
        self.text.draw_text(&format!("Upper 25% FPS: {:.2}", upper / amt), 10., 22., Weight::Medium, WHITE, 12.);
        self.text.draw_text(&format!("Lower 25% FPS: {:.2}", lower / amt), 10., 34., Weight::Medium, WHITE, 12.);
        let mut sorted_ticks = self.differences.iter().collect::<Vec<_>>();
        sorted_ticks.sort_by(|a, b| { a.partial_cmp(b).unwrap() });
        let amt_ticks = (sorted_ticks.len() / 4) as f64;
        let upper_ticks = sorted_ticks.iter().rev().take(sorted_ticks.len() / 4).cloned().sum::<f64>();
        let lower_ticks = sorted_ticks.iter().take(sorted_ticks.len() / 4).cloned().sum::<f64>();
        self.text.draw_text(&format!("Average ms between ticks: {:.2}", (self.differences.iter().sum::<f64>() / self.differences.len() as f64) * 1000.), 10., 46., Weight::Medium, WHITE, 12.);
        self.text.draw_text(&format!("Upper 25% ticks: {:.2}", (upper_ticks / amt_ticks) * 1000.), 10., 58., Weight::Medium, WHITE, 12.);
        self.text.draw_text(&format!("Lower 25% ticks: {:.2}", (lower_ticks / amt_ticks) * 1000.), 10., 70., Weight::Medium, WHITE, 12.);
    }
}

struct NetworkInputProvider {
    just_pressed: HashSet<Input>,
    current: HashSet<Input>,
}
impl InputProvider for NetworkInputProvider {
    fn peek(&mut self) {
    }
    fn consume(&mut self) {
        self.just_pressed.clear();
    }
    fn key_just_pressed(&self, input: Input) -> bool {
        self.just_pressed.contains(&input)
    }
    fn key_down(&self, input: Input) -> bool {
        self.current.contains(&input)
    }
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Updater for Game {
    fn update(&mut self, inputs: &Inputs, ticks: u64) {
        // for input in INPUTS {
        //     if inputs.key_just_pressed(*input) {
        //         self.network.send(ClientToServer::Input { input: *input, up: true }.serialize_json().as_bytes());
        //     } else if inputs.key_just_released(*input) {
        //         self.network.send(ClientToServer::Input { input: *input, up: false }.serialize_json().as_bytes());
        //     }
        // }

        // while let Some(bytes) = self.network.try_recv() {
        //     let msg = ServerToClient::deserialize_json(str::from_utf8(&bytes).unwrap()).unwrap();
        //     match msg {
        //     ServerToClient::Join { client_id, field } => {
        //         self.fields.push(FieldAndGraphics::new(Some((Inputs::new(), NetworkInputProvider {
        //             just_pressed: HashSet::new(),
        //             current: HashSet::new(),
        //         })), field, client_id));
        //     }
        //     ServerToClient::Leave { client_id } => {
        //         self.fields.retain(|f| { f.client_id != client_id });
        //     }
        //     ServerToClient::Input { client_id, input, up } => {
        //         if let Some(field) = self.fields.iter_mut().find(|it| it.client_id == client_id) {
        //             if let Some((_inputs, provider)) = &mut field.inputs_override {
        //                 if up {
        //                     provider.just_pressed.insert(input);
        //                     provider.current.insert(input);
        //                 } else {
        //                     provider.current.remove(&input);
        //                 }
        //             }
        //         }
        //     }
        //     ServerToClient::Tick { client_id } => {
        //         if let Some(field) = self.fields.iter_mut().find(|it| it.client_id == client_id) {
        //             if let Some((ref mut inner, ref mut provider)) = &mut field.inputs_override {
        //                 inner.tick(ticks, provider);
        //                 field.field.update(&inner, &mut self.sounds, &mut field.cubes);
        //             }
        //         }
        //     }
        //     }
        // }

        // self.replay.replay_tick(inputs);
        // for field in &mut self.fields {
        //     if let Some(_) = field.inputs_override {
        //         // inner.tick(ticks, provider);
        //         // field.field.update(&inner, &mut self.sounds, &mut field.cubes);
        //     } else {
        //         field.field.update(&inputs, &mut self.sounds, &mut field.cubes);
        //     }
        //     field.cubes.tick();
        // }
        // self.network.send(ClientToServer::Tick {}.serialize_json().as_bytes());
        self.fps.push_back(get_fps());
        while self.fps.len() >= 60*10 {
            self.fps.pop_front();
        }
        self.differences.push_back(get_time() - self.last_tick);
        while self.differences.len() >= 60*5 {
            self.differences.pop_front();
        }
        self.last_tick = get_time();
    }

    fn draw(&mut self) {
        set_default_camera();
        clear_background(LIGHTGRAY);
        self.graphics.draw_background();
        let gl = unsafe { get_internal_gl() }.quad_gl;

        self.draw_perf();
        return;

        for (idx, field) in self.fields.iter().enumerate() {
            self.draw_field(
                field,
                Mat4::from_translation(vec3(0., 0., idx as f32 * 15.)),
            );

            gl.push_model_matrix(Mat4::from_translation(vec3(0., 0., idx as f32 * 15.)));
            for cube in &field.cubes.cubes {
                gl.push_model_matrix(Mat4::from_translation(Vec3::new(
                    cube.z,
                    cube.y - (WELL_ROWS as f32) / 2. + 0.5,
                    cube.x - (WELL_COLS as f32) / 2. + 0.5,
                )));
                gl.push_model_matrix(Mat4::from_rotation_z(cube.rz));
                draw_cube(
                    Vec3::new(0., 0., 0.),
                    Vec3::new(1., 1., 1.),
                    None,
                    color(cube.color),
                );
                gl.pop_model_matrix();
                gl.pop_model_matrix();
            }
            gl.pop_model_matrix();
        }
    }
}

#[macroquad::main("Edrefis")]
async fn main() {
    macroquad::rand::srand(macroquad::miniquad::date::now() as u64);
    let my_id = macroquad::rand::rand();

    let mut ticker = Ticker::new(Game {
        fields: vec![
            FieldAndGraphics::new(None, Field::new(), my_id),
        ],
            // if cfg!(target_arch = "wasm32") {
            //     vec![FieldAndGraphics::new(None)]
            // } else {
            //     vec![
            //         FieldAndGraphics::new(None),
            //         // FieldAndGraphics::new({
            //         //     let mut file = std::fs::File::open("replay.json").unwrap();
            //         //     let mut contents = String::new();
            //         //     file.read_to_string(&mut contents).unwrap();
            //         //     let replay = Replay::deserialize_json(&contents).unwrap();
            //         //     Some(Inputs::new(Box::new(ReplayInputProvider::new(replay))))
            //         // }),
            //     ]
            // },
        my_id,
        network: {
            #[cfg(not(target_arch = "wasm32"))]
            let mut socket = QuadSocket::connect("blackquill.cc:8088").unwrap();
            #[cfg(target_arch = "wasm32")]
            let mut socket = QuadSocket::connect("wss://1293045598395830332.discordsays.com/.proxy/api").unwrap();
            #[cfg(target_arch = "wasm32")]
            {
                while socket.is_wasm_websocket_connected() == false {
                    next_frame().await;
                }
            }
            socket.send(ClientToServer::Join { client_id: my_id }.serialize_json().as_bytes());
            socket
        },
        graphics: Graphics::new(),
        text: Text::new().unwrap(),
        sounds: ClientSounds::new().await.unwrap(),
        replay: Replay::new(10),
        fps: {
            let mut it = VecDeque::new();
            it.push_back(60);
            it
        },
        last_tick: get_time(),
        differences: VecDeque::new(),
    });
    ticker.run().await
}
