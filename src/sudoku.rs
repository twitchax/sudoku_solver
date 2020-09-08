use std::{fmt::{Formatter, Display}};
use once_cell::sync::Lazy;

#[derive(PartialEq)]
pub enum SetResult {
    Set,
    NotSet
}

type SudokuMatrix = Box<[u8; 81]>;
type ConstraintTable = Box<[[bool; 10]; 9]>;
type RowColSqArray = [(usize, usize, usize); 81];

// [ARoney] NOTE: The inner arrays for the constraint table are 10 since the arrays likely have to be aligned anyway.
// That way, no need to compute the correct entry with a `-1` every time.
pub struct Sudoku {
    matrix: SudokuMatrix,
    constraint_order: SudokuMatrix,
    row_constraint_table: ConstraintTable,
    col_constraint_table: ConstraintTable,
    sq_constraint_table: ConstraintTable,
}

impl Clone for Sudoku {
    fn clone(&self) -> Self {
        let mut new = self.memberwise_clone();
        new.update_constraint_order();
        new
    }
}

impl Sudoku {
    pub fn from_str(s: &str) -> Self {
        let matrix = Box::new([0; 81]);
        let constraint_order = Box::new([0; 81]);

        let row_constraint_table = Box::new([[false; 10]; 9]);
        let col_constraint_table = Box::new([[false; 10]; 9]);
        let sq_constraint_table = Box::new([[false; 10]; 9]);

        let mut sudoku = Self { matrix, constraint_order, row_constraint_table, col_constraint_table, sq_constraint_table };
        
        // Prepare the matrix.
        for (k, entry_string) in s.split_whitespace().enumerate() {
            assert!(k < 81);

            let entry = match entry_string.parse::<u8>() {
                Ok(i) => i,
                _ => 0
            };

            sudoku.unchecked_set_at(k, entry);
        }

        // Order the cell indices by constraint-amount, from most to least.
        sudoku.update_constraint_order();

        sudoku
    }

    pub fn update_constraint_order(&mut self) {
        let mut index_constraint_pairs = self.matrix.iter().enumerate().map(|(i, _)| {
            let constraint = self.constraint_level_at(i);
            (i as u8, constraint)
        }).collect::<Vec<(u8, u8)>>();

        index_constraint_pairs.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        for (k, v) in self.constraint_order.iter_mut().enumerate() {
            *v = index_constraint_pairs[k].0;
        }
    }

    pub fn _get(&self, pos: usize) -> u8 {
        let pos = self.constraint_order[pos] as usize;
        self.get_at(pos)
    }

    pub fn has_value(&self, pos: usize) -> bool {
        let pos = self.constraint_order[pos] as usize;
        self.get_at(pos) > 0
    }

    pub fn set(&mut self, pos: usize, value: u8) -> SetResult {
        let pos = self.constraint_order[pos] as usize;
        self.set_at(pos, value)
    }

    pub fn erase(&mut self, pos: usize) {
        let pos = self.constraint_order[pos] as usize;
        self.unchecked_set_at(pos, 0);
    }

    fn get_at(&self, pos: usize) -> u8 {
        assert!(pos < 81);
        self.matrix[pos]
    }

    fn get_row_col(&self, row: usize, col: usize) -> u8 {
        self.get_at(row * 9 + col)
    }

    fn set_at(&mut self, pos: usize, value: u8) -> SetResult {
        if !self.can_place(pos, value) {
            return SetResult::NotSet;
        }

        self.unchecked_set_at(pos, value);
        SetResult::Set
    }

    fn can_place(&self, pos: usize, value: u8) -> bool {
        let (row, col, sq) = ROW_COL_SQ_MAP[pos];

        let num = value as usize;

        !self.row_constraint_table[row][num] && !self.col_constraint_table[col][num] && !self.sq_constraint_table[sq][num]
    }

    fn unchecked_set_at(&mut self, pos: usize, value: u8) {
        let location = &mut self.matrix[pos];
        // If there was an old value, clean up the table items for that cell.
        if *location > 0 {
            let (row, col, sq) = ROW_COL_SQ_MAP[pos];
            let num = *location as usize;

            self.row_constraint_table[row][num] = false;
            self.col_constraint_table[col][num] = false;
            self.sq_constraint_table[sq][num] = false;
        }
        
        // If there is a new value, set the table items for that cell.
        if value > 0 {
            let (row, col, sq) = ROW_COL_SQ_MAP[pos];
            let num = value as usize;

            self.row_constraint_table[row][num] = true;
            self.col_constraint_table[col][num] = true;
            self.sq_constraint_table[sq][num] = true;
        }

        *location = value;
    }

    fn constraint_level_at(&self, pos: usize) -> u8 {
        // If there is already a value in the cell, then make it one of the last cells to look at.
        if self.get_at(pos) > 0 {
            return 0;
        }

        let (row, col, sq) = ROW_COL_SQ_MAP[pos];
        
        let mut sum = 0;
        for k in 1..10 {
            if self.row_constraint_table[row][k] || self.col_constraint_table[col][k] || self.sq_constraint_table[sq][k] {
                sum += 1;
            }
        }
        
        sum
    }

    fn memberwise_clone(&self) -> Self {
        Self { matrix: self.matrix.clone(), constraint_order: self.constraint_order.clone(), row_constraint_table: self.row_constraint_table.clone(), col_constraint_table: self.col_constraint_table.clone(), sq_constraint_table: self.sq_constraint_table.clone() }
    }
}

impl Display for Sudoku {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for row in 0..9 {
            for col in 0..9 {
                write!(f, "{} ", self.get_row_col(row, col))?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

static ROW_COL_SQ_MAP: Lazy<RowColSqArray> = Lazy::new(|| {
    let mut row_col_sq_map = [(0, 0, 0); 81];

    for (k, v) in row_col_sq_map.iter_mut().enumerate() {
        let row = k / 9;
        let col = k % 9;
        let sq = 3 * (row / 3) + col / 3;

        *v = (row, col, sq);
    }

    row_col_sq_map
});