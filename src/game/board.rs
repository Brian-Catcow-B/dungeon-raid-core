use crate::game::randomizer::WeightedRandomizer;
use crate::game::tile::{Tile, TilePosition, Wind8};

struct Board {
    // access by [y][x] where [0][0] is top left corner
    tiles: Vec<Vec<Tile>>,
    randomizer: WeightedRandomizer,
    selection_start: Option<TilePosition>,
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
            selection_start: None,
        }
    }

    fn position_valid(&self, pos: TilePosition) -> bool {
        self.tiles.len() > 0
            && (pos.y as usize) < self.tiles.len()
            && (pos.x as usize) < self.tiles[0].len()
    }

    fn remove_selection_starting_at(&mut self, mut pos: TilePosition) {
        loop {
            if self.position_valid(pos) {
                let relative_next = self.tiles[pos.y as usize][pos.x as usize].next_selection;
                match relative_next {
                    Wind8::None => return,
                    _ => pos = pos + TilePosition::from(relative_next),
                };
            } else {
                unreachable!("remove_selection_starting_at found that either the given position or one of the positions down the selection trail points off the board");
            }
        }
    }

    pub fn select_tile(&mut self, position_to_select: TilePosition) -> bool {
        match self.selection_start {
            Some(pos) => {
                let mut p: TilePosition = pos;
                loop {
                    // TODO: make this limited to the number of tiles in case there is some sort of invalid board
                    if self.position_valid(p) {
                        let relative_next = self.tiles[p.y as usize][p.x as usize].next_selection;
                        match relative_next {
                            Wind8::None => {
                                match Wind8::try_from(position_to_select - p) {
                                    Ok(w8) => match w8 {
                                        Wind8::None => {},
                                        _ => {
                                            self.tiles[p.y as usize][p.x as usize].next_selection = w8;
                                            return true;
                                        },
                                    }
                                    Err(_) => return false,
                                };
                            },
                            _ => {
                                p = p + TilePosition::from(relative_next);
                                if p == position_to_select {
                                    self.remove_selection_starting_at(p);
                                    return true;
                                }
                            },
                        }
                    } else {
                        unreachable!("in select_tile, one of the tiles in the selection trail points off the board");
                    }
                }
            }
            None => {
                self.selection_start = Some(position_to_select);
                return true;
            }
        }
    }

    pub fn get_selection_trail(&self) -> Vec<TilePosition> {
        let mut trail: Vec<TilePosition> = vec![];
        match self.selection_start {
            Some(pos) => {
                let mut p: TilePosition = pos;
                loop {
                    // TODO: limit this loop
                    if self.position_valid(p) {
                        trail.push(p);
                        let relative_next = self.tiles[p.y as usize][p.x as usize].next_selection;
                        match relative_next {
                            Wind8::None => break,
                            _ => p = p + TilePosition::from(relative_next),
                        };
                    } else {
                        unreachable!("in get_selection_trail, one of the tiles in the selection trail points off the board");
                    }
                }
            },
            None => {},
        };
        trail
    }
}
