use std::fmt::{Formatter, Display};

use crate::helpers::Res;

#[derive(PartialEq)]
pub enum SetResult {
    Set,
    NotSet
}

#[derive(Clone)]
pub struct Sudoku {
    matrix: Vec<Option<u8>>
}

impl Sudoku {
    pub fn new() -> Self {
        let matrix = vec![None; 81];
        Self { matrix }
    }

    pub fn from(s: &str) -> Res<Self> {
        let mut sudoku = Self::new();
        
        for (k, entry_string) in s.split_whitespace().enumerate() {
            assert!(k < 81);

            let entry = match entry_string.parse::<u8>() {
                Ok(i) => Some(i),
                _ => None
            };
            sudoku.unchecked_set_at(k, entry);
        }

        Ok(sudoku)
    }

    pub fn _get_at(&self, pos: usize) -> Option<u8> {
        assert!(pos < 81);
        self.matrix[pos]
    }

    pub fn get_at_ref<'a>(&'a self, pos: usize) -> &'a Option<u8> {
        assert!(pos < 81);
        &self.matrix[pos]
    }

    pub fn get_at_mut<'a>(&'a mut self, pos: usize) -> &'a mut Option<u8> {
        assert!(pos < 81);
        &mut self.matrix[pos]
    }

    pub fn _get(&self, row: usize, col: usize) -> Option<u8> {
        self._get_at(row * 9 + col)
    }

    pub fn get_ref<'a>(&'a self, row: usize, col: usize) -> &'a Option<u8> {
        self.get_at_ref(row * 9 + col)
    }

    pub fn get_mut<'a>(&'a mut self, row: usize, col: usize) -> &'a mut Option<u8> {
        self.get_at_mut(row * 9 + col)
    }

    pub fn erase_at(&mut self, pos: usize) {
        *self.get_at_mut(pos) = None;
    }

    pub fn set_at(&mut self, pos: usize, value: u8) -> SetResult {
        let row = pos / 9;
        let col = pos % 9;

        self.set(row, col, value)
    }

    pub fn set(&mut self, row: usize, col: usize, value: u8) -> SetResult {
        if !self.can_place(row, col, value) {
            return SetResult::NotSet;
        }

        self.unchecked_set(row, col, Some(value));
        SetResult::Set
    }

    pub fn can_place(&self, row: usize, col: usize, value: u8) -> bool {
        let sq = 3 * (row / 3) + col / 3;
        let mut all_clashes = self.iter_row(row).chain(self.iter_col(col)).chain(self.iter_sq(sq));

        !all_clashes.any(|c| *c == value)
    }

    pub fn iter_row<'a>(&'a self, row: usize) -> impl Iterator<Item = &'a u8> {
        let indices = (row * 9) .. ((row + 1) * 9);

        self.values_from_indices(indices)
    }

    pub fn iter_col<'a>(&'a self, col: usize) -> impl Iterator<Item = &'a u8> {
        let indices = ( col .. 81 ).step_by(9);

        self.values_from_indices(indices)
    }

    pub fn iter_sq<'a>(&'a self, sq: usize) -> impl Iterator<Item = &'a u8> {
        let s = (sq / 3) * 27 + (sq % 3) * 3;
        let indices = 
                  ( (s +  0) .. (s +  0 + 3) )
            .chain( (s +  9) .. (s +  9 + 3) )
            .chain( (s + 18) .. (s + 18 + 3) );

        self.values_from_indices(indices)
    }

    fn unchecked_set_at(&mut self, pos: usize, value: Option<u8>) {
        *self.get_at_mut(pos) = value;
    }

    fn unchecked_set(&mut self, row: usize, col: usize, value: Option<u8>) {
        *self.get_mut(row, col) = value;
    }

    fn values_from_indices<'a>(&'a self, indices: impl Iterator<Item = usize>) -> impl Iterator<Item = &'a u8> {
        // The borrow of `self` is "moved" into the closure here so that the closure will not
        // live longer than the 'a of self borrowed here.
        indices
            .map(move |i| self.matrix[i].as_ref())
            .filter(|c| c != &None)
            .map(|c| c.unwrap())
    }
}

impl Display for Sudoku {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for row in 0..9 {
            for col in 0..9 {
                let o = match self.get_ref(row, col) {
                    Some(i) => *i,
                    _ => 0
                };

                write!(f, "{} ", o)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}