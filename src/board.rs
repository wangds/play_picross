// board.rs

#[derive(Clone,Copy,Debug)]
pub enum Tile {
    Empty,
    Filled,
    CrossedOut
}

pub struct Board {
    width: usize,
    height: usize,
    tiles: Vec<Tile>
}

impl Board {
    pub fn new(width: usize, height: usize) -> Board {
        assert!(width > 0 && height > 0);
        let ts = vec![Tile::Empty; width * height];

        Board {
            width: width,
            height: height,
            tiles: ts
        }
    }

    pub fn get(&self, x: u32, y: u32) -> Option<Tile> {
        let xx = x as usize;
        let yy = y as usize;
        if xx < self.width && yy < self.height {
            Some(self.tiles[self.width * yy + xx])
        } else {
            None
        }
    }

    pub fn set(&mut self, x: u32, y: u32, state: Tile) {
        let xx = x as usize;
        let yy = y as usize;
        if xx < self.width && yy < self.height {
            self.tiles[self.width * yy + xx] = state;
        }
    }
}
