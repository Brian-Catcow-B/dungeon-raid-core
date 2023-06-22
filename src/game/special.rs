use crate::game::being::{Being, BeingType};
use crate::game::randomizer::WeightedRandomizer;
use crate::game::Game;

#[derive(Copy, Clone)]
pub enum SpecialType {
    Boss,
    Unstable,
    WeaponsMaster,
    Undead,
    COUNT,
}

impl TryFrom<usize> for SpecialType {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Boss),
            1 => Ok(Self::Unstable),
            2 => Ok(Self::WeaponsMaster),
            3 => Ok(Self::Undead),
            _ => Err("invalid value given to SpecialType::TryFrom<usize>"),
        }
    }
}

impl From<SpecialType> for Being {
    fn from(value: SpecialType) -> Being {
        let num_den = match value {
            SpecialType::Boss => (2, 1),
            SpecialType::Unstable => (4, 3),
            SpecialType::WeaponsMaster => (1, 1),
            SpecialType::Undead => (4, 3),
            SpecialType::COUNT => unreachable!(""),
        };
        Being::new(BeingType::Special, num_den.0, num_den.1)
    }
}

type Reanimated = bool;
#[derive(Copy, Clone)]
pub enum SpecialInfo {
    Boss,
    Unstable,
    WeaponsMaster,
    Undead(Reanimated),
}

impl From<SpecialType> for SpecialInfo {
    fn from(value: SpecialType) -> SpecialInfo {
        match value {
            SpecialType::Boss => Self::Boss,
            SpecialType::Unstable => Self::Unstable,
            SpecialType::WeaponsMaster => Self::WeaponsMaster,
            SpecialType::Undead => Self::Undead(false),
            SpecialType::COUNT => unreachable!(""),
        }
    }
}

impl SpecialType {
    pub fn name_description(self) -> (&'static str, &'static str) {
        match self {
            Self::Boss => ("Boss", "A much stronger enemy"),
            Self::Unstable => ("Unstable", "Teleports to a random tile every turn"),
            Self::WeaponsMaster => ("Weapons Master", "Attacks cannot be blunted"),
            Self::Undead => (
                "Undead",
                "When killed the first time, reanimates with half HP",
            ),
            Self::COUNT => unreachable!(""),
        }
    }
}

pub struct SpecialGenerator {
    type_randomizer: WeightedRandomizer,
}

impl Default for SpecialGenerator {
    fn default() -> Self {
        let mut type_randomizer = WeightedRandomizer::default();
        for st in 0..(SpecialType::COUNT as usize) {
            type_randomizer.set_weight(st, 1);
        }
        Self { type_randomizer }
    }
}

#[derive(Copy, Clone)]
pub struct Special {
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
            SpecialType::WeaponsMaster => {}
            _ => self.being.blunt(blunting),
        }
    }

    pub fn end_of_turn(&mut self, game: &mut Game) {}
}

impl SpecialGenerator {
    pub fn get(&mut self) -> Special {
        let special_type =
            SpecialType::try_from(self.type_randomizer.weighted_random().expect("")).expect("");
        Special {
            special_type,
            special_info: SpecialInfo::from(special_type),
            being: Being::from(special_type),
        }
    }
}
