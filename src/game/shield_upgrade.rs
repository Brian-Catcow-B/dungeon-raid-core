use crate::game::improvement_choices::ImprovementChoiceDisplay;
use crate::game::randomizer::{WeightedRandomizer, WeightedRandomizerType};
use crate::game::stat_modifiers::{
    BaseDamageDecrease, BaseDamageIncrease, DefenseIncrease, HitPointsPerPotionIncrease,
    PercentGoldPerCoinIncrease, PercentUpgradePointsPerShieldIncrease,
};

#[derive(Copy, Clone)]
pub enum ShieldUpgradeType {
    Defense,
    BaseDamage,
    Blunting,
    GoldPerCoin,
    HitPointsPerPotion,
    UpgradePointsPerShield,
    COUNT,
}

impl TryFrom<usize> for ShieldUpgradeType {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Defense),
            1 => Ok(Self::BaseDamage),
            2 => Ok(Self::Blunting),
            3 => Ok(Self::GoldPerCoin),
            4 => Ok(Self::HitPointsPerPotion),
            5 => Ok(Self::UpgradePointsPerShield),
            _ => Err("invalid value given to ShieldUpgradeType::TryFrom<usize>"),
        }
    }
}

#[derive(Copy, Clone)]
pub enum ShieldUpgradeInfo {
    Defense(DefenseIncrease),
    BaseDamage(BaseDamageIncrease),
    Blunting(BaseDamageDecrease),
    GoldPerCoin(PercentGoldPerCoinIncrease),
    HitPointsPerPotion(HitPointsPerPotionIncrease),
    UpgradePointsPerShield(PercentUpgradePointsPerShieldIncrease),
}

pub struct ShieldUpgradeGenerator {
    shield_upgrade_type_randomizer: WeightedRandomizer,
}

impl Default for ShieldUpgradeGenerator {
    fn default() -> Self {
        let mut shield_upgrade_type_randomizer =
            WeightedRandomizer::new(WeightedRandomizerType::MetaSubAllOnObtain);
        for sut in 0..(ShieldUpgradeType::COUNT as usize) {
            shield_upgrade_type_randomizer.set_weight(sut, 1);
        }
        Self {
            shield_upgrade_type_randomizer,
        }
    }
}

pub struct ShieldUpgrade {
    pub shield_upgrade_type: ShieldUpgradeType,
    pub shield_upgrade_info: ShieldUpgradeInfo,
}

impl From<&ShieldUpgrade> for ImprovementChoiceDisplay {
    fn from(value: &ShieldUpgrade) -> Self {
        let mut description = String::from("Upgrade ");
        description += match value.shield_upgrade_type {
            ShieldUpgradeType::Defense => "Defense by ",
            ShieldUpgradeType::BaseDamage => "Base Damage by ",
            ShieldUpgradeType::Blunting => "Blunting by ",
            ShieldUpgradeType::GoldPerCoin => "Gold per Coin by ",
            ShieldUpgradeType::HitPointsPerPotion => "Hit Points per Potion by ",
            ShieldUpgradeType::UpgradePointsPerShield => "Upgrade-Points per Shield by ",
            ShieldUpgradeType::COUNT => unreachable!(""),
        };
        let info_string;
        match value.shield_upgrade_info {
            ShieldUpgradeInfo::Defense(val) => info_string = format!("{}", val),
            ShieldUpgradeInfo::BaseDamage(val) => info_string = format!("{}", val),
            ShieldUpgradeInfo::Blunting(val) => info_string = format!("{}", val),
            ShieldUpgradeInfo::GoldPerCoin(val) => info_string = format!("{}%", val),
            ShieldUpgradeInfo::HitPointsPerPotion(val) => info_string = format!("{}", val),
            ShieldUpgradeInfo::UpgradePointsPerShield(val) => info_string = format!("{}%", val),
        };
        description += info_string.as_str();
        Self { description }
    }
}

impl ShieldUpgradeGenerator {
    pub fn get(&mut self) -> ShieldUpgrade {
        let shield_upgrade_type = ShieldUpgradeType::try_from(
            self.shield_upgrade_type_randomizer
                .weighted_random()
                .expect(""),
        )
        .expect("");
        // TODO: make random "lucky" rolls for stat upgrades
        let shield_upgrade_info = match shield_upgrade_type {
            ShieldUpgradeType::Defense => ShieldUpgradeInfo::Defense(1),
            ShieldUpgradeType::BaseDamage => ShieldUpgradeInfo::BaseDamage(1),
            ShieldUpgradeType::Blunting => ShieldUpgradeInfo::Blunting(1),
            ShieldUpgradeType::GoldPerCoin => ShieldUpgradeInfo::GoldPerCoin(25),
            ShieldUpgradeType::HitPointsPerPotion => ShieldUpgradeInfo::HitPointsPerPotion(1),
            ShieldUpgradeType::UpgradePointsPerShield => {
                ShieldUpgradeInfo::UpgradePointsPerShield(25)
            }
            ShieldUpgradeType::COUNT => unreachable!(""),
        };
        ShieldUpgrade {
            shield_upgrade_type,
            shield_upgrade_info,
        }
    }

    pub fn reset(&mut self) {
        self.shield_upgrade_type_randomizer.reset_metadata();
    }
}
