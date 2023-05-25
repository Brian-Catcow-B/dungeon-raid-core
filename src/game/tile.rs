use std::ops::Add;

enum Wind8 {
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

pub struct TilePosition {
    y: isize,
    x: isize,
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

enum TileType {
    None,
    Heart,
    Shield,
    Coin,
    Enemy,
    Boss,
}

pub struct Tile {
    tile_type: TileType,
    connected_to_tile: Wind8,
}

impl Default for Tile {
    fn default() -> Tile {
        Tile {
            tile_type: TileType::None,
            connected_to_tile: Wind8::None,
        }
    }
}

impl Tile {
    fn new(tile_type: TileType) -> Tile {
        Tile {
            tile_type,
            connected_to_tile: Wind8::None,
        }
    }
}
