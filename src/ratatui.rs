//! Vibe-coded terminal UI implementation using ratatui.
//!
//! Navigation: Arrow keys or hjkl (vim-style)
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
//! - **Height >= 15**: Header and footer with borders, cell size based on available space
//! - **Height >= 11**: Header and footer without borders (compact), smaller cells
//! - **Height < 11**: No header/footer, minimal grid only
//!
//! Cell configurations based on available grid area:
//! - **57x39+**: Large cells (5x3) with shared borders and 3x3 separators
//! - **48x30+**: Medium cells (4x2) with shared borders and 3x3 separators
//! - **37x19+**: Compact cells (3x1) with shared borders and 3x3 separators
//! - **29x11+**: Small cells (3x1) with 3x3 separators only
//! - **20x11+**: Tiny cells (2x1) with 3x3 separators
//! - **18x9+**: Minimal cells (2x1) with no decorations
//! - **9x9+**: Ultra-minimal (1x1) cells

use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use crate::{Colour, SudokuModel};

pub fn main(sudoku_model: SudokuModel) -> io::Result<()> {
    // Check if we're running in a terminal
    if !crossterm::tty::IsTty::is_tty(&io::stdout()) {
        eprintln!("Error: This application requires a terminal (TTY) to run.");
        eprintln!("Please run it directly in a terminal, not through a pipe or non-TTY environment.");
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not running in a TTY",
        ));
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state and run
    let mut app = App::new(sudoku_model);
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

struct App {
    model: SudokuModel,
    cursor_x: usize,
    cursor_y: usize,
    should_quit: bool,
}

