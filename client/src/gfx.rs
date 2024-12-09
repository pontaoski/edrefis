// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use logic::{piece::Piece, well::{Block, Well, WELL_COLS, WELL_ROWS}};
use macroquad::prelude::*;

pub const SRC_BLOCK_SIZE: f32 = 8.0;
pub const MULT: f32 = 4.;
pub const DST_BLOCK_SIZE: f32 = MULT * 8.0;
pub const DST_PIXEL_SIZE: f32 = MULT;

pub struct Graphics {
    blocks: Texture2D,
    background: Texture2D,
}

pub fn color(block: Block) -> Color {
    match block {
    Block::Red => Color::new(1.0, 0.0, 0.18823529411764706, 1.0),
    Block::Orange => Color::new(1.0, 0.4392156862745098, 0.0, 1.0),
    Block::Yellow => Color::new(1.0, 0.7647058823529411, 0.0, 1.0),
    Block::Green => Color::new(0.4588235294117647, 0.9333333333333333, 0.2235294117647059, 1.0),
    Block::Cyan => Color::new(0.0, 0.9411764705882353, 0.8274509803921568, 1.0),
    Block::Blue => Color::new(0.25098039215686274, 0.6235294117647059, 0.9725490196078431, 1.0),
    Block::Purple => Color::new(0.7137254901960784, 0.47058823529411764, 0.9607843137254902, 1.0),
    }
}
pub fn texture_index(block: Block) -> i32 {
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

impl Graphics {
    pub fn new() -> Graphics {
        let blocks = Texture2D::from_file_with_format(include_bytes!("./tiles.png"), None);
        blocks.set_filter(FilterMode::Nearest);
        let background = Texture2D::from_file_with_format(include_bytes!("./background.png"), None);
        Graphics {
            blocks,
            background,
        }
    }
    pub fn draw_background(&self) {
        draw_texture_ex(
            &self.background,
            0., 0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(screen_width(), screen_height())),
                source: Some(Rect::new(0., 0., self.background.size().x, self.background.size().y)),
                ..Default::default()
            }
        );
    }
    pub fn draw_block_at(&self, x: f32, y: f32, num: i32) {
        draw_texture_ex(&self.blocks, x, y, WHITE, DrawTextureParams {
            dest_size: Some(Vec2::new(DST_BLOCK_SIZE, DST_BLOCK_SIZE)),
            source: Some(Rect::new(num as f32 * SRC_BLOCK_SIZE as f32, 0., SRC_BLOCK_SIZE, SRC_BLOCK_SIZE)),
            ..Default::default()
        });
    }
    pub fn draw_well(&self, well: &Well, greyscale: bool) {
        for (i, row) in well.blocks.iter().enumerate() {
            for (j, col) in row.iter().enumerate() {
                if let Some(block) = col {
                    let bx = j as f32 * DST_BLOCK_SIZE;
                    let by = i as f32 * DST_BLOCK_SIZE;
                    self.draw_block_at(bx, by, if greyscale { 7 } else { texture_index(block.color) });
                    draw_rectangle(bx, by, DST_BLOCK_SIZE, DST_BLOCK_SIZE, Color::new(0., 0., 0., 0.2));
                }
            }
        }
    }
    pub fn draw_outlines(&self, well: &Well) {
        let pixel_color = Color::new(0.9, 0.9, 0.9, 0.8);

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
                        draw_rectangle(bx, by, DST_BLOCK_SIZE, DST_PIXEL_SIZE, pixel_color);
                        top = true;
                    }
                    if check(0, 1) {
                        draw_rectangle(bx, by + DST_BLOCK_SIZE - DST_PIXEL_SIZE, DST_BLOCK_SIZE, DST_PIXEL_SIZE, pixel_color);
                        bottom = true;
                    }
                    if check(-1, 0) {
                        draw_rectangle(bx, by, DST_PIXEL_SIZE, DST_BLOCK_SIZE, pixel_color);
                        left = true;
                    }
                    if check(1, 0) {
                        draw_rectangle(bx + DST_BLOCK_SIZE - DST_PIXEL_SIZE, by, DST_PIXEL_SIZE, DST_BLOCK_SIZE, pixel_color);
                        right = true;
                    }

                    if !left && !top && check(-1, -1) {
                        draw_rectangle(bx, by, DST_PIXEL_SIZE, DST_PIXEL_SIZE, pixel_color);
                    }
                    if !right && !top && check(1, -1) {
                        draw_rectangle(bx + DST_BLOCK_SIZE - DST_PIXEL_SIZE, by, DST_PIXEL_SIZE, DST_PIXEL_SIZE, pixel_color);
                    }
                    if !left && !bottom && check(-1, 1) {
                        draw_rectangle(bx, by + DST_BLOCK_SIZE - DST_PIXEL_SIZE, DST_PIXEL_SIZE, DST_PIXEL_SIZE, pixel_color);
                    }
                    if !right && !bottom && check(1, 1) {
                        draw_rectangle(bx + DST_BLOCK_SIZE - DST_PIXEL_SIZE, by + DST_BLOCK_SIZE - DST_PIXEL_SIZE, DST_PIXEL_SIZE, DST_PIXEL_SIZE, pixel_color);
                    }
                }
            }
        }
    }
    pub fn draw_piece(&self, piece: &Piece, darkening: f32) {
        self.draw_piece_at(piece, piece.x, piece.y, darkening);
    }
    pub fn draw_piece_at(&self, piece: &Piece, x: i32, y: i32, darkening: f32) {
        for (i, row) in piece.rotations.piece_map()[piece.rotation].iter().enumerate() {
            for (j, col) in row.iter().enumerate() {
                if *col {
                    let bx = (x + j as i32) as f32 * DST_BLOCK_SIZE;
                    let by = (y + i as i32) as f32 * DST_BLOCK_SIZE;
                    self.draw_block_at(bx, by, texture_index(piece.color));
                    draw_rectangle(bx, by, DST_BLOCK_SIZE, DST_BLOCK_SIZE, Color::new(0., 0., 0., darkening));
                }
            }
        }
    }
}
