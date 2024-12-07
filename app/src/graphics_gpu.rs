// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::rc::Rc;

use cgmath::{Vector2, Vector3};
use logic::{piece::Piece, well::{Block, Well, WELL_COLS, WELL_ROWS}};
use sdl2::image::ImageRWops;
use crate::gpu::{rectangle, Camera2D, State};

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
        target: &wgpu::TextureView,
        well: &Well,
        piece: &Piece,
        state: &mut State,
    ) -> Result<(), String> {
        state.set_camera(&Camera2D::from_rect(Vector2::new(0., 0.), Vector2::new(WELL_COLS as f32, WELL_ROWS as f32)));
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

        state.do_draw(target)
    }
    pub fn render(&self, well: &Well, piece: &Piece, state: &mut State) -> Result<(), String> {
        let frame = state
            .surface
            .get_current_texture()
            .map_err(|e| e.to_string())?;

        let output = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_well(&output, well, piece, state)?;

        frame.present();

        Ok(())
    }
}