impl App {
    fn new(model: SudokuModel) -> Self {
        Self {
            model,
            cursor_x: 0,
            cursor_y: 0,
            should_quit: false,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            // Quit
            KeyCode::Esc | KeyCode::Char('q') => {
                self.should_quit = true;
            }
            // Navigation - Arrow keys
            KeyCode::Up => {
                if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                }
            }
            KeyCode::Down => {
                if self.cursor_y < 8 {
                    self.cursor_y += 1;
                }
            }
            KeyCode::Left => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_x < 8 {
                    self.cursor_x += 1;
                }
            }
            // Navigation - Vim style (hjkl)
            KeyCode::Char('h') => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            KeyCode::Char('j') => {
                if self.cursor_y < 8 {
                    self.cursor_y += 1;
                }
            }
            KeyCode::Char('k') => {
                if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                }
            }
            KeyCode::Char('l') => {
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

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
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
struct LayoutConfig {
    cell_width: u16,
    cell_height: u16,
    show_grid_border: bool,
    show_cell_borders: bool,
    separator_size: u16,
}

impl LayoutConfig {
    fn from_size(width: u16, height: u16) -> Self {
        // Select layout based on what actually fits in the available area
        // With shared borders: cell*9 + 8 internal + separators + grid_border
        
        // Large: 5*9 + 8 + 2 + 2 = 57w, 3*9 + 8 + 2 + 2 = 39h (cell=5x3, shared borders)
        if width >= 57 && height >= 39 {
            Self {
                cell_width: 5,
                cell_height: 3,
                show_grid_border: true,
                show_cell_borders: true,
                separator_size: 1,
            }
        }
        // Medium: 4*9 + 8 + 2 + 2 = 48w, 2*9 + 8 + 2 + 2 = 30h (cell=4x2, shared borders)
        else if width >= 48 && height >= 30 {
            Self {
                cell_width: 4,
                cell_height: 2,
                show_grid_border: true,
                show_cell_borders: true,
                separator_size: 1,
            }
        }
        // Compact with borders: 3*9 + 8 + 2 = 37w, 1*9 + 8 + 2 = 19h (cell=3x1, shared borders)
        else if width >= 37 && height >= 19 {
            Self {
                cell_width: 3,
                cell_height: 1,
                show_grid_border: false,
                show_cell_borders: true,
                separator_size: 1,
            }
        }
        // Compact no cell borders: 3*9 + 2 = 29w, 1*9 + 2 = 11h (cell=3x1, just separators)
        else if width >= 29 && height >= 11 {
            Self {
                cell_width: 3,
                cell_height: 1,
                show_grid_border: false,
                show_cell_borders: false,
                separator_size: 1,
            }
        }
        // Small: 2*9 + 2 = 20w, 1*9 + 2 = 11h (cell=2x1, separators)
        else if width >= 20 && height >= 11 {
            Self {
                cell_width: 2,
                cell_height: 1,
                show_grid_border: false,
                show_cell_borders: false,
                separator_size: 1,
            }
        }
        // Minimal: 2*9 = 18w, 1*9 = 9h (cell=2x1, no separators)
        else if width >= 18 && height >= 9 {
            Self {
                cell_width: 2,
                cell_height: 1,
                show_grid_border: false,
                show_cell_borders: false,
                separator_size: 0,
            }
        }
        // Ultra-minimal: 1*9 = 9w, 1*9 = 9h (cell=1x1)
        else {
            Self {
                cell_width: 1,
                cell_height: 1,
                show_grid_border: false,
                show_cell_borders: false,
                separator_size: 0,
            }
        }
    }

    fn grid_width(&self) -> u16 {
        if self.show_cell_borders {
            // With shared borders: cell*9 + 8 internal borders + 2 separators + 2 grid borders
            self.cell_width * 9 + 8 + self.separator_size * 2 + if self.show_grid_border { 2 } else { 0 }
        } else {
            self.cell_width * 9 + self.separator_size * 2 + if self.show_grid_border { 2 } else { 0 }
        }
    }

    fn grid_height(&self) -> u16 {
        if self.show_cell_borders {
            // With shared borders: cell*9 + 8 internal borders + 2 separators + 2 grid borders
            self.cell_height * 9 + 8 + self.separator_size * 2 + if self.show_grid_border { 2 } else { 0 }
        } else {
            self.cell_height * 9 + self.separator_size * 2 + if self.show_grid_border { 2 } else { 0 }
        }
    }
    
    fn cell_total_width(&self) -> u16 {
        if self.show_cell_borders {
            self.cell_width + 1 // Each cell + 1 for the border line after it
        } else {
            self.cell_width
        }
    }
    
    fn cell_total_height(&self) -> u16 {
        if self.show_cell_borders {
            self.cell_height + 1 // Each cell + 1 for the border line after it
        } else {
            self.cell_height
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.area();
    
    // Prioritize showing header/footer even with smaller cells
    // Header/footer each need 1 line minimum (3 with borders)
    let show_header = size.height >= 11; // At least 9 for grid + 1 for header + 1 for footer
    let show_instructions = size.height >= 11;
    
    // Decide if we have room for borders on header/footer
    let header_borders = size.height >= 15; // 9 grid + 3 header + 3 footer
    let margin = if size.width >= 49 && size.height >= 31 { 1 } else { 0 };

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
        if header_borders {
            constraints.push(Constraint::Length(3)); // With borders
        } else {
            constraints.push(Constraint::Length(1)); // Without borders
        }
    }

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(margin)
        .constraints(constraints)
        .split(size);

    let mut chunk_idx = 0;

    // Title (if shown)
    if show_header {
        let title_text = Line::from(vec![
            Span::styled("Sudoku", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        
        let title = if header_borders {
            Paragraph::new(title_text)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL))
        } else {
            Paragraph::new(title_text)
                .alignment(Alignment::Center)
        };
        f.render_widget(title, chunks[chunk_idx]);
        chunk_idx += 1;
    }

    // Sudoku grid - select config based on actual available grid area
    let grid_area = chunks[chunk_idx];
    let config = LayoutConfig::from_size(grid_area.width, grid_area.height);
    render_sudoku_grid(f, app, grid_area, &config);
    chunk_idx += 1;

    // Instructions (if shown)
    if show_instructions {
        let instructions_text = if header_borders {
            vec![
                Line::from("Arrows/hjkl: Move | 1-9: Set value | 0/⌫: Clear | +/-: Inc/Dec | ESC/q: Quit"),
            ]
        } else {
            vec![
                Line::from("↑↓←→/hjkl:Move 1-9:Set 0/⌫:Clear +/-:Inc/Dec ESC/q:Quit"),
            ]
        };
        
        let instructions = if header_borders {
            Paragraph::new(instructions_text)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL))
        } else {
            Paragraph::new(instructions_text)
                .alignment(Alignment::Center)
        };
        f.render_widget(instructions, chunks[chunk_idx]);
    }
}

fn render_sudoku_grid(f: &mut Frame, app: &App, area: Rect, config: &LayoutConfig) {
    // Calculate grid dimensions based on config
    let grid_width = config.grid_width();
    let grid_height = config.grid_height();

    // Center the grid and ensure it fits within area
    let grid_area = Rect {
        x: area.x + (area.width.saturating_sub(grid_width)) / 2,
        y: area.y + (area.height.saturating_sub(grid_height)) / 2,
        width: grid_width.min(area.width),
        height: grid_height.min(area.height),
    };
    
    // Ensure grid_area is within frame bounds
    if grid_area.x >= f.area().width || grid_area.y >= f.area().height {
        return; // Grid would be completely off-screen
    }
    
    let frame_area = f.area();
    let max_x = frame_area.width;
    let max_y = frame_area.height;

    // Render grid border (if enabled)
    let inner = if config.show_grid_border {
        let grid_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));
        f.render_widget(grid_block, grid_area);
        grid_area.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        })
    } else {
        grid_area
    };

    // Render grid lines (shared cell borders) FIRST so cells draw on top
    if config.show_cell_borders {
        render_grid_lines(f, inner, config, max_x, max_y);
    }
    
    // Render each cell
    let cell_total_width = config.cell_total_width();
    let cell_total_height = config.cell_total_height();
    
    for y in 0..9 {
        for x in 0..9 {
            // Calculate position accounting for shared borders and 3x3 separators
            let border_offset_x = if config.show_cell_borders { x as u16 } else { 0 };
            let border_offset_y = if config.show_cell_borders { y as u16 } else { 0 };
            let sep_offset_x = (x as u16 / 3) * config.separator_size;
            let sep_offset_y = (y as u16 / 3) * config.separator_size;
            
            let cell_x = inner.x + (x as u16) * config.cell_width + border_offset_x + sep_offset_x;
            let cell_y = inner.y + (y as u16) * config.cell_height + border_offset_y + sep_offset_y;

            // Skip cells that would be outside frame bounds
            if cell_x >= max_x || cell_y >= max_y {
                continue;
            }

            let cell_area = Rect {
                x: cell_x,
                y: cell_y,
                width: config.cell_width.min(max_x.saturating_sub(cell_x)),
                height: config.cell_height.min(max_y.saturating_sub(cell_y)),
            };

            render_cell(f, app, cell_area, x, y, config);
        }
    }

    // Render 3x3 grid separators (if enabled) - thicker lines to distinguish 3x3 boxes
    if config.separator_size > 0 {
        // Calculate actual rendered grid dimensions based on inner area and frame bounds
        let actual_grid_height = (cell_total_height * 9 + config.separator_size * 2)
            .min(inner.height)
            .min(max_y.saturating_sub(inner.y));
        let actual_grid_width = (cell_total_width * 9 + config.separator_size * 2)
            .min(inner.width)
            .min(max_x.saturating_sub(inner.x));
        
        for i in 1..3 {
            // Vertical separators
            let sep_x = inner.x + (i as u16) * (cell_total_width * 3 + config.separator_size) - 1;
            // Only render separators within frame bounds
            if sep_x < max_x && sep_x < inner.x + inner.width {
                for y in 0..actual_grid_height {
                    let y_pos = inner.y + y;
                    // Check frame bounds before rendering
                    if y_pos < max_y {
                        let sep_area = Rect {
                            x: sep_x,
                            y: y_pos,
                            width: 1,
                            height: 1,
                        };
                        let sep = Paragraph::new("│").style(Style::default().fg(Color::DarkGray));
                        f.render_widget(sep, sep_area);
                    }
                }
            }

            // Horizontal separators
            let sep_y = inner.y + (i as u16) * (cell_total_height * 3 + config.separator_size) - 1;
            // Only render separators within frame bounds
            if sep_y < max_y && sep_y < inner.y + inner.height {
                for x in 0..actual_grid_width {
                    let x_pos = inner.x + x;
                    // Check frame bounds before rendering
                    if x_pos < max_x {
                        let sep_area = Rect {
                            x: x_pos,
                            y: sep_y,
                            width: 1,
                            height: 1,
                        };
                        let sep = Paragraph::new("─").style(Style::default().fg(Color::DarkGray));
                        f.render_widget(sep, sep_area);
                    }
                }
            }
        }
    }
}

