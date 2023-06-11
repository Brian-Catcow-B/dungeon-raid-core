use crate::game::abilities::AbilityType;
use crate::game::improvement_choices::ImprovementChoiceDisplay;
use crate::game::randomizer::{WeightedRandomizer, WeightedRandomizerType};

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
    stat_level_up_type_randomizer: WeightedRandomizer,
    generation: usize,
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
            stat_level_up_type_randomizer,
            generation: 0,
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

impl ExperiencePointLevelUpGenerator {
    pub fn get(&mut self) -> ExperiencePointLevelUp {
        let xplu = if self.generation < 2 {
            // first 2 are ability options
            let atype =
                AbilityType::try_from(self.ability_type_randomizer.weighted_random().expect(""))
                    .expect("");
            ExperiencePointLevelUp {
                experience_point_level_up_info: ExperiencePointLevelUpInfo::Ability(atype),
            }
        } else {
            // give the rest as stat options
            let slutype = StatLevelUpType::try_from(
                self.stat_level_up_type_randomizer
                    .weighted_random()
                    .expect(""),
            )
            .expect("");
            ExperiencePointLevelUp {
                experience_point_level_up_info: ExperiencePointLevelUpInfo::Stat(
                    StatLevelUpInfo::from(slutype),
                ),
            }
        };

        self.generation += 1;
        xplu
    }

    pub fn reset(&mut self) {
        self.ability_type_randomizer.reset_metadata();
        self.stat_level_up_type_randomizer.reset_metadata();
        self.generation = 0;
    }
}
