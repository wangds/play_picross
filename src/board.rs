// board.rs

#[derive(Clone,Copy,Eq,PartialEq)]
pub enum Tile {
    Empty,
    Filled,
    CrossedOut
}

#[derive(Clone)]
pub struct Board {
    pub width: usize,
    pub height: usize,
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

    fn at(&self, x: usize, y: usize) -> Tile {
        self.tiles[self.width * y + x]
    }

    pub fn get(&self, x: u32, y: u32) -> Option<Tile> {
        let xx = x as usize;
        let yy = y as usize;
        if xx < self.width && yy < self.height {
            Some(self.at(xx, yy))
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

    pub fn get_completed_column_segments_from_head(&self, x: usize) -> Vec<u32> {
        let mut segments = Vec::new();

        if x < self.width {
            let mut y = 0;

            while y < self.height {
                let mut count = 0;

                if self.at(x, y) == Tile::Empty {
                    break;
                }

                while y < self.height && self.at(x, y) == Tile::CrossedOut {
                    y = y + 1;
                }

                while y < self.height && self.at(x, y) == Tile::Filled {
                    count = count + 1;
                    y = y + 1;
                }

                if count > 0
                    && (y >= self.height || self.at(x, y) == Tile::CrossedOut) {
                    segments.push(count);
                }
            }
        }

        segments
    }

    pub fn get_completed_column_segments_from_tail(&self, x: usize) -> Vec<u32> {
        let mut segments = Vec::new();

        if x < self.width {
            let mut y = self.height;

            while y > 0 {
                let mut count = 0;

                if self.at(x, y - 1) == Tile::Empty {
                    break;
                }

                while y > 0 && self.at(x, y - 1) == Tile::CrossedOut {
                    y = y - 1;
                }

                while y > 0 && self.at(x, y - 1) == Tile::Filled {
                    count = count + 1;
                    y = y - 1;
                }

                if count > 0
                    && (y == 0 || self.at(x, y - 1) == Tile::CrossedOut) {
                    segments.push(count);
                }
            }
        }

        segments
    }

    pub fn get_completed_row_segments_from_head(&self, y: usize) -> Vec<u32> {
        let mut segments = Vec::new();

        if y < self.height {
            let mut x = 0;

            while x < self.width {
                let mut count = 0;

                if self.at(x, y) == Tile::Empty {
                    break;
                }

                while x < self.width && self.at(x, y) == Tile::CrossedOut {
                    x = x + 1;
                }

                while x < self.width && self.at(x, y) == Tile::Filled {
                    count = count + 1;
                    x = x + 1;
                }

                if count > 0
                    && (x >= self.width || self.at(x, y) == Tile::CrossedOut) {
                    segments.push(count);
                }
            }
        }

        segments
    }

    pub fn get_completed_row_segments_from_tail(&self, y: usize) -> Vec<u32> {
        let mut segments = Vec::new();

        if y < self.height {
            let mut x = self.width;

            while x > 0 {
                let mut count = 0;

                if self.at(x - 1, y) == Tile::Empty {
                    break;
                }

                while x > 0 && self.at(x - 1, y) == Tile::CrossedOut {
                    x = x - 1;
                }

                while x > 0 && self.at(x - 1, y) == Tile::Filled {
                    count = count + 1;
                    x = x - 1;
                }

                if count > 0
                    && (x == 0 || self.at(x - 1, y) == Tile::CrossedOut) {
                    segments.push(count);
                }
            }
        }

        segments
    }
}
