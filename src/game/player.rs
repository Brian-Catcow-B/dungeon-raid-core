use crate::game::being::{Being, BeingType};
use crate::game::coin_purchase::{CoinPurchase, CoinPurchaseInfo};
use crate::game::shield_upgrade::{ShieldUpgrade, ShieldUpgradeInfo};
use crate::game::stat_modifiers::PlayerStatModifiers;

pub struct Player {
    pub being: Being,
    pub coin_cents: usize,
    pub excess_shield_cents: usize,
    pub experience_points: usize,
    pub stat_modifiers: PlayerStatModifiers,
}

pub const COIN_CENTS_PER_PURCHASE: usize = 10000;
pub const EXCESS_SHIELD_CENTS_PER_UPGRADE: usize = 400;
pub const EXPERIENCE_POINT_CENTS_PER_LEVEL_UP: usize = 100;

impl Default for Player {
    fn default() -> Self {
        Self {
            being: Being::new(BeingType::Player),
            coin_cents: 0,
            excess_shield_cents: 0,
            experience_points: 0,
            stat_modifiers: PlayerStatModifiers::default(),
        }
    }
}

type NumRollovers = usize;
fn rollover_add(val_into: &mut usize, val_other: usize, exclusive_max: usize) -> NumRollovers {
    *val_into += val_other;
    let divi = *val_into as usize / exclusive_max as usize;
    let modu = *val_into % exclusive_max;
    *val_into = modu;
    divi
}

pub type PlayerIsDead = bool;
pub type NumPurchases = usize;
pub type NumUpgrades = usize;
pub type NumLevelUps = usize;
impl Player {
    pub fn take_damage(&mut self, damage: usize) -> PlayerIsDead {
        self.being.take_damage(damage)
    }

    pub fn output_damage(&self, num_enemies: usize, num_weapons: usize) -> usize {
        self.being.output_damage(num_enemies, num_weapons)
    }

    pub fn add_hit_points(&mut self, heart_potions_collected: usize) -> usize {
        self.being.add_hit_points(heart_potions_collected * self.stat_modifiers.hit_points_per_potion)
    }

    pub fn add_coins(&mut self, coin_tiles_collected: usize) -> NumPurchases {
        rollover_add(&mut self.coin_cents, coin_tiles_collected * self.stat_modifiers.percent_gold_per_coin, COIN_CENTS_PER_PURCHASE)
    }

    fn add_excess_shields(&mut self, excess_shields_to_add: usize) -> NumUpgrades {
        rollover_add(
            &mut self.excess_shield_cents,
            excess_shields_to_add * self.stat_modifiers.percent_upgrade_points_per_shield,
            EXCESS_SHIELD_CENTS_PER_UPGRADE,
        )
    }

    pub fn add_shields(&mut self, shield_tiles_collected: usize) -> NumUpgrades {
        let excess = self.being.add_shields(shield_tiles_collected);
        self.add_excess_shields(excess)
    }

    pub fn add_experience_points(&mut self, experience_point_tiles_collected: usize) -> NumLevelUps {
        rollover_add(
            &mut self.experience_points,
            experience_point_tiles_collected,
            EXPERIENCE_POINT_CENTS_PER_LEVEL_UP,
        )
    }

    // improvements

    pub fn apply_upgrade(&mut self, upgrade: &ShieldUpgrade) {
        match upgrade.shield_upgrade_info {
            ShieldUpgradeInfo::Defense(def_inc) => {
                self.being.max_shields += def_inc;
                self.add_shields(def_inc);
            }
            ShieldUpgradeInfo::BaseDamage(base_dmg_inc) => {
                self.being.base_output_damage += base_dmg_inc;
            }
            ShieldUpgradeInfo::Blunting(blunting) => {
                self.stat_modifiers.blunting += blunting;
            }
            ShieldUpgradeInfo::GoldPerCoin(gold_per_coin_inc) => {
                self.stat_modifiers.percent_gold_per_coin += gold_per_coin_inc;
            }
            ShieldUpgradeInfo::HitPointsPerPotion(hp_per_potion_inc) => {
                self.stat_modifiers.hit_points_per_potion += hp_per_potion_inc;
            }
            ShieldUpgradeInfo::UpgradePointsPerShield(up_per_shield_inc) => {
                self.stat_modifiers.percent_upgrade_points_per_shield += up_per_shield_inc;
            }
        };
    }

    pub fn apply_purchase(&mut self, purchase: &CoinPurchase) {
        match purchase.coin_purchase_info {
            CoinPurchaseInfo::Defense(def_inc) => {
                self.being.max_shields += def_inc;
                self.add_shields(def_inc);
            }
            CoinPurchaseInfo::Attack(weap_dmg_inc) => {
                self.being.weapon_output_damage += weap_dmg_inc;
            }
        };
    }
}
