use crate::game::being::{Being, BeingType};
use crate::game::randomizer::WeightedRandomizer;
use crate::game::tile::{TileInfo, TilePosition, TileType};
use crate::game::Game;

pub type SpecialIdentifier = usize;
pub type ModifiesBoard = bool;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SpecialType {
    Boss,
    Unstable,
    Precise,
    Undead,
    Resourceful,
    COUNT,
}

impl TryFrom<usize> for SpecialType {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Boss),
            1 => Ok(Self::Unstable),
            2 => Ok(Self::Precise),
            3 => Ok(Self::Undead),
            4 => Ok(Self::Resourceful),
            _ => Err("invalid value given to SpecialType::TryFrom<usize>"),
        }
    }
}

impl From<SpecialType> for Being {
    fn from(value: SpecialType) -> Being {
        let num_den = match value {
            SpecialType::Boss => (2, 1),
            SpecialType::Unstable => (4, 3),
            SpecialType::Precise => (1, 1),
            SpecialType::Undead => (4, 3),
            SpecialType::Resourceful => (1, 1),
            SpecialType::COUNT => unreachable!(""),
        };
        let mut being = Being::new(BeingType::Special, num_den.0, num_den.1);
        if value == SpecialType::Resourceful {
            being.max_shields = 8;
        }
        being
    }
}

type Reanimated = bool;
#[derive(Copy, Clone)]
pub enum SpecialInfo {
    Boss,
    Unstable,
    Precise,
    Undead(Reanimated),
    Resourceful,
}

impl From<SpecialType> for SpecialInfo {
    fn from(value: SpecialType) -> SpecialInfo {
        match value {
            SpecialType::Boss => Self::Boss,
            SpecialType::Unstable => Self::Unstable,
            SpecialType::Precise => Self::Precise,
            SpecialType::Undead => Self::Undead(false),
            SpecialType::Resourceful => Self::Resourceful,
            SpecialType::COUNT => unreachable!(""),
        }
    }
}

impl SpecialType {
    pub fn name_description(self) -> (&'static str, &'static str) {
        match self {
            Self::Boss => ("Boss", "A much stronger enemy"),
            Self::Unstable => ("Unstable", "Teleports to a random tile every turn"),
            Self::Precise => ("Precise", "Attacks cannot be blunted"),
            Self::Undead => (
                "Undead",
                "When killed the first time, reanimates with half HP",
            ),
			Self::Resourceful => ("Resourceful", "At the end of each turn, defense gets set to the number of surrounding shields, attack is premanently increased by the number of surrounding swords, and health is increased by the number of surrounding health potions"),
            Self::COUNT => unreachable!(""),
        }
    }
}

pub struct SpecialGenerator {
    unused_id: SpecialIdentifier,
    type_randomizer: WeightedRandomizer,
}

impl Default for SpecialGenerator {
    fn default() -> Self {
        let mut type_randomizer = WeightedRandomizer::default();
        for st in 0..(SpecialType::COUNT as usize) {
            type_randomizer.set_weight(st, 1);
        }
        Self {
            unused_id: 0,
            type_randomizer,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Special {
    pub id: SpecialIdentifier,
    pub special_type: SpecialType,
    pub special_info: SpecialInfo,
    pub being: Being,
}

impl Special {
    pub fn take_damage(&mut self, damage: usize) -> bool {
        let killed = self.being.take_damage(damage);
        match self.special_info {
            SpecialInfo::Undead(ref mut reanimated) => {
                if *reanimated {
                    killed
                } else {
                    *reanimated = true;
                    self.being.hit_points = self.being.max_hit_points / 2;
                    false
                }
            }
            _ => killed,
        }
    }

    pub fn output_damage(&self, num_enemies: usize, num_weapons: usize) -> usize {
        self.being.output_damage(num_enemies, num_weapons)
    }

    pub fn blunt(&mut self, blunting: usize) {
        match self.special_type {
            SpecialType::Precise => {}
            _ => self.being.blunt(blunting),
        }
    }

    pub fn end_of_turn(game: &mut Game, tile_position: &TilePosition) -> ModifiesBoard {
        let num_surrounding_shields = game
            .board
            .num_surrounding_tiles_of_type(tile_position, TileType::Shield);
        let num_surrounding_swords = game
            .board
            .num_surrounding_tiles_of_type(tile_position, TileType::Sword);
        let num_surrounding_potions = game
            .board
            .num_surrounding_tiles_of_type(tile_position, TileType::Potion);
        if let TileInfo::Special(ref mut special) = game.board.mut_tile_at(tile_position).tile_info
        {
            match special.special_info {
                SpecialInfo::Boss => false,
                SpecialInfo::Unstable => {
                    game.board.swap_position_with_random_other(tile_position);
                    true
                }
                SpecialInfo::Precise => false,
                SpecialInfo::Undead(_) => false,
                SpecialInfo::Resourceful => {
                    special.being.shields = num_surrounding_shields;
                    special.being.base_output_damage += num_surrounding_swords;
                    special.being.add_hit_points(num_surrounding_potions);
                    false
                }
            }
        } else {
            unreachable!("Special::end_of_turn called with a TilePosition that does not correspond to a TileInfo::Special Tile");
        }
    }
}

impl SpecialGenerator {
    pub fn get(&mut self) -> Special {
        let special_type =
            SpecialType::try_from(self.type_randomizer.weighted_random().expect("")).expect("");
        let id = self.unused_id;
        self.unused_id += 1;
        Special {
            id,
            special_type,
            special_info: SpecialInfo::from(special_type),
            being: Being::from(special_type),
        }
    }
}
