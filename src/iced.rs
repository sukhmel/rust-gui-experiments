//! No means to connect to external events, not very flexible, but somewhat easy to understand.

use iced::border::Radius;
use iced::font::Weight;
use iced::widget::button::Status;
use iced::widget::{Column, Row, button};
use iced::{Background, Border, Color, Element, Font, Pixels, Settings, Task, window};

use crate::{Colour, SudokuModel};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Click(usize, usize),
}

pub const CELL_SIZE: f32 = 50.0;
pub const WINDOW_SIZE: f32 = CELL_SIZE * 11.0;

pub fn main(sudoku_model: SudokuModel) -> iced::Result {
    let window_settings = window::Settings {
        size: iced::Size {
            width: WINDOW_SIZE,
            height: WINDOW_SIZE,
        },
        resizable: false,
        decorations: true,
        ..Default::default()
    };
    let settings: Settings = Settings {
        default_text_size: Pixels(CELL_SIZE / 1.75),
        default_font: Font {
            weight: Weight::Bold,
            ..Default::default()
        },
        ..Default::default()
    };

    iced::application("Sudoku", SudokuModel::update, SudokuModel::view)
        .settings(settings)
        .window(window_settings)
        .run_with(move || (sudoku_model, Task::none()))
}

impl SudokuModel {
    pub fn view(&self) -> Column<Message> {
        let default_border = Border {
            color: Color::from_rgb(0.6, 0.6, 0.6),
            width: 1.0,
            radius: Radius {
                top_left: CELL_SIZE / 6.0,
                top_right: CELL_SIZE / 6.0,
                bottom_right: CELL_SIZE / 6.0,
                bottom_left: CELL_SIZE / 6.0,
            },
        };
        let default_style = button::Style {
            background: Some(Background::Color(Color::WHITE)),
            border: default_border,
            ..Default::default()
        };
        let black = {
            let mut result = default_style.clone();
            result.text_color = Color::BLACK;
            result
        };
        let red = {
            let mut result = default_style.clone();
            result.text_color = Color::from_rgb(0.8, 0.0, 0.0);
            result
        };
        let green = {
            let mut result = default_style.clone();
            result.text_color = Color::from_rgb(0.0, 0.6, 0.0);
            result
        };
        Column::with_children((0..9).flat_map(|y| {
            let mut children = vec![];
            if y % 3 == 0 {
                children.push(Element::from(iced::widget::vertical_space()))
            }
            let row = Element::from(Row::with_children((0..9).flat_map(move |x| {
                let mut children = vec![];
                if x % 3 == 0 {
                    children.push(Element::from(iced::widget::horizontal_space()))
                }
                let enabled = self.get(x, y).enabled;
                children.push(Element::from(
                    button(self.text(x, y))
                        .on_press_maybe(enabled.then_some(Message::Click(x, y)))
                        .width(CELL_SIZE)
                        .height(CELL_SIZE)
                        .style(move |_, status| {
                            let mut style = match self.colour(x, y) {
                                Colour::Black => black.clone(),
                                Colour::Red => red.clone(),
                                Colour::Green => green.clone(),
                            };
                            match status {
                                Status::Active => {
                                    style.background =
                                        Some(Background::Color(Color::from_rgb(0.9, 0.9, 0.9)));
                                }
                                Status::Hovered => {
                                    style.background =
                                        Some(Background::Color(Color::from_rgb(0.8, 0.8, 0.8)));
                                }
                                Status::Pressed => {
                                    style.background =
                                        Some(Background::Color(Color::from_rgb(0.7, 0.7, 0.7)));
                                }
                                Status::Disabled => {
                                    style.background = Some(Background::Color(Color::WHITE));
                                    let mut border = default_border.clone();
                                    border.width = border.width * 1.5;
                                    border.color = Color::BLACK;
                                    style.border = border;
                                }
                            }
                            style
                        }),
                ));
                if x == 8 {
                    children.push(Element::from(iced::widget::horizontal_space()))
                }
                children.into_iter()
            })));
            children.push(row);
            if y == 8 {
                children.push(Element::from(iced::widget::vertical_space()))
            }
            children.into_iter()
        }))
        .width(WINDOW_SIZE)
        .height(WINDOW_SIZE)
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Click(x, y) => self.add(x, y, 1),
        }
    }
}
