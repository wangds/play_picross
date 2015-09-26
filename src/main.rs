// main.rs

extern crate sdl2;
extern crate sdl2_image;

mod action;
mod board;
mod gfx;
mod gui;
mod puzzle;

use action::PicrossAction;
use gui::Gui;
use puzzle::Puzzle;

fn main() {
    let mut gui = Gui::new();
    let mut puzzle = Puzzle::new(10, 10);
    let mut quit = false;

    while !quit {
        match gui.read_input(puzzle.get_board()) {
            PicrossAction::NoOp => {},
            PicrossAction::Quit => quit = true,
            PicrossAction::Undo => puzzle.undo(),
            PicrossAction::Redo => puzzle.redo(),
            PicrossAction::Update(new_b) => puzzle.update(new_b)
        }

        gui.draw_to_screen(puzzle.get_board());
    }
}