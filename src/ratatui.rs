//! Terminal UI implementation using `ratatui`.
//!
//! Reimplemented practically from scratch after vibe-coded version proved unfixable.
//!
//! Navigation: Arrow keys or hjkl
//! Input: Number keys (1-9) to set values, 0/Backspace/Delete to clear
//! Value adjustment: +/- keys to increment/decrement
//! Quit: Press ESC or 'q'
//!
//! The selected cell is highlighted with a border (or background in minimal mode).
//! Colors indicate:
//! - White: Normal state
//! - Red: Conflict detected
//! - Green: Row/column/box complete
//! - Cyan: Fixed cells (initial puzzle)
//!
//! ## Responsive Layout
//!
//! The UI automatically adapts to different terminal sizes, prioritizing header/footer:
//! - **Height < 9**: Not enough space for display
//! - **Height == 9**: No header/footer
//! - **Height < 13**: No header, simple footer
//! - **Height < 15**: Simple header and footer
//! - **Height < 17**: Header with borders, simple footer
//! - **Height >= 17**: Header and footer with borders
//!
//! Cell configurations based on available grid area (with collapsed borders):
//! - **9x9**: Simple 1x1 cells
//! - **11x11**: Simple 1x1 cells with separators
//! - **17x17**: Overlapping 3x3 cells with collapsed borders and collapsed separators
//! - **19x19**: Overlapping 3x3 cells with separators, borders and border around, all collapsed
//!
//! These modes are not displaying correctly yet, because some maths is off:
//! - **23x23**: Overlapping 3x3 cells with separators and collapsed borders
//! - **25x25**: Overlapping 3x3 cells with separators, collapsed borders and border around
//! - **29x29**: Separate 3x3 cells with borders and separators
//! - **31x31**: Separate 3x3 cells with borders, separators and border around

use std::io;
use std::io::Stdout;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::backend::Backend;
use ratatui::symbols::line::{DOUBLE_HORIZONTAL, DOUBLE_VERTICAL, HORIZONTAL, Set, VERTICAL};
use ratatui::text::Text;
use ratatui::widgets::Wrap;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{Colour, SudokuModel};

pub const DOUBLE_HORIZONTAL_PLAIN_DOWN: &str = "╤";
pub const DOUBLE_HORIZONTAL_PLAIN_UP: &str = "╧";
pub const DOUBLE_HORIZONTAL_RIGHT_PLAIN_VERTICAL: &str = "╞";
pub const DOUBLE_HORIZONTAL_LEFT_PLAIN_VERTICAL: &str = "╡";
pub const DOUBLE_HORIZONTAL_PLAIN_CROSS: &str = "╪";
pub const DOUBLE_RIGHT_PLAIN_DOWN: &str = "╒";
pub const DOUBLE_RIGHT_PLAIN_UP: &str = "╘";
pub const DOUBLE_LEFT_PLAIN_DOWN: &str = "╕";
pub const DOUBLE_LEFT_PLAIN_UP: &str = "╛";

pub const DOUBLE_VERTICAL_DOWN_PLAIN_HORIZONTAL: &str = "╥";
pub const DOUBLE_VERTICAL_UP_PLAIN_HORIZONTAL: &str = "╨";
pub const DOUBLE_VERTICAL_PLAIN_RIGHT: &str = "╟";
pub const DOUBLE_VERTICAL_PLAIN_LEFT: &str = "╢";
pub const DOUBLE_VERTICAL_PLAIN_CROSS: &str = "╫";
pub const DOUBLE_DOWN_PLAIN_RIGHT: &str = "╓";
pub const DOUBLE_UP_PLAIN_RIGHT: &str = "╙";
pub const DOUBLE_DOWN_PLAIN_LEFT: &str = "╖";
pub const DOUBLE_UP_PLAIN_LEFT: &str = "╜";

