pub const IMPROVEMENT_CHOICE_SET_SIZE: usize = 3;

use crate::game::coin_purchase::{CoinPurchase, CoinPurchaseGenerator};
use crate::game::shield_upgrade::{ShieldUpgrade, ShieldUpgradeGenerator};

#[derive(Copy, Clone)]
pub enum ImprovementType {
    Shields,
    Coins,
}

impl ImprovementType {
    fn header(self) -> &'static str {
        match self {
            Self::Shields => "Upgrade a stat",
            Self::Coins => "Purchase an item",
        }
    }
}

// always IMPROVEMENT_CHOICE_SET_SIZE elements in each vector
pub enum ImprovementInfo {
    ShieldUpgradeInfo(Vec<ShieldUpgrade>),
    CoinPurchaseInfo(Vec<CoinPurchase>),
}

#[derive(Default)]
pub struct ImprovementChoiceSetGenerator {
    shield_upgrade_generator: ShieldUpgradeGenerator,
    coin_purchase_generator: CoinPurchaseGenerator,
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
}

impl ImprovementChoiceSetGenerator {
    pub fn get(&mut self, improvement_type: ImprovementType) -> ImprovementChoiceSet {
        match improvement_type {
            ImprovementType::Shields => {
                let mut shield_upgrades: Vec<ShieldUpgrade> =
                    Vec::with_capacity(IMPROVEMENT_CHOICE_SET_SIZE);
                let mut displays: Vec<ImprovementChoiceDisplay> =
                    Vec::with_capacity(IMPROVEMENT_CHOICE_SET_SIZE);
                for pushing_idx in 0..IMPROVEMENT_CHOICE_SET_SIZE {
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
                }
            }
            ImprovementType::Coins => {
                let mut coin_purchases: Vec<CoinPurchase> =
                    Vec::with_capacity(IMPROVEMENT_CHOICE_SET_SIZE);
                let mut displays: Vec<ImprovementChoiceDisplay> =
                    Vec::with_capacity(IMPROVEMENT_CHOICE_SET_SIZE);
                for pushing_idx in 0..IMPROVEMENT_CHOICE_SET_SIZE {
                    coin_purchases.push(self.coin_purchase_generator.get());
                    displays.push(ImprovementChoiceDisplay::from(&coin_purchases[pushing_idx]));
                }
                self.coin_purchase_generator.reset();
                ImprovementChoiceSet {
                    improvement_type,
                    header: improvement_type.header(),
                    displays,
                    info: ImprovementInfo::CoinPurchaseInfo(coin_purchases),
                }
            }
        }
    }
}
