use crate::game::being::Being;
use crate::game::player::Player;
use crate::game::randomizer::{Weight, WeightedRandomizer, WeightedRandomizerType};
use crate::game::special::SpecialGenerator;
use crate::game::stat_modifiers::BaseDamageDecrease;
use crate::game::tile::{Tile, TileInfo, TilePosition, TileType, Wind8};

use std::io::Write;
const _LOG_FILE: &'static str = "core_log.txt";
fn _clear_logs() {
    let mut file = std::fs::File::create(_LOG_FILE).expect("failed to create file");
    write!(&mut file, "").expect("failed to write file");
}
fn _log(msg: &String) {
    let mut file = std::fs::File::options()
        .append(true)
        .create(true)
        .open(_LOG_FILE)
        .expect("failed to create file");
    writeln!(&mut file, "{}", msg).expect("failed to write file");
}

pub struct Board {
    // access by [y][x] where [0][0] is top left corner
    tiles: Vec<Vec<Tile>>,
    tile_randomizer: WeightedRandomizer,
    special_exists: bool,
    pub selection_start: Option<TilePosition>,
}

const MIN_DESTRUCTION_SELECTION: usize = 3;
const WR_EXP_ERR_STR: &'static str =
    "weighted_random should only return None if nothing has been added to the randomizer";
const TT_EXP_ERR_STR: &'static str = "TileType::TryFrom<usize> shouldn't fail because the usize is from a WeightedRandomizer with only the TileType's added";
const TI_EXP_ERR_STR: &'static str =
    "TileInfo::TryFrom<(TileType, &Being, &Being)> shouldn't fail in this situation";

impl Board {
    pub fn new(
        w: usize,
        h: usize,
        enemy: &Being,
        special_generator: &mut SpecialGenerator,
    ) -> Board {
        _clear_logs();

        // tile randomizer

        let mut tile_randomizer = WeightedRandomizer::default();
        for tt in 0..(TileType::COUNT as usize) {
            let tile_type =
                TileType::try_from(tt).expect("TileType::try_from errored where it never should");
            let default_weight = Weight::try_from(tile_type)
                .expect("Weight::try_from errored where it never should");
            tile_randomizer.set_weight(tt, default_weight);
        }

        // create the board

        let mut b = Self {
            tiles: vec![],
            tile_randomizer,
            special_exists: false,
            selection_start: None,
        };

        // tiles

        for _ in 0..w {
            let new_idx = b.tiles.len();
            b.tiles.push(vec![]);
            for _ in 0..h {
                b.tiles[new_idx].push(Tile::default());
            }
        }
        b.apply_gravity_and_randomize_new_tiles(enemy, special_generator);

        b
    }

    pub fn incoming_damage(&self) -> usize {
        let mut dmg = 0;
        for col in self.tiles.iter() {
            for tile in col.iter() {
                dmg += tile.tile_info.output_damage();
            }
        }
        dmg
    }