pub const DOUBLE_SIDES_PLAIN: Set = Set {
    vertical: DOUBLE_VERTICAL,
    horizontal: HORIZONTAL,
    top_right: DOUBLE_DOWN_PLAIN_LEFT,
    top_left: DOUBLE_DOWN_PLAIN_RIGHT,
    bottom_right: DOUBLE_UP_PLAIN_LEFT,
    bottom_left: DOUBLE_UP_PLAIN_RIGHT,
    vertical_left: DOUBLE_VERTICAL_PLAIN_LEFT,
    vertical_right: DOUBLE_VERTICAL_PLAIN_RIGHT,
    horizontal_down: DOUBLE_VERTICAL_DOWN_PLAIN_HORIZONTAL,
    horizontal_up: DOUBLE_VERTICAL_UP_PLAIN_HORIZONTAL,
    cross: DOUBLE_VERTICAL_PLAIN_CROSS,
};

pub const PLAIN_SIDES_DOUBLE: Set = Set {
    vertical: VERTICAL,
    horizontal: DOUBLE_HORIZONTAL,
    top_right: DOUBLE_LEFT_PLAIN_DOWN,
    top_left: DOUBLE_RIGHT_PLAIN_DOWN,
    bottom_right: DOUBLE_LEFT_PLAIN_UP,
    bottom_left: DOUBLE_RIGHT_PLAIN_UP,
    vertical_left: DOUBLE_HORIZONTAL_RIGHT_PLAIN_VERTICAL,
    vertical_right: DOUBLE_HORIZONTAL_LEFT_PLAIN_VERTICAL,
    horizontal_down: DOUBLE_HORIZONTAL_PLAIN_DOWN,
    horizontal_up: DOUBLE_HORIZONTAL_PLAIN_UP,
    cross: DOUBLE_HORIZONTAL_PLAIN_CROSS,
};

pub const EMPTY_SET: Set = Set {
    vertical: " ",
    horizontal: " ",
    top_right: " ",
    top_left: " ",
    bottom_right: " ",
    bottom_left: " ",
    vertical_left: " ",
    vertical_right: " ",
    horizontal_down: " ",
    horizontal_up: " ",
    cross: " ",
};

pub fn main(sudoku_model: SudokuModel) -> io::Result<()> {
    // Check if we're running in a terminal
    if !crossterm::tty::IsTty::is_tty(&io::stdout()) {
        eprintln!("Error: This application requires a terminal (TTY) to run.");
        eprintln!(
            "Please run it directly in a terminal, not through a pipe or non-TTY environment."
        );
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not running in a TTY",
        ));
    }

    // Setup terminal and restore on exit
    let res = {
        let mut terminal_guard = TerminalGuard::new()?;
        let mut app = App::new(sudoku_model);

        run_app(terminal_guard.terminal(), &mut app)
    };

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

