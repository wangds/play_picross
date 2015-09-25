// action.rs

use board::Board;

pub enum PicrossAction {
    Quit,
    Undo,
    Redo,
    Update(Board)
}
