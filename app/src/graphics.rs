// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use logic::piece::Piece;
use logic::well::{Block, Well, WELL_COLS, WELL_ROWS};
use sdl2 as sdl;

use sdl::image::LoadTexture;
use sdl::rect::Rect;
use sdl::pixels::Color;
use sdl::video::{Window, WindowContext};
use sdl::render::{Texture, TextureCreator, Canvas};

pub struct Graphics<'a> {
    blocks: Texture<'a>,
    background: Texture<'a>,
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

const SOURCE_BLOCK_SIZE: u32 = 8;
const DESTINATION_PIXEL_SIZE: u32 = 3;
const DESTINATION_BLOCK_SIZE: u32 = SOURCE_BLOCK_SIZE * DESTINATION_PIXEL_SIZE;

impl Graphics<'_> {
    pub fn new<'a>(texture_creator: &'a TextureCreator<WindowContext>) -> Result<Graphics<'a>, String> {

        Ok(Graphics {
            blocks: texture_creator.load_texture_bytes(include_bytes!("gfx/tiles.png"))?,
            background: texture_creator.load_texture_bytes(include_bytes!("gfx/background.png"))?,
        })
    }
    pub fn draw_background(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.copy(&self.background, None, None)
    }
    pub fn draw_block_at(&self, canvas: &mut Canvas<Window>, x: i32, y: i32, block: Block) -> Result<(), String> {
        canvas.copy(
            &self.blocks,
            Some(Rect::new(texture_index(block) * SOURCE_BLOCK_SIZE as i32, 0, SOURCE_BLOCK_SIZE, SOURCE_BLOCK_SIZE)),
            Some(Rect::new(x, y, DESTINATION_BLOCK_SIZE, DESTINATION_BLOCK_SIZE)),
        )
    }
    pub fn draw_well_background(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 140));
        canvas.fill_rect(Rect::new(0, 0, WELL_COLS as u32 * DESTINATION_BLOCK_SIZE, WELL_ROWS as u32 * DESTINATION_BLOCK_SIZE))?;
        Ok(())
    }
    pub fn draw_well(&self, canvas: &mut Canvas<Window>, well: &Well) -> Result<(), String> {
        for (i, row) in well.blocks.iter().enumerate() {
            for (j, col) in row.iter().enumerate() {
                if let Some(block) = col {
                    let bx = j as i32 * DESTINATION_BLOCK_SIZE as i32;
                    let by = i as i32 * DESTINATION_BLOCK_SIZE as i32;
                    self.draw_block_at(canvas, bx, by, *block)?;
                    canvas.set_draw_color(Color::RGBA(0,0,0,60));
                    canvas.fill_rect(Rect::new(bx, by, DESTINATION_BLOCK_SIZE, DESTINATION_BLOCK_SIZE))?;
                }
            }
        }
        Ok(())
    }
    pub fn well_viewport<O: FnMut(&mut Canvas<Window>) -> Result<(), String>>(canvas: &mut Canvas<Window>, mut inner: O) -> Result<(), String> {
        let (w, h) = canvas.output_size()?;

        canvas.set_viewport(Some(Rect::new(w as i32/2 - (DESTINATION_BLOCK_SIZE as i32*WELL_COLS as i32)/2, h as i32/2 - (DESTINATION_BLOCK_SIZE as i32*WELL_ROWS as i32)/2, w, h)));
        inner(canvas)?;
        canvas.set_viewport(None);
        Ok(())
    }
    pub fn well_side_viewport<O: FnMut(&mut Canvas<Window>) -> Result<(), String>>(canvas: &mut Canvas<Window>, mut inner: O) -> Result<(), String> {
        let (w, h) = canvas.output_size()?;

        canvas.set_viewport(Some(Rect::new(w as i32/2 - (DESTINATION_BLOCK_SIZE as i32*WELL_COLS as i32), h as i32/2 - (DESTINATION_BLOCK_SIZE as i32*WELL_ROWS as i32)/2, w, h)));
        inner(canvas)?;
        canvas.set_viewport(None);
        Ok(())
    }
    pub fn draw_outlines(&self, canvas: &mut Canvas<Window>, well: &Well) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(204, 204, 204, 204));

        let mut draw_rectangle = |x, y, w, h| {
            canvas.fill_rect(Rect::new(x as i32, y as i32, w, h))
        };

        for (i, row) in well.blocks.iter().enumerate() {
            for (j, col) in row.iter().enumerate() {
                if col.is_some() {
                    let bx = j as u32 * DESTINATION_BLOCK_SIZE;
                    let by = i as u32 * DESTINATION_BLOCK_SIZE;

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
                        draw_rectangle(bx, by, DESTINATION_BLOCK_SIZE, DESTINATION_PIXEL_SIZE)?;
                        top = true;
                    }
                    if check(0, 1) {
                        draw_rectangle(bx, by + DESTINATION_BLOCK_SIZE - DESTINATION_PIXEL_SIZE, DESTINATION_BLOCK_SIZE, DESTINATION_PIXEL_SIZE)?;
                        bottom = true;
                    }
                    if check(-1, 0) {
                        draw_rectangle(bx, by, DESTINATION_PIXEL_SIZE, DESTINATION_BLOCK_SIZE)?;
                        left = true;
                    }
                    if check(1, 0) {
                        draw_rectangle(bx + DESTINATION_BLOCK_SIZE - DESTINATION_PIXEL_SIZE, by, DESTINATION_PIXEL_SIZE, DESTINATION_BLOCK_SIZE)?;
                        right = true;
                    }

                    if !left && !top && check(-1, -1) {
                        draw_rectangle(bx, by, DESTINATION_PIXEL_SIZE, DESTINATION_PIXEL_SIZE)?;
                    }
                    if !right && !top && check(1, -1) {
                        draw_rectangle(bx + DESTINATION_BLOCK_SIZE - DESTINATION_PIXEL_SIZE, by, DESTINATION_PIXEL_SIZE, DESTINATION_PIXEL_SIZE)?;
                    }
                    if !left && !bottom && check(-1, 1) {
                        draw_rectangle(bx, by + DESTINATION_BLOCK_SIZE - DESTINATION_PIXEL_SIZE, DESTINATION_PIXEL_SIZE, DESTINATION_PIXEL_SIZE)?;
                    }
                    if !right && !bottom && check(1, 1) {
                        draw_rectangle(bx + DESTINATION_BLOCK_SIZE - DESTINATION_PIXEL_SIZE, by + DESTINATION_BLOCK_SIZE - DESTINATION_PIXEL_SIZE, DESTINATION_PIXEL_SIZE, DESTINATION_PIXEL_SIZE)?;
                    }
                }
            }
        }
        Ok(())
    }
    pub fn draw_piece_at(&self, canvas: &mut Canvas<Window>, piece: &Piece, x: i32, y: i32, darkening: u8) -> Result<(), String> {
        for (i, row) in piece.rotations.piece_map()[piece.rotation]
            .iter()
            .enumerate()
        {
            for (j, col) in row.iter().enumerate() {
                if *col {
                    let bx = (x + j as i32) * DESTINATION_BLOCK_SIZE as i32;
                    let by = (y + i as i32) * DESTINATION_BLOCK_SIZE as i32;
                    self.draw_block_at(canvas, bx, by, piece.color)?;
                    canvas.set_draw_color(Color::RGBA(0,0,0,darkening));
                    canvas.fill_rect(Rect::new(bx, by, DESTINATION_BLOCK_SIZE, DESTINATION_BLOCK_SIZE))?;
                }
            }
        }
        Ok(())
    }
    pub fn draw_piece(&self, canvas: &mut Canvas<Window>, piece: &Piece, darkening: u8) -> Result<(), String> {
        self.draw_piece_at(canvas, piece, piece.x, piece.y, darkening)
    }
}