struct TerminalGuard {
    terminal: Option<Terminal<CrosstermBackend<Stdout>>>,
}

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            terminal: Some(terminal),
        })
    }

    fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        self.terminal.as_mut().unwrap()
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut terminal = std::mem::take(&mut self.terminal).unwrap();
        let _ = disable_raw_mode();
        let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
        let _ = terminal.show_cursor();
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum BorderStyle {
    None,
    Plain,
    Double,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum State {
    Neutral,
    Good,
    Bad,
}

struct Cell {
    left: BorderStyle,
    right: BorderStyle,
    top: BorderStyle,
    bottom: BorderStyle,
    continued_left: bool,
    continued_right: bool,
    continued_up: bool,
    continued_down: bool,
    x: u16,
    y: u16,
    h: u16,
    w: u16,
    state: State,
    selected: bool,
    enabled: bool,
    separate: bool,
    text: &'static str,
}

impl Cell {
    fn has_borders(&self) -> bool {
        self.left != BorderStyle::None
            || self.right != BorderStyle::None
            || self.top != BorderStyle::None
            || self.bottom != BorderStyle::None
    }

    fn get_borders(&self) -> Borders {
        let mut borders = Borders::empty();
        if self.left != BorderStyle::None {
            borders |= Borders::LEFT;
        }
        if self.right != BorderStyle::None {
            borders |= Borders::RIGHT;
        }
        if self.top != BorderStyle::None {
            borders |= Borders::TOP;
        }
        if self.bottom != BorderStyle::None {
            borders |= Borders::BOTTOM;
        }
        borders
    }

    fn get_border_set(&self) -> symbols::border::Set {
        let mut result = symbols::border::EMPTY;
        let (top_left_set, bottom_left_set) = if self.left == BorderStyle::Double {
            let top_left_set = if self.top == BorderStyle::Double {
                symbols::line::DOUBLE
            } else if self.top == BorderStyle::Plain {
                DOUBLE_SIDES_PLAIN
            } else {
                EMPTY_SET
            };
            let bottom_left_set = if self.bottom == BorderStyle::Double {
                symbols::line::DOUBLE
            } else if self.bottom == BorderStyle::Plain {
                DOUBLE_SIDES_PLAIN
            } else {
                EMPTY_SET
            };
            (top_left_set, bottom_left_set)
        } else if self.left == BorderStyle::Plain {
            let top_left_set = if self.top == BorderStyle::Double {
                PLAIN_SIDES_DOUBLE
            } else if self.top == BorderStyle::Plain {
                symbols::line::NORMAL
            } else {
                EMPTY_SET
            };
            let bottom_left_set = if self.bottom == BorderStyle::Double {
                PLAIN_SIDES_DOUBLE
            } else if self.bottom == BorderStyle::Plain {
                symbols::line::NORMAL
            } else {
                EMPTY_SET
            };
            (top_left_set, bottom_left_set)
        } else {
            (EMPTY_SET, EMPTY_SET)
        };

        let (top_right_set, bottom_right_set) = if self.right == BorderStyle::Double {
            let top_right_set = if self.top == BorderStyle::Double {
                symbols::line::DOUBLE
            } else if self.top == BorderStyle::Plain {
                DOUBLE_SIDES_PLAIN
            } else {
                EMPTY_SET
            };
            let bottom_right_set = if self.bottom == BorderStyle::Double {
                symbols::line::DOUBLE
            } else if self.bottom == BorderStyle::Plain {
                DOUBLE_SIDES_PLAIN
            } else {
                EMPTY_SET
            };
            (top_right_set, bottom_right_set)
        } else if self.right == BorderStyle::Plain {
            let top_right_set = if self.top == BorderStyle::Double {
                PLAIN_SIDES_DOUBLE
            } else if self.top == BorderStyle::Plain {
                symbols::line::NORMAL
            } else {
                EMPTY_SET
            };
            let bottom_right_set = if self.bottom == BorderStyle::Double {
                PLAIN_SIDES_DOUBLE
            } else if self.bottom == BorderStyle::Plain {
                symbols::line::NORMAL
            } else {
                EMPTY_SET
            };
            (top_right_set, bottom_right_set)
        } else {
            (EMPTY_SET, EMPTY_SET)
        };

        if self.continued_left && self.continued_up {
            result.top_left = top_left_set.cross;
        } else if self.continued_left {
            result.top_left = top_left_set.horizontal_down;
        } else if self.continued_up {
            result.top_left = top_left_set.vertical_right;
        } else {
            result.top_left = top_left_set.top_left;
        }

        if self.continued_left && self.continued_down {
            result.bottom_left = bottom_left_set.cross;
        } else if self.continued_left {
            result.bottom_left = bottom_left_set.horizontal_up;
        } else if self.continued_down {
            result.bottom_left = bottom_left_set.vertical_right;
        } else {
            result.bottom_left = bottom_left_set.bottom_left;
        }

        if self.continued_right && self.continued_up {
            result.top_right = top_right_set.cross;
        } else if self.continued_right {
            result.top_right = top_right_set.horizontal_down;
        } else if self.continued_up {
            result.top_right = top_right_set.vertical_left;
        } else {
            result.top_right = top_right_set.top_right;
        }

        if self.continued_right && self.continued_down {
            result.bottom_right = bottom_right_set.cross;
        } else if self.continued_right {
            result.bottom_right = bottom_right_set.horizontal_up;
        } else if self.continued_down {
            result.bottom_right = bottom_right_set.vertical_left;
        } else {
            result.bottom_right = bottom_right_set.bottom_right;
        }

        if self.right == BorderStyle::Double {
            result.vertical_right = symbols::line::DOUBLE.vertical;
        } else if self.right == BorderStyle::Plain {
            result.vertical_right = symbols::line::NORMAL.vertical;
        }
        if self.top == BorderStyle::Double {
            result.horizontal_top = symbols::line::DOUBLE.horizontal;
        } else if self.top == BorderStyle::Plain {
            result.horizontal_top = symbols::line::NORMAL.horizontal;
        }
        if self.bottom == BorderStyle::Double {
            result.horizontal_bottom = symbols::line::DOUBLE.horizontal;
        } else if self.bottom == BorderStyle::Plain {
            result.horizontal_bottom = symbols::line::NORMAL.horizontal;
        }
        if self.left == BorderStyle::Double {
            result.vertical_left = symbols::line::DOUBLE.vertical;
        } else if self.left == BorderStyle::Plain {
            result.vertical_left = symbols::line::NORMAL.vertical;
        }

        result
    }
}

struct App {
    model: SudokuModel,
    cursor_x: usize,
    cursor_y: usize,
    should_quit: bool,
    debug: bool,
}

impl App {
    fn new(model: SudokuModel) -> Self {
        Self {
            model,
            cursor_x: 0,
            cursor_y: 0,
            should_quit: false,
            debug: false,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            // Debug grid info
            KeyCode::Char('d') => {
                self.debug = !self.debug;
            }
            // Quit
            KeyCode::Esc | KeyCode::Char('q') => {
                self.should_quit = true;
            }
            // Navigation - Arrow keys
            KeyCode::Up | KeyCode::Char('k') => {
                if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.cursor_y < 8 {
                    self.cursor_y += 1;
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.cursor_x < 8 {
                    self.cursor_x += 1;
                }
            }
            // Number input
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let digit = c.to_digit(10).unwrap() as u8;
                if self.model.get(self.cursor_x, self.cursor_y).enabled {
                    self.model.set(self.cursor_x, self.cursor_y, digit);
                }
            }
            // Clear cell
            KeyCode::Backspace | KeyCode::Delete => {
                if self.model.get(self.cursor_x, self.cursor_y).enabled {
                    self.model.set(self.cursor_x, self.cursor_y, 0);
                }
            }
            // Increment/decrement
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if self.model.get(self.cursor_x, self.cursor_y).enabled {
                    self.model.add(self.cursor_x, self.cursor_y, 1);
                }
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                if self.model.get(self.cursor_x, self.cursor_y).enabled {
                    self.model.add(self.cursor_x, self.cursor_y, -1);
                }
            }
            _ => {}
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.handle_key(key);
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

/// Layout configuration based on available grid area size
#[derive(Debug)]
struct LayoutConfig {
    cell_h: u16,
    cell_w: u16,
    outer_border: bool,
    cell_border: bool,
    cell_collapsed: bool,
    separators_visible: bool,
    separators_collapsed: bool,
}

impl LayoutConfig {
    fn from_size(width: u16, height: u16) -> Self {
        // Now I see that this should have been done differently, because now some sizes will cause
        // flags that don't make sense together. Probably it would work if cell sizes are chosen
        // first, then area size is calculated in multiples of cell sizes, but it didn't work right
        // away, and I don't want to troubleshoot longer.
        let cell_h = if height < 17 {
            1
        } else {
            //if height < 29
            2
        };
        let cell_w = if width < 23 {
            1
        } else if width < 32 {
            2
        } else if width < 50 {
            3
        } else {
            5
        };
        let cell_border = cell_w > 2 && cell_h > 1;
        let cell_collapsed = width < 39 || height < 29;
        let separators_collapsed =
            height < 11 || width < 11 || width > 16 && width < 21 || height > 16 && height < 21;
        let separators_visible =
            width > 10 && height > 10 && !(width > 20 && width < 29 || height > 20 && height < 29);
        let outer_border = height > 18 && height < 29 || width > 60 && height > 30;
        Self {
            cell_h,
            cell_w,
            outer_border,
            cell_border,
            cell_collapsed,
            separators_visible,
            separators_collapsed,
        }
    }

    fn grid_width(&self) -> u16 {
        self.grid_size(self.cell_w)
    }
    fn grid_height(&self) -> u16 {
        // overrides to fix sloppy coordinates math, that led to negative offset for some sizes
        if self.cell_h == 2
            && self.outer_border
            && self.cell_border
            && !self.cell_collapsed
            && self.separators_visible
            && !self.separators_collapsed
        {
            return 29;
        }
        if self.cell_h == 2
            && !self.outer_border
            && self.cell_border
            && !self.cell_collapsed
            && self.separators_visible
            && !self.separators_collapsed
        {
            return 27;
        }
        if self.cell_h == 2
            && self.outer_border
            && self.cell_border
            && self.cell_collapsed
            && !self.separators_visible
            && !self.separators_collapsed
        {
            return 21;
        }
        if self.cell_h == 2
            && self.outer_border
            && self.cell_border
            && self.cell_collapsed
            && self.separators_visible
            && self.separators_collapsed
        {
            return 19;
        }
        if self.cell_h == 2
            && !self.outer_border
            && self.cell_border
            && self.cell_collapsed
            && self.separators_visible
            && self.separators_collapsed
        {
            return 17;
        }
        if self.cell_h == 1
            && !self.outer_border
            && !self.cell_border
            && self.cell_collapsed
            && self.separators_visible
            && !self.separators_collapsed
        {
            return 9;
        }
        if self.cell_h == 1
            && !self.outer_border
            && !self.cell_border
            && self.cell_collapsed
            && !self.separators_visible
            && self.separators_collapsed
        {
            return 9;
        }

        self.grid_size(self.cell_h)
    }
    fn grid_size(&self, cell_size: u16) -> u16 {
        let mut result = 9 * cell_size;
        if !self.cell_collapsed {
            result += 9;
        } else if self.cell_border {
            result -= 7;
        }

        if self.outer_border {
            result += 1;
        } else if cell_size > 1 {
            result -= 1;
        }
        if self.separators_visible && !self.separators_collapsed {
            result += 2;
        }

        result
    }
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.area();

    // Prioritize footer even with smaller cells
    let show_instructions = size.height > 9;
    let show_header = size.height > 12; // header is less important than footer and separators

    // To keep visuals consistent, header will have borders earlier than footer
    let header_borders = size.height > 14;
    let footer_borders = size.height > 16;

    // Build constraints dynamically
    let mut constraints = vec![];
    if show_header {
        if header_borders {
            constraints.push(Constraint::Length(3)); // With borders
        } else {
            constraints.push(Constraint::Length(1)); // Without borders
        }
    }
    constraints.push(Constraint::Min(0)); // Grid
    if show_instructions {
        if footer_borders {
            constraints.push(Constraint::Length(3)); // With borders
        } else {
            constraints.push(Constraint::Length(1)); // Without borders
        }
    }

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(size);

    let mut chunk_idx = 0;

    // Title (if shown)
    if show_header {
        f.render_widget(
            render_bordered_text("Sudoku", header_borders, true),
            chunks[chunk_idx],
        );
        chunk_idx += 1;
    }

    // Sudoku grid - select config based on actual available grid area
    let grid_area = chunks[chunk_idx];
    let config = LayoutConfig::from_size(grid_area.width, grid_area.height);
    render_sudoku_grid(f, app, grid_area, &config);
    chunk_idx += 1;

    // Instructions (if shown)
    if show_instructions {
        let area = chunks[chunk_idx];

        let instructions = if area.width < 9 {
            "Q"
        } else if area.width < 12 {
            "⇆ ⇅ 0-9 Q"
        } else if area.width < 14 {
            "⇆ ⇅ 0-9 ⌫ Q"
        } else if area.width < 16 {
            "⇆ ⇅ ± 0-9 ⌫ Q"
        } else if area.width < 17 {
            "⇆ ⇅ ± 0-9 ⌫ ␛ Q"
        } else if area.width < 19 {
            "←↓↑→ ± 0-9 ⌫ ␛ Q"
        } else if area.width < 24 {
            "←↓↑→ ± 0-9 ⌫ Esc/Q"
        } else if area.width < 26 {
            "←↓↑→/hjkl ± 0-9 ⌫ Esc/Q"
        } else if area.width < 56 {
            "←↓↑→/hjkl -/+ 0-9 ⌫ Esc/Q"
        } else if area.width < 76 {
            "↑↓←→/hjkl:Move 1-9:Set 0/⌫:Clear +/-:Inc/Dec ESC/q:Quit"
        } else {
            "Arrows/hjkl: Move | 1-9: Set value | 0/⌫: Clear | +/-: Inc/Dec | ESC/q: Quit"
        };

        f.render_widget(
            render_bordered_text(instructions, footer_borders, false),
            area,
        );
    }
}

fn render_bordered_text(text: &str, with_borders: bool, bold: bool) -> Paragraph<'_> {
    let mut text_style = Style::default();
    if bold {
        text_style = text_style.add_modifier(Modifier::BOLD);
    }
    let content = Line::from(vec![Span::styled(text, text_style)]);

    let title = if with_borders {
        Paragraph::new(content).alignment(Alignment::Center).block(
            Block::default()
                .border_style(Style::default().add_modifier(Modifier::BOLD))
                .border_set(symbols::border::ROUNDED)
                .borders(Borders::ALL),
        )
    } else {
        Paragraph::new(content).alignment(Alignment::Center)
    };

    title
}

fn render_sudoku_grid(f: &mut Frame, app: &App, area: Rect, config: &LayoutConfig) {
    // Calculate grid dimensions based on config
    let grid_width = config.grid_width();
    let grid_height = config.grid_height();

    // Center the grid and ensure it fits within area
    let inner = Rect {
        x: area.x + (area.width.saturating_sub(grid_width)) / 2,
        y: area.y + (area.height.saturating_sub(grid_height)) / 2,
        width: grid_width,
        height: grid_height,
    };

    if grid_height > area.height || grid_width > area.width {
        f.render_widget(
            Paragraph::new(
                Line::from(vec![
                    Span::styled("terminal too small to render, press ", Style::default()),
                    Span::styled(
                        "ESC",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Yellow),
                    ),
                    Span::styled(" or ", Style::default()),
                    Span::styled(
                        "Q",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Yellow),
                    ),
                    Span::styled(" to quit", Style::default()),
                ])
                .alignment(Alignment::Left),
            )
            .wrap(Wrap { trim: true }),
            area,
        );
        return;
    }

    let frame_area = f.area();
    let max_x = frame_area.width;
    let max_y = frame_area.height;

    // Cells will need to overlap if using collapsed borders
    let cell_stride_y = config.cell_h + if config.cell_collapsed { 0 } else { 1 };
    let cell_stride_x = config.cell_w + if config.cell_collapsed { 0 } else { 1 }
        - if config.cell_border && config.cell_collapsed {
            1
        } else {
            0
        };
    let separator_stride = if config.separators_collapsed { 0 } else { 1 };

    let mut cells = Vec::with_capacity(81);

    for y in 0..9 {
        for x in 0..9 {
            let (correction_w, correction_x) = get_correction(config, x);
            let (correction_h, correction_y) = get_correction(config, y);
            // Position cells
            let cell_x = inner.x + (x as u16) * cell_stride_x + (x as u16 / 3) * separator_stride
                - correction_x;
            let cell_y = inner.y + (y as u16) * cell_stride_y + (y as u16 / 3) * separator_stride
                - correction_y;

            // Skip cells that would be outside frame bounds
            if cell_x + config.cell_w > max_x || cell_y + config.cell_h > max_y {
                continue;
            }

            let cell_w = config.cell_w - correction_w;
            let cell_h = config.cell_h - correction_h;

            let enabled = app.model.get(x, y).enabled;
            let selected = app.cursor_x == x && app.cursor_y == y;
            let state = match app.model.colour(x, y) {
                Colour::Black => State::Neutral,
                Colour::Red => State::Bad,
                Colour::Green => State::Good,
            };
            let value = app.model.get(x, y).text();
            let border_left = if config.cell_border {
                if x == 0 && !config.outer_border && config.separators_collapsed {
                    BorderStyle::None
                } else if x % 3 == 0 && config.separators_visible && config.separators_collapsed {
                    BorderStyle::Double
                } else {
                    BorderStyle::Plain
                }
            } else {
                BorderStyle::None
            };
            let border_right = if config.cell_border {
                if x == 8 && !config.outer_border && config.separators_collapsed {
                    BorderStyle::None
                } else if x % 3 == 2 && config.separators_visible && config.separators_collapsed {
                    BorderStyle::Double
                } else {
                    BorderStyle::Plain
                }
            } else {
                BorderStyle::None
            };
            let border_top = if config.cell_border {
                if y == 0 && !config.outer_border && config.separators_collapsed {
                    BorderStyle::None
                } else if y % 3 == 0 && config.separators_visible && config.separators_collapsed {
                    BorderStyle::Double
                } else {
                    BorderStyle::Plain
                }
            } else {
                BorderStyle::None
            };
            let border_bottom = if config.cell_border {
                if y == 8 && !config.outer_border && config.separators_collapsed {
                    BorderStyle::None
                } else if y % 3 == 2 && config.separators_visible && config.separators_collapsed {
                    BorderStyle::Double
                } else {
                    BorderStyle::Plain
                }
            } else {
                BorderStyle::None
            };

            // a & b & c || a & b & !c & d
            cells.push(Cell {
                left: border_left,
                right: border_right,
                top: border_top,
                bottom: border_bottom,
                continued_left: x > 0
                    && config.cell_collapsed
                    && (config.separators_collapsed || x % 3 != 0),
                continued_right: x < 8
                    && config.cell_collapsed
                    && (config.separators_collapsed || x % 3 != 2),
                continued_up: y > 0
                    && config.cell_collapsed
                    && (config.separators_collapsed || y % 3 != 0),
                continued_down: y < 8
                    && config.cell_collapsed
                    && (config.separators_collapsed || y % 3 != 2),
                x: cell_x,
                y: cell_y,
                w: cell_w,
                h: cell_h,
                state,
                selected,
                enabled,
                separate: !config.cell_collapsed,
                text: value,
            });
        }
    }

    cells.sort_by(|a, b| {
        a.selected
            .cmp(&b.selected)
            .then_with(|| (!a.enabled).cmp(&(!b.enabled)))
            .then_with(|| a.state.cmp(&b.state))
    });

    for cell in cells {
        render_cell(f, cell)
    }

    render_separators(f, config, inner, cell_stride_y, cell_stride_x);

    if app.debug {
        f.render_widget(
            Text::from(format!(
                "area: {area:#?}\ngrid: {}x{}\ninner: {inner:#?}\nconfig: {config:#?}\n",
                grid_width, grid_height
            )),
            area,
        );
    }
}

