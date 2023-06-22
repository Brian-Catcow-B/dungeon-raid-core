mod board;
use board::Board;

pub mod tile;
use tile::{Tile, TilePosition, TileType};

mod collection_multipliers;
use collection_multipliers::CollectionMultipliers;

mod randomizer;

mod being;
use being::{Being, BeingType};

mod player;
use player::{Player, PlayerIsDead};

mod special;
use special::SpecialGenerator;

mod stat_modifiers;

mod abilities;
use abilities::AbilityType;

mod coin_purchase;
mod experience_point_level_up;
use experience_point_level_up::ExperiencePointLevelUpInfo;
mod shield_upgrade;

pub mod improvement_choices;
use improvement_choices::{
    ImprovementChoiceSet, ImprovementChoiceSetGenerator, ImprovementInfo, ImprovementType,
};

pub struct Game {
    turns_passed: usize,
    most_recent_special_kill_turn: usize,
    min_turns_between_specials: usize,
    board: Board,
    player: Player,
    enemy: Being,
    special_generator: SpecialGenerator,
    improvement_choice_set_generator: ImprovementChoiceSetGenerator,
    improvement_choice_set: Option<ImprovementChoiceSet>,
    improvement_queue: Vec<ImprovementType>,
    collection_multipliers: CollectionMultipliers,
}

pub const DEFAULT_BOARD_WIDTH: usize = 6;
pub const DEFAULT_BOARD_HEIGHT: usize = 6;

pub const ABILITY_SLOTS: usize = 4;

const INITIAL_MIN_TURNS_BETWEEN_SPECIALS: usize = 2;

impl Default for Game {
    fn default() -> Game {
        let enemy = Being::new(BeingType::Enemy, 1, 1);
        let mut special_generator = SpecialGenerator::default();
        Game {
            turns_passed: 0,
            most_recent_special_kill_turn: 0,
            min_turns_between_specials: INITIAL_MIN_TURNS_BETWEEN_SPECIALS,
            board: Board::new(
                DEFAULT_BOARD_WIDTH,
                DEFAULT_BOARD_HEIGHT,
                &enemy,
                &mut special_generator,
            ),
            player: Player::default(),
            enemy,
            special_generator,
            improvement_choice_set_generator: ImprovementChoiceSetGenerator::default(),
            improvement_choice_set: None,
            improvement_queue: vec![],
            collection_multipliers: CollectionMultipliers::default(),
        }
    }
}

impl Game {
    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn improvement_choice_set(&self) -> &Option<ImprovementChoiceSet> {
        &self.improvement_choice_set
    }

    pub fn incoming_damage(&self) -> usize {
        self.board.incoming_damage()
    }

    pub fn special(&self) -> Option<Tile> {
        self.board.special()
    }

    pub fn apply_incoming_damage(&mut self) -> PlayerIsDead {
        let player_has_shields = self.player.being.shields > 0;
        let player_is_dead = self.player.take_damage(self.board.incoming_damage());
        if player_has_shields {
            self.board
                .apply_blunting(self.player.stat_modifiers.blunting);
        }

        player_is_dead
    }

    pub fn select_tile(&mut self, tile_position: TilePosition) -> bool {
        self.board.select_tile(tile_position)
    }

    fn step_improvement_queue(&mut self) {
        match self.improvement_queue.pop() {
            Some(imp_type) => {
                self.improvement_choice_set =
                    Some(self.improvement_choice_set_generator.get(imp_type))
            }
            None => self.improvement_choice_set = None,
        }
    }

    pub fn drop_selection(&mut self) -> bool {
        let (slash, vec) = self.board.drop_selection(
            &self.player,
            self.collection_multipliers.weapon_collection_multiplier,
        );
        let (mut potions, mut shields, mut coins, mut experience_points, mut special_killed) =
            (0, 0, 0, 0, false);
        for tile in vec.iter() {
            match tile.tile_type {
                TileType::Potion => potions += 1,
                TileType::Shield => {
                    shields += self.collection_multipliers.shield_collection_multiplier
                }
                TileType::Coin => coins += self.collection_multipliers.coin_collection_multiplier,
                TileType::Sword => {}
                TileType::Enemy => experience_points += 1,
                TileType::Special => {
                    experience_points += 15;
                    special_killed = true;
                }
                TileType::COUNT | TileType::None => {
                    unreachable!("drop_selection went over invalid TileType")
                }
            };
        }
        self.collection_multipliers = CollectionMultipliers::default();

        if slash {
            // collection
            if potions > 0 {
                self.player.add_hit_points(potions);
            }
            if shields > 0 {
                let num_upgrades = self.player.add_shields(shields);
                for _ in 0..num_upgrades {
                    self.improvement_queue.push(ImprovementType::Shields);
                }
            }
            if coins > 0 {
                let num_purchases = self.player.add_coins(coins);
                for _ in 0..num_purchases {
                    self.improvement_queue.push(ImprovementType::Coins);
                }
            }
            if experience_points > 0 {
                let num_level_ups = self.player.add_experience_points(experience_points);
                for _ in 0..num_level_ups {
                    self.improvement_queue
                        .push(ImprovementType::ExperiencePoints);
                }
            }
            // cooldowns down by 1
            for ability_opt in self.player.abilities.iter_mut() {
                match ability_opt {
                    Some(ref mut a) => {
                        if a.running_cooldown > 0 {
                            a.running_cooldown -= 1
                        }
                    }
                    None => {}
                };
            }
            // number of turns passed up by 1
            self.turns_passed += 1;
            // update min_turns_between_specials
            let mtbs_modifier = self.turns_passed / 25;
            self.min_turns_between_specials = if INITIAL_MIN_TURNS_BETWEEN_SPECIALS > mtbs_modifier
            {
                INITIAL_MIN_TURNS_BETWEEN_SPECIALS - mtbs_modifier
            } else {
                0
            };
            // update most_recent_special_kill_turn
            if special_killed {
                self.most_recent_special_kill_turn = self.turns_passed;
            }
            // handle special spawn stuff
            if self.turns_passed - self.most_recent_special_kill_turn
                >= self.min_turns_between_specials
            {
                self.board
                    .activate_special_spawns(self.turns_passed, self.most_recent_special_kill_turn);
            }
        }

        self.step_improvement_queue();

        slash
    }

    pub fn choose_improvements(&mut self, indeces: &Vec<usize>) {
        match self.improvement_choice_set {
            Some(ref set) => {
                match set.info {
                    ImprovementInfo::ShieldUpgradeInfo(ref vec_shield_upgrade) => {
                        for given_idx in indeces.iter() {
                            self.player.apply_upgrade(&vec_shield_upgrade[*given_idx]);
                        }
                    }
                    ImprovementInfo::CoinPurchaseInfo(ref vec_coin_purchase) => {
                        for given_idx in indeces.iter() {
                            self.player.apply_purchase(&vec_coin_purchase[*given_idx]);
                        }
                    }
                    ImprovementInfo::ExperiencePointLevelUpInfo(
                        ref vec_experience_point_level_up,
                    ) => {
                        for given_idx in indeces.iter() {
                            let lvl_up = &vec_experience_point_level_up[*given_idx];
                            let maybe_ability_level = self.player.apply_level_up(lvl_up);
                            if let ExperiencePointLevelUpInfo::Ability(atype) =
                                lvl_up.experience_point_level_up_info
                            {
                                self.improvement_choice_set_generator
                                    .ability_upgraded(atype, maybe_ability_level);
                            }
                        }
                    }
                };
            }
            None => {}
        };
        self.step_improvement_queue();
    }

    pub fn cast_ability(&mut self, index: usize) -> bool {
        let ability_opt = &mut self.player.abilities[index];
        match ability_opt {
            Some(ref mut a) => {
                if a.running_cooldown > 0 {
                    return false;
                }
                match a.ability_type {
                    AbilityType::DoubleShieldCollection => {
                        self.collection_multipliers.shield_collection_multiplier *= 2
                    }
                    AbilityType::DoubleCoinCollection => {
                        self.collection_multipliers.coin_collection_multiplier *= 2
                    }
                    AbilityType::DoubleWeaponCollection => {
                        self.collection_multipliers.weapon_collection_multiplier *= 2
                    }
                    AbilityType::EnemiesToCoins => {
                        self.board.replace_tiles(
                            TileType::Enemy,
                            TileType::Coin,
                            &self.enemy,
                            &mut self.special_generator,
                        );
                    }
                    AbilityType::ScrambleBoard => {
                        self.board.scramble();
                    }
                    AbilityType::COUNT => unreachable!(""),
                };
                a.put_on_cooldown();
                true
            }
            None => false,
        }
    }

    pub fn apply_gravity_and_randomize_new_tiles(&mut self) {
        self.board
            .apply_gravity_and_randomize_new_tiles(&self.enemy, &mut self.special_generator);
    }

    pub fn get_tile(&self, tile_position: TilePosition) -> Option<Tile> {
        self.board.get_tile(tile_position)
    }

    pub fn get_selection_start(&self) -> Option<TilePosition> {
        self.board.selection_start
    }
}
