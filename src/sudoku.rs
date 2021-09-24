use std::{fmt::{Formatter, Display}};
use log::{debug, info};
use once_cell::sync::Lazy;

// TODO:
//   * Add a "constraint level" member to the map.
//   * Update constraint level for only the cells that would change when a value is set.
//   * Fix sort to merely swap the newly set constraint level with "one less than" the new level of that cell.

#[derive(PartialEq)]
pub enum SetResult {
    Set,
    NotSet
}

type SudokuMatrix = [u8; 81];
type ConstraintTable = [[u8; 10]; 9]; // NOTE: This has an inner size of 16 to allow for SIMD u8x16; the space difference is negligible.
type RowColSqMap = [(usize, usize, usize); 81];
type AreaToIndexArrayMap = [[usize; 9]; 9];

use rayon::prelude::*;
use core_simd::*;

// [ARoney] NOTE: The inner arrays for the constraint table are 10 since the arrays likely have to be aligned anyway.
// That way, no need to compute the correct entry with a `-1` every time.
pub struct Sudoku {
    matrix: SudokuMatrix,
    constraint_order: SudokuMatrix,
    constraint_level: SudokuMatrix,
    available_values: ConstraintTable,
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
        let matrix = [0; 81];
        let constraint_order = [0; 81];
        let constraint_level = [0; 81];

        let available_values = ConstraintTable::default();

        let row_constraint_table = ConstraintTable::default();
        let col_constraint_table = ConstraintTable::default();
        let sq_constraint_table = ConstraintTable::default();
 
        let mut sudoku = Self { matrix, constraint_order, constraint_level, available_values, row_constraint_table, col_constraint_table, sq_constraint_table };
        
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
        sudoku.compute_original_constraint_level();
        sudoku.update_constraint_order();

        sudoku
    }

    fn compute_original_constraint_level(&mut self) {
        // for (k, _) in self.matrix.iter().enumerate() {
        //     self.constraint_level[k] = self.constraint_level_at(k)
        // }
    }

    pub fn update_constraint_order(&mut self) {
        let mut index_constraint_pairs = [(0u8, 0u8); 81];
        self.matrix.iter().enumerate().map(|(i, _)| {
            let constraint = self.constraint_level_at(i);
            (i as u8, constraint)
        }).for_each(|p| index_constraint_pairs[p.0 as usize] = p);

        // let mut index_constraint_pairs = [(0u8, 0u8); 81];
        // self.constraint_level.iter().enumerate().map(|(i, entry)| (i, entry))
        //     .for_each(|p| index_constraint_pairs[p.0] = (p.0 as u8, *p.1));

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

        self.row_constraint_table[row][num] != 1 && self.col_constraint_table[col][num] != 1 && self.sq_constraint_table[sq][num] != 1
    }

    fn unchecked_set_at(&mut self, pos: usize, value: u8) {
        let location = &mut self.matrix[pos];
        let (row, col, sq) = ROW_COL_SQ_MAP[pos];

        // If there was an old value, clean up the table items for that cell.
        if *location > 0 {
            let num = *location as usize;

            self.row_constraint_table[row][num] = 0;
            self.col_constraint_table[col][num] = 0;
            self.sq_constraint_table[sq][num] = 0;
        }
        
        // If there is a new value, set the table items for that cell.
        if value > 0 {
            let num = value as usize;

            self.row_constraint_table[row][num] = 1;
            self.col_constraint_table[col][num] = 1;
            self.sq_constraint_table[sq][num] = 1;
        }

        *location = value;
    }

    fn constraint_level_at(&self, pos: usize) -> u8 {
        // If there is already a value in the cell, then make it one of the last cells to look at.
        if self.get_at(pos) > 0 {
            return 0;
        }

        let (row, col, sq) = ROW_COL_SQ_MAP[pos];

        let s = u8x16::splat_apply(0, self.sq_constraint_table[sq]);
        let r = u8x16::splat_apply(0, self.row_constraint_table[row]);
        let c = u8x16::splat_apply(0, self.col_constraint_table[col]);
        
        (s | r | c).horizontal_sum()
    }

    fn memberwise_clone(&self) -> Self {
        Self { matrix: self.matrix, constraint_order: self.constraint_order, constraint_level: self.constraint_level, available_values: self.available_values, row_constraint_table: self.row_constraint_table, col_constraint_table: self.col_constraint_table, sq_constraint_table: self.sq_constraint_table }
    }
}

trait SplatApply<T, const LANES: usize>
where
    T: SimdElement,
    LaneCount<LANES>: SupportedLaneCount
{
    fn splat_apply<const L: usize>(default: T, array: [T; L]) -> Self;
}

impl<T, const LANES: usize> SplatApply<T, LANES> for Simd<T, LANES>
where
    T: SimdElement,
    LaneCount<LANES>: SupportedLaneCount
{
    #[inline(always)]
    fn splat_apply<const L: usize>(default: T, array: [T; L]) -> Self {
        let mut result = Self::splat(default);
        array.iter().enumerate().for_each(|(i, &v)| result[i] = v);

        result
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

/// Static map from sudoku global position to row, col, and square positions.
static ROW_COL_SQ_MAP: Lazy<RowColSqMap> = Lazy::new(|| {
    let mut row_col_sq_map = [(0, 0, 0); 81];

    for (k, v) in row_col_sq_map.iter_mut().enumerate() {
        let row = k / 9;
        let col = k % 9;
        let sq = 3 * (row / 3) + col / 3;

        *v = (row, col, sq);
    }

    row_col_sq_map
});

// static ROW_INDEXES_MAP: Lazy<AreaToIndexArrayMap> = Lazy::new(|| {
//     let mut row_index_map = [[0; 9]; 9];

//     for (k, v) in row_index_map.iter_mut().enumerate() {
//         let indexes = (k * 9 .. (k + 1) * 9).collect::<Vec<_>>().try_into().expect("Could not concert Vec to array.");

//         *v = indexes;
//     }

//     row_index_map
// });

// static COL_INDEXES_MAP: Lazy<AreaToIndexArrayMap> = Lazy::new(|| {
//     let mut col_index_map = [[0; 9]; 9];

//     for (k, v) in col_index_map.iter_mut().enumerate() {
//         let indexes = (k .. k + 72).step_by(9).collect::<Vec<_>>().try_into().expect("Could not concert Vec to array.");

//         *v = indexes;
//     }

//     col_index_map
// });

// static SQ_INDEXES_MAP: Lazy<AreaToIndexArrayMap> = Lazy::new(|| {
//     let mut row_index_map = [[0; 9]; 9];

//     for (k, v) in row_index_map.iter_mut().enumerate() {
//         let upper_left_index = (k / 3) * 27 + (k % 3) * 3;


//         let top = (k * 9 .. (k + 1) * 9).collect::<Vec<_>>().try_into().expect("Could not concert Vec to array.");


//         let indexes = .step_by(9).collect::<Vec<_>>().try_into().expect("Could not concert Vec to array.");

//         *v = indexes;
//     }

//     row_index_map
// });