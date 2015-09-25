// puzzle.rs

use board::Board;

type History = Vec<Board>;

pub struct Puzzle {
    width: usize,
    height: usize,
    history: History,
    curr_history: usize
}

impl Puzzle {
    pub fn new(width: usize, height: usize) -> Puzzle {
        assert!(width > 0 && height > 0);
        let mut h: History = Vec::new();
        let b = Board::new(width, height);

        h.push(b);

        Puzzle {
            width: width,
            height: height,
            history: h,
            curr_history: 0
        }
    }

    pub fn get_board(&self) -> &Board {
        &self.history[self.curr_history]
    }

    pub fn undo(&mut self) {
        if self.curr_history > 0 {
            self.curr_history = self.curr_history - 1;
        }
    }

    pub fn redo(&mut self) {
        if self.curr_history + 1 < self.history.len() {
            self.curr_history = self.curr_history + 1;
        }
    }

    pub fn update(&mut self, board: Board) {
        assert!(board.width == self.width && board.height == self.height);

        while self.history.len() > self.curr_history + 1 {
            self.history.pop();
        }
        self.history.push(board);
        self.curr_history = self.history.len() - 1;
    }
}