    pub fn special(&self) -> Option<Tile> {
        if !self.special_exists {
            return None;
        }
        for col in self.tiles.iter() {
            for tile in col.iter() {
                if tile.tile_type == TileType::Special {
                    return Some(*tile);
                }
            }
        }
        unreachable!("self.special_exists seems to indicate that the special does indeed exist, but it was not found");
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

    fn num_weapons_and_beings_in_selection(&self) -> (usize, usize) {
        let mut num_weapons: usize = 0;
        let mut num_beings: usize = 0;
        match self.selection_start {
            Some(pos) => {
                assert!(self.position_valid(pos));
                if !self.tiles[pos.y as usize][pos.x as usize]
                    .tile_type
                    .connects_with(TileType::Sword)
                {
                    return (0, 0);
                }
                let mut p = pos;
                loop {
                    match self.tiles[p.y as usize][p.x as usize].tile_type {
                        TileType::Sword => num_weapons += 1,
                        TileType::Enemy | TileType::Special => num_beings += 1,
                        _ => {}
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

    pub fn drop_selection(
        &mut self,
        player: &Player,
        weapon_collection_multiplier: usize,
    ) -> (bool, Vec<Tile>) {
        let slash = self.selection_slashes();
        let (num_weapons, num_beings) = if slash {
            let (nw, nb) = self.num_weapons_and_beings_in_selection();
            (nw * weapon_collection_multiplier, nb)
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
                    if slash
                        && self.tiles[p.y as usize][p.x as usize]
                            .slash(player.output_damage(num_beings, num_weapons))
                    {
                        destructing_tiles.push(self.tiles[p.y as usize][p.x as usize]);
                        if self.tiles[p.y as usize][p.x as usize].tile_type == TileType::Special {
                            self.special_exists = false;
                        }
                        self.tiles[p.y as usize][p.x as usize] = Tile::default();
                    }
                    self.tiles[p.y as usize][p.x as usize].next_selection = Wind8::None;
                    match relative_next {
                        Wind8::None => break,
                        _ => p = p + TilePosition::from(relative_next),
                    };
                }
            }
            None => {}
        }
        (slash, destructing_tiles)
    }

    pub fn get_tile(&self, tile_position: TilePosition) -> Option<Tile> {
        if self.position_valid(tile_position) {
            Some(self.tiles[tile_position.y as usize][tile_position.x as usize])
        } else {
            None
        }
    }

    pub fn apply_blunting(&mut self, blunting: BaseDamageDecrease) {
        for col in self.tiles.iter_mut() {
            for tile in col.iter_mut() {
                match tile.tile_info {
                    TileInfo::Enemy(ref mut b) => b.blunt(blunting),
                    TileInfo::Special(ref mut s) => s.blunt(blunting),
                    _ => {}
                }
            }
        }
    }

    pub fn apply_gravity_and_randomize_new_tiles(
        &mut self,
        enemy: &Being,
        special_generator: &mut SpecialGenerator,
    ) {
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
                            self.tiles[y + num_falling][x] = self.tiles[y][x];
                            self.tiles[y][x] = Tile::default();
                        }
                    }
                };
            }
            for i in 0..num_falling {
                let y = num_falling - i - 1;
                let tile_type = TileType::try_from(
                    self.tile_randomizer
                        .weighted_random()
                        .expect(WR_EXP_ERR_STR),
                )
                .expect(TT_EXP_ERR_STR);
                // if we got TileType::Special, set the weight to zero, which then gets handled
                // later by Self::activate_special_spawns
                if tile_type == TileType::Special {
                    self.tile_randomizer
                        .set_weight(TileType::Special as usize, 0);
                    self.special_exists = true;
                }
                let tile_info = TileInfo::try_from((tile_type, enemy, &mut *special_generator))
                    .expect(TI_EXP_ERR_STR);
                self.tiles[y][x] = Tile::new(tile_type, tile_info);
            }
        }
    }

    // overall_turns_passed is expected to be >= most_recent_special_kill_turn
    pub fn activate_special_spawns(
        &mut self,
        overall_turns_passed: usize,
        most_recent_special_kill_turn: usize,
    ) {
        // The idea is that we scale the special's weight with the number of turns that have passed
        // along with the the number of turns that have passed since the most recent special kill.
        // Also, if a special enemy already exists, we drop the weight considerably
        let enemy_weight = Weight::try_from(TileType::Enemy).expect("");
        let special_div_set = if self.special_exists { (5, 6) } else { (2, 3) };
        let special_weight = std::cmp::min(
            (overall_turns_passed / special_div_set.0)
                - (most_recent_special_kill_turn / special_div_set.1),
            enemy_weight,
        );
        self.tile_randomizer
            .set_weight(TileType::Special as usize, special_weight);
    }

    // ability functions

    pub fn replace_tiles(
        &mut self,
        from: TileType,
        to: TileType,
        enemy: &Being,
        special_generator: &mut SpecialGenerator,
    ) {
        for col in self.tiles.iter_mut() {
            for tile in col.iter_mut() {
                if tile.tile_type == from {
                    let tile_info = TileInfo::try_from((to, enemy, &mut *special_generator))
                        .expect(TI_EXP_ERR_STR);
                    *tile = Tile::new(to, tile_info);
                }
            }
        }
    }

    pub fn scramble(&mut self) {
        // oh boy here we go
        self.selection_start = None;
        let mut randomizer = WeightedRandomizer::new(WeightedRandomizerType::MetaSubAllOnObtain);
        let h = self.tiles.len();
        if h == 0 || self.tiles[0].len() == 0 {
            return;
        }
        let w = self.tiles[0].len();
        // convention: for given val, w, h; y = val % h and x = val / w
        for val in 0..(w * h) {
            randomizer.set_weight(val, 1);
        }
        let first_idx_2d = randomizer.weighted_random().expect("");
        let first_pos = TilePosition::new((first_idx_2d % h) as isize, (first_idx_2d / w) as isize);
        let mut first = self.tiles[first_pos.y as usize][first_pos.x as usize];
        first.next_selection = Wind8::None;
        let mut target_pos = first_pos;
        for _ in 0..(w * h) {
            let value_opt = randomizer.weighted_random();
            match value_opt {
                Some(value) => {
                    let rand_tile_pos =
                        TilePosition::new((value % h) as isize, (value / w) as isize);
                    self.tiles[target_pos.y as usize][target_pos.x as usize] =
                        self.tiles[rand_tile_pos.y as usize][rand_tile_pos.x as usize];
                    self.tiles[target_pos.y as usize][target_pos.x as usize].next_selection =
                        Wind8::None;
                    target_pos = rand_tile_pos;
                }
                None => {
                    self.tiles[target_pos.y as usize][target_pos.x as usize] = first;
                    return;
                }
            };
        }
        unreachable!("while scrambling the board, the for loop ran too many iterations based on the logic set in place");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::being::{BeingType, ENEMY_START_DMG};
    use crate::game::{DEFAULT_BOARD_HEIGHT, DEFAULT_BOARD_WIDTH};

    fn testhelp_custom_random_board(
        w: usize,
        h: usize,
        enemy: &Being,
        boss: &Being,
        randomizer_tiles: &Vec<TileType>,
    ) -> Board {
        let mut r = WeightedRandomizer::default();
        for tile_type in randomizer_tiles.iter() {
            r.add_to_weight(*tile_type as usize, 1);
        }

        // create the board

        let mut b = Board {
            tiles: vec![],
            tile_randomizer: r,
            selection_start: None,
        };

        // tiles

        for _ in 0..w {
            let new_idx = b.tiles.len();
            b.tiles.push(vec![]);
            for _ in 0..h {
                b.tiles[new_idx].push(Tile::default());
            }
        }
        b.apply_gravity_and_randomize_new_tiles(enemy, boss);

        b
    }

    #[test]
    fn test_incoming_damage() {
        let enemy = Being::new(BeingType::Enemy);
        let boss = Being::new(BeingType::Special);
        let mut b = testhelp_custom_random_board(
            DEFAULT_BOARD_WIDTH,
            DEFAULT_BOARD_HEIGHT,
            &enemy,
            &boss,
            &vec![TileType::Enemy],
        );

        assert_eq!(
            b.incoming_damage(),
            ENEMY_START_DMG * DEFAULT_BOARD_WIDTH as isize * DEFAULT_BOARD_HEIGHT as isize
        );

        let mut tp;
        for _ in 0..1000 {
            tp = TilePosition::new(0, 0);
            for _ in 0..3 {
                b.select_tile(tp);
                tp = tp + TilePosition::try_from(Wind8::R).expect("");
            }
            assert_eq!(
                b.incoming_damage(),
                ENEMY_START_DMG * DEFAULT_BOARD_WIDTH as isize * DEFAULT_BOARD_HEIGHT as isize
            );
        }
    }
}
