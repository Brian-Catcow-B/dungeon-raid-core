pub const IMPROVEMENT_CHOICE_SET_SIZE: usize = 3;

use crate::game::coin_purchase::{CoinPurchase, CoinPurchaseGenerator};
use crate::game::experience_point_level_up::{
    ExperiencePointLevelUp, ExperiencePointLevelUpGenerator,
};
use crate::game::shield_upgrade::{ShieldUpgrade, ShieldUpgradeGenerator};

#[derive(Copy, Clone)]
pub enum ImprovementType {
    Shields,
    Coins,
    ExperiencePoints,
}

impl ImprovementType {
    fn header(self) -> &'static str {
        match self {
            Self::Shields => "Upgrade a stat",
            Self::Coins => "Purchase an item",
            Self::ExperiencePoints => "Level Up a facet",
        }
    }

    pub fn x_choose_y(self) -> (usize, usize) {
        match self {
            Self::Shields => (3, 1),
            Self::Coins => (3, 1),
            Self::ExperiencePoints => (4, 2),
        }
    }
}

// always IMPROVEMENT_CHOICE_SET_SIZE elements in each vector
pub enum ImprovementInfo {
    ShieldUpgradeInfo(Vec<ShieldUpgrade>),
    CoinPurchaseInfo(Vec<CoinPurchase>),
    ExperiencePointLevelUpInfo(Vec<ExperiencePointLevelUp>),
}

#[derive(Default)]
pub struct ImprovementChoiceSetGenerator {
    shield_upgrade_generator: ShieldUpgradeGenerator,
    coin_purchase_generator: CoinPurchaseGenerator,
    experience_point_level_up_generator: ExperiencePointLevelUpGenerator,
}

pub struct ImprovementChoiceDisplay {
    pub description: String,
}

pub struct ImprovementChoiceSet {
    pub improvement_type: ImprovementType,
    pub header: &'static str,
    // always IMPROVEMENT_CHOICE_SET_SIZE elements in this vector
    pub displays: Vec<ImprovementChoiceDisplay>,
    pub info: ImprovementInfo,
    pub num_to_choose: usize,
}

impl ImprovementChoiceSetGenerator {
    pub fn get(&mut self, improvement_type: ImprovementType) -> ImprovementChoiceSet {
        let (num_choices, num_to_choose) = improvement_type.x_choose_y();
        let mut displays: Vec<ImprovementChoiceDisplay> = Vec::with_capacity(num_choices);
        match improvement_type {
            ImprovementType::Shields => {
                let mut shield_upgrades: Vec<ShieldUpgrade> = Vec::with_capacity(num_choices);
                for pushing_idx in 0..num_choices {
                    shield_upgrades.push(self.shield_upgrade_generator.get());
                    displays.push(ImprovementChoiceDisplay::from(
                        &shield_upgrades[pushing_idx],
                    ));
                }
                self.shield_upgrade_generator.reset();
                ImprovementChoiceSet {
                    improvement_type,
                    header: improvement_type.header(),
                    displays,
                    info: ImprovementInfo::ShieldUpgradeInfo(shield_upgrades),
                    num_to_choose,
                }
            }
            ImprovementType::Coins => {
                let mut coin_purchases: Vec<CoinPurchase> = Vec::with_capacity(num_choices);
                for pushing_idx in 0..num_choices {
                    coin_purchases.push(self.coin_purchase_generator.get());
                    displays.push(ImprovementChoiceDisplay::from(&coin_purchases[pushing_idx]));
                }
                self.coin_purchase_generator.reset();
                ImprovementChoiceSet {
                    improvement_type,
                    header: improvement_type.header(),
                    displays,
                    info: ImprovementInfo::CoinPurchaseInfo(coin_purchases),
                    num_to_choose,
                }
            }
            ImprovementType::ExperiencePoints => {
                let mut experience_point_level_ups: Vec<ExperiencePointLevelUp> =
                    Vec::with_capacity(num_choices);
                for pushing_idx in 0..num_choices {
                    experience_point_level_ups.push(self.experience_point_level_up_generator.get());
                    displays.push(ImprovementChoiceDisplay::from(
                        &experience_point_level_ups[pushing_idx],
                    ));
                }
                self.experience_point_level_up_generator.reset();
                ImprovementChoiceSet {
                    improvement_type,
                    header: improvement_type.header(),
                    displays,
                    info: ImprovementInfo::ExperiencePointLevelUpInfo(experience_point_level_ups),
                    num_to_choose,
                }
            }
        }
    }
}
