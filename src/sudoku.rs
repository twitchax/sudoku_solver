use std::{fmt::{Formatter, Display}};
use once_cell::sync::OnceCell;

static CONSTRAINT_MAP: OnceCell<[[usize; 27]; 81]> = OnceCell::new();

#[derive(PartialEq)]
pub enum SetResult {
    Set,
    NotSet
}

pub struct Sudoku {
    matrix: Vec<Option<u8>>,
    constraint_order: Vec<usize>
}

impl Clone for Sudoku {
    fn clone(&self) -> Self {
        let mut new = self.memberwise_clone();
        new.update_constraint_order();
        new
    }
}

impl Sudoku {
    pub fn new() -> Self {
        // Ensure the constraint map was created.
        Sudoku::ensure_constraint_map();

        let matrix = vec![None; 81];
        let constraint_order = vec![0; 81];
        Self { matrix, constraint_order }
    }

    pub fn from_str(s: &str) -> Self {
        let mut sudoku = Self::new();
        
        // Prepare the matrix.
        for (k, entry_string) in s.split_whitespace().enumerate() {
            assert!(k < 81);

            let entry = match entry_string.parse::<u8>() {
                Ok(i) => Some(i),
                _ => None
            };
            sudoku.unchecked_set_at(k, entry);
        }

        // Order the cell indices by constraint-amount, from most to least.
        sudoku.update_constraint_order();

        sudoku
    }

    pub fn ensure_constraint_map() {
        CONSTRAINT_MAP.get_or_init(|| {
            let mut constraint_map = [[0; 27]; 81];

            for row in 0..9 {
                for col in 0..9 {
                    let sq = 3 * (row / 3) + col / 3;
                    let s = (sq / 3) * 27 + (sq % 3) * 3;

                    let row_range = (row * 9) .. ((row + 1) * 9);
                    let col_range = ( col .. 81 ).step_by(9);
                    let sq_range = ( (s     ) .. (s      + 3) )
                             .chain( (s +  9) .. (s +  9 + 3) )
                             .chain( (s + 18) .. (s + 18 + 3) );

                    
                    let mut total_range = row_range.chain(col_range).chain(sq_range);

                    for k in 0..27 {
                        constraint_map[row * 9 + col][k] = total_range.next().unwrap();
                    }
                }
            }

            constraint_map
        });
    }

    pub fn update_constraint_order(&mut self) {
        let mut index_constraint_pairs = self.matrix.iter().enumerate().map(|(i, _)| {
            let constraint = self.constraint_level_at(i);
            (i, constraint)
        }).collect::<Vec<(usize, u8)>>();

        index_constraint_pairs.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        for k in 0..81 {
            self.constraint_order[k] = index_constraint_pairs[k].0;
        }
    }

    pub fn get(&self, pos: usize) -> Option<u8> {
        let pos = self.constraint_order[pos];
        self.get_at(pos)
    }

    pub fn set(&mut self, pos: usize, value: u8) -> SetResult {
        let pos = self.constraint_order[pos];
        self.set_at(pos, value)
    }

    pub fn erase(&mut self, pos: usize) {
        let pos = self.constraint_order[pos];
        self.erase_at(pos);
    }

    fn get_at(&self, pos: usize) -> Option<u8> {
        assert!(pos < 81);
        self.matrix[pos]
    }

    fn get_row_col(&self, row: usize, col: usize) -> Option<u8> {
        self.get_at(row * 9 + col)
    }

    fn set_at(&mut self, pos: usize, value: u8) -> SetResult {
        if !self.can_place(pos, value) {
            return SetResult::NotSet;
        }

        self.unchecked_set_at(pos, Some(value));
        SetResult::Set
    }

    fn erase_at(&mut self, pos: usize) {
        self.matrix[pos]= None;
    }

    fn can_place(&self, pos: usize, value: u8) -> bool {
        !self.values_from_constraint_map(pos).any(|c| *c == value)
    }

    fn constraint_level_at(&self, pos: usize) -> u8 {
        // If there is already a value in the cell, then make it one of the last cells to look at.
        if self.get_at(pos).is_some() {
            return 0;
        }

        let mut num_count = [0; 9];
        for c in self.values_from_constraint_map(pos).map(|c| *c as usize) {
            num_count[c - 1] = 1;
        }

        num_count.iter().sum()
    }

    fn unchecked_set_at(&mut self, pos: usize, value: Option<u8>) {
        self.matrix[pos] = value;
    }

    fn values_from_indices(&self, indices: impl Iterator<Item = &'static usize>) -> impl Iterator<Item = &u8>
    {
        // The borrow of `self` is "moved" into the closure here so that the closure will not
        // live longer than the 'a of self borrowed here.
        indices
            .map(move |i| self.matrix[*i].as_ref())
            .filter(|c| c.is_some())
            .map(|c| c.unwrap())
    }

    fn values_from_constraint_map(&self, pos: usize) -> impl Iterator<Item = &u8> {
        let constraint_map = CONSTRAINT_MAP.get().unwrap();
        let indices = &constraint_map[pos];

        self.values_from_indices(indices.iter())
    }

    fn memberwise_clone(&self) -> Self {
        Self { matrix: self.matrix.clone(), constraint_order: self.constraint_order.clone() }
    }
}

impl Display for Sudoku {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for row in 0..9 {
            for col in 0..9 {
                let o = match self.get_row_col(row, col) {
                    Some(i) => i,
                    _ => 0
                };

                write!(f, "{} ", o)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}