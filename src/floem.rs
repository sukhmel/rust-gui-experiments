//! Seems most flexible, but requires to attach signals to update everything.
//!
//! On macOS, the window icon doesn't seem to work, and closing the window does not finish the app.
//! There will be `exit_on_close` in future floem versions to fix this.

use std::array;
use std::cell::RefCell;

use floem::IntoView;
use floem::event::EventPropagation;
use floem::kurbo::Size;
use floem::peniko::Color;
use floem::prelude::{RwSignal, button, h_stack_from_iter, v_stack_from_iter};
use floem::reactive::{SignalGet, SignalUpdate, create_signal, create_updater};
use floem::style::StyleValue;
use floem::views::Decorators;
use floem::window::{Icon, WindowConfig};
use itertools::Itertools;

use crate::{Colour, SudokuModel};

pub fn main(sudoku_model: SudokuModel) {
    let icon = image::ImageReader::open("www/favicon.png")
        .unwrap()
        .decode()
        .unwrap()
        .to_rgba8();
    let window_config = WindowConfig::default()
        .window_icon(Icon::from_rgba(icon.to_vec(), icon.width(), icon.height()).unwrap())
        .title("Sudoku")
        .size(Size {
            width: 585.0,
            height: 585.0,
        });
    floem::Application::new()
        .window(move |_app| sudoku_model.into_view(), Some(window_config))
        .run();
}

impl SudokuModel {
    fn into_view(self) -> impl IntoView {
        let colours = array::from_fn::<_, 9, _>(|x| {
            array::from_fn::<_, 9, _>(|y| RwSignal::<Colour>::new(self.colour(x, y)))
        });
        let text = array::from_fn::<_, 9, _>(|x| {
            array::from_fn::<_, 9, _>(|y| RwSignal::<String>::new(self.text(x, y).to_string()))
        });
        let enabled = array::from_fn::<_, 9, _>(|x| {
            array::from_fn::<_, 9, _>(|y| RwSignal::<bool>::new(self.get(x, y).enabled))
        });
        let (on_click, click) = create_signal((0usize, 0usize, 0i8));
        let sudoku = RefCell::new(self);
        create_updater(
            move || on_click.get(),
            move |(x, y, v)| {
                sudoku.borrow_mut().add(x, y, v);
                text[x][y].set(sudoku.borrow().get(x, y).text().to_string());
                for x in 0..9 {
                    for y in 0..9 {
                        colours[x][y].set(sudoku.borrow().colour(x, y))
                    }
                }
            },
        );
        let buttons: Vec<Vec<_>> = (0..9)
            .map(|y| {
                (0..9)
                    .map(|x| {
                        let button = button(text[x][y]);

                        button
                            .action(move || click.set((x, y, 1)))
                            .on_secondary_click(move |_| {
                                click.set((x, y, -1));
                                EventPropagation::Stop
                            })
                            .disabled(move || !enabled[x][y].get())
                            .style(move |s| {
                                s.width(15)
                                    .height(15)
                                    .disabled(|s| s.color(colours[x][y].get()))
                                    .color(colours[x][y].get())
                            })
                    })
                    .collect()
            })
            .collect();

        v_stack_from_iter(buttons.into_iter().chunks(3).into_iter().map(|chunk| {
            v_stack_from_iter(chunk.into_iter().map(|buttons| {
                h_stack_from_iter(
                    buttons
                        .into_iter()
                        .chunks(3)
                        .into_iter()
                        .map(|chunk| h_stack_from_iter(chunk.into_iter())),
                )
                .style(|s| s.gap(15))
            }))
        }))
        .style(|s| {
            s.gap(15)
                .padding_left(15)
                .padding_top(15)
                .max_width(225)
                .max_height(225)
        })
        .window_scale(|| 3.0)
    }
}

impl From<Colour> for StyleValue<Color> {
    fn from(c: Colour) -> Self {
        match c {
            Colour::Black => StyleValue::Val(Color::BLACK),
            Colour::Red => StyleValue::Val(Color::RED),
            Colour::Green => StyleValue::Val(Color::GREEN),
        }
    }
}
