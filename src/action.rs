// action.rs

use board::Board;

pub enum PicrossAction {
    NoOp,
    Quit,

    // New(filename)
    New(String),

    Undo,
    Redo,
    Update(Board),

    AutoFill,
}
