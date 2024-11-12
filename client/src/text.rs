// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use macroquad::{prelude::*, Error};

pub struct Text {
    bold: Font,
    medium: Font,
}

pub enum Weight {
    Bold,
    Medium,
}

impl Text {
    pub fn new() -> Result<Text, Error> {
        let bold = load_ttf_font_from_bytes(include_bytes!("font/HankenGrotesk-Bold.ttf"))?;
        let medium = load_ttf_font_from_bytes(include_bytes!("font/HankenGrotesk-Medium.ttf"))?;
        Ok(Text { bold, medium })
    }
    fn font(&self, weight: Weight) -> Option<&Font> {
        match weight {
            Weight::Bold => Some(&self.bold),
            Weight::Medium => Some(&self.medium),
        }
    }
    pub fn measure_text(&self, text: &str, weight: Weight, size: f32) -> TextDimensions {
        measure_text(text, self.font(weight), size as u16, 1.)
    }
    pub fn draw_text(&self, text: &str, x: f32, y: f32, weight: Weight, color: Color, size: f32) -> TextDimensions {
        // let (font_size, font_scale, font_scale_aspect) = camera_font_scale(size);
        draw_text_ex(text, x, y, TextParams {
            font: self.font(weight),
            font_size: size as u16,
            font_scale: 1.,
            font_scale_aspect: 1.,
            color,
            ..Default::default()
        })
    }
}
