use crate::game::abilities::{Ability, AbilityType};
use crate::game::being::{Being, BeingType};
use crate::game::coin_purchase::{CoinPurchase, CoinPurchaseInfo};
use crate::game::experience_point_level_up::{
    ExperiencePointLevelUp, ExperiencePointLevelUpInfo, StatLevelUpInfo,
};
use crate::game::shield_upgrade::{ShieldUpgrade, ShieldUpgradeInfo};
use crate::game::stat_modifiers::PlayerStatModifiers;
use crate::game::ABILITY_SLOTS;

pub struct Player {
    pub being: Being,
    pub coin_cents: usize,
    pub excess_shield_cents: usize,
    pub experience_point_cents: usize,
    pub stat_modifiers: PlayerStatModifiers,
    pub abilities: Vec<Option<Ability>>,
}

pub const COIN_CENTS_PER_PURCHASE: usize = 1000;
pub const EXCESS_SHIELD_CENTS_PER_UPGRADE: usize = 1000;
pub const EXPERIENCE_POINT_CENTS_PER_LEVEL_UP: usize = 300;

impl Default for Player {
    fn default() -> Self {
        let mut abilities = Vec::with_capacity(ABILITY_SLOTS);
        for _ in 0..ABILITY_SLOTS {
            abilities.push(None);
        }
        Self {
            being: Being::new(BeingType::Player, 1, 1),
            coin_cents: 0,
            excess_shield_cents: 0,
            experience_point_cents: 0,
            stat_modifiers: PlayerStatModifiers::default(),
            abilities,
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

    pub fn add_hit_points(&mut self, potions_collected: usize) -> usize {
        self.being
            .add_hit_points(potions_collected * self.stat_modifiers.hit_points_per_potion)
    }

    pub fn add_coins(&mut self, coin_tiles_collected: usize) -> NumPurchases {
        rollover_add(
            &mut self.coin_cents,
            coin_tiles_collected * self.stat_modifiers.percent_gold_per_coin,
            COIN_CENTS_PER_PURCHASE,
        )
    }

    fn add_excess_shields(&mut self, excess_shields_to_add: usize) -> NumUpgrades {
        rollover_add(
            &mut self.excess_shield_cents,
            excess_shields_to_add * self.stat_modifiers.percent_upgrade_points_per_shield,
            EXCESS_SHIELD_CENTS_PER_UPGRADE,
        )
    }

    pub fn add_shields(&mut self, shield_tiles_collected: usize) -> NumUpgrades {
        let excess = self
            .being
            .add_shields(shield_tiles_collected, self.stat_modifiers.armor_per_shield);
        self.add_excess_shields(excess)
    }

    pub fn add_experience_points(
        &mut self,
        experience_point_tiles_collected: usize,
    ) -> NumLevelUps {
        rollover_add(
            &mut self.experience_point_cents,
            experience_point_tiles_collected * self.stat_modifiers.percent_xp_per_experience_point,
            EXPERIENCE_POINT_CENTS_PER_LEVEL_UP,
        )
    }

    // improvements

    pub fn apply_upgrade(&mut self, upgrade: &ShieldUpgrade) {
        match upgrade.shield_upgrade_info {
            ShieldUpgradeInfo::Defense(def_inc) => {
                self.being.max_shields += def_inc;
                self.being.shields += def_inc;
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
                self.being.shields += def_inc;
            }
            CoinPurchaseInfo::Attack(weap_dmg_inc) => {
                self.being.weapon_output_damage += weap_dmg_inc;
            }
        };
    }

    fn handle_ability_level_up(&mut self, ability_type: AbilityType) -> usize {
        for ability_opt in self.abilities.iter_mut() {
            match ability_opt {
                Some(ref mut a) => {
                    if a.ability_type == ability_type {
                        a.level_up();
                        return a.current_level;
                    }
                }
                None => {
                    *ability_opt = Some(Ability::new(ability_type));
                    return ability_opt.as_ref().expect("").current_level;
                }
            }
        }
        unreachable!("it should be impossible to level up an ability that the player doesn't have if there are no more slots available for a new one");
    }

    // returns the level of the leveled up ability if an ability was upgraded and 0 otherwise
    pub fn apply_level_up(&mut self, level_up: &ExperiencePointLevelUp) -> usize {
        match level_up.experience_point_level_up_info {
            ExperiencePointLevelUpInfo::Ability(atype) => self.handle_ability_level_up(atype),
            ExperiencePointLevelUpInfo::Stat(sluinfo) => {
                match sluinfo {
                    StatLevelUpInfo::MaxHitPoints(max_hp_inc) => {
                        self.being.max_hit_points += max_hp_inc;
                        self.being.hit_points += max_hp_inc;
                    }
                    StatLevelUpInfo::BaseOutputDamage(bod_inc) => {
                        self.being.base_output_damage += bod_inc;
                    }
                    StatLevelUpInfo::ArmorPerShield(aps_inc) => {
                        self.stat_modifiers.armor_per_shield += aps_inc;
                    }
                };
                0
            }
        }
    }
}
