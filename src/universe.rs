use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use color_eyre::{Result, eyre::Error};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use rand::{Rng, SeedableRng};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect, Size},
    style::{Color, Stylize},
    symbols::Marker,
    text::Text,
    widgets::{
        Block, Widget,
        canvas::{Canvas, Points},
    },
};

use crate::cell::Cell;

pub struct Universe {
    speed: Option<u32>,
    grid: Vec<Vec<Cell>>,
    seed: Option<u64>,
    density: Option<f64>,
    path: Option<PathBuf>,
    exit: bool,
    marker: Marker,
    size: Size,
}

impl Universe {
    fn with_speed(speed: u32) -> Self {
        if speed == 0 {
            panic!("speed must be greater than 0");
        }

        Self {
            speed: Some(speed),
            seed: None,
            size: Size::default(),
            density: None,
            grid: vec![],
            path: None,
            marker: Marker::Block,
            exit: false,
        }
    }

    pub fn new(seed: u64, density: f64, speed: u32, size: Size) -> Self {
        if !(0.0..=1.0).contains(&density) {
            panic!("density must be between 0 and 1");
        }

        let mut universe = Self {
            seed: Some(seed),
            density: Some(density),
            size,
            ..Self::with_speed(speed)
        };

        universe.initialize_random();

        universe
    }

    pub fn from_plaintext_file(path: PathBuf, speed: u32, size: Size) -> Self {
        let mut universe = Self {
            path: Some(path),
            speed: Some(speed),
            size,
            ..Self::with_speed(speed)
        };

        universe.parse_text_file().unwrap();

        universe
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let speed = match self.speed {
            Some(speed) => speed,
            None => {
                log::warn!("speed not set, using default speed of 60");
                60
            }
        };

        let tick_rate = Duration::from_millis(1000 / speed as u64);
        let mut last_tick = Instant::now();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_press(key);
                }
            }

            if last_tick.elapsed() >= tick_rate {
                let grid = Self::compute_next_generation(self);
                self.set_grid(grid);
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let header = Text::from_iter([
            "Conway's Game of Life".bold(),
            "<q> Quit | <enter> Change Marker".into(),
        ]);

        let vertical_layout = Layout::vertical([
            Constraint::Length(header.height() as u16), // Header area
            Constraint::Min(0),                         // Canvas takes remaining space
        ]);

        let [header_area, canvas_area] = vertical_layout.areas(frame.area());

        frame.render_widget(header.centered(), header_area);
        frame.render_widget(self.draw_canvas(canvas_area), canvas_area);
    }

    fn draw_canvas(&self, area: Rect) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::bordered().title("Universe"))
            .marker(self.marker)
            .x_bounds([0.0, f64::from(area.width)])
            .y_bounds([0.0, f64::from(area.height)])
            .paint(move |ctx| {
                let points = self
                    .grid
                    .iter()
                    .enumerate()
                    .flat_map(|(x, row)| {
                        row.iter().enumerate().filter_map(move |(y, cell)| {
                            if cell.is_alive() {
                                Some((y as f64, x as f64))
                            } else {
                                None
                            }
                        })
                    })
                    .collect::<Vec<(f64, f64)>>();
                ctx.draw(&Points {
                    coords: &points,
                    color: Color::White,
                });
            })
    }

    fn handle_key_press(&mut self, key: event::KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Enter => {
                self.marker = match self.marker {
                    Marker::Dot => Marker::Braille,
                    Marker::Braille => Marker::Block,
                    Marker::Block => Marker::HalfBlock,
                    Marker::HalfBlock => Marker::Bar,
                    Marker::Bar => Marker::Dot,
                };
            }
            _ => {}
        }
    }

    pub fn initialize_random(&mut self) {
        let density = self.density.unwrap_or(0.5);
        let seed = self.seed.unwrap_or(0);

        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let width = self.size.width as usize;
        let height = self.size.height as usize;

        for _ in 0..height {
            let row: Vec<Cell> = (0..width)
                .map(|_| Cell::new(rng.random_bool(density)))
                .collect();
            self.grid.push(row);
        }
    }

    pub fn parse_text_file(&mut self) -> Result<(), Error> {
        let path = self.path.as_ref().ok_or(Error::msg("file path not set"))?;
        let file_contents = std::fs::read_to_string(path)?;

        let grid_width = self.size.width as usize;
        let grid_height = self.size.height as usize;

        // Read and filter empty and comment lines
        let pattern_lines: Vec<&str> = file_contents
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with("!"))
            .collect();

        // Calculate pattern dimensions
        let pattern_height = pattern_lines.len();
        let pattern_width = pattern_lines
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0);

        // Ensure grid is large enough
        if pattern_width > grid_width || pattern_height > grid_height {
            return Err(Error::msg("Grid too small for pattern"));
        }

        // Calculate centering offsets
        let vert_pad = grid_height.saturating_sub(pattern_height);
        let top_pad = vert_pad / 2;
        let left_pad = (grid_width.saturating_sub(pattern_width)) / 2;

        // Build vertically centered grid
        self.grid.clear();
        for _ in 0..top_pad {
            self.grid.push(vec![Cell::default(); grid_width]);
        }

        for line in pattern_lines {
            let mut row = vec![Cell::default(); grid_width];
            let start_idx = left_pad;
            let end_idx = (left_pad + line.len()).min(grid_width);

            line.chars()
                .take(end_idx - start_idx)
                .enumerate()
                .for_each(|(i, c)| {
                    //start from left_pad to horizontally center the pattern
                    row[start_idx + i] = Cell::new(c != '.');
                });

            self.grid.push(row);
        }

        // Fill remaining rows for vertical centering
        while self.grid.len() < grid_height {
            self.grid.push(vec![Cell::default(); grid_width]);
        }
        Ok(())
    }

    pub fn compute_next_generation(&mut self) -> Vec<Vec<Cell>> {
        let current_grid = &self.grid;
        let rows = current_grid.len();
        let cols = if rows > 0 { current_grid[0].len() } else { 0 };

        (0..rows)
            .map(|x| {
                (0..cols)
                    .map(|y| Cell::new(Self::tick(rows, cols, current_grid, x, y)))
                    .collect()
            })
            .collect()
    }

    fn tick(rows: usize, cols: usize, current_grid: &[Vec<Cell>], x: usize, y: usize) -> bool {
        let cell = &current_grid[x][y];

        // Pre computed neighbor offsets
        const NEIGHBOR_OFFSETS: [(i32, i32); 8] = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        let alive_neighbors = NEIGHBOR_OFFSETS
            .iter()
            .filter_map(|(dx, dy)| {
                let x = x as i32 + dx;
                let y = y as i32 + dy;
                (x >= 0 && x < rows as i32 && y >= 0 && y < cols as i32)
                    .then(|| current_grid[x as usize][y as usize].is_alive())
            })
            .filter(|&alive| alive)
            .count();

        // Apply Conway's Game of Life rules
        matches!(
            (cell.is_alive(), alive_neighbors),
            (true, 2 | 3) | (false, 3)
        )
    }

    pub fn set_grid(&mut self, grid: Vec<Vec<Cell>>) {
        self.grid = grid;
    }
}
