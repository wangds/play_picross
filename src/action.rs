// action.rs

use board::Board;

pub enum PicrossAction {
    NoOp,
    Quit,
    Undo,
    Redo,
    Update(Board)
}
