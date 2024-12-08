// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::rc::Rc;
use glam::{Vec2, Vec3, Vec3Swizzles};
use logic::{field::{level_to_gravity, Field}, piece::Piece, well::{Block, Well, WELL_COLS, WELL_ROWS}};
use sdl2::image::ImageRWops;
use crate::{gpu::{parallelogram, rectangle, Camera2D, Camera3D, State}, lerp};

fn texture_index(block: Block) -> i32 {
    match block {
        Block::Red => 0,
        Block::Orange => 1,
        Block::Yellow => 2,
        Block::Green => 3,
        Block::Cyan => 4,
        Block::Blue => 5,
        Block::Purple => 6,
    }
}


// fn color(block: Block) -> wgpu::Color {
//     match block {
//     Block::Red => wgpu::Color { r: 1.0, g: 0.0, b: 0.18823529411764706, a: 1.0 },
//     Block::Orange => wgpu::Color { r: 1.0, g: 0.4392156862745098, b: 0.0, a: 1.0 },
//     Block::Yellow => wgpu::Color { r: 1.0, g: 0.7647058823529411, b: 0.0, a: 1.0 },
//     Block::Green => wgpu::Color { r: 0.4588235294117647, g: 0.9333333333333333, b: 0.2235294117647059, a: 1.0 },
//     Block::Cyan => wgpu::Color { r: 0.0, g: 0.9411764705882353, b: 0.8274509803921568, a: 1.0 },
//     Block::Blue => wgpu::Color { r: 0.25098039215686274, g: 0.6235294117647059, b: 0.9725490196078431, a: 1.0 },
//     Block::Purple => wgpu::Color { r: 0.7137254901960784, g: 0.47058823529411764, b: 0.9607843137254902, a: 1.0 },
//     }
// }

fn tilemap_position(block: Block) -> Vec2 {
    Vec2::new(texture_index(block) as f32 * 1. / 12., 0.)
}

const TILEMAP_WIDTH: f32 = 1. / 12.;
const TILEMAP_HEIGHT: f32 = 1.;

pub struct Graphics {
    tilemap: Rc<wgpu::BindGroup>,
    well: (Rc<wgpu::BindGroup>, Rc<wgpu::TextureView>),
    score_buffer: glyphon::Buffer,
}

