// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use crate::well::Block;

pub trait Cubes {
    fn spawn_cube(&mut self, x: i32, y: i32, color: Block);
}

pub trait Sounds {
    fn block_spawn(&mut self, color: Block);
    fn line_clear(&mut self);
    fn lock(&mut self);
    fn land(&mut self);
}
