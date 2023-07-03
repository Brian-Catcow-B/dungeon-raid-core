use crate::game::being::{Being, BeingType};
use crate::game::randomizer::WeightedRandomizer;
use crate::game::tile::{TileInfo, TilePosition, TileType};
use crate::game::Game;

pub type SpecialIdentifier = usize;
pub type ModifiesBoard = bool;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SpecialType {
    Boss,
    Chaotic,
    Precise,
    Undead,
    Resourceful,
    Enlightener,
    Kamikaze,
    COUNT,
}

impl TryFrom<usize> for SpecialType {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Boss),
            1 => Ok(Self::Chaotic),
            2 => Ok(Self::Precise),
            3 => Ok(Self::Undead),
            4 => Ok(Self::Resourceful),
            5 => Ok(Self::Enlightener),
            6 => Ok(Self::Kamikaze),
            _ => Err("invalid value given to SpecialType::TryFrom<usize>"),
        }
    }
}

impl From<SpecialType> for Being {
    fn from(value: SpecialType) -> Being {
        let num_den = match value {
            SpecialType::Boss => (2, 1),
            SpecialType::Chaotic => (4, 3),
            SpecialType::Precise => (1, 1),
            SpecialType::Undead => (4, 3),
            SpecialType::Resourceful => (1, 1),
            SpecialType::Enlightener => (2, 3),
            SpecialType::Kamikaze => (9, 7),
            SpecialType::COUNT => unreachable!(""),
        };
        let mut being = Being::new(BeingType::Special, num_den.0, num_den.1);
        if value == SpecialType::Resourceful {
            // the shields being set to 0 should be overridden because
            // end_of_turn is run on specials after they spawn, but in
            // case something ends up depending on shields being <= max_shields...
            being.shields = 0;
            being.max_shields = 8;
        }
        being
    }
}

type Reanimated = bool;

type TurnsUntilEnlighten = usize;
macro_rules! ENLIGHTEN_COOLDOWN_MACRO {
    () => {
        3 // GAME_BALANCE: 4?
    };
}
const ENLIGHTEN_COOLDOWN: usize = ENLIGHTEN_COOLDOWN_MACRO!();

type TurnsUntilKamikaze = usize;
macro_rules! KAMIKAZE_COUNTDOWN_MACRO {
    () => {
        4
    };
}
const KAMIKAZE_COUNTDOWN: usize = KAMIKAZE_COUNTDOWN_MACRO!();

#[derive(Copy, Clone)]
pub enum SpecialInfo {
    Boss,
    Chaotic,
    Precise,
    Undead(Reanimated),
    Resourceful,
    Enlightener(TurnsUntilEnlighten),
    Kamikaze(TurnsUntilKamikaze),
}

impl From<SpecialType> for SpecialInfo {
    fn from(value: SpecialType) -> SpecialInfo {
        match value {
            SpecialType::Boss => Self::Boss,
            SpecialType::Chaotic => Self::Chaotic,
            SpecialType::Precise => Self::Precise,
            SpecialType::Undead => Self::Undead(false),
            SpecialType::Resourceful => Self::Resourceful,
            SpecialType::Enlightener => Self::Enlightener(ENLIGHTEN_COOLDOWN + 1),
            SpecialType::Kamikaze => Self::Kamikaze(KAMIKAZE_COUNTDOWN + 1),
            SpecialType::COUNT => unreachable!(""),
        }
    }
}

