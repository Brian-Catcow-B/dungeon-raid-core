mod board;
use board::Board;

mod randomizer;

pub mod tile;
use tile::{Tile, TilePosition};

pub struct Game {
    board: Board,
}

impl Default for Game {
    fn default() -> Game {
        Game {
            board: Board::new(6, 6),
        }
    }
}

impl Game {
    pub fn select_tile(&mut self, tile_position: TilePosition) -> bool {
        self.board.select_tile(tile_position)
    }

    pub fn drop_selection(&mut self) -> Vec<TilePosition> {
        self.board.drop_selection()
    }

    pub fn apply_gravity_and_randomize_new_tiles(&mut self) {
        self.board.apply_gravity_and_randomize_new_tiles();
    }

    pub fn get_tile(&self, tile_position: TilePosition) -> Option<Tile> {
        self.board.get_tile(tile_position)
    }
}
