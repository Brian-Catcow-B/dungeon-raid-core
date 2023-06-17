#[derive(Copy, Clone, PartialEq, Eq)]

#[allow(clippy::upper_case_acronyms)]
pub enum AbilityType {
    DoubleShieldCollection,
    DoubleCoinCollection,
    DoubleWeaponCollection,
    EnemiesToCoins,
    ScrambleBoard,
    COUNT,
}
pub const MAX_ABILITY_LEVEL: usize = 10;
pub type AbilityCooldown = usize;

impl TryFrom<usize> for AbilityType {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::DoubleShieldCollection),
            1 => Ok(Self::DoubleCoinCollection),
            2 => Ok(Self::DoubleWeaponCollection),
            3 => Ok(Self::EnemiesToCoins),
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
            Self::DoubleWeaponCollection => {
                ("Sharpened Blades", "Doubles all weapon damage this turn")
            }
            Self::EnemiesToCoins => ("Touch of Midas", "Turns every non-boss enemy to gold"),
            Self::ScrambleBoard => ("Gambler's Shuffle", "Randomizes the position of each tile"),
            Self::COUNT => unreachable!(""),
        }
    }

    pub fn base_cooldown(self) -> usize {
        match self {
            Self::DoubleShieldCollection => 19,
            Self::DoubleCoinCollection => 17,
            Self::DoubleWeaponCollection => 15,
            Self::EnemiesToCoins => 23,
            Self::ScrambleBoard => 14,
            Self::COUNT => unreachable!(""),
        }
    }
}

pub struct Ability {
    pub ability_type: AbilityType,
    pub cooldown: AbilityCooldown,
    pub running_cooldown: AbilityCooldown,
    pub current_level: usize,
}

impl Ability {
    pub fn new(ability_type: AbilityType) -> Self {
        Self {
            ability_type,
            cooldown: ability_type.base_cooldown(),
            running_cooldown: 0,
            current_level: 9,
        }
    }

    pub fn level_up(&mut self) {
        self.current_level += 1;
        self.cooldown -= 1;
    }

    pub fn put_on_cooldown(&mut self) {
        self.running_cooldown = self.cooldown;
    }
}
