// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use logic::{hooks::Cubes, well::{Block, WELL_COLS, WELL_ROWS}};

#[derive(Debug, Copy, Clone)]
pub struct Cube {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rz: f32,
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
    pub drz: f32,
    pub ddy: f32,
    pub color: Block,
}

pub fn lerp(a: f32, b: f32, f: f32) -> f32 {
    a * (1.0 - f) + (b * f)
}

pub struct ClientCubes {
    pub cubes: Vec<Cube>,
    cooldown: u32,
}
impl Cubes for ClientCubes {
    fn spawn_cube(&mut self, x: i32, y: i32, color: Block) {
        self.cubes.push(Cube {
            x: x as f32,
            y: y as f32,
            z: 0.,
            rz: 0.,

            color,
            dx: (x as f32 - (WELL_COLS as f32) / 2.) / 40.,
            dy: -0.28,
            ddy: {
                let base = lerp(0.045, 0.025, y as f32 / WELL_ROWS as f32);

                let horiz = lerp(1.0, 0.75, (x as f32 - (WELL_COLS as f32) / 2.).abs() / (WELL_COLS as f32) / 2.);

                base * horiz
            },
            dz: -0.02,
            drz: -0.1,
        });
        self.cooldown = 41;
    }
}
impl ClientCubes {
    pub fn new() -> ClientCubes {
        ClientCubes {
            cubes: vec![],
            cooldown: 0
        }
    }
    pub fn tick(&mut self) {
        for cube in &mut self.cubes {
            cube.x += cube.dx;
            cube.y += cube.dy;
            cube.z += cube.dz;
            cube.rz += cube.drz;
            cube.dy += cube.ddy;
        }
        self.cooldown = self.cooldown.wrapping_sub(1);
        if self.cooldown == 0 {
            self.cubes.clear();
        }
    }
}