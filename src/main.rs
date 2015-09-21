// main.rs

mod board;

use board::Board;
use board::Tile;

fn main() {
    let mut b = Board::new(10, 10);
    b.set(0,0,Tile::Filled);
    println!("{:?}", b.get(0,0).unwrap());
    b.set(0,1,Tile::CrossedOut);
    println!("{:?}", b.get(0,1).unwrap());
}
