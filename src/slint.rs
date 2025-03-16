//! Convenient, boilerplate can be moved out to `*.slint` files, but requires to build those, and not too flexible.
//!
//! Copyright (C) 2025  Vladislav Sukhmel
//!
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program.  If not, see <http://www.gnu.org/licenses/>.

use slint::platform::PointerEventButton;
use slint::private_unstable_api::re_exports::PointerEventKind;
use slint::{Color, Model, PlatformError};

use crate::{Colour, SudokuModel};

slint::include_modules!();

impl From<Colour> for Color {
    fn from(value: Colour) -> Self {
        match value {
            Colour::Black => Color::from_rgb_u8(64, 64, 64),
            Colour::Red => Color::from_rgb_u8(128, 32, 32),
            Colour::Green => Color::from_rgb_u8(32, 128, 32),
        }
    }
}

pub fn main(mut sudoku_model: SudokuModel) -> Result<(), PlatformError> {
    let ui = MainWindow::new()?;
    let tiles = (0..9)
        .flat_map(|y| {
            (0..9).map(move |x| TileData {
                color: sudoku_model.colour(x, y).into(),
                enabled: sudoku_model.get(x, y).enabled,
                text: sudoku_model.text(x, y).into(),
            })
        })
        .collect::<Vec<_>>();
    let tiles_model = std::rc::Rc::new(slint::VecModel::from(tiles));

    ui.set_tiles(tiles_model.clone().into());

    ui.on_click(move |event, x, y| {
        // info!(?event);
        let value = match event.kind {
            PointerEventKind::Up => match event.button {
                PointerEventButton::Left => 1,
                PointerEventButton::Right => -1,
                _ => return,
            },
            _ => return,
        };
        let x = x as usize;
        let y = y as usize;
        sudoku_model.add(x, y, value);
        for x in 0..9 {
            for y in 0..9 {
                tiles_model.set_row_data(
                    x + 9 * y,
                    TileData {
                        color: sudoku_model.colour(x, y).into(),
                        enabled: sudoku_model.get(x, y).enabled,
                        text: sudoku_model.text(x, y).into(),
                    },
                );
            }
        }
    });

    ui.run()?;

    Ok(())
}
