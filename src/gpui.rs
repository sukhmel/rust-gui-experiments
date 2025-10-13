//! gpui implementation of the Sudoku game
//!
//! Seemed to be a bit of an overkill in speed, and complexity.
//!
//! Useful links:
//! * <https://matinaniss.github.io/gpui-book/>
//! * <https://github.com/zed-industries/zed/tree/main/crates/gpui/examples>

use gpui::{
    App, Application, Bounds, Context, Hsla, IntoElement, KeyBinding, MouseButton, ParentElement,
    Render, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, prelude::*,
    px, rgb,
};

use crate::{Colour, SudokuModel};

pub fn main(sudoku_model: SudokuModel) {
    Application::new().run(move |cx: &mut App| {
        // required to make sure the app exits after the window is closed
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let bounds = Bounds::centered(None, gpui::size(px(585.), px(585.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Sudoku".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| SudokuApp {
                    model: sudoku_model,
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}

struct SudokuApp {
    model: SudokuModel,
}

/// Lightens a color by adding grey to it (similar to egui's hover effect)
fn lighten_color(color: Hsla) -> Hsla {
    Hsla {
        h: color.h,
        s: color.s,
        l: (color.l + 0.15).min(1.0), // Increase lightness by 15%, capped at 1.0
        a: color.a,
    }
}

impl Render for SudokuApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Build the grid imperatively to avoid borrow checker issues
        let mut block_rows = Vec::new();

        for top_y in 0..3 {
            let mut block_row = div().flex().flex_row().gap(px(15.));

            for top_x in 0..3 {
                let mut block = div().flex().flex_col().gap(px(1.));

                for inner_y in 0..3 {
                    let mut cell_row = div().flex().flex_row().gap(px(1.));

                    for inner_x in 0..3 {
                        let x = top_x * 3 + inner_x;
                        let y = top_y * 3 + inner_y;
                        let cell = self.render_cell(x, y, cx);
                        cell_row = cell_row.child(cell);
                    }

                    block = block.child(cell_row);
                }

                block_row = block_row.child(block);
            }

            block_rows.push(block_row);
        }

        // Main container with dark grey background
        let mut grid = div().flex().flex_col().gap(px(15.));
        for row in block_rows {
            grid = grid.child(row);
        }

        div()
            .flex()
            .flex_col()
            .bg(rgb(0x1b1b1b))
            .size_full()
            .p(px(13.5))
            .child(grid)
    }
}

impl SudokuApp {
    fn render_cell(&mut self, x: usize, y: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let text = self.model.text(x, y).to_string();
        let colour = self.model.colour(x, y);
        let color: Hsla = colour.into();
        let enabled = self.model.get(x, y).enabled;

        let mut cell = div()
            .flex()
            .items_center()
            .justify_center()
            .w(px(58.))
            .h(px(58.))
            .bg(color)
            .border_1()
            .border_color(rgb(0x999999))
            .text_size(px(32.))
            .child(text);

        // Add white text color for black backgrounds
        if enabled {
            cell = cell.text_color(gpui::white());
        } else {
            cell = cell.text_color(rgb(0xaaaaaa));
        }

        // Only add click handlers if the cell is enabled
        if enabled {
            // Create a lighter shade for hover by adding gray
            let hover_color = lighten_color(color);

            cell = cell
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |app, _event, _window, _cx| {
                        app.model.add(x, y, 1);
                        _cx.notify();
                    }),
                )
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(move |app, _event, _window, _cx| {
                        app.model.add(x, y, -1);
                        _cx.notify();
                    }),
                )
                .cursor_pointer()
                .hover(|style| style.bg(hover_color));
        } else {
            // Disabled cells have a stronger border
            cell = cell.border_2().border_color(gpui::black());
        }

        cell
    }
}

impl From<Colour> for Hsla {
    fn from(c: Colour) -> Self {
        match c {
            Colour::Black => rgb(0x000000).into(),
            Colour::Red => rgb(0x8b0000).into(),
            Colour::Green => rgb(0x006400).into(),
        }
    }
}