fn get_correction(config: &LayoutConfig, x: usize) -> (u16, u16) {
    if !config.outer_border && config.separators_visible {
        if config.separators_collapsed {
            match x {
                0 => (1, 0),
                8 => (1, 1),
                _ => (0, 1),
            }
        } else {
            (0, 1)
        }
    } else {
        (0, 0)
    }
}

fn render_separators(
    f: &mut Frame,
    config: &LayoutConfig,
    inner: Rect,
    cell_stride_y: u16,
    cell_stride_x: u16,
) {
    if !config.separators_collapsed && config.separators_visible {
        for y in 0..3 {
            for x in 0..3 {
                let mut border_set = symbols::border::DOUBLE;
                border_set.bottom_right = if x < 2 {
                    if y < 2 {
                        symbols::line::DOUBLE.cross
                    } else {
                        symbols::line::DOUBLE.horizontal_up
                    }
                } else {
                    if y < 2 {
                        symbols::line::DOUBLE.vertical_left
                    } else {
                        symbols::line::DOUBLE.bottom_right
                    }
                };
                border_set.top_right = if x < 2 {
                    if y == 0 {
                        symbols::line::DOUBLE.horizontal_down
                    } else {
                        symbols::line::DOUBLE.cross
                    }
                } else {
                    if y == 0 {
                        symbols::line::DOUBLE.top_right
                    } else {
                        symbols::line::DOUBLE.vertical_left
                    }
                };
                border_set.bottom_left = if x == 0 {
                    if y < 2 {
                        symbols::line::DOUBLE.vertical_right
                    } else {
                        symbols::line::DOUBLE.bottom_left
                    }
                } else {
                    if y < 2 {
                        symbols::line::DOUBLE.cross
                    } else {
                        symbols::line::DOUBLE.horizontal_up
                    }
                };
                border_set.top_left = if x == 0 {
                    if y == 0 {
                        symbols::line::DOUBLE.top_left
                    } else {
                        symbols::line::DOUBLE.vertical_right
                    }
                } else {
                    if y == 0 {
                        symbols::line::DOUBLE.horizontal_down
                    } else {
                        symbols::line::DOUBLE.cross
                    }
                };

                let mut borders = Borders::ALL;

                if !config.outer_border {
                    if x == 0 {
                        borders ^= Borders::LEFT;
                    }
                    if x == 2 {
                        borders ^= Borders::RIGHT;
                    }
                    if y == 0 {
                        borders ^= Borders::TOP;
                    }
                    if y == 2 {
                        borders ^= Borders::BOTTOM;
                    }
                }

                let offset = if config.outer_border {
                    if !config.cell_collapsed { 1 } else { 0 }
                } else {
                    1
                };
                let (width, height, x, y) = if config.outer_border
                    && !config.cell_collapsed
                    && !config.separators_collapsed
                {
                    let stride_x = 3 * cell_stride_x + 1;
                    let stride_y = 3 * cell_stride_y + 1;
                    (
                        if x == 0 {
                            3 * cell_stride_x + 3
                        } else {
                            3 * cell_stride_x + 2
                        },
                        3 * cell_stride_y + 2,
                        inner.x + x * stride_x - 1 - if x == 0 { 1 } else { 0 },
                        inner.y + y * stride_y - 1,
                    )
                } else {
                    let stride_x = 3 * cell_stride_x;
                    let stride_y = 3 * cell_stride_y;
                    (
                        3 * cell_stride_x + 1,
                        3 * cell_stride_y + 1 + if y == 1 { 1 } else { 0 },
                        inner.x + x * stride_x - offset,
                        inner.y + y * stride_y - offset + if y == 2 { 1 } else { 0 },
                    )
                };

                f.render_widget(
                    Block::default()
                        .borders(borders)
                        .border_style(Style::default().fg(Color::DarkGray))
                        .border_set(border_set),
                    Rect {
                        x,
                        y,
                        width,
                        height,
                    },
                );
            }
        }
    }
}