impl SpecialType {
    pub fn name_description(self) -> (&'static str, &'static str) {
        match self {
            Self::Boss => ("Boss", "A much stronger enemy"),
            Self::Chaotic => ("Chaotic", "Teleports to a random tile every turn"),
            Self::Precise => ("Precise", "Attacks cannot be blunted"),
            Self::Undead => (
                "Undead",
                "When killed the first time, reanimates with half HP",
            ),
			Self::Resourceful => ("Resourceful", "For surrounding tiles, armor = shields, attack += swords, health += health potions"),
            // RENAME: maybe "regular monster" will be called something different
            Self::Enlightener => ("Enlightener", concat!("Every ", ENLIGHTEN_COOLDOWN_MACRO!(), " turns, a regular monster into a special monster")),
            Self::Kamikaze => ("Kamikaze", concat!("Explodes after ", KAMIKAZE_COUNTDOWN_MACRO!(), " turns, dealing half the player's max HP and destroying the surrounding tiles")),
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

    const END_OF_TURN_TILE_INFO_NOT_SPECIAL: &str = "Special::end_of_turn called with a TilePosition that does not correspond to a TileInfo::Special Tile";
    const SPECIAL_TYPE_SPECIAL_INFO_MISMATCH: &str = "SpecialType and SpecialInfo mismatch";
    pub fn end_of_turn(game: &mut Game, tile_position: &TilePosition) -> ModifiesBoard {
        let special_type =
            if let TileInfo::Special(ref special) = game.board.tile_at(tile_position).tile_info {
                special.special_type
            } else {
                unreachable!("{}", Self::END_OF_TURN_TILE_INFO_NOT_SPECIAL);
            };
        match special_type {
            SpecialType::Boss => false,
            SpecialType::Chaotic => {
                game.board.swap_position_with_random_other(tile_position);
                true
            }
            SpecialType::Precise => false,
            SpecialType::Undead => false,
            SpecialType::Resourceful => {
                let num_surrounding_shields = game
                    .board
                    .num_surrounding_tiles_of_type(tile_position, TileType::Shield);
                let num_surrounding_swords = game
                    .board
                    .num_surrounding_tiles_of_type(tile_position, TileType::Sword);
                let num_surrounding_potions = game
                    .board
                    .num_surrounding_tiles_of_type(tile_position, TileType::Potion);
                if let TileInfo::Special(ref mut special) =
                    game.board.mut_tile_at(tile_position).tile_info
                {
                    special.being.shields = num_surrounding_shields;
                    special.being.base_output_damage += num_surrounding_swords;
                    special.being.add_hit_points(num_surrounding_potions);
                } else {
                    unreachable!("{}", Self::END_OF_TURN_TILE_INFO_NOT_SPECIAL);
                }
                false
            }
            SpecialType::Enlightener => {
                if let TileInfo::Special(ref mut special) =
                    game.board.mut_tile_at(tile_position).tile_info
                {
                    if let SpecialInfo::Enlightener(ref mut turns_until_enlighten) =
                        special.special_info
                    {
                        if *turns_until_enlighten == 0 {
                            *turns_until_enlighten = ENLIGHTEN_COOLDOWN;
                            match game.board.random_tile_of_type(TileType::Enemy) {
                                Some(tile_position) => {
                                    //let tile_info = TileInfo::from((TileType::Special, &game.enemy, &mut game.special_generator));
                                    //*game.board.mut_tile_at(&tile_position) = Tile::new(TileType::Special, tile_info);
                                    game.board.replace_tile(
                                        &tile_position,
                                        TileType::Special,
                                        &game.enemy,
                                        &mut game.special_generator,
                                    );
                                    true
                                }
                                None => false,
                            }
                        } else {
                            *turns_until_enlighten -= 1;
                            false
                        }
                    } else {
                        unreachable!("{}", Self::SPECIAL_TYPE_SPECIAL_INFO_MISMATCH);
                    }
                } else {
                    unreachable!("{}", Self::END_OF_TURN_TILE_INFO_NOT_SPECIAL);
                }
            }
            SpecialType::Kamikaze => {
                if let TileInfo::Special(ref mut special) =
                    game.board.mut_tile_at(tile_position).tile_info
                {
                    if let SpecialInfo::Kamikaze(ref mut turns_until_kamikaze) =
                        special.special_info
                    {
                        if *turns_until_kamikaze == 0 {
                            game.board.destroy_3x3_centered_at(
                                tile_position,
                                &game.enemy,
                                &mut game.special_generator,
                            );
                            game.player
                                .take_damage(game.player.being.max_hit_points / 2);
                            true
                        } else {
                            *turns_until_kamikaze -= 1;
                            false
                        }
                    } else {
                        unreachable!("{}", Self::SPECIAL_TYPE_SPECIAL_INFO_MISMATCH);
                    }
                } else {
                    unreachable!("{}", Self::END_OF_TURN_TILE_INFO_NOT_SPECIAL);
                }
            }
            SpecialType::COUNT => unreachable!(""),
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