impl Graphics {
    pub fn new(state: &mut State) -> Result<Graphics, String> {
        let png = sdl2::rwops::RWops::from_bytes(include_bytes!("gfx/tiles.png"))?.load_png()?;

        let tilemap = state.upload_texture(&png);
        let well = state.create_texture(WELL_COLS as u32 * 8, WELL_ROWS as u32 * 8);
        let mut buffer = state.create_buffer();
        Graphics::score_text(&mut buffer, state, 0, 0);

        Ok(Graphics {
            tilemap,
            well,
            score_buffer: buffer,
        })
    }
    pub fn score_text(buffer: &mut glyphon::Buffer, state: &mut State, gravity: i32, level: u32) {
        let attrs = glyphon::Attrs::new().family(glyphon::Family::Name("Hanken Grotesk")).weight(glyphon::Weight::MEDIUM).color(glyphon::Color::rgba(255, 255, 255, 180));

        let is_20g = gravity >= 256;
        let gravity_amount = if !is_20g {
            gravity / 2
        } else {
            gravity / 256
        };

        state.set_buffer_text(buffer, [
            ("Gravity\n", attrs.metrics(glyphon::Metrics::relative(24., 1.2))),
            (&format!("{}", gravity_amount), attrs.metrics(glyphon::Metrics::relative(32., 1.2)).weight(glyphon::Weight::BOLD).color(glyphon::Color::rgba(255, 255, 255, 255))),
            if is_20g {
                ("G", attrs.metrics(glyphon::Metrics::relative(32., 1.2)))
            } else {
                (" /128", attrs.metrics(glyphon::Metrics::relative(24., 1.2)))
            },
            ("\n", attrs),
            ("Level\n", attrs.metrics(glyphon::Metrics::relative(24., 1.2))),
            (&format!("{}", level), attrs.metrics(glyphon::Metrics::relative(32., 1.2)).weight(glyphon::Weight::BOLD).color(glyphon::Color::rgba(255, 255, 255, 255))),
            (" /", attrs.metrics(glyphon::Metrics::relative(24., 1.2))),
            ("100\n", attrs.metrics(glyphon::Metrics::relative(24., 1.2))),
        ], attrs);
    }
    pub fn queue_well_bg(state: &mut State) {
        let well_width = WELL_COLS as f32;
        let well_height = WELL_ROWS as f32;
        let wall = wgpu::Color { r: 0.77625, g: 0.96804, b: 1.00513, a: 0.1 };

        // well bg
        state.queue_draw(
            parallelogram(
                Vec3::new(well_width / -2., well_height / -2., -1.),
                well_width * Vec3::X,
                well_height * Vec3::Y,
                Vec2::ZERO,
                Vec2::X,
                Vec2::Y,
                wgpu::Color { r: 0., g: 0., b: 0., a: 0.4 },
            )
        );

        // bottom
        state.queue_draw(
            parallelogram(
                Vec3::new(well_width / -2., well_height / -2., -1.),
                well_width * Vec3::X,
                2. * Vec3::Z,
                Vec2::ZERO,
                Vec2::X,
                Vec2::Y,
                wall,
            )
        );

        // left
        state.queue_draw(
            parallelogram(
                Vec3::new(well_width / -2., well_height / -2., -1.),
                well_height * Vec3::Y,
                2. * Vec3::Z,
                Vec2::ZERO,
                Vec2::X,
                Vec2::Y,
                wall,
            )
        );

        // right
        state.queue_draw(
            parallelogram(
                Vec3::new(well_width / 2., well_height / -2., -1.),
                well_height * Vec3::Y,
                2. * Vec3::Z,
                Vec2::ZERO,
                Vec2::X,
                Vec2::Y,
                wall,
            )
        );
    }
    pub fn render_well(
        &self,
        well: &Well,
        piece: &Piece,
        state: &mut State,
    ) -> Result<(), String> {
        state.set_camera(&Camera2D::from_rect(Vec2::new(0., 0.), Vec2::new(WELL_COLS as f32, WELL_ROWS as f32), Some(self.well.1.clone())));
        state.start_render_pass(Some(wgpu::Color { r: 0., g: 0., b: 0., a: 0. }));

        state.set_texture(Some(self.tilemap.clone()));

        for (i, row) in well.blocks.iter().enumerate() {
            for (j, col) in row.iter().enumerate() {
                if let Some(block) = col {
                    let bx = j as f32;
                    let by = i as f32;

                    state.queue_draw(rectangle(Vec3::new(bx, by, 0.), 1., 1., tilemap_position(*block), TILEMAP_WIDTH, TILEMAP_HEIGHT, wgpu::Color::WHITE));
                }
            }
        }

        for (i, row) in piece.rotations.piece_map()[piece.rotation]
            .iter()
            .enumerate()
        {
            for (j, col) in row.iter().enumerate() {
                if *col {
                    let bx = piece.x as f32 + j as f32;
                    let by = piece.y as f32 + i as f32;

                    state.queue_draw(rectangle(Vec3::new(bx, by, 0.), 1., 1., tilemap_position(piece.color), TILEMAP_WIDTH, TILEMAP_HEIGHT, wgpu::Color::WHITE));
                }
            }
        }

        state.do_draw()?;

        state.set_texture(None);

        for (i, row) in well.blocks.iter().enumerate() {
            for (j, col) in row.iter().enumerate() {
                if col.is_some() {
                    let bx = j as f32;
                    let by = i as f32;

                    state.queue_draw(rectangle(Vec3::new(bx, by, 0.), 1., 1., Vec2::new(0., 0.), 1., 1., wgpu::Color { r: 0., g: 0., b: 0., a: 0.5 }));
                }
            }
        }

        let pixel_color = wgpu::Color { r: 0.9, g: 0.9, b: 0.9, a: 0.4 };
        const DST_BLOCK_SIZE: f32 = 1.;
        const DST_PIXEL_SIZE: f32 = 1. / 8.;

        for (i, row) in well.blocks.iter().enumerate() {
            for (j, col) in row.iter().enumerate() {
                if col.is_some() {
                    let bx = j as f32 * DST_BLOCK_SIZE;
                    let by = i as f32 * DST_BLOCK_SIZE;

                    let check = |dx: i32, dy: i32| {
                        let row_idx = i as i32+dy;
                        let col_idx = j as i32+dx;
                        if row_idx < 0 || col_idx < 0 {
                            false
                        } else if row_idx as usize >= WELL_ROWS || col_idx as usize >= WELL_COLS {
                            false
                        } else {
                            well.blocks[row_idx as usize][col_idx as usize].is_none()
                        }
                    };

                    let mut top = false;
                    let mut left = false;
                    let mut right = false;
                    let mut bottom = false;

                    if check(0, -1) {
                        state.queue_draw(rectangle(Vec3::new(bx, by, 0.), DST_BLOCK_SIZE, DST_PIXEL_SIZE, Vec2::ZERO, 1., 1., pixel_color));
                        top = true;
                    }
                    if check(0, 1) {
                        state.queue_draw(rectangle(Vec3::new(bx, by + DST_BLOCK_SIZE - DST_PIXEL_SIZE, 0.), DST_BLOCK_SIZE, DST_PIXEL_SIZE, Vec2::ZERO, 1., 1., pixel_color));
                        bottom = true;
                    }
                    if check(-1, 0) {
                        state.queue_draw(rectangle(Vec3::new(bx, by, 0.), DST_PIXEL_SIZE, DST_BLOCK_SIZE, Vec2::ZERO, 1., 1., pixel_color));
                        left = true;
                    }
                    if check(1, 0) {
                        state.queue_draw(rectangle(Vec3::new(bx + DST_BLOCK_SIZE - DST_PIXEL_SIZE, by, 0.), DST_PIXEL_SIZE, DST_BLOCK_SIZE, Vec2::ZERO, 1., 1., pixel_color));
                        right = true;
                    }

                    if !left && !top && check(-1, -1) {
                        state.queue_draw(rectangle(Vec3::new(bx, by, 0.), DST_PIXEL_SIZE, DST_PIXEL_SIZE, Vec2::ZERO, 1., 1., pixel_color));
                    }
                    if !right && !top && check(1, -1) {
                        state.queue_draw(rectangle(Vec3::new(bx + DST_BLOCK_SIZE - DST_PIXEL_SIZE, by, 0.), DST_PIXEL_SIZE, DST_PIXEL_SIZE, Vec2::ZERO, 1., 1., pixel_color));
                    }
                    if !left && !bottom && check(-1, 1) {
                        state.queue_draw(rectangle(Vec3::new(bx, by + DST_BLOCK_SIZE - DST_PIXEL_SIZE, 0.), DST_PIXEL_SIZE, DST_PIXEL_SIZE, Vec2::ZERO, 1., 1., pixel_color));
                    }
                    if !right && !bottom && check(1, 1) {
                        state.queue_draw(rectangle(Vec3::new(bx + DST_BLOCK_SIZE - DST_PIXEL_SIZE, by + DST_BLOCK_SIZE - DST_PIXEL_SIZE, 0.), DST_PIXEL_SIZE, DST_PIXEL_SIZE, Vec2::ZERO, 1., 1., pixel_color));
                    }
                }
            }
        }

        for (i, row) in piece.rotations.piece_map()[piece.rotation]
            .iter()
            .enumerate()
        {
            for (j, col) in row.iter().enumerate() {
                if *col {
                    let bx = piece.x as f32 + j as f32;
                    let by = piece.y as f32 + i as f32;

                    state.queue_draw(rectangle(Vec3::new(bx, by, 0.), 1., 1., Vec2::new(0., 0.), 1., 1., wgpu::Color { r: 0., g: 0., b: 0., a: lerp(0.8, 0., piece.ticks_to_lock as f32 / 30.) as f64 }));
                }
            }
        }
        state.do_draw()?;
        state.complete_render_pass()?;

        Ok(())
    }
    pub fn render(&mut self, field: &Field, well: &Well, piece: &Piece, state: &mut State) -> Result<(), String> {
        self.render_well(well, piece, state)?;

        state.set_camera(&Camera3D::default());

        state.start_render_pass(Some(wgpu::Color { r: 0.05, g: 0.05, b: 0.1, a: 1.0 }));
        state.set_texture(None);

        Graphics::queue_well_bg(state);
        state.do_draw()?;

        state.set_texture(Some(self.well.0.clone()));

        let well_width = WELL_COLS as f32;
        let well_height = WELL_ROWS as f32;
        state.queue_draw(
            parallelogram(
                Vec3::new(well_width / -2., well_height / -2., 0.),
                well_width * Vec3::X,
                well_height * Vec3::Y,
                Vec2::ZERO,
                Vec2::X,
                Vec2::Y,
                wgpu::Color::WHITE,
            )
        );
        state.do_draw()?;

        let point = state.world_to_view(Vec3::new(well_width / 2. + 1., well_height / 2., 0.));
        Graphics::score_text(&mut self.score_buffer, state, level_to_gravity(field.level), field.level);
        state.draw_text(&mut self.score_buffer, point)?;

        state.complete_render_pass()?;

        state.present()?;

        Ok(())
    }
}