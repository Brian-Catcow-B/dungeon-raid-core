use crate::game::being::Being;
use crate::game::player::Player;
use crate::game::randomizer;
use crate::game::randomizer::{Weight, WeightedRandomizer, WeightedRandomizerType};
use crate::game::special::{SpecialGenerator, SpecialIdentifier};
use crate::game::stat_modifiers::BaseDamageDecrease;
use crate::game::tile::{Tile, TileInfo, TilePosition, TileType, Wind8};

use std::io::Write;
const _LOG_FILE: &str = "core_log.txt";
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
    w: usize,
    h: usize,
    // access by [y][x] where [0][0] is top left corner
    tiles: Vec<Vec<Tile>>,
    tile_randomizer: WeightedRandomizer,
    num_specials: usize,
    pub selection_start: Option<TilePosition>,
}

const MIN_DESTRUCTION_SELECTION: usize = 3;
const WR_EXP_ERR_STR: &str =
    "weighted_random should only return None if nothing has been added to the randomizer";
const TT_EXP_ERR_STR: &str = "TileType::TryFrom<usize> shouldn't fail because the usize is from a WeightedRandomizer with only the TileType's added";

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
        // DEBUG: always start with a boss
        tile_randomizer.set_weight(TileType::Special as usize, 1000);

        // create the board

        let mut b = Self {
            w,
            h,
            tiles: vec![],
            tile_randomizer,
            num_specials: 0,
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

    pub fn num_tiles(&self) -> usize {
        self.w * self.h
    }

    pub fn tile_at(&self, tp: &TilePosition) -> &Tile {
        &self.tiles[tp.y as usize][tp.x as usize]
    }

    pub fn mut_tile_at(&mut self, tp: &TilePosition) -> &mut Tile {
        &mut self.tiles[tp.y as usize][tp.x as usize]
    }

    pub fn specials(
        &self,
        omit_ids: &Vec<SpecialIdentifier>,
    ) -> Vec<(TilePosition, Tile, SpecialIdentifier)> {
        if self.num_specials <= omit_ids.len() {
            return vec![];
        }
        let mut specials_vec = Vec::with_capacity(self.num_specials);
        for (y, col) in self.tiles.iter().enumerate() {
            for (x, tile) in col.iter().enumerate() {
                if tile.tile_type == TileType::Special {
                    if let TileInfo::Special(special) = tile.tile_info {
                        if !omit_ids.contains(&special.id) {
                            specials_vec.push((
                                TilePosition::new(y as isize, x as isize),
                                *tile,
                                special.id,
                            ));
                        }
                        if specials_vec.len() + omit_ids.len() == self.num_specials {
                            return specials_vec;
                        }
                    } else {
                        unreachable!("tile.tile_type was TileType::Special, but if let TileInfo::Special(special) = tile.tile_info gave false");
                    }
                }
            }
        }
        unreachable!("self.num_specials and the number of specials found in the tiles differ");
    }

    pub fn select_tile(&mut self, position_to_select: &TilePosition) -> bool {
        match self.selection_start {
            Some(ref pos) => {
                let start_tile_type = self.tile_at(pos).tile_type;
                if !start_tile_type.connects_with(self.tile_at(position_to_select).tile_type) {
                    return false;
                }
                let mut p: TilePosition = *pos;
                let num_tiles = self.num_tiles();
                for _ in 0..num_tiles {
                    if p == *position_to_select {
                        self.remove_selection_starting_at(&p);
                        return true;
                    }
                    let relative_next = self.tile_at(&p).next_selection;
                    match relative_next {
                        Wind8::None => {
                            match Wind8::try_from(*position_to_select - p) {
                                Ok(w8) => match w8 {
                                    Wind8::None => return false,
                                    _ => {
                                        self.mut_tile_at(&p).next_selection = w8;
                                        return true;
                                    }
                                },
                                Err(_) => return false,
                            };
                        }
                        _ => {
                            p = p + TilePosition::from(relative_next);
                        }
                    };
                }
                unreachable!("selection loops");
            }
            None => {
                self.selection_start = Some(*position_to_select);
                true
            }
        }
    }

    pub fn drop_selection(
        &mut self,
        player: &Player,
        weapon_collection_multiplier: usize,
    ) -> (bool, Vec<Tile>) {
        let hit = self.selection_hits();
        let (num_weapons, num_beings) = if hit {
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
                let num_tiles = self.num_tiles();
                let mut found_the_end = false;
                for _ in 0..num_tiles {
                    let relative_next = self.tile_at(&p).next_selection;
                    if hit
                        && self
                            .mut_tile_at(&p)
                            .hit(player.output_damage(num_beings, num_weapons))
                    {
                        destructing_tiles.push(*self.tile_at(&p));
                        self.destroy_tile(&p);
                    }
                    self.mut_tile_at(&p).next_selection = Wind8::None;
                    match relative_next {
                        Wind8::None => {
                            found_the_end = true;
                            break;
                        }
                        _ => p = p + TilePosition::from(relative_next),
                    };
                }
                assert!(found_the_end);
            }
            None => {}
        }
        (hit, destructing_tiles)
    }

    pub fn get_tile(&self, tile_position: &TilePosition) -> Option<Tile> {
        if self.position_valid(tile_position) {
            Some(*self.tile_at(tile_position))
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
        for x in 0..self.w {
            let mut num_falling = 0;
            for y in (0..self.h).rev() {
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
                }
                let tile_info = TileInfo::from((tile_type, enemy, &mut *special_generator));
                self.meta_create_tile(tile_type);
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
        let special_div_set = if self.num_specials > 0 {
            (5, 6)
        } else {
            (2, 3)
        };
        let special_weight = std::cmp::min(
            (overall_turns_passed / special_div_set.0)
                - (most_recent_special_kill_turn / special_div_set.1),
            enemy_weight,
        );
        self.tile_randomizer
            .set_weight(TileType::Special as usize, special_weight);
    }

    // util

    fn position_valid(&self, tile_pos: &TilePosition) -> bool {
        tile_pos.y >= 0
            && tile_pos.x >= 0
            && (tile_pos.y as usize) < self.h
            && (tile_pos.x as usize) < self.w
    }

    fn position_in_selection(&self, tile_pos: &TilePosition) -> bool {
        match self.selection_start {
            Some(ref pos) => {
                let mut p = *pos;
                let num_tiles = self.num_tiles();
                for _ in 0..num_tiles {
                    if p == *tile_pos {
                        return true;
                    }
                    let n = self.tile_at(&p).next_selection;
                    match n {
                        Wind8::None => return false,
                        _ => p = p + TilePosition::from(n),
                    }
                }
                unreachable!("selection loops");
            }
            None => false,
        }
    }

    fn selection_hits(&self) -> bool {
        let mut num_tiles = 0;
        match self.selection_start {
            Some(pos) => {
                let mut p: TilePosition = pos;
                num_tiles += 1;
                let num_tiles_in_board = self.num_tiles();
                for _ in 0..num_tiles_in_board {
                    let relative_next = self.tile_at(&p).next_selection;
                    match relative_next {
                        Wind8::None => return false,
                        _ => p = p + TilePosition::from(relative_next),
                    };
                    num_tiles += 1;
                    if num_tiles >= MIN_DESTRUCTION_SELECTION {
                        return true;
                    }
                }
                unreachable!("selection loops");
            }
            None => false,
        }
    }

    fn remove_selection_starting_at(&mut self, pos: &TilePosition) {
        let num_tiles = self.num_tiles();
        let mut p = *pos;
        for _ in 0..num_tiles {
            let relative_next = self.tile_at(&p).next_selection;
            self.mut_tile_at(&p).next_selection = Wind8::None;
            match relative_next {
                Wind8::None => return,
                _ => p = p + TilePosition::from(relative_next),
            };
        }
        unreachable!("selection loops");
    }

    fn enforce_selection_valid(&mut self) {
        // connectedness
        for y in 0..self.h {
            for x in 0..self.w {
                let p = TilePosition::new(y as isize, x as isize);
                if self.tile_at(&p).next_selection != Wind8::None && !self.position_in_selection(&p)
                {
                    self.mut_tile_at(&p).next_selection = Wind8::None;
                }
            }
        }
        // type alignment
        let num_tiles = self.num_tiles();
        let mut p = match self.selection_start {
            Some(starting_pos) => starting_pos,
            None => return,
        };
        let t = self.tile_at(&p).tile_type;
        for _ in 0..num_tiles {
            let w8 = self.tile_at(&p).next_selection;
            match w8 {
                Wind8::None => return,
                _ => {
                    let n = p + TilePosition::from(w8);
                    if !self.tile_at(&n).tile_type.connects_with(t) {
                        self.remove_selection_starting_at(&p);
                        return;
                    }
                    p = n;
                }
            }
        }
    }

    fn num_weapons_and_beings_in_selection(&self) -> (usize, usize) {
        let mut num_weapons: usize = 0;
        let mut num_beings: usize = 0;
        match self.selection_start {
            Some(ref pos) => {
                if !self.tile_at(pos).tile_type.connects_with(TileType::Sword) {
                    return (0, 0);
                }
                let mut p = *pos;
                let num_tiles = self.num_tiles();
                let mut found_the_end = false;
                for _ in 0..num_tiles {
                    match self.tile_at(&p).tile_type {
                        TileType::Sword => num_weapons += 1,
                        TileType::Enemy | TileType::Special => num_beings += 1,
                        _ => {}
                    };
                    let relative_next = self.tile_at(&p).next_selection;
                    match relative_next {
                        Wind8::None => {
                            found_the_end = true;
                            break;
                        }
                        _ => {
                            p = p + TilePosition::from(relative_next);
                        }
                    };
                }
                assert!(found_the_end);
            }
            None => {}
        };
        (num_weapons, num_beings)
    }

    fn meta_destroy_tile(&mut self, tile_pos: &TilePosition) {
        if self.tile_at(tile_pos).tile_type == TileType::Special {
            self.num_specials -= 1;
        }
    }

    fn destroy_tile(&mut self, tile_pos: &TilePosition) {
        self.meta_destroy_tile(tile_pos);
        *self.mut_tile_at(tile_pos) = Tile::default();
    }

    fn meta_create_tile(&mut self, tile_type: TileType) {
        if tile_type == TileType::Special {
            self.num_specials += 1;
        }
    }

    fn serialize_tile_position(&self, tile_pos: &TilePosition) -> usize {
        tile_pos.y as usize + tile_pos.x as usize * self.h
    }

    fn deserialize_tile_position(&self, s_tile_pos: usize) -> TilePosition {
        TilePosition::new(
            (s_tile_pos % self.h) as isize,
            (s_tile_pos / self.h) as isize,
        )
    }

    // special end of turn

    pub fn random_tile_of_type(&self, tile_type: TileType) -> Option<TilePosition> {
        let mut randomizer = WeightedRandomizer::default();
        for y in 0..self.h {
            for x in 0..self.w {
                if self.tiles[y][x].tile_type == tile_type {
                    randomizer.set_weight(
                        self.serialize_tile_position(&TilePosition::new(y as isize, x as isize)),
                        1,
                    );
                }
            }
        }
        match randomizer.weighted_random() {
            Some(s_tile_pos) => Some(self.deserialize_tile_position(s_tile_pos)),
            None => None,
        }
    }

    pub fn replace_tile(
        &mut self,
        tile_pos: &TilePosition,
        replace_type: TileType,
        enemy: &Being,
        special_generator: &mut SpecialGenerator,
    ) {
        let tile_info = TileInfo::from((replace_type, enemy, &mut *special_generator));
        self.meta_destroy_tile(tile_pos);
        self.meta_create_tile(replace_type);
        *self.mut_tile_at(tile_pos) = Tile::new(replace_type, tile_info);
        self.enforce_selection_valid();
    }

    pub fn swap_positions(&mut self, tp1: &TilePosition, tp2: &TilePosition) {
        let tmp = *self.tile_at(tp2);
        *self.mut_tile_at(tp2) = *self.tile_at(tp1);
        *self.mut_tile_at(tp1) = tmp;
        self.enforce_selection_valid();
    }

    pub fn swap_position_with_random_other(&mut self, tp: &TilePosition) {
        let serialized_tp = self.serialize_tile_position(tp);
        // get value in [0, num_tiles - 2]
        let mut serialized_random_other_tp =
            randomizer::evenly_distributed_random(self.num_tiles() - 2);
        // map serialized_tp to self.num_tiles() - 1
        if serialized_random_other_tp == serialized_tp {
            serialized_random_other_tp = self.num_tiles() - 1;
        }
        let random_other_tp = self.deserialize_tile_position(serialized_random_other_tp);
        self.swap_positions(tp, &random_other_tp);
        self.enforce_selection_valid();
    }

    pub fn num_surrounding_tiles_of_type(
        &self,
        tile_position: &TilePosition,
        tile_type: TileType,
    ) -> usize {
        let mut num_surrounding = 0;
        for w8_num in 0..8 {
            let w8 = Wind8::try_from(w8_num as u8).expect("");
            let p = *tile_position + TilePosition::from(w8);
            if self.position_valid(&p) && self.tile_at(&p).tile_type == tile_type {
                num_surrounding += 1;
            }
        }
        num_surrounding
    }

    pub fn destroy_3x3_centered_at(
        &mut self,
        center_pos: &TilePosition,
        enemy: &Being,
        special_generator: &mut SpecialGenerator,
    ) -> Vec<Tile> {
        let mut destroyed_tiles = Vec::with_capacity(3 * 3);
        for w8_num in 0..8 {
            let w8 = Wind8::try_from(w8_num as u8).expect("");
            let p = *center_pos + TilePosition::from(w8);
            if self.position_valid(&p) {
                destroyed_tiles.push(*self.tile_at(&p));
                self.destroy_tile(&p);
            }
        }
        destroyed_tiles.push(*self.tile_at(center_pos));
        self.destroy_tile(center_pos);
        self.apply_gravity_and_randomize_new_tiles(enemy, special_generator);
        self.enforce_selection_valid();
        destroyed_tiles
    }

    // ability functions

    pub fn replace_tiles(
        &mut self,
        from: TileType,
        to: TileType,
        enemy: &Being,
        special_generator: &mut SpecialGenerator,
    ) {
        for y in 0..self.h {
            for x in 0..self.w {
                let p = TilePosition::new(y as isize, x as isize);
                if self.tile_at(&p).tile_type == from {
                    let tile_info = TileInfo::from((to, enemy, &mut *special_generator));
                    self.meta_destroy_tile(&p);
                    self.meta_create_tile(to);
                    *self.mut_tile_at(&p) = Tile::new(to, tile_info);
                }
            }
        }
        self.enforce_selection_valid();
    }

    pub fn scramble(&mut self) {
        // oh boy here we go
        self.selection_start = None;
        let mut randomizer = WeightedRandomizer::new(WeightedRandomizerType::MetaSubAllOnObtain);
        let num_tiles = self.num_tiles();
        for val in 0..num_tiles {
            randomizer.set_weight(val, 1);
        }
        let first_idx_2d = randomizer.weighted_random().expect("");
        let first_pos: TilePosition = self.deserialize_tile_position(first_idx_2d);
        let mut first = *self.tile_at(&first_pos);
        first.next_selection = Wind8::None;
        let mut target_pos = first_pos;
        for _ in 0..num_tiles {
            let value_opt = randomizer.weighted_random();
            match value_opt {
                Some(value) => {
                    let rand_tile_pos = self.deserialize_tile_position(value);
                    *self.mut_tile_at(&target_pos) = *self.tile_at(&rand_tile_pos);
                    self.mut_tile_at(&target_pos).next_selection = Wind8::None;
                    target_pos = rand_tile_pos;
                }
                None => {
                    *self.mut_tile_at(&target_pos) = first;
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