fn render_grid_lines(f: &mut Frame, inner: Rect, config: &LayoutConfig, max_x: u16, max_y: u16) {
    // Draw horizontal lines between rows (but not at 3x3 boundaries which are drawn as separators)
    for row in 1..9 {
        // Skip rows that are 3x3 boundaries
        if row % 3 == 0 && config.separator_size > 0 {
            continue;
        }
        
        // Line is drawn AFTER row-1 and BEFORE row
        // Position: cell_height * row + (row - 1 for previous lines) + separator offsets
        let line_y = inner.y + config.cell_height * row as u16 + row as u16 - 1 + (row as u16 / 3) * config.separator_size;
        
        if line_y >= max_y {
            break;
        }
        
        // Draw horizontal line across all cells in this row
        for col in 0..9 {
            let cell_x = inner.x + config.cell_width * col as u16 + col as u16 + (col as u16 / 3) * config.separator_size;
            
            if cell_x >= max_x {
                break;
            }
            
            // Draw line for the width of this cell
            for offset in 0..config.cell_width {
                let x = cell_x + offset;
                if x < max_x {
                    let area = Rect { x, y: line_y, width: 1, height: 1 };
                    f.render_widget(Paragraph::new("─").style(Style::default().fg(Color::DarkGray)), area);
                }
            }
        }
    }
    
    // Draw vertical lines between columns (but not at 3x3 boundaries)
    for col in 1..9 {
        // Skip columns that are 3x3 boundaries
        if col % 3 == 0 && config.separator_size > 0 {
            continue;
        }
        
        // Line is drawn AFTER col-1 and BEFORE col
        let line_x = inner.x + config.cell_width * col as u16 + col as u16 - 1 + (col as u16 / 3) * config.separator_size;
        
        if line_x >= max_x {
            break;
        }
        
        // Draw vertical line across all cells in this column
        for row in 0..9 {
            let cell_y = inner.y + config.cell_height * row as u16 + row as u16 + (row as u16 / 3) * config.separator_size;
            
            if cell_y >= max_y {
                break;
            }
            
            // Draw line for the height of this cell
            for offset in 0..config.cell_height {
                let y = cell_y + offset;
                if y < max_y {
                    let area = Rect { x: line_x, y, width: 1, height: 1 };
                    f.render_widget(Paragraph::new("│").style(Style::default().fg(Color::DarkGray)), area);
                }
            }
        }
    }
}

