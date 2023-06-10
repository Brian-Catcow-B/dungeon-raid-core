pub type DefenseIncrease = usize;
pub type WeaponDamageIncrease = usize;
pub type BaseDamageIncrease = usize;
pub type BaseDamageDecrease = usize;
pub type PercentGoldPerCoinIncrease = usize;
pub type HitPointsPerPotionIncrease = usize;
pub type PercentUpgradePointsPerShieldIncrease = usize;

pub struct PlayerStatModifiers {
    //pub defense_increase: DefenseIncrease,
    //pub weapon_damage_increase: WeaponDamageIncrease,
    //pub base_damage_increase: BaseDamageIncrease,
    pub blunting: BaseDamageDecrease,
    pub percent_gold_per_coin: PercentGoldPerCoinIncrease,
    pub hit_points_per_potion: HitPointsPerPotionIncrease,
    pub percent_upgrade_points_per_shield: PercentUpgradePointsPerShieldIncrease,
}

impl Default for PlayerStatModifiers {
    fn default() -> Self {
        Self {
            blunting: 0,
            percent_gold_per_coin: 100,
            hit_points_per_potion: 1,
            percent_upgrade_points_per_shield: 100,
        }
    }
}
