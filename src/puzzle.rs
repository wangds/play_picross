// puzzle.rs

use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::num;
use std::path::Path;

use board::Board;

type History = Vec<Board>;
pub type Rule = Vec<u32>;
pub type Rules<'a> = (&'a Vec<Rule>, &'a Vec<Rule>);

// PuzzleReaderResult(width, height, col_rules, row_rules)
type PuzzleReaderResult = (usize, usize, Vec<Rule>, Vec<Rule>);

#[derive(Debug)]
enum PuzzleReaderError {
    Io(io::Error),
    ParseInt(num::ParseIntError),

    ParseError(String, Box<PuzzleReaderError>),

    InvalidDimensions(usize, usize),
    IncompletePuzzle,
    TooManyRules,
    RuleTooLong,
}

pub struct Puzzle {
    width: usize,
    height: usize,
    col_rules: Vec<Rule>,
    row_rules: Vec<Rule>,
    history: History,
    curr_history: usize
}

impl Puzzle {
    pub fn new(width: usize, height: usize) -> Puzzle {
        assert!(width > 0 && height > 0);
        let col_rules = vec![Vec::new(); width];
        let row_rules = vec![Vec::new(); height];

        Puzzle::new_with_rules(width, height, col_rules, row_rules)
    }

    fn new_with_rules(width: usize, height: usize,
            col_rules: Vec<Rule>, row_rules: Vec<Rule>) -> Puzzle {
        assert!(width > 0 && height > 0);
        assert!(col_rules.len() == width && row_rules.len() == height);
        let mut h: History = Vec::new();
        let b = Board::new(width, height);

        h.push(b);

        Puzzle {
            width: width,
            height: height,
            col_rules: col_rules,
            row_rules: row_rules,
            history: h,
            curr_history: 0
        }
    }

    pub fn load_file(filename: &String) -> Option<Puzzle> {
        match read_file(filename) {
            Ok((width, height, col_rules, row_rules)) =>
                Some(Puzzle::new_with_rules(width, height, col_rules, row_rules)),

            Err(e) => {
                println!("{}: {}", filename, e);
                None
            }
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

fn read_file(filename: &String) -> Result<PuzzleReaderResult, PuzzleReaderError> {
    let path = Path::new(filename);
    let file = try!(File::open(path));
    let reader = BufReader::new(file);
    let mut dim: Option<(usize,usize)> = None;
    let mut row_rules = Vec::new();
    let mut col_rules = Vec::new();

    for line in reader.lines() {
        if let Err(e) = line {
            return Err(PuzzleReaderError::Io(e))
        }

        let ln = line.unwrap();
        if ln.is_empty() || ln.starts_with("#") {
            continue
        }

        if dim.is_none() {
            let ws: Vec<&str> = ln.splitn(2, 'x').collect();
            if ws.len() == 2 {
                let possible_w = ws[0].trim().parse::<usize>();
                let possible_h = ws[1].trim().parse::<usize>();
                if possible_w.is_ok() && possible_h.is_ok() {
                    let w = possible_w.unwrap();
                    let h = possible_h.unwrap();

                    if w <= 0 || h <= 0 {
                        return Err(PuzzleReaderError::InvalidDimensions(w, h))
                    }

                    dim = Some((w, h));
                }
                if dim.is_none() {
                    break
                }
            }
        } else {
            let (width, height) = dim.unwrap();

            let (max_value, max_elements) =
                if row_rules.len() < height {
                    (width, height)
                } else {
                    (height, width)
                };

            // Add some context.
            let rules =
                try!(read_rules(&ln, max_value, max_elements).map_err(|e|
                        PuzzleReaderError::ParseError(ln.clone(), Box::new(e))));

            if !rules.is_empty() {
                if row_rules.len() < height {
                    row_rules.push(rules);
                } else {
                    col_rules.push(rules);
                }
            }

            if col_rules.len() >= height {
                break
            }
        }
    }

    if let Some((width, height)) = dim {
        if row_rules.len() == height && col_rules.len() == width {
            return Ok((width, height, col_rules, row_rules))
        }
    }

    Err(PuzzleReaderError::IncompletePuzzle)
}

fn read_rules(ln: &String, max_value: usize, max_elements: usize)
    -> Result<Rule, PuzzleReaderError>
{
    let maybe_vs: Vec<&str> = ln.split_whitespace().collect();
    let mut rules = Vec::new();
    let mut sum = 0;

    for maybe_v in maybe_vs.iter() {
        let v = try!(maybe_v.parse::<u32>());
        rules.push(v);
        sum = sum + v;
    }

    if rules.len() == 0 {
        // do not fail for lines with no numbers
        return Ok(rules)
    }

    if rules.len() * 2 - 1 > max_elements {
        return Err(PuzzleReaderError::TooManyRules)
    }

    sum = sum + (rules.len() as u32 - 1);
    if sum > max_value as u32 {
        return Err(PuzzleReaderError::RuleTooLong)
    }

    Ok(rules)
}

impl fmt::Display for PuzzleReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PuzzleReaderError::Io(ref err) =>
                write!(f, "IO error: {}", err),

            PuzzleReaderError::ParseInt(ref err) =>
                write!(f, "Parse error: {}", err),

            PuzzleReaderError::ParseError(ref ln, ref err) =>
                write!(f, "Error on line '{}': {}", ln, err),

            PuzzleReaderError::InvalidDimensions(w, h) =>
                write!(f, "Invalid dimensions ({} x {})", w, h),

            PuzzleReaderError::IncompletePuzzle
            | PuzzleReaderError::RuleTooLong
            | PuzzleReaderError::TooManyRules =>
                write!(f, "{}", error::Error::description(self)),
        }
    }
}

impl error::Error for PuzzleReaderError {
    fn description(&self) -> &str {
        match *self {
            PuzzleReaderError::Io(ref err) =>
                err.description(),

            PuzzleReaderError::ParseInt(ref err) =>
                err.description(),

            PuzzleReaderError::ParseError(..) =>
                "Error parsing line",

            PuzzleReaderError::InvalidDimensions(..) =>
                "Invalid dimensions",

            PuzzleReaderError::IncompletePuzzle =>
                "Puzzle incomplete",

            PuzzleReaderError::TooManyRules =>
                "Too many rules for board dimensions",

            PuzzleReaderError::RuleTooLong =>
                "Rule length exceeds for board dimensions"
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            PuzzleReaderError::Io(ref err) => Some(err),
            PuzzleReaderError::ParseInt(ref err) => Some(err),

            PuzzleReaderError::ParseError(_, ref box_err) =>
                Some(&**box_err),

            PuzzleReaderError::InvalidDimensions(..)
            | PuzzleReaderError::IncompletePuzzle
            | PuzzleReaderError::TooManyRules
            | PuzzleReaderError::RuleTooLong => None
        }
    }
}

impl From<io::Error> for PuzzleReaderError {
    fn from(err: io::Error) -> PuzzleReaderError {
        PuzzleReaderError::Io(err)
    }
}

impl From<num::ParseIntError> for PuzzleReaderError {
    fn from(err: num::ParseIntError) -> PuzzleReaderError {
        PuzzleReaderError::ParseInt(err)
    }
}
