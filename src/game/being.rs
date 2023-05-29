pub enum BeingType {
    Player,
    Enemy,
    Boss,
}

pub struct Being {
    being_type: BeingType,
    base_output_damage: isize,
    weapon_output_damage: isize,
    hit_points: isize,
    max_hit_points: isize,
    shields: isize,
    max_shields: isize,
}

const PLAYER_START_HP: isize = 20;
const PLAYER_START_SH: isize = 3;
const ENEMY_START_HP: isize = 5;
const ENEMY_START_SH: isize = 0;
const BOSS_START_HP: isize = 50;
const BOSS_START_SH: isize = 4;

type BeingIsDead = bool;
impl Being {
    pub fn new(being_type: BeingType) -> Self {
        match being_type {
            BeingType::Player => Self {
                being_type,
                base_output_damage: 3,
                weapon_output_damage: 3,
                hit_points: PLAYER_START_HP,
                max_hit_points: PLAYER_START_HP,
                shields: PLAYER_START_SH,
                max_shields: PLAYER_START_SH,
            },
            BeingType::Enemy => Self {
                being_type,
                base_output_damage: 1,
                weapon_output_damage: 0,
                hit_points: ENEMY_START_HP,
                max_hit_points: ENEMY_START_HP,
                shields: ENEMY_START_SH,
                max_shields: ENEMY_START_SH,
            },
            BeingType::Boss => Self {
                being_type,
                base_output_damage: 7,
                weapon_output_damage: 3,
                hit_points: BOSS_START_HP,
                max_hit_points: BOSS_START_HP,
                shields: BOSS_START_SH,
                max_shields: BOSS_START_SH,
            },
        }
    }

    pub fn take_damage(&mut self, mut damage: isize) -> BeingIsDead {
        if damage <= self.shields {
            self.shields -= damage;
        } else {
            damage -= self.shields;
            self.shields = 0;
            self.hit_points -= damage;
        }
        self.hit_points <= 0
    }

    pub fn output_damage(&self, num_enemies: isize, num_weapons: isize) -> isize {
        let mut dmg = num_weapons * self.weapon_output_damage;
        if num_enemies > 0 {
            dmg += self.base_output_damage;
        }
        dmg
    }

    pub fn add_hit_points(&mut self, mut hit_points_to_add: isize) -> isize {
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

    pub fn add_shields(&mut self, mut shields_to_add: isize) -> isize {
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
}
