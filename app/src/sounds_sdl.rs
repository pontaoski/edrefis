// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use logic::{hooks::Sounds, well::Block};
use sdl2::{self as sdl, mixer::LoaderRWops};

use sdl::mixer::Chunk;

pub struct ClientSounds {
    lock: Chunk,
    land: Chunk,
    lineclear: Chunk,
    pieces1: Chunk,
    pieces2: Chunk,
    pieces3: Chunk,
    pieces4: Chunk,
    pieces5: Chunk,
    pieces6: Chunk,
    pieces7: Chunk,
}

const LOCK: &'static [u8] = include_bytes!("audio/lock.wav");
const LAND: &'static [u8] = include_bytes!("audio/land.wav");
const LINECLEAR: &'static [u8] = include_bytes!("audio/lineclear.wav");
const PIECES1: &'static [u8] = include_bytes!("audio/pieces1.wav");
const PIECES2: &'static [u8] = include_bytes!("audio/pieces2.wav");
const PIECES3: &'static [u8] = include_bytes!("audio/pieces3.wav");
const PIECES4: &'static [u8] = include_bytes!("audio/pieces4.wav");
const PIECES5: &'static [u8] = include_bytes!("audio/pieces5.wav");
const PIECES6: &'static [u8] = include_bytes!("audio/pieces6.wav");
const PIECES7: &'static [u8] = include_bytes!("audio/pieces7.wav");

impl Sounds for ClientSounds {
    fn line_clear(&mut self) {
        sdl::mixer::Channel::all().play(&self.lineclear, 0).unwrap();
    }
    fn block_spawn(&mut self, color: Block) {
        sdl::mixer::Channel::all().play(match color {
            Block::Yellow => &self.pieces1,
            Block::Blue => &self.pieces2,
            Block::Orange => &self.pieces3,
            Block::Green => &self.pieces4,
            Block::Purple => &self.pieces5,
            Block::Cyan => &self.pieces6,
            Block::Red => &self.pieces7,
        }, 0).unwrap();
    }
    fn lock(&mut self) {
        sdl::mixer::Channel::all().play(&self.lock, 0).unwrap();
    }
    fn land(&mut self) {
        sdl::mixer::Channel::all().play(&self.land, 0).unwrap();
    }
}

impl ClientSounds {
    pub fn new() -> Result<ClientSounds, String> {
        Ok(
            ClientSounds {
                lock: sdl::rwops::RWops::from_bytes(LOCK)?.load_wav()?,
                land: sdl::rwops::RWops::from_bytes(LAND)?.load_wav()?,
                lineclear: sdl::rwops::RWops::from_bytes(LINECLEAR)?.load_wav()?,
                pieces1: sdl::rwops::RWops::from_bytes(PIECES1)?.load_wav()?,
                pieces2: sdl::rwops::RWops::from_bytes(PIECES2)?.load_wav()?,
                pieces3: sdl::rwops::RWops::from_bytes(PIECES3)?.load_wav()?,
                pieces4: sdl::rwops::RWops::from_bytes(PIECES4)?.load_wav()?,
                pieces5: sdl::rwops::RWops::from_bytes(PIECES5)?.load_wav()?,
                pieces6: sdl::rwops::RWops::from_bytes(PIECES6)?.load_wav()?,
                pieces7: sdl::rwops::RWops::from_bytes(PIECES7)?.load_wav()?,
            }
        )
    }
}
