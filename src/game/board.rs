use crate::game::randomizer::{Weight, WeightedRandomizer};
use crate::game::tile::{Tile, TilePosition, TileType, TileInfo, Wind8};
use crate::game::player::Player;
use crate::game::being::{BeingType, Being};

pub struct Board {
    // access by [y][x] where [0][0] is top left corner
    tiles: Vec<Vec<Tile>>,
    tile_randomizer: WeightedRandomizer,
    pub selection_start: Option<TilePosition>,
}

const MIN_DESTRUCTION_SELECTION: usize = 3;
const WR_EXP_ERR_STR: &'static str =
    "weighted_random should only return None if nothing has been added to the randomizer";
const TT_EXP_ERR_STR: &'static str = "TileType::TryFrom<usize> shouldn't fail because the usize is from a WeightedRandomizer with only the TileType's added";
const TI_EXP_ERR_STR: &'static str = "TileInfo::TryFrom<(TileType, &Being, &Being)> shouldn't fail in this situation";

impl Board {
    pub fn new(w: usize, h: usize, enemy: &Being, boss: &Being) -> Board {
        // tile randomizer

        let mut tile_randomizer = WeightedRandomizer::default();
        for tt in 0..(TileType::COUNT as usize) {
            let tile_type =
                TileType::try_from(tt).expect("TileType::try_from errored where it never should");
            let default_weight = Weight::try_from(tile_type)
                .expect("Weight::try_from errored where it never should");
            tile_randomizer.set_weight(tt, default_weight);
        }

        // tiles

        let mut tiles = vec![];
        for _ in 0..w {
            let new_idx = tiles.len();
            tiles.push(vec![]);
            for _ in 0..h {
                let weighted_random_value =
                    tile_randomizer.weighted_random().expect(WR_EXP_ERR_STR);
                let tile_type = TileType::try_from(weighted_random_value).expect(TT_EXP_ERR_STR);
                let tile_info = TileInfo::try_from((tile_type, enemy, boss)).expect(TI_EXP_ERR_STR);
                tiles[new_idx].push(Tile::new(tile_type, tile_info));
            }
        }

        Board {
            tiles,
            tile_randomizer,
            selection_start: None,
        }
    }

    fn position_valid(&self, pos: TilePosition) -> bool {
        self.tiles.len() > 0
            && (pos.y as usize) < self.tiles.len()
            && (pos.x as usize) < self.tiles[0].len()
    }

    fn remove_selection_starting_at(&mut self, mut pos: TilePosition) {
        loop {
            if self.position_valid(pos) {
                let relative_next = self.tiles[pos.y as usize][pos.x as usize].next_selection;
                self.tiles[pos.y as usize][pos.x as usize].next_selection = Wind8::None;
                match relative_next {
                    Wind8::None => return,
                    _ => pos = pos + TilePosition::from(relative_next),
                };
            } else {
                unreachable!("remove_selection_starting_at found that either the given position or one of the positions down the selection trail points off the board");
            }
        }
    }

    pub fn select_tile(&mut self, position_to_select: TilePosition) -> bool {
        if !self.position_valid(position_to_select) {
            unreachable!("select_tile called with a TilePosition parameter that is invalid");
        }
        match self.selection_start {
            Some(pos) => {
                let start_tile_type;
                if self.position_valid(pos) {
                    start_tile_type = self.tiles[pos.y as usize][pos.x as usize].tile_type;
                } else {
                    unreachable!(
                        "in select tile, the selection_start is Some(pos) where pos is invalid"
                    );
                }
                if !start_tile_type.connects_with(
                    self.tiles[position_to_select.y as usize][position_to_select.x as usize]
                        .tile_type,
                ) {
                    return false;
                }
                let mut p: TilePosition = pos;
                loop {
                    if p == position_to_select {
                        self.remove_selection_starting_at(p);
                        return true;
                    }
                    // TODO: make this limited to the number of tiles in case there is some sort of invalid board
                    let relative_next = self.tiles[p.y as usize][p.x as usize].next_selection;
                    match relative_next {
                        Wind8::None => {
                            match Wind8::try_from(position_to_select - p) {
                                Ok(w8) => match w8 {
                                    Wind8::None => return false,
                                    _ => {
                                        self.tiles[p.y as usize][p.x as usize].next_selection = w8;
                                        return true;
                                    }
                                },
                                Err(_) => return false,
                            };
                        }
                        _ => {
                            p = p + TilePosition::from(relative_next);
                            if !self.position_valid(p) {
                                unreachable!("in select_tile, one of the tiles in the selection trail points off the board; position: (x, y) {} {}", p.x, p.y);
                            }
                        }
                    };
                }
            }
            None => {
                self.selection_start = Some(position_to_select);
                return true;
            }
        }
    }

