#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AbilityType {
    DoubleShieldCollection,
    DoubleCoinCollection,
    DoubleWeaponDamage,
    EnemiesToGold,
    ScrambleBoard,
    COUNT,
}
pub type AbilityCooldown = usize;

impl TryFrom<usize> for AbilityType {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::DoubleShieldCollection),
            1 => Ok(Self::DoubleCoinCollection),
            2 => Ok(Self::DoubleWeaponDamage),
            3 => Ok(Self::EnemiesToGold),
            4 => Ok(Self::ScrambleBoard),
            _ => Err("Invalid value given to AbilityType::TryFrom<usize>"),
        }
    }
}

impl AbilityType {
    pub fn name_description(self) -> (&'static str, &'static str) {
        match self {
            Self::DoubleShieldCollection => (
                "Obsidian Defense",
                "Doubles all shield collection this turn",
            ),
            Self::DoubleCoinCollection => {
                ("Plentiful Bounty", "Doubles all coin collection this turn")
            }
            Self::DoubleWeaponDamage => ("Sharpened Blades", "Doubles all weapon damage this turn"),
            Self::EnemiesToGold => ("Touch of Midas", "Turns every non-boss enemy to gold"),
            Self::ScrambleBoard => ("Gambler's Shuffle", "Randomizes the position of each tile"),
            Self::COUNT => unreachable!(""),
        }
    }

    pub fn base_cooldown(self) -> usize {
        match self {
            Self::DoubleShieldCollection => 19,
            Self::DoubleCoinCollection => 17,
            Self::DoubleWeaponDamage => 15,
            Self::EnemiesToGold => 23,
            Self::ScrambleBoard => 14,
            Self::COUNT => unreachable!(""),
        }
    }
}

pub struct Ability {
    pub ability_type: AbilityType,
    pub current_cooldown: AbilityCooldown,
    pub current_level: usize,
}

impl Ability {
    pub fn new(ability_type: AbilityType) -> Self {
        Self {
            ability_type,
            current_cooldown: ability_type.base_cooldown(),
            current_level: 1,
        }
    }

    pub fn level_up(&mut self) {
        self.current_level += 1;
        self.current_cooldown -= 1;
    }
}
