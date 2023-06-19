use std::{collections::HashSet, fmt::{Formatter, Display}};
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
type ConstraintTable = [[u8; 16]; 9]; // NOTE: This has an inner size of 16 to allow for SIMD u8x16; the space difference is negligible.
type RowColSqMap = [(usize, usize, usize); 81];
type AreaToIndexArrayMap = [[usize; 9]; 9];

use rayon::prelude::*;
use core_simd::*;

// [ARoney] NOTE: The inner arrays for the constraint table are 10 since the arrays likely have to be aligned anyway.
// That way, no need to compute the correct entry with a `-1` every time.
#[derive(Debug, Clone)]
pub struct Sudoku {
    pub matrix: SudokuMatrix,
    pub constraint_order: SudokuMatrix,
    pub constraint_level: SudokuMatrix,
    pub available_values: ConstraintTable,
    pub row_constraint_table: ConstraintTable,
    pub col_constraint_table: ConstraintTable,
    pub sq_constraint_table: ConstraintTable,
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
        sudoku.update_constraint_order();

        sudoku
    }

    pub fn update_constraint_order(&mut self) {
        // Create some pairs for sorting.
        let mut index_constraint_pairs = [(0u8, 0u8); 81];
        self.constraint_level.iter().enumerate()
            .for_each(|(i, entry)| index_constraint_pairs[i] = (i as u8, *entry));

        // Sort the pairs.
        index_constraint_pairs.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        // Updated the constraint order vector.
        for (k, v) in self.constraint_order.iter_mut().enumerate() {
            *v = index_constraint_pairs[k].0;
        }
    }

    pub fn get_max_constraint_pos(&self) -> usize {
        let mut index = 0;
        let mut max = 0;

        for k in 0..81 {
            let level = self.constraint_level[k];
            if level == 8 {
                return k;
            } else if level > max {
                index = k;
                max = level;
            }
        }
        
        index
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

    pub fn unchecked_set(&mut self, pos: usize, value: u8) {
        let pos = self.constraint_order[pos] as usize;
        self.unchecked_set_at(pos, value);
    }

    pub fn can_place(&mut self, pos: usize, value: u8) -> bool {
        let pos = self.constraint_order[pos] as usize;
        self.can_place_at(pos, value)
    }

    pub fn can_place_what(&mut self, pos: usize) -> [u8; 16] {
        let pos = self.constraint_order[pos] as usize;
        self.can_place_what_at(pos)
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
        if !self.can_place_at(pos, value) {
            return SetResult::NotSet;
        }

        self.unchecked_set_at(pos, value);
        SetResult::Set
    }

    fn can_place_at(&self, pos: usize, value: u8) -> bool {
        let (row, col, sq) = ROW_COL_SQ_MAP[pos];

        let num = value as usize;

        self.row_constraint_table[row][num] != 1 && self.col_constraint_table[col][num] != 1 && self.sq_constraint_table[sq][num] != 1
    }

    fn can_place_what_at(&self, pos: usize) -> [u8; 16] {
        let (row, col, sq) = ROW_COL_SQ_MAP[pos];

        let max_less_one = u8x16::splat(254);
        
        let s = u8x16::from_array(self.sq_constraint_table[sq]);
        let r = u8x16::from_array(self.row_constraint_table[row]);
        let c = u8x16::from_array(self.col_constraint_table[col]);

        let lane_nums = u8x16::from_array([0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8]);

        let result = (!(s | r | c) - max_less_one) * lane_nums;

        result.to_array()
    }

    fn unchecked_set_at(&mut self, pos: usize, value: u8) {
        let (row, col, sq) = ROW_COL_SQ_MAP[pos];
        
        // If there is a new value, set the constraint table items for that cell.
        if value > 0 {
            let num = value as usize;

            self.row_constraint_table[row][num] = 1;
            self.col_constraint_table[col][num] = 1;
            self.sq_constraint_table[sq][num] = 1;
        }

        // Update the value in the sudoku.
        self.matrix[pos] = value;
        
        // Update the constraint table items for the affected cells.
        for i in ROW_INDEXES_MAP[row] {
            self.constraint_level[i] = self.constraint_level_at(i);
        }

        for i in COL_INDEXES_MAP[col] {
            self.constraint_level[i] = self.constraint_level_at(i);
        }

        for i in SQ_INDEXES_MAP[sq] {
            self.constraint_level[i] = self.constraint_level_at(i);
        }
    }

    fn constraint_level_at(&self, pos: usize) -> u8 {
        // If there is already a value in the cell, then make it one of the last cells to look at.
        if self.get_at(pos) > 0 {
            return 0;
        }

        let (row, col, sq) = ROW_COL_SQ_MAP[pos];

        let s = u8x16::from_array(self.sq_constraint_table[sq]);
        let r = u8x16::from_array(self.row_constraint_table[row]);
        let c = u8x16::from_array(self.col_constraint_table[col]);
        
        (s | r | c).horizontal_sum()
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

static ROW_INDEXES_MAP: Lazy<AreaToIndexArrayMap> = Lazy::new(|| {
    let mut row_index_map = [[0; 9]; 9];

    for (k, v) in row_index_map.iter_mut().enumerate() {
        let indexes = (k * 9 .. (k + 1) * 9).collect::<Vec<_>>().try_into().expect("Could not convert Vec to array.");

        *v = indexes;
    }

    row_index_map
});

static COL_INDEXES_MAP: Lazy<AreaToIndexArrayMap> = Lazy::new(|| {
    let mut col_index_map = [[0; 9]; 9];

    for (k, v) in col_index_map.iter_mut().enumerate() {
        let indexes = (k .. k + 73).step_by(9).collect::<Vec<_>>().try_into().expect("Could not concert Vec to array.");

        *v = indexes;
    }

    col_index_map
});

static SQ_INDEXES_MAP: Lazy<AreaToIndexArrayMap> = Lazy::new(|| {
    let mut row_index_map = [[0; 9]; 9];

    for (k, v) in row_index_map.iter_mut().enumerate() {
        let upper_left_index = (k / 3) * 27 + (k % 3) * 3;

        let top: [usize; 3] = (upper_left_index .. upper_left_index + 3).collect::<Vec<_>>().try_into().expect("Could not convert Vec to array.");
        let middle: [usize; 3] = (upper_left_index + 9 .. upper_left_index + 9 + 3).collect::<Vec<_>>().try_into().expect("Could not convert Vec to array.");
        let bottom: [usize; 3] = (upper_left_index + 18 .. upper_left_index + 18 + 3).collect::<Vec<_>>().try_into().expect("Could not convert Vec to array.");

        let indexes = top.into_iter().chain(middle.into_iter()).chain(bottom.into_iter()).collect::<Vec<_>>().try_into().expect("Could not convert Vec to array.");

        *v = indexes;
    }

    row_index_map
});