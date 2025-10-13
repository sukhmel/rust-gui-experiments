#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

//! Very responsive and seems to be very flexible.
//!
//! Downside is that some adjustments are hard to perform or find, like [central panel margins],
//! default [button on-hover background], changing [text size and colour] on a same button,
//! changing button [fill for default and hover] (in this case it was me not digging enough),
//! [adding support for tray icon] ([also by replacing `winit` with `tao`]), etc.
//!
//! Rule of thumb seems to be to search for configurations in the global `ui` struct, apply them in
//! scope, and then add the widget. Modifying the widget itself is for less idiomatic changes (?).
//!
//! Some more examples can be found on <https://egui.info>
//! 
//! [central panel margins]: https://github.com/emilk/egui/discussions/4365#discussioncomment-11371627
//! [button on-hover background]: https://github.com/emilk/egui/discussions/3356
//! [text size and colour]: https://github.com/emilk/egui/discussions/4518
//! [fill for default and hover]: https://github.com/emilk/egui/discussions/7627
//! [adding support for tray icon]: https://github.com/hoothin/RustClock
//! [also by replacing `winit` with `tao`]: https://github.com/sidit77/headset-controller

use std::ops::Add;
use eframe::egui::{Button, Context, IconData};
use eframe::{Frame, egui};
use egui::Color32;

use crate::{Colour, SudokuModel};

pub fn main(sudoku_model: SudokuModel) -> eframe::Result {
    let favicon = image::ImageReader::open("www/favicon.png")
        .unwrap()
        .decode()
        .unwrap()
        .to_rgba8();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([585.0, 585.0])
            .with_icon(IconData {
                rgba: favicon.to_vec(),
                width: favicon.width(),
                height: favicon.height(),
            })
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Sudoku",
        options,
        Box::new(|_cc| Ok(Box::new(sudoku_model))),
    )
}

impl eframe::App for SudokuModel {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.set_pixels_per_point(3.5);

        egui::CentralPanel::default()
            // Margins set otherwise seem to be ignored
            .frame(
                egui::Frame::default()
                    .inner_margin(13.5)
                    .fill(Color32::from_gray(27)),
            )
            .show(ctx, |ui| {
                // buttons we create are too small, and will by default be extra padded
                ui.spacing_mut().interact_size = egui::vec2(30.0, 30.0);

                egui::Grid::new("top_grid")
                    .spacing([15.0, 15.0])
                    .show(ui, |ui| {
                        for top_y in 0..3 {
                            for top_x in 0..3 {
                                egui::Grid::new(format!("grid_{}_{}", top_x, top_y))
                                    .spacing([1.0, 1.0])
                                    .show(ui, |ui| {
                                        for inner_y in 0..3 {
                                            for inner_x in 0..3 {
                                                let x = top_x * 3 + inner_x;
                                                let y = top_y * 3 + inner_y;
                                                let color: Color32 = self.colour(x, y).into();
                                                let enabled = self.get(x, y).enabled;
                                                let text = self.text(x, y);

                                                let button = Button::new(text)
                                                    .frame(true)
                                                    .min_size(egui::vec2(30.0, 30.0));

                                                let response = ui
                                                    .scope(|ui| {
                                                        let styles = ui.style_mut();
                                                        styles.visuals.widgets.inactive.weak_bg_fill = color;
                                                        styles.visuals.widgets.hovered.weak_bg_fill = color.add(Color32::from_gray(27));
                                                        ui.add_enabled(enabled, button)
                                                    })
                                                    .inner;

                                                if response.clicked() {
                                                    self.add(x, y, 1);
                                                }
                                                if response.secondary_clicked() {
                                                    self.add(x, y, -1);
                                                }
                                                if enabled {
                                                    response.on_hover_cursor(egui::CursorIcon::PointingHand);
                                                }
                                            }
                                            ui.end_row();
                                        }
                                    });
                            }
                            ui.end_row();
                        }
                    })
            });
    }
}

impl From<Colour> for Color32 {
    fn from(c: Colour) -> Self {
        match c {
            Colour::Black => Color32::BLACK,
            Colour::Red => Color32::DARK_RED,
            Colour::Green => Color32::DARK_GREEN,
        }
    }
}
