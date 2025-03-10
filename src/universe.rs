use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use color_eyre::{eyre::Error, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use rand::{Rng, SeedableRng};
use ratatui::{
    layout::{Constraint, Layout, Rect, Size},
    style::{Color, Stylize},
    symbols::Marker,
    text::Text,
    widgets::{
        canvas::{Canvas, Points},
        Block, Widget,
    },
    DefaultTerminal, Frame,
};

use crate::cell::Cell;

pub struct Universe {
    speed: u32,
    grid: Vec<Vec<Cell>>,
    marker: Marker,
    exit: bool,
    size: Size,
}

pub struct UniverseBuilder {
    size: Size,
    speed: u32,
    initialization: UniverseInitialization,
}

enum UniverseInitialization {
    Random { seed: u64, density: f64 },
    File(PathBuf),
}

impl UniverseBuilder {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            speed: 30,
            initialization: UniverseInitialization::Random {
                seed: 1,
                density: 0.5,
            },
        }
    }

    pub fn speed(mut self, speed: u32) -> Self {
        self.speed = speed;
        self
    }

    pub fn random(mut self, seed: u64, density: f64) -> Self {
        self.initialization = UniverseInitialization::Random { seed, density };
        self
    }

    pub fn with_file(mut self, path: PathBuf) -> Self {
        self.initialization = UniverseInitialization::File(path);
        self
    }

    pub fn build(self) -> Result<Universe> {
        let mut universe = Universe {
            speed: self.speed,
            grid: vec![],
            marker: Marker::Block,
            exit: false,
            size: self.size,
        };

        match self.initialization {
            UniverseInitialization::Random { seed, density } => {
                if !(0.0..=1.0).contains(&density) {
                    return Err(Error::msg("Density must be in range [0,1]"));
                }
                universe.initialize_random(seed, density);
            }
            UniverseInitialization::File(path) => {
                universe.parse_text_file(path)?;
            }
        }

        Ok(universe)
    }
}

impl Universe {
    fn initialize_random(&mut self, seed: u64, density: f64) {
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

    fn parse_text_file(&mut self, path: PathBuf) -> Result<(), Error> {
        path.try_exists()?;

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

    /// Runs the simulation loop until the user exits. <br />
    /// It computes the next generation of the grid at a fixed speed. <br />
    /// The speed parameter controls the frames per second of the simulation.
    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let tick_rate = Duration::from_millis(1000 / self.speed as u64);
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

    /// Applies the rules of Life to each cell in the grid to compute the next generation.
    fn compute_next_generation(&self) -> Vec<Vec<Cell>> {
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

    /// Applies the rules of Life to a single cell in the grid. <br />
    /// Returns true if the cell should be alive in the next generation.
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

    fn set_grid(&mut self, grid: Vec<Vec<Cell>>) {
        self.grid = grid;
    }
}
