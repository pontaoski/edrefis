// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use macroquad::{audio::{load_sound_from_bytes, play_sound_once, Sound}, Error};

use logic::{hooks::Sounds, well::Block};

pub struct ClientSounds {
    lock: Sound,
    land: Sound,
    lineclear: Sound,
    piece1: Sound,
    piece2: Sound,
    piece3: Sound,
    piece4: Sound,
    piece5: Sound,
    piece6: Sound,
    piece7: Sound,
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
        play_sound_once(&self.lineclear);
    }
    fn lock(&mut self) {
        play_sound_once(&self.lock);
    }
    fn land(&mut self) {
        play_sound_once(&self.land);
    }
    fn block_spawn(&mut self, block: Block) {
        match block {
        Block::Yellow => play_sound_once(&self.piece1),
        Block::Blue => play_sound_once(&self.piece2),
        Block::Orange => play_sound_once(&self.piece3),
        Block::Green => play_sound_once(&self.piece4),
        Block::Purple => play_sound_once(&self.piece5),
        Block::Cyan => play_sound_once(&self.piece6),
        Block::Red => play_sound_once(&self.piece7),
        }
    }
}
impl ClientSounds {
    pub async fn new() -> Result<ClientSounds, Error> {
        Ok(
            ClientSounds {
                lock: load_sound_from_bytes(LOCK).await?,
                land: load_sound_from_bytes(LAND).await?,
                lineclear: load_sound_from_bytes(LINECLEAR).await?,
                piece1: load_sound_from_bytes(PIECES1).await?,
                piece2: load_sound_from_bytes(PIECES2).await?,
                piece3: load_sound_from_bytes(PIECES3).await?,
                piece4: load_sound_from_bytes(PIECES4).await?,
                piece5: load_sound_from_bytes(PIECES5).await?,
                piece6: load_sound_from_bytes(PIECES6).await?,
                piece7: load_sound_from_bytes(PIECES7).await?,
            }
        )
    }
}
