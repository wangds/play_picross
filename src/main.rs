// main.rs

mod action;
mod board;
mod puzzle;

use action::PicrossAction;
use board::Board;
use board::Tile;
use puzzle::Puzzle;

fn main() {
    let mut puzzle = Puzzle::new(10, 10);
    let mut b = Board::new(10, 10);
    let mut quit = false;
    let mut iter = 0;

    while !quit {
        match read_input(&mut b, iter) {
            PicrossAction::Quit => quit = true,
            PicrossAction::Undo => puzzle.undo(),
            PicrossAction::Redo => puzzle.redo(),
            PicrossAction::Update(new_b) => puzzle.update(new_b)
        }

        // dodgy.
        if iter == 0 {
            println!("{:?}", puzzle.get_board().get(0,0).unwrap());
        } else {
            println!("{:?}", puzzle.get_board().get(0,1).unwrap());
        }

        iter = iter + 1;
    }
}

fn read_input(board: &Board, iter: u8) -> PicrossAction {
    if iter == 0 {
        let mut b = board.clone();
        b.set(0,0,Tile::Filled);
        PicrossAction::Update(b)
    } else if iter == 1 {
        let mut b = board.clone();
        b.set(0,1,Tile::CrossedOut);
        PicrossAction::Update(b)
    } else if iter == 2 {
        PicrossAction::Undo
    } else if iter == 3 {
        PicrossAction::Redo
    } else {
        PicrossAction::Quit
    }
}