    fn selection_slashes(&self) -> bool {
        let mut num_tiles = 0;
        match self.selection_start {
            Some(pos) => {
                let mut p: TilePosition = pos;
                num_tiles += 1;
                loop {
                    let relative_next = self.tiles[p.y as usize][p.x as usize].next_selection;

                    match relative_next {
                        Wind8::None => return false,
                        _ => p = p + TilePosition::from(relative_next),
                    };
                    num_tiles += 1;
                    if num_tiles >= MIN_DESTRUCTION_SELECTION {
                        return true;
                    }
                }
            }
            None => return false,
        }
    }

    fn num_weapons_and_beings_in_selection(&self) -> (isize, isize) {
        let mut num_weapons: isize = 0;
        let mut num_beings: isize = 0;
        match self.selection_start {
            Some(pos) => {
                if !self.position_valid(pos) {
                    unreachable!("in num_weapons_in_selection, selection_start is Some(pos) but pos is not a valid position");
                }
                if !self.tiles[pos.y as usize][pos.x as usize]
                    .tile_type
                    .connects_with(TileType::Sword)
                {
                    return (0, 0);
                }
                let mut p = pos;
                loop {
                    match self.tiles[p.y as usize][pos.x as usize].tile_type {
                        TileType::Sword => num_weapons += 1,
                        TileType::Enemy | TileType::Boss => num_beings += 1,
                        _ => {},
                    };
                    let relative_next = self.tiles[p.y as usize][p.x as usize].next_selection;
                    match relative_next {
                        Wind8::None => break,
                        _ => {
                            p = p + TilePosition::from(relative_next);
                            assert!(self.position_valid(p));
                        }
                    };
                }
            }
            None => {}
        };
        (num_weapons, num_beings)
    }

    pub fn drop_selection(&mut self, player: &Player) -> Vec<Tile> {
        let slash = self.selection_slashes();
        let (num_weapons, num_beings) = if slash {
            self.num_weapons_and_beings_in_selection()
        } else {
            (0, 0)
        };
        let mut destructing_tiles: Vec<Tile> = vec![];
        match self.selection_start {
            Some(pos) => {
                self.selection_start = None;
                let mut p = pos;
                loop {
                    // TODO: limit this loop
                    assert!(self.position_valid(p));
                    let relative_next = self.tiles[p.y as usize][p.x as usize].next_selection;
                    if slash {
                        if self.tiles[p.y as usize][p.x as usize].slash(player.output_damage(num_beings, num_weapons)) {
                            destructing_tiles.push(self.tiles[p.y as usize][p.x as usize]);
                            self.tiles[p.y as usize][p.x as usize] = Tile::default();
                        }
                    } else {
                        self.tiles[p.y as usize][p.x as usize].next_selection = Wind8::None;
                    }
                    match relative_next {
                        Wind8::None => break,
                        _ => p = p + TilePosition::from(relative_next),
                    };
                }
            }
            None => {}
        }
        destructing_tiles
    }

    pub fn get_tile(&self, tile_position: TilePosition) -> Option<Tile> {
        if self.position_valid(tile_position) {
            Some(self.tiles[tile_position.y as usize][tile_position.x as usize])
        } else {
            None
        }
    }

    pub fn apply_gravity_and_randomize_new_tiles(&mut self, enemy: &Being, boss: &Being) {
        let h = self.tiles.len();
        if h == 0 || self.tiles[0].len() == 0 {
            return;
        }
        let w = self.tiles[0].len();
        for x in 0..w {
            let mut num_falling = 0;
            for y in (0..h).rev() {
                match self.tiles[y][x].tile_type {
                    TileType::None => num_falling += 1,
                    _ => {
                        if num_falling > 0 {
                            self.tiles[y + num_falling][x].tile_type = self.tiles[y][x].tile_type;
                            self.tiles[y][x].tile_type = TileType::None;
                        }
                    }
                };
            }
            for i in 0..num_falling {
                let tile_type = 
                TileType::try_from(
                    self.tile_randomizer
                        .weighted_random()
                        .expect(WR_EXP_ERR_STR),
                )
                .expect(TT_EXP_ERR_STR);
                let tile_info = TileInfo::try_from((tile_type, enemy, boss)).expect(TI_EXP_ERR_STR);
                self.tiles[num_falling - i - 1][x] = Tile::new(tile_type, tile_info);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_tile() {
        let mut b = Board::new(6, 6);
        b.select_tile(TilePosition::new(0, 0));
        b.select_tile(TilePosition::new(1, 0));
        b.select_tile(TilePosition::new(0, 1));
        b.select_tile(TilePosition::new(0, 0));
        assert!(
            b.get_tile(TilePosition::new(0, 0))
                .expect("how is (0, 0) not a tile?")
                .next_selection
                == Wind8::None
        );
    }
}
