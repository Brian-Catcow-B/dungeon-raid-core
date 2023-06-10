use crate::game::stat_modifiers::BaseDamageDecrease;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BeingType {
    Player,
    Enemy,
    Boss,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Being {
    pub being_type: BeingType,
    pub base_output_damage: usize,
    pub weapon_output_damage: usize,
    pub hit_points: usize,
    pub max_hit_points: usize,
    pub shields: usize,
    pub max_shields: usize,
}

const PLAYER_START_BASE_DMG: usize = 3;
const PLAYER_START_WEAPON_DMG: usize = 3;
const PLAYER_START_HP: usize = 50;
const PLAYER_START_SH: usize = 3;

const ENEMY_START_DMG: usize = 2;
const ENEMY_START_HP: usize = 5;
const ENEMY_START_SH: usize = 0;

const BOSS_START_DMG: usize = 7;
const BOSS_START_HP: usize = 50;
const BOSS_START_SH: usize = 4;

const MIN_BASE_DAMAGE: usize = 1;

pub type BeingIsDead = bool;
impl Being {
    pub fn new(being_type: BeingType) -> Self {
        match being_type {
            BeingType::Player => Self {
                being_type,
                base_output_damage: PLAYER_START_BASE_DMG,
                weapon_output_damage: PLAYER_START_WEAPON_DMG,
                hit_points: PLAYER_START_HP,
                max_hit_points: PLAYER_START_HP,
                shields: PLAYER_START_SH,
                max_shields: PLAYER_START_SH,
            },
            BeingType::Enemy => Self {
                being_type,
                base_output_damage: ENEMY_START_DMG,
                weapon_output_damage: 0,
                hit_points: ENEMY_START_HP,
                max_hit_points: ENEMY_START_HP,
                shields: ENEMY_START_SH,
                max_shields: ENEMY_START_SH,
            },
            BeingType::Boss => Self {
                being_type,
                base_output_damage: BOSS_START_DMG,
                weapon_output_damage: 3,
                hit_points: BOSS_START_HP,
                max_hit_points: BOSS_START_HP,
                shields: BOSS_START_SH,
                max_shields: BOSS_START_SH,
            },
        }
    }

    pub fn take_damage(&mut self, mut damage: usize) -> BeingIsDead {
        if damage <= self.shields {
            self.shields -= damage;
        } else {
            damage -= self.shields;
            self.shields = 0;
            self.hit_points -= damage;
        }
        self.hit_points <= 0
    }

    pub fn output_damage(&self, num_enemies: usize, num_weapons: usize) -> usize {
        let mut dmg = num_weapons * self.weapon_output_damage;
        if num_enemies > 0 {
            dmg += self.base_output_damage;
        }
        dmg
    }

    pub fn add_hit_points(&mut self, mut hit_points_to_add: usize) -> usize {
        let missing_hp = self.max_hit_points - self.hit_points;
        if hit_points_to_add <= missing_hp {
            self.hit_points += hit_points_to_add;
            return 0;
        } else {
            hit_points_to_add -= missing_hp;
            self.hit_points = self.max_hit_points;
            return hit_points_to_add;
        }
    }

    pub fn add_shields(&mut self, mut shields_to_add: usize) -> usize {
        let missing_sh = self.max_shields - self.shields;
        if shields_to_add <= missing_sh {
            self.shields += shields_to_add;
            return 0;
        } else {
            shields_to_add -= missing_sh;
            self.shields = self.max_shields;
            return shields_to_add;
        }
    }

    pub fn blunt(&mut self, blunting: BaseDamageDecrease) {
        if self.base_output_damage - MIN_BASE_DAMAGE <= blunting {
            self.base_output_damage = MIN_BASE_DAMAGE;
        }
        else {
            self.base_output_damage -= blunting;
        }
    }
}
