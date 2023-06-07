use crate::game::improvement_choices::ImprovementChoiceDisplay;
use crate::game::randomizer::{WeightedRandomizer, WeightedRandomizerType};
use crate::game::stat_modifier_types::{DefenseIncrease, WeaponDamageIncrease};

pub enum CoinPurchaseType {
    Defense,
    Attack,
}

impl TryFrom<usize> for CoinPurchaseType {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Defense),
            1 => Ok(Self::Attack),
            _ => Err("invalid value given to CoinPurchaseType::TryFrom<usize>"),
        }
    }
}

pub enum CoinPurchaseInfo {
    Defense(DefenseIncrease),
    Attack(WeaponDamageIncrease),
}

#[derive(Copy, Clone)]
pub enum CoinPurchasePieceType {
    Helmet,
    Breastplate,
    Legguards,
    Greaves,
    Weapon,
    COUNT,
}

impl TryFrom<usize> for CoinPurchasePieceType {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Helmet),
            1 => Ok(Self::Breastplate),
            2 => Ok(Self::Legguards),
            3 => Ok(Self::Greaves),
            4 => Ok(Self::Weapon),
            _ => Err("invalid value given to CoinPurchasePieceType::TryFrom<usize>"),
        }
    }
}

impl TryFrom<CoinPurchasePieceType> for CoinPurchaseType {
    type Error = &'static str;

    fn try_from(value: CoinPurchasePieceType) -> Result<Self, Self::Error> {
        match value {
            CoinPurchasePieceType::Helmet
            | CoinPurchasePieceType::Breastplate
            | CoinPurchasePieceType::Legguards
            | CoinPurchasePieceType::Greaves => Ok(Self::Defense),
            CoinPurchasePieceType::Weapon => Ok(Self::Attack),
            _ => Err("invalid value given to CoinPurchaseCategory::TryFrom<CoinPurchasePieceType>"),
        }
    }
}

pub struct CoinPurchaseGenerator {
    piece_type_randomizer: WeightedRandomizer,
    defense_increase_randomizer: WeightedRandomizer,
    attack_increase_randomizer: WeightedRandomizer,
}

impl Default for CoinPurchaseGenerator {
    fn default() -> Self {
        let mut piece_type_randomizer =
            WeightedRandomizer::new(WeightedRandomizerType::MetaSubAllOnObtain);
        for pt in 0..(CoinPurchasePieceType::COUNT as usize) {
            piece_type_randomizer.set_weight(pt, 1);
        }
        let mut defense_increase_randomizer = WeightedRandomizer::default();
        defense_increase_randomizer.set_weight(1, 99);
        defense_increase_randomizer.set_weight(2, 1);
        let mut attack_increase_randomizer = WeightedRandomizer::default();
        attack_increase_randomizer.set_weight(1, 99);
        attack_increase_randomizer.set_weight(2, 1);
        Self {
            piece_type_randomizer,
            defense_increase_randomizer,
            attack_increase_randomizer,
        }
    }
}

pub struct CoinPurchase {
    pub coin_purchase_type: CoinPurchaseType,
    pub coin_purchase_info: CoinPurchaseInfo,
    pub coin_purchase_piece_type: CoinPurchasePieceType,
}

impl From<&CoinPurchase> for ImprovementChoiceDisplay {
    fn from(value: &CoinPurchase) -> Self {
        let mut description = String::from("Upgrade ");
        description += match value.coin_purchase_piece_type {
            CoinPurchasePieceType::Helmet => "helmet's ",
            CoinPurchasePieceType::Breastplate => "breastplate's ",
            CoinPurchasePieceType::Legguards => "legguards' ",
            CoinPurchasePieceType::Greaves => "greaves' ",
            CoinPurchasePieceType::Weapon => "weapon's ",
            CoinPurchasePieceType::COUNT => unreachable!(""),
        };
        description += match value.coin_purchase_type {
            CoinPurchaseType::Defense => "defense by ",
            CoinPurchaseType::Attack => "attack by ",
        };
        let inc_string: String;
        match value.coin_purchase_info {
            CoinPurchaseInfo::Defense(inc) => inc_string = format!("{}", inc),
            CoinPurchaseInfo::Attack(inc) => inc_string = format!("{}", inc),
        };
        description += inc_string.as_str();
        Self { description }
    }
}

impl CoinPurchaseGenerator {
    pub fn get(&mut self) -> CoinPurchase {
        let coin_purchase_piece_type = CoinPurchasePieceType::try_from(
            self.piece_type_randomizer.weighted_random().expect(""),
        )
        .expect("");
        let coin_purchase_type = CoinPurchaseType::try_from(coin_purchase_piece_type).expect("");
        let coin_purchase_info = match coin_purchase_type {
            CoinPurchaseType::Defense => CoinPurchaseInfo::Defense(
                self.defense_increase_randomizer
                    .weighted_random()
                    .expect(""),
            ),
            CoinPurchaseType::Attack => CoinPurchaseInfo::Attack(
                self.attack_increase_randomizer.weighted_random().expect(""),
            ),
        };
        CoinPurchase {
            coin_purchase_type,
            coin_purchase_info,
            coin_purchase_piece_type,
        }
    }

    pub fn reset(&mut self) {
        self.piece_type_randomizer.reset_metadata();
    }
}
