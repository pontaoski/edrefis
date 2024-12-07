// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::rc::Rc;

use cgmath::{Vector2, Vector3, Zero};
use logic::{piece::Piece, well::{Block, Well, WELL_COLS, WELL_ROWS}};
use sdl2::image::ImageRWops;
use crate::{gpu::{rectangle, Camera2D, State}, lerp};

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


fn color(block: Block) -> wgpu::Color {
    match block {
    Block::Red => wgpu::Color { r: 1.0, g: 0.0, b: 0.18823529411764706, a: 1.0 },
    Block::Orange => wgpu::Color { r: 1.0, g: 0.4392156862745098, b: 0.0, a: 1.0 },
    Block::Yellow => wgpu::Color { r: 1.0, g: 0.7647058823529411, b: 0.0, a: 1.0 },
    Block::Green => wgpu::Color { r: 0.4588235294117647, g: 0.9333333333333333, b: 0.2235294117647059, a: 1.0 },
    Block::Cyan => wgpu::Color { r: 0.0, g: 0.9411764705882353, b: 0.8274509803921568, a: 1.0 },
    Block::Blue => wgpu::Color { r: 0.25098039215686274, g: 0.6235294117647059, b: 0.9725490196078431, a: 1.0 },
    Block::Purple => wgpu::Color { r: 0.7137254901960784, g: 0.47058823529411764, b: 0.9607843137254902, a: 1.0 },
    }
}

fn tilemap_position(block: Block) -> Vector2<f32> {
    Vector2::new(texture_index(block) as f32 * 1. / 12., 0.)
}

const TILEMAP_WIDTH: f32 = 1. / 12.;
const TILEMAP_HEIGHT: f32 = 1.;

pub struct Graphics {
    tilemap: Rc<wgpu::BindGroup>,
}

impl Graphics {
    pub fn new(state: &State) -> Result<Graphics, String> {
        let png = sdl2::rwops::RWops::from_bytes(include_bytes!("gfx/tiles.png"))?.load_png()?;

        let tilemap = state.upload_texture(&png);

        Ok(Graphics {
            tilemap,
        })
    }
    pub fn render_well(
        &self,
        well: &Well,
        piece: &Piece,
        state: &mut State,
    ) -> Result<(), String> {
        state.set_camera(&Camera2D::from_rect(Vector2::new(0., 0.), Vector2::new(WELL_COLS as f32, WELL_ROWS as f32), None));
        state.set_texture(Some(self.tilemap.clone()));

        for (i, row) in well.blocks.iter().enumerate() {
            for (j, col) in row.iter().enumerate() {
                if let Some(block) = col {
                    let bx = j as f32;
                    let by = i as f32;

                    state.queue_draw(rectangle(Vector3::new(bx, by, 0.), 1., 1., tilemap_position(*block), TILEMAP_WIDTH, TILEMAP_HEIGHT, wgpu::Color::WHITE));
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

                    state.queue_draw(rectangle(Vector3::new(bx, by, 0.), 1., 1., tilemap_position(piece.color), TILEMAP_WIDTH, TILEMAP_HEIGHT, wgpu::Color::WHITE));
                }
            }
        }

        state.do_draw(Some(wgpu::Color { r: 0.05, g: 0.05, b: 0.1, a: 1.0 }))?;

        state.set_texture(None);

        for (i, row) in well.blocks.iter().enumerate() {
            for (j, col) in row.iter().enumerate() {
                if col.is_some() {
                    let bx = j as f32;
                    let by = i as f32;

                    state.queue_draw(rectangle(Vector3::new(bx, by, 0.), 1., 1., Vector2::new(0., 0.), 1., 1., wgpu::Color { r: 0., g: 0., b: 0., a: 0.5 }));
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
                        state.queue_draw(rectangle(Vector3::new(bx, by, 0.), DST_BLOCK_SIZE, DST_PIXEL_SIZE, Vector2::zero(), 1., 1., pixel_color));
                        top = true;
                    }
                    if check(0, 1) {
                        state.queue_draw(rectangle(Vector3::new(bx, by + DST_BLOCK_SIZE - DST_PIXEL_SIZE, 0.), DST_BLOCK_SIZE, DST_PIXEL_SIZE, Vector2::zero(), 1., 1., pixel_color));
                        bottom = true;
                    }
                    if check(-1, 0) {
                        state.queue_draw(rectangle(Vector3::new(bx, by, 0.), DST_PIXEL_SIZE, DST_BLOCK_SIZE, Vector2::zero(), 1., 1., pixel_color));
                        left = true;
                    }
                    if check(1, 0) {
                        state.queue_draw(rectangle(Vector3::new(bx + DST_BLOCK_SIZE - DST_PIXEL_SIZE, by, 0.), DST_PIXEL_SIZE, DST_BLOCK_SIZE, Vector2::zero(), 1., 1., pixel_color));
                        right = true;
                    }

                    if !left && !top && check(-1, -1) {
                        state.queue_draw(rectangle(Vector3::new(bx, by, 0.), DST_PIXEL_SIZE, DST_PIXEL_SIZE, Vector2::zero(), 1., 1., pixel_color));
                    }
                    if !right && !top && check(1, -1) {
                        state.queue_draw(rectangle(Vector3::new(bx + DST_BLOCK_SIZE - DST_PIXEL_SIZE, by, 0.), DST_PIXEL_SIZE, DST_PIXEL_SIZE, Vector2::zero(), 1., 1., pixel_color));
                    }
                    if !left && !bottom && check(-1, 1) {
                        state.queue_draw(rectangle(Vector3::new(bx, by + DST_BLOCK_SIZE - DST_PIXEL_SIZE, 0.), DST_PIXEL_SIZE, DST_PIXEL_SIZE, Vector2::zero(), 1., 1., pixel_color));
                    }
                    if !right && !bottom && check(1, 1) {
                        state.queue_draw(rectangle(Vector3::new(bx + DST_BLOCK_SIZE - DST_PIXEL_SIZE, by + DST_BLOCK_SIZE - DST_PIXEL_SIZE, 0.), DST_PIXEL_SIZE, DST_PIXEL_SIZE, Vector2::zero(), 1., 1., pixel_color));
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

                    state.queue_draw(rectangle(Vector3::new(bx, by, 0.), 1., 1., Vector2::new(0., 0.), 1., 1., wgpu::Color { r: 0., g: 0., b: 0., a: lerp(0.8, 0., piece.ticks_to_lock as f32 / 30.) as f64 }));
                }
            }
        }

        state.do_draw(None)?;

        Ok(())
    }
    pub fn render(&self, well: &Well, piece: &Piece, state: &mut State) -> Result<(), String> {
        self.render_well(well, piece, state)?;

        state.present()?;

        Ok(())
    }
}