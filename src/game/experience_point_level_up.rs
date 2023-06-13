use crate::game::abilities::AbilityType;
use crate::game::abilities::MAX_ABILITY_LEVEL;
use crate::game::improvement_choices::ImprovementChoiceDisplay;
use crate::game::randomizer::{WeightedRandomizer, WeightedRandomizerType};
use crate::game::ABILITY_SLOTS;

pub enum ExperiencePointLevelUpType {
    Ability,
    Stat,
}

impl TryFrom<usize> for ExperiencePointLevelUpType {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Ability),
            1 => Ok(Self::Stat),
            _ => Err("invalid value given to ExperiencePointLevelUpType::TryFrom<usize>"),
        }
    }
}

pub enum ExperiencePointLevelUpInfo {
    Ability(AbilityType),
    Stat(StatLevelUpInfo),
}

pub enum StatLevelUpType {
    MaxHitPoints,
    BaseOutputDamage,
    COUNT,
}

impl TryFrom<usize> for StatLevelUpType {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::MaxHitPoints),
            1 => Ok(Self::BaseOutputDamage),
            _ => Err("invalid value given to StatLevelUpType::TryFrom<usize>"),
        }
    }
}

#[derive(Copy, Clone)]
pub enum StatLevelUpInfo {
    // TODO: type this stuff
    MaxHitPoints(usize),
    BaseOutputDamage(usize),
}

impl From<StatLevelUpType> for StatLevelUpInfo {
    fn from(value: StatLevelUpType) -> Self {
        match value {
            StatLevelUpType::MaxHitPoints => Self::MaxHitPoints(10),
            StatLevelUpType::BaseOutputDamage => Self::BaseOutputDamage(1),
            StatLevelUpType::COUNT => unreachable!(""),
        }
    }
}

impl StatLevelUpInfo {
    pub fn name_description(self) -> (&'static str, String) {
        match self {
            Self::MaxHitPoints(hp_inc) => {
                ("Health", format!("Increase max hit points by {}", hp_inc))
            }
            Self::BaseOutputDamage(bod_inc) => {
                ("Damage", format!("Increase base damage by {}", bod_inc))
            }
        }
    }
}

pub struct ExperiencePointLevelUpGenerator {
    ability_type_randomizer: WeightedRandomizer,
    chosen_ability_type_randomizer: WeightedRandomizer,
    stat_level_up_type_randomizer: WeightedRandomizer,
    generation: usize,
    chosen_abilities: Vec<AbilityType>,
}

impl Default for ExperiencePointLevelUpGenerator {
    fn default() -> Self {
        let mut ability_type_randomizer =
            WeightedRandomizer::new(WeightedRandomizerType::MetaSubAllOnObtain);
        for at in 0..(AbilityType::COUNT as usize) {
            ability_type_randomizer.set_weight(at, 1);
        }
        let mut stat_level_up_type_randomizer =
            WeightedRandomizer::new(WeightedRandomizerType::MetaSubAllOnObtain);
        for st in 0..(StatLevelUpType::COUNT as usize) {
            stat_level_up_type_randomizer.set_weight(st, 1);
        }
        Self {
            ability_type_randomizer,
            chosen_ability_type_randomizer: WeightedRandomizer::new(
                WeightedRandomizerType::MetaSubAllOnObtain,
            ),
            stat_level_up_type_randomizer,
            generation: 0,
            chosen_abilities: Vec::with_capacity(ABILITY_SLOTS),
        }
    }
}

pub struct ExperiencePointLevelUp {
    pub experience_point_level_up_info: ExperiencePointLevelUpInfo,
}

impl From<&ExperiencePointLevelUp> for ImprovementChoiceDisplay {
    fn from(value: &ExperiencePointLevelUp) -> Self {
        let mut description;
        match value.experience_point_level_up_info {
            ExperiencePointLevelUpInfo::Ability(ref atype) => {
                description = String::from("ABILITY: ");
                let (name, desc) = atype.name_description();
                description += name;
                description += ". ";
                description += desc;
            }
            ExperiencePointLevelUpInfo::Stat(ref slutype) => {
                description = String::from("STAT: ");
                let (name, desc) = slutype.name_description();
                description += name;
                description += ". ";
                description += desc.as_str();
            }
        };
        Self { description }
    }
}

const NUM_ABILITY_OPTIONS: usize = 2;
impl ExperiencePointLevelUpGenerator {
    pub fn get(&mut self) -> Option<ExperiencePointLevelUp> {
        let xplu_opt = if self.generation < NUM_ABILITY_OPTIONS {
            // first are ability options
            if ABILITY_SLOTS - self.chosen_abilities.len() <= self.generation {
                // level up existing abilities (since we can't give more
                // options than there are available ability slots)
                let value_opt = self.chosen_ability_type_randomizer.weighted_random();
                // value_opt could be None since abilities are removed from
                // chosen_ability_type_randomizer when they hit max level
                match value_opt {
                    Some(value) => {
                        let atype = AbilityType::try_from(value).expect("");
                        Some(ExperiencePointLevelUp {
                            experience_point_level_up_info: ExperiencePointLevelUpInfo::Ability(
                                atype,
                            ),
                        })
                    }
                    None => None,
                }
            } else {
                // level up potentially unique, potentially existing abilities
                let atype = AbilityType::try_from(
                    self.ability_type_randomizer.weighted_random().expect(""),
                )
                .expect("");
                Some(ExperiencePointLevelUp {
                    experience_point_level_up_info: ExperiencePointLevelUpInfo::Ability(atype),
                })
            }
        } else {
            // give the rest as stat options
            let slutype = StatLevelUpType::try_from(
                self.stat_level_up_type_randomizer
                    .weighted_random()
                    .expect(""),
            )
            .expect("");
            Some(ExperiencePointLevelUp {
                experience_point_level_up_info: ExperiencePointLevelUpInfo::Stat(
                    StatLevelUpInfo::from(slutype),
                ),
            })
        };

        self.generation += 1;
        xplu_opt
    }

    pub fn reset(&mut self) {
        // note we do NOT reset the chosen_ability_type_randomizer
        // because that stores important metadata pertaining to abilities
        // that are at their max level
        self.ability_type_randomizer.reset_metadata();
        self.stat_level_up_type_randomizer.reset_metadata();
        self.generation = 0;
    }

    pub fn ability_upgraded(&mut self, ability_type: AbilityType, ability_level: usize) {
        let mut exists = false;
        for atype in self.chosen_abilities.iter() {
            if *atype == ability_type {
                exists = true;
                break;
            }
        }
        if exists {
            if ability_level == MAX_ABILITY_LEVEL {
                self.chosen_ability_type_randomizer
                    .meta_remove_value(ability_type as usize);
            }
        } else {
            self.chosen_abilities.push(ability_type);
            self.chosen_ability_type_randomizer
                .set_weight(ability_type as usize, 1);
        }
    }
}
