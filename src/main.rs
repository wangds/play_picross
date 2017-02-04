// main.rs

extern crate sdl2;

#[cfg(feature = "flic")]
extern crate flic;

mod action;
mod board;
mod gfx;
mod gui;
mod puzzle;

use std::env;

use action::PicrossAction;
use gui::Gui;
use puzzle::Puzzle;

fn main() {
    let mut gui = Gui::new();
    let mut puzzle = Puzzle::new(10, 10);
    let mut quit = false;

    if env::args().count() > 1 {
        let filename = env::args().nth(1).unwrap();
        if let Some(p) = Puzzle::load_file(&filename) {
            puzzle = p;
        }
    }

    gui.on_new_puzzle(&puzzle);

    while !quit {
        match gui.read_input(puzzle.get_board()) {
            PicrossAction::NoOp => {},
            PicrossAction::Quit => quit = true,

            PicrossAction::New(filename) =>
                if let Some(p) = Puzzle::load_file(&filename) {
                    puzzle = p;
                    gui.on_new_puzzle(&puzzle);
                },

            PicrossAction::Undo => puzzle.undo(),
            PicrossAction::Redo => puzzle.redo(),
            PicrossAction::Update(new_b) => puzzle.update(new_b),

            PicrossAction::AutoFill =>
                if let Some(new_b) = puzzle.get_board().autofill(puzzle.get_rules()) {
                    puzzle.update(new_b);
                }
        }

        gui.draw_to_screen(puzzle.get_rules(), puzzle.get_board());
    }
}