fn render_cell(f: &mut Frame, app: &App, area: Rect, x: usize, y: usize, config: &LayoutConfig) {
    // Skip rendering if area is empty or invalid
    if area.width == 0 || area.height == 0 {
        return;
    }
    
    let value = app.model.get(x, y);
    let colour = app.model.colour(x, y);
    let is_selected = app.cursor_x == x && app.cursor_y == y;

    // Determine colors
    let fg_color = match colour {
        Colour::Black => Color::White,
        Colour::Red => Color::Red,
        Colour::Green => Color::Green,
    };

    let mut style = Style::default().fg(fg_color);
    if is_selected {
        style = style.add_modifier(Modifier::BOLD);
    }
    if !value.enabled {
        style = style.add_modifier(Modifier::BOLD);
    }

    // Add background/highlighting for selected cell
    if is_selected {
        if config.show_cell_borders {
            // With shared borders, use background and underline
            style = style.bg(Color::DarkGray).add_modifier(Modifier::UNDERLINED);
        } else {
            // Without borders, just use background
            style = style.bg(Color::DarkGray);
        }
    }

    let text = value.text();
    let cell_content = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(style);

    if config.show_cell_borders {
        // Using shared borders (grid lines), so just render cell content
        
        // Center text vertically in the cell
        let text_area = if config.cell_height > 1 {
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
    } else {
        // Render without borders (minimal mode)
        // For single-line cells, no need to adjust vertical position
        let text_area = if config.cell_height > 1 {
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

