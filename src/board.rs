// board.rs

use std::collections::HashMap;

use puzzle::Rule;
use puzzle::Rules;

#[derive(Clone,Copy,Eq,PartialEq)]
pub enum Tile {
    Empty,
    Filled,
    CrossedOut
}

#[derive(Clone,Copy,Eq,PartialEq)]
enum AutoFillTile {
    // State of board is filled or crossed out.
    Filled,
    CrossedOut,

    // State of board is empty, replaced as solutions are discovered.
    NoSolutionFound,
    CanBeFilled,
    CanBeCrossedOut,
    CanBeAnything
}

#[derive(Clone,Copy,Eq,PartialEq)]
enum AutoFillResult {
    // Too many solutions, no need to find any more
    SearchEnded,

    // Descendant found a solution.
    // Backtrack and look for other solutions.
    SolutionFound,

    // Some earlier choices lead to a conflict.
    // Backtrack and look for other solutions.
    Conflict,
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

    pub fn autofill(&self, rules: Rules) -> Option<Board> {
        type WorkXYT = (u32, u32, Tile);
        let (col_rules, row_rules) = rules;
        let mut work: Vec<WorkXYT> = Vec::new();

        for (row, rule) in row_rules.iter().enumerate() {
            let mut trial = self.make_row_slice(row);
            let mut accum = trial.clone();
            let mut cache = HashMap::new();

            try_autofill(&mut trial, 0, rule, 0, &mut accum, &mut cache);

            for (x, &t) in accum.iter().enumerate() {
                if t == AutoFillTile::CanBeFilled {
                    work.push((x as u32, row as u32, Tile::Filled))
                } else if t == AutoFillTile::CanBeCrossedOut {
                    work.push((x as u32, row as u32, Tile::CrossedOut))
                }
            }
        }

        for (col, rule) in col_rules.iter().enumerate() {
            let mut trial = self.make_col_slice(col);
            let mut accum = trial.clone();
            let mut cache = HashMap::new();

            try_autofill(&mut trial, 0, rule, 0, &mut accum, &mut cache);

            for (y, &t) in accum.iter().enumerate() {
                if t == AutoFillTile::CanBeFilled {
                    work.push((col as u32, y as u32, Tile::Filled))
                } else if t == AutoFillTile::CanBeCrossedOut {
                    work.push((col as u32, y as u32, Tile::CrossedOut))
                }
            }
        }

        if !work.is_empty() {
            let mut b = self.clone();
            for &(x, y, t) in work.iter() {
                b.set(x, y, t);
            }

            Some(b)
        } else {
            None
        }
    }

    fn make_row_slice(&self, y: usize) -> Vec<AutoFillTile> {
        self.tiles[self.width * y .. self.width * (y + 1)].iter()
            .map(|&t| make_autofill_tile(t)).collect()
    }

    fn make_col_slice(&self, x: usize) -> Vec<AutoFillTile> {
        (0..self.height).map(|y| self.width * y + x)
            .map(|i| make_autofill_tile(self.tiles[i])).collect()
    }
}

fn make_autofill_tile(t: Tile) -> AutoFillTile {
    match t {
        Tile::Empty => AutoFillTile::NoSolutionFound,
        Tile::Filled => AutoFillTile::Filled,
        Tile::CrossedOut => AutoFillTile::CrossedOut
    }
}

fn try_autofill(
        trial: &mut Vec<AutoFillTile>, pos: usize,
        rule: &Rule, rule_idx: usize,
        accum: &mut Vec<AutoFillTile>,
        cache: &mut HashMap<(usize, usize), AutoFillResult>)
        -> AutoFillResult {
    assert!(pos <= trial.len() && rule_idx <= rule.len());
    let key = (pos, rule_idx);

    // base case
    {
        let maybe_visited = cache.get(&key);
        if pos == trial.len() || maybe_visited.is_some() {
            if maybe_visited.map_or(rule_idx != rule.len(),
                    |&v| v == AutoFillResult::Conflict) {
                return AutoFillResult::Conflict
            }

            for (a, &mut t) in accum[0..pos].iter_mut().zip(trial) {
                *a = combine_solutions(t, *a);
            }

            if accum.iter().any(|&t| is_unassigned_unique_solution(t)) {
                return AutoFillResult::SolutionFound
            } else {
                return AutoFillResult::SearchEnded
            }
        }
    }

    // not enough space
    if rule_idx < rule.len() {
        let remaining_space = trial.len() - pos;
        let mut required_space = rule.len() - rule_idx - 1;
        for i in rule_idx..rule.len() {
            required_space = required_space + (rule[i] as usize);
        }
        if required_space > remaining_space {
            return AutoFillResult::Conflict
        }
    }

    let mut result = AutoFillResult::Conflict;

    // try empty
    if trial[pos] != AutoFillTile::Filled {
        if trial[pos] != AutoFillTile::CrossedOut {
            trial[pos] = AutoFillTile::CanBeCrossedOut;
        }

        let r = try_autofill(trial, pos + 1, rule, rule_idx, accum, cache);
        if r == AutoFillResult::SearchEnded {
            return AutoFillResult::SearchEnded
        } else if r == AutoFillResult::SolutionFound {
            result = r;
        }
    }

    // try filled
    if trial[pos] != AutoFillTile::CrossedOut
        && rule_idx < rule.len()
        && can_begin_fill(trial, pos, rule[rule_idx] as usize) {

        let mut rule_len = rule[rule_idx] as usize;

        for i in 0..rule_len {
            if trial[pos + i] != AutoFillTile::Filled {
                trial[pos + i] = AutoFillTile::CanBeFilled;
            }
        }

        if pos + rule_len < trial.len() {
            if trial[pos + rule_len] != AutoFillTile::CrossedOut {
                trial[pos + rule_len] = AutoFillTile::CanBeCrossedOut;
            }
            rule_len = rule_len + 1;
        }

        let r = try_autofill(
                trial, pos + rule_len, rule, rule_idx + 1, accum, cache);
        if r == AutoFillResult::SearchEnded {
            return AutoFillResult::SearchEnded
        } else if r == AutoFillResult::SolutionFound {
            result = r;
        }
    }

    cache.insert(key, result);
    result
}

fn can_begin_fill(slice: &Vec<AutoFillTile>, pos: usize, len: usize) -> bool {
    (pos + len <= slice.len())
    && slice[pos .. pos + len].iter().all(|&t| t != AutoFillTile::CrossedOut)
    && (pos + len == slice.len()
        || slice[pos + len] != AutoFillTile::Filled)
}

fn combine_solutions(trial: AutoFillTile, accum: AutoFillTile) -> AutoFillTile {
    use self::AutoFillTile::*;
    assert!(trial != NoSolutionFound);

    match accum {
        Filled | CrossedOut | CanBeAnything => accum,

        CanBeFilled =>
            if trial == CanBeCrossedOut {
                CanBeAnything
            } else {
                CanBeFilled
            },

        CanBeCrossedOut =>
            if trial == CanBeFilled {
                CanBeAnything
            } else {
                CanBeCrossedOut
            },

        NoSolutionFound => trial
    }
}

fn is_unassigned_unique_solution(t: AutoFillTile) -> bool {
    t == AutoFillTile::CanBeFilled || t == AutoFillTile::CanBeCrossedOut
}
