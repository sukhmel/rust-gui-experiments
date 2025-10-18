use std::collections::HashSet;

#[cfg(feature = "floem")]
pub mod floem;

#[cfg(feature = "iced")]
pub mod iced;

#[cfg(feature = "slint")]
pub mod slint;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "egui")]
pub mod egui;

#[cfg(feature = "gpui")]
pub mod gpui;

#[cfg(feature = "xilem")]
pub mod xilem;

#[cfg(feature = "leptos")]
pub mod leptos;

#[cfg(feature = "rui")]
pub mod rui;

#[cfg(feature = "ratatui")]
pub mod ratatui;

#[cfg(feature = "kas")]
pub mod kas;

#[derive(Debug, Clone, Copy)]
pub struct SudokuValue {
    value: u8,
    enabled: bool,
}

const VALUES: [&str; 10] = [" ", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
impl SudokuValue {
    pub fn text(&self) -> &'static str {
        VALUES[self.value as usize]
    }
}

impl Default for SudokuValue {
    fn default() -> Self {
        Self {
            value: 0,
            enabled: true,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SudokuCell {
    values: [[SudokuValue; 3]; 3],
}

impl SudokuCell {
    pub fn new() -> SudokuCell {
        SudokuCell::default()
    }
}

impl From<[[u8; 9]; 9]> for SudokuModel {
    fn from(value: [[u8; 9]; 9]) -> Self {
        let mut result = SudokuModel::default();
        for (x, col) in value.iter().enumerate() {
            for (y, num) in col.iter().enumerate() {
                if *num != 0 {
                    result.set(x, y, *num);
                    result.set_enabled(x, y, false);
                }
            }
        }
        result
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct SudokuModel {
    cells: [[SudokuCell; 3]; 3],
}

#[derive(Debug, Clone, Copy)]
pub enum Colour {
    Black,
    Red,
    Green,
}

impl SudokuModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn example() -> Self {
        Self::from([
            [1, 6, 7, 8, 9, 2, 3, 4, 5],
            [4, 2, 8, 0, 0, 0, 0, 0, 0],
            [5, 9, 3, 0, 0, 0, 0, 0, 0],
            [6, 0, 0, 4, 0, 0, 0, 0, 0],
            [7, 0, 0, 0, 5, 0, 0, 0, 0],
            [8, 0, 0, 0, 0, 6, 0, 0, 0],
            [9, 0, 0, 0, 0, 0, 7, 0, 0],
            [2, 0, 0, 0, 0, 0, 0, 8, 0],
            [3, 0, 0, 0, 0, 0, 0, 0, 9],
        ])
    }

    pub fn text(&self, x: usize, y: usize) -> &str {
        self.get(x, y).text()
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut SudokuValue {
        let top_x = x / 3;
        let top_y = y / 3;
        let cell_x = x % 3;
        let cell_y = y % 3;
        &mut self.cells[top_x][top_y].values[cell_x][cell_y]
    }

    pub fn get(&self, x: usize, y: usize) -> &SudokuValue {
        let top_x = x / 3;
        let top_y = y / 3;
        let cell_x = x % 3;
        let cell_y = y % 3;
        &self.cells[top_x][top_y].values[cell_x][cell_y]
    }

    pub fn set(&mut self, x: usize, y: usize, value: u8) {
        let target = self.get_mut(x, y);
        if target.enabled {
            target.value = if value == u8::MAX { 9 } else { value % 10 };
        }
    }

    pub fn set_enabled(&mut self, x: usize, y: usize, enabled: bool) {
        let target = self.get_mut(x, y);
        target.enabled = enabled;
    }

    pub fn add(&mut self, x: usize, y: usize, value: i8) {
        self.set(x, y, self.get(x, y).value.wrapping_add_signed(value));
    }

    pub fn colour(&self, x: usize, y: usize) -> Colour {
        let top_x = x / 3;
        let top_y = y / 3;
        let cell_x = x % 3;
        let cell_y = y % 3;
        let cell = &self.cells[top_x][top_y];
        let target = cell.values[cell_x][cell_y].value;
        let mut cell_values = HashSet::<u8>::from_iter(1..=9);
        let mut row_values = HashSet::<u8>::from_iter(1..=9);
        let mut col_values = HashSet::<u8>::from_iter(1..=9);
        if target != 0 {
            for lookup_x in 0..3 {
                for lookup_y in 0..3 {
                    let value = cell.values[lookup_x][lookup_y].value;
                    cell_values.remove(&value);
                    if lookup_x == cell_x && lookup_y == cell_y {
                        continue;
                    }
                    if target == value {
                        return Colour::Red;
                    }
                }
            }
            for lookup_x in 0..9 {
                let value = self.get(lookup_x, y).value;
                row_values.remove(&value);
                if x != lookup_x && target == value {
                    return Colour::Red;
                }
            }
            for lookup_y in 0..9 {
                let value = self.get(x, lookup_y).value;
                col_values.remove(&value);
                if y != lookup_y && target == value {
                    return Colour::Red;
                }
            }
        }
        if cell_values.is_empty() || row_values.is_empty() || col_values.is_empty() {
            Colour::Green
        } else {
            Colour::Black
        }
    }
}
