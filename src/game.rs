mod board;
use board::Board;

pub mod tile;
use tile::{Tile, TilePosition, TileType};

mod randomizer;

mod being;
use being::{Being, BeingIsDead, BeingType};

mod player;
use player::Player;

mod coin_purchase;
mod improvement_choices;
mod shield_upgrade;
mod stat_modifier_types;
use improvement_choices::{ImprovementChoiceSet, ImprovementChoiceSetGenerator, ImprovementType, ImprovementInfo};

pub struct Game {
    board: Board,
    player: Player,
    enemy: Being,
    boss: Being,
    improvement_choice_set_generator: ImprovementChoiceSetGenerator,
    improvement_choice_set: Option<ImprovementChoiceSet>,
    improvement_queue: Vec<ImprovementType>,
}

pub const DEFAULT_BOARD_WIDTH: usize = 6;
pub const DEFAULT_BOARD_HEIGHT: usize = 6;

impl Default for Game {
    fn default() -> Game {
        let enemy = Being::new(BeingType::Enemy);
        let boss = Being::new(BeingType::Boss);
        Game {
            board: Board::new(DEFAULT_BOARD_WIDTH, DEFAULT_BOARD_HEIGHT, &enemy, &boss),
            player: Player::default(),
            enemy,
            boss,
            improvement_choice_set_generator: ImprovementChoiceSetGenerator::default(),
            improvement_choice_set: None,
            improvement_queue: vec![],
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

    pub fn incoming_damage(&self) -> isize {
        self.board.incoming_damage()
    }

    pub fn apply_incoming_damage(&mut self) -> BeingIsDead {
        self.player.take_damage(self.board.incoming_damage())
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

    pub fn drop_selection(&mut self) -> Vec<Tile> {
        let vec = self.board.drop_selection(&self.player);
        let (mut hearts, mut shields, mut coins) = (0, 0, 0);
        for tile in vec.iter() {
            match tile.tile_type {
                TileType::Heart => hearts += 1,
                TileType::Shield => shields += 1,
                TileType::Coin => coins += 1,
                TileType::Sword => {}
                TileType::Enemy => { /*TODO: add xp*/ }
                TileType::Boss => { /*TODO: add xp*/ }
                TileType::COUNT | TileType::None => {
                    unreachable!("drop_selection went over invalid TileType")
                }
            };
        }
        if hearts > 0 {
            self.player.add_hit_points(hearts);
        }
        // TODO: handle upgrade/purchase/lvl up
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

        self.step_improvement_queue();

        vec
    }

    pub fn choose_improvement(&mut self, index: usize) -> bool {
        match self.improvement_choice_set {
            Some(ref set) => {
                match set.info {
                    ImprovementInfo::ShieldUpgradeInfo(ref vec_shield_upgrade) => self.player.apply_upgrade(&vec_shield_upgrade[index]),
                    ImprovementInfo::CoinPurchaseInfo(ref vec_coin_purchase) => self.player.apply_purchase(&vec_coin_purchase[index]),
                };
                true
            },
            None => false,
        }
    }

    pub fn apply_gravity_and_randomize_new_tiles(&mut self) {
        self.board
            .apply_gravity_and_randomize_new_tiles(&self.enemy, &self.boss);
    }

    pub fn get_tile(&self, tile_position: TilePosition) -> Option<Tile> {
        self.board.get_tile(tile_position)
    }

    pub fn get_selection_start(&self) -> Option<TilePosition> {
        self.board.selection_start
    }
}
