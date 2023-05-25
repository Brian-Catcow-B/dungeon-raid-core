use crate::game::randomizer::WeightedRandomizer;
use crate::game::tile::Tile;

struct Board {
    // access by [y][x] where [0][0] is top left corner
    tiles: Vec<Vec<Tile>>,
    randomizer: WeightedRandomizer,
}

impl Board {
    pub fn new(w: usize, h: usize) -> Board {
        let mut tiles = vec![];
        for _ in 0..w {
            let new_idx = tiles.len();
            tiles.push(vec![]);
            for _ in 0..h {
                tiles[new_idx].push(Tile::default());
            }
        }

        Board {
            tiles,
            randomizer: WeightedRandomizer::default(),
        }
    }
}

