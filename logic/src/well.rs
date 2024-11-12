// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use nanoserde::{DeJson, SerJson};

#[derive(Copy, Clone, Debug, Eq, PartialEq, SerJson, DeJson)]
pub enum Block {
    Red,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Purple,
}

pub const WELL_COLS: usize = 10;
pub const WELL_ROWS: usize = 21;

#[derive(SerJson, DeJson, Clone)]
pub struct Well {
    pub blocks: [[Option<Block>; WELL_COLS]; WELL_ROWS],
}

impl Well {
    pub fn new() -> Well {
        Well {
            blocks: [[None; WELL_COLS]; WELL_ROWS]
        }
    }
    pub fn do_clear(&mut self) -> Vec<(i32, [Option<Block>; WELL_COLS])> {
        let mut cleared = vec![];
        for ri in 0..self.blocks.len() {
            if self.blocks[ri].iter().all(|b| b.is_some()) {
                cleared.push((ri as i32, self.blocks[ri]));
                self.blocks[ri] = [None; WELL_COLS];
            }
        }
        cleared
    }
    pub fn commit_clear(&mut self, vec: &Vec<i32>) {
        for idx in vec {
            self.blocks[0..*idx as usize+1].rotate_right(1);
        }
    }
}
