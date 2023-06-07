use crate::game::being::{Being, BeingType};
use crate::game::coin_purchase::{CoinPurchase, CoinPurchaseInfo};
use crate::game::shield_upgrade::{ShieldUpgrade, ShieldUpgradeInfo};
use crate::game::stat_modifiers::PlayerStatModifiers;

pub struct Player {
    pub being: Being,
    pub coins: isize,
    pub excess_shields: isize,
    pub experience_points: isize,
    pub stat_modifiers: PlayerStatModifiers,
}

pub const COINS_PER_PURCHASE: isize = 10;
pub const EXCESS_SHIELDS_PER_UPGRADE: isize = 10;
pub const EXPERIENCE_POINTS_PER_LEVEL_UP: isize = 100;

impl Default for Player {
    fn default() -> Self {
        Self {
            being: Being::new(BeingType::Player),
            coins: 0,
            excess_shields: 0,
            experience_points: 0,
            stat_modifiers: PlayerStatModifiers::default(),
        }
    }
}

type NumRollovers = usize;
fn rollover_add(val_into: &mut isize, val_other: isize, exclusive_max: isize) -> NumRollovers {
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
    pub fn take_damage(&mut self, damage: isize) -> PlayerIsDead {
        self.being.take_damage(damage)
    }

    pub fn output_damage(&self, num_enemies: isize, num_weapons: isize) -> isize {
        self.being.output_damage(num_enemies, num_weapons)
    }

    pub fn add_hit_points(&mut self, hit_points_to_add: isize) -> isize {
        self.being.add_hit_points(hit_points_to_add)
    }

    pub fn add_coins(&mut self, coins_to_add: isize) -> NumPurchases {
        rollover_add(&mut self.coins, coins_to_add, COINS_PER_PURCHASE)
    }

    fn add_excess_shields(&mut self, excess_shields_to_add: isize) -> NumUpgrades {
        rollover_add(
            &mut self.excess_shields,
            excess_shields_to_add,
            EXCESS_SHIELDS_PER_UPGRADE,
        )
    }

    pub fn add_shields(&mut self, shields_to_add: isize) -> NumUpgrades {
        let excess = self.being.add_shields(shields_to_add);
        self.add_excess_shields(excess)
    }

    pub fn add_experience_points(&mut self, experience_points_to_add: isize) -> NumLevelUps {
        rollover_add(
            &mut self.experience_points,
            experience_points_to_add,
            EXPERIENCE_POINTS_PER_LEVEL_UP,
        )
    }

    // improvements

    pub fn apply_upgrade(&mut self, upgrade: &ShieldUpgrade) {
        match upgrade.shield_upgrade_info {
            ShieldUpgradeInfo::Defense(def_inc) => {
                self.being.max_shields += def_inc as isize;
                self.add_shields(def_inc as isize);
            }
            ShieldUpgradeInfo::BaseDamage(base_dmg_inc) => {
                self.being.base_output_damage += base_dmg_inc as isize;
            }
            ShieldUpgradeInfo::Blunting(blunting) => {
                self.stat_modifiers.blunting += blunting;
            }
            ShieldUpgradeInfo::GoldPerCoin(gold_per_coin_inc) => {
                self.stat_modifiers.gold_per_coin += gold_per_coin_inc;
            }
            ShieldUpgradeInfo::HitPointsPerPotion(hp_per_potion_inc) => {
                self.stat_modifiers.hit_points_per_potion += hp_per_potion_inc;
            }
            ShieldUpgradeInfo::UpgradePointsPerShield(up_per_shield_inc) => {
                self.stat_modifiers.upgrade_points_per_shield += up_per_shield_inc;
            }
        };
    }

    pub fn apply_purchase(&mut self, purchase: &CoinPurchase) {
        match purchase.coin_purchase_info {
            CoinPurchaseInfo::Defense(def_inc) => {
                self.being.max_shields += def_inc as isize;
                self.add_shields(def_inc as isize);
            }
            CoinPurchaseInfo::Attack(weap_dmg_inc) => {
                self.being.weapon_output_damage += weap_dmg_inc as isize;
            }
        };
    }
}
