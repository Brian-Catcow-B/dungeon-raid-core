mod board;
use board::Board;

pub mod tile;
use tile::{Tile, TilePosition};

mod randomizer;

mod being;
use being::{BeingType, Being};

mod player;
use player::Player;

pub struct Game {
    board: Board,
    player: Player,
    enemy: Being,
    boss: Being,
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
        }
    }
}

impl Game {
    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn incoming_damage(&self) -> isize {
        self.board.incoming_damage
    }

    pub fn select_tile(&mut self, tile_position: TilePosition) -> bool {
        self.board.select_tile(tile_position)
    }

    pub fn drop_selection(&mut self) -> Vec<Tile> {
        self.board.drop_selection(&self.player)
    }

    pub fn apply_gravity_and_randomize_new_tiles(&mut self) {
        self.board.apply_gravity_and_randomize_new_tiles(&self.enemy, &self.boss);
    }

    pub fn get_tile(&self, tile_position: TilePosition) -> Option<Tile> {
        self.board.get_tile(tile_position)
    }

    pub fn get_selection_start(&self) -> Option<TilePosition> {
        self.board.selection_start
    }
}
