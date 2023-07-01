use crate::game::being::Being;
use crate::game::randomizer::Weight;
use crate::game::special::{Special, SpecialGenerator};
use std::ops::Add;
use std::ops::Sub;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Wind8 {
    U,
    UR,
    R,
    DR,
    D,
    DL,
    L,
    UL,
    None,
}

impl TryFrom<u8> for Wind8 {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Wind8::U),
            1 => Ok(Wind8::UR),
            2 => Ok(Wind8::R),
            3 => Ok(Wind8::DR),
            4 => Ok(Wind8::D),
            5 => Ok(Wind8::DL),
            6 => Ok(Wind8::L),
            7 => Ok(Wind8::UL),
            8 => Ok(Wind8::None),
            _ => Err("Invalid value given to Wind8::try_from<u8>"),
        }
    }
}

impl TryFrom<TilePosition> for Wind8 {
    type Error = &'static str;

    fn try_from(value: TilePosition) -> Result<Self, Self::Error> {
        match value.y {
            -1 => match value.x {
                -1 => Ok(Wind8::UL),
                0 => Ok(Wind8::U),
                1 => Ok(Wind8::UR),
                _ => Err("cannot convert TilePosition to Wind8 for values of x outside of [-1, 1]"),
            },
            0 => match value.x {
                -1 => Ok(Wind8::L),
                0 => Ok(Wind8::None),
                1 => Ok(Wind8::R),
                _ => Err("cannot convert TilePosition to Wind8 for values of x outside of [-1, 1]"),
            },
            1 => match value.x {
                -1 => Ok(Wind8::DL),
                0 => Ok(Wind8::D),
                1 => Ok(Wind8::DR),
                _ => Err("cannot convert TilePosition to Wind8 for values of x outside of [-1, 1]"),
            },
            _ => Err("cannot convert TilePosition to Wind8 for values of x outside of [-1, 1]"),
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub struct TilePosition {
    pub y: isize,
    pub x: isize,
}

impl Add for TilePosition {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            y: self.y + other.y,
            x: self.x + other.x,
        }
    }
}

impl Sub for TilePosition {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            y: self.y - other.y,
            x: self.x - other.x,
        }
    }
}

impl TilePosition {
    pub fn new(y: isize, x: isize) -> Self {
        Self { y, x }
    }
}

impl From<Wind8> for TilePosition {
    fn from(ow: Wind8) -> Self {
        match ow {
            Wind8::U => Self::new(-1, 0),
            Wind8::UR => Self::new(-1, 1),
            Wind8::R => Self::new(0, 1),
            Wind8::DR => Self::new(1, 1),
            Wind8::D => Self::new(1, 0),
            Wind8::DL => Self::new(1, -1),
            Wind8::L => Self::new(0, -1),
            Wind8::UL => Self::new(-1, -1),
            Wind8::None => Self::new(0, 0),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TileType {
    Potion,
    Shield,
    Coin,
    Sword,
    Enemy,
    Special,
    COUNT,
    None,
}

impl TryFrom<usize> for TileType {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Potion),
            1 => Ok(Self::Shield),
            2 => Ok(Self::Coin),
            3 => Ok(Self::Sword),
            4 => Ok(Self::Enemy),
            5 => Ok(Self::Special),
            _ => Err("Invalid value for converting usize->TileType"),
        }
    }
}

impl TryFrom<TileType> for Weight {
    type Error = &'static str;

    fn try_from(value: TileType) -> Result<Self, Self::Error> {
        match value {
            TileType::Potion => Ok(100),
            TileType::Shield => Ok(100),
            TileType::Coin => Ok(100),
            TileType::Sword => Ok(80),
            TileType::Enemy => Ok(60),
            TileType::Special => Ok(0),
            _ => Err("Invalid value for converting TileType->Weight"),
        }
    }
}

impl TileType {
    pub fn connects_with(self, other: TileType) -> bool {
        if self == other {
            return true;
        }
        match self {
            Self::Sword | Self::Enemy | Self::Special => match other {
                Self::Sword | Self::Enemy | Self::Special => true,
                _ => false,
            },
            _ => false,
        }
    }
}

#[derive(Copy, Clone)]
pub enum TileInfo {
    Enemy(Being),
    Special(Special),
    None,
}

impl From<(TileType, &Being, &mut SpecialGenerator)> for TileInfo {
    fn from(value: (TileType, &Being, &mut SpecialGenerator)) -> Self {
        match value.0 {
            TileType::Potion | TileType::Shield | TileType::Coin | TileType::Sword => Self::None,
            TileType::Enemy => Self::Enemy(*value.1),
            TileType::Special => Self::Special(value.2.get()),
            TileType::None | TileType::COUNT => unreachable!(""),
        }
    }
}

impl TileInfo {
    pub fn output_damage(&self) -> usize {
        match self {
            Self::Enemy(ref b) => b.output_damage(1, 0),
            Self::Special(ref s) => s.output_damage(1, 0),
            _ => 0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Tile {
    pub tile_type: TileType,
    pub tile_info: TileInfo,
    pub next_selection: Wind8,
}

impl Default for Tile {
    fn default() -> Tile {
        Tile {
            tile_type: TileType::None,
            tile_info: TileInfo::None,
            next_selection: Wind8::None,
        }
    }
}

pub type Destroyed = bool;
impl Tile {
    pub fn new(tile_type: TileType, tile_info: TileInfo) -> Tile {
        Tile {
            tile_type,
            tile_info,
            next_selection: Wind8::None,
        }
    }

    pub fn slash(&mut self, damage: usize) -> Destroyed {
        match self.tile_info {
            TileInfo::Enemy(ref mut being) => being.take_damage(damage),
            TileInfo::Special(ref mut special) => special.take_damage(damage),
            _ => true,
        }
    }
}
