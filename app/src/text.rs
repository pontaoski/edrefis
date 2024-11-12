// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use sdl2 as sdl;
use sdl::ttf::Font;
use sdl::ttf::Sdl2TtfContext as TtfContext;
use sdl::pixels::Color;
use sdl::rwops::RWops;
use sdl::render::{Canvas, TextureQuery, TextureCreator};
use sdl::video::{Window, WindowContext};
use sdl::rect::Rect;

pub struct Text<'a> {
    medium: Font<'a, 'static>,
    bold: Font<'a, 'static>,
}

#[derive(Copy, Clone, Debug)]
pub enum Weight {
    Bold,
    Medium,
}

impl Text<'_> {
    pub fn new<'a>(context: &'a TtfContext) -> Result<Text<'_>, String> {
        let bold = context.load_font_from_rwops(RWops::from_bytes(include_bytes!("font/HankenGrotesk-Bold.ttf"))?, 24)?;
        let medium = context.load_font_from_rwops(RWops::from_bytes(include_bytes!("font/HankenGrotesk-Medium.ttf"))?, 24)?;

        Ok(
            Text {
                medium,
                bold,
            }
        )
    }
    pub fn draw_text(&self, canvas: &mut Canvas<Window>, texture_creator: &mut TextureCreator<WindowContext>, text: &str, x: i32, y: i32, weight: Weight, color: Color) -> Result<(), String> {
        let surface = match weight {
        Weight::Medium => &self.medium,
        Weight::Bold => &self.bold,
        }.render(text)
         .blended(color)
         .map_err(|e| e.to_string())?;

        let texture = surface.as_texture(texture_creator).map_err(|e| e.to_string())?;
        let TextureQuery { width, height, .. } = texture.query();

        canvas.copy(&texture, None, Some(Rect::new(x, y, width, height)))?;

        Ok(())
    }
}