fn render_cell(f: &mut Frame, cell: Cell) {
    let area = Rect {
        x: cell.x,
        y: cell.y,
        width: cell.w,
        height: cell.h + if cell.has_borders() { 1 } else { 0 },
    };

    // Skip rendering if area is empty or invalid
    if area.width == 0 || area.height == 0 {
        return;
    }

    let text = cell.text;
    let is_selected = cell.selected;

    // Determine colors
    let fg_color = match cell.state {
        State::Neutral => Color::White,
        State::Bad => Color::Red,
        State::Good => Color::Green,
    };

    let mut style = Style::default().fg(fg_color);
    if !cell.enabled {
        style = style.add_modifier(Modifier::BOLD);
    }

    // Highlight selected cell
    if is_selected {
        style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
    }

    if cell.has_borders() {
        // Use collapsed borders approach from ratatui docs
        // Determine which borders this cell should render
        let borders = cell.get_borders();
        let border_set = cell.get_border_set();
        let border_style = Style::default().fg(if cell.separate {
            if is_selected {
                Color::Yellow
            } else if !cell.enabled {
                Color::Cyan
            } else {
                Color::DarkGray
            }
        } else {
            Color::DarkGray
        });

        let block = Block::default()
            .borders(borders)
            .border_set(border_set)
            .border_style(border_style);

        let inner_area = block.inner(area);
        f.render_widget(block, area);

        // Render text centered in the inner area
        if inner_area.width > 0 && inner_area.height > 0 {
            let text_widget = Paragraph::new(text)
                .alignment(Alignment::Center)
                .style(style);
            let text_area = if inner_area.height > 1 {
                Rect {
                    x: inner_area.x,
                    y: inner_area.y + (inner_area.height / 2),
                    width: inner_area.width,
                    height: 1,
                }
            } else {
                inner_area
            };
            f.render_widget(text_widget, text_area);
        }
    } else {
        // Render without borders (minimal mode)
        let cell_content = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(style);

        let text_area = if cell.h > 1 {
            Rect {
                x: area.x,
                y: area.y + (area.height / 2),
                width: area.width,
                height: 1,
            }
        } else {
            area
        };
        f.render_widget(cell_content, text_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_size_17() {
        let config = LayoutConfig::from_size(129, 17);
        assert_eq!(config.grid_height(), 17, "{config:?}");
    }

    #[test]
    fn from_size_21() {
        let config = LayoutConfig::from_size(129, 21);
        assert_eq!(config.grid_height(), 22, "{config:?}");
    }
}
