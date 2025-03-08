use std::{fmt::Display, path::PathBuf};

use color_eyre::{Result, eyre::Error};
use rand::{Rng, SeedableRng};

use crate::cell::Cell;

pub struct Universe {
    dimension: u32,
    grid: Vec<Vec<Cell>>,
    seed: Option<u64>,
    density: Option<f64>,
    path: Option<PathBuf>,
    alive_char: String,
    dead_char: String,
}

impl Universe {
    fn default(dimension: u32) -> Self {
        if dimension == 0 {
            panic!("dimension must be greater than 0");
        }

        Self {
            seed: None,
            density: None,
            dimension,
            grid: vec![],
            path: None,
            alive_char: String::from("█"),
            dead_char: String::from(" "),
        }
    }

    pub fn new(dimension: u32, seed: u64, density: f64) -> Self {
        if !(0.0..=1.0).contains(&density) {
            panic!("density must be between 0 and 1");
        }

        let mut universe = Self {
            seed: Some(seed),
            density: Some(density),
            ..Self::default(dimension)
        };

        universe.initialize_random();

        universe
    }

    pub fn from_plaintext_file(dimension: u32, path: Option<PathBuf>) -> Self {
        let mut universe = Self {
            path,
            ..Self::default(dimension)
        };

        universe.parse_text_file().unwrap();

        universe
    }

    pub fn initialize_random(&mut self) {
        let density = match self.density {
            Some(density) => density,
            None => {
                log::warn!("density not set, using default density of 0.5");
                0.5
            }
        };

        let seed = match self.seed {
            Some(seed) => seed,
            None => {
                log::warn!("seed not set, using default seed of 0");
                0
            }
        };

        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        for _ in 0..self.dimension {
            let row: Vec<Cell> = vec![Cell::new(rng.random_bool(density)); self.dimension as usize];
            self.grid.push(row);
        }
    }

    pub fn parse_text_file(&mut self) -> Result<(), Error> {
        let path = match &self.path {
            Some(path) => {
                if !path.exists() {
                    return Err(Error::msg("file does not exist"));
                }

                path
            }
            None => {
                return Err(Error::msg("file path not set"));
            }
        };

        let file_contents = std::fs::read_to_string(path)?;

        let dimension = self.dimension as usize;

        // Read and filter empty and comment lines
        let mut lines: Vec<&str> = file_contents
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with("!"))
            .collect();

        // Truncate or pad rows to match dimension
        lines.truncate(dimension);
        while lines.len() < dimension {
            lines.push("");
        }

        self.grid.clear();

        for line in lines {
            let mut row: Vec<Cell> = line
                .chars()
                .map(|c| Cell::new(c != '.'))
                .take(dimension)
                .collect();

            // Pad with dead cells if the row is too short
            if row.len() < dimension {
                row.resize(dimension, Cell::default());
            }

            self.grid.push(row);
        }

        while self.grid.len() < dimension {
            self.grid.push(vec![Cell::default(); dimension]);
        }

        Ok(())
    }

    pub fn compute_next_generation(&mut self) -> Vec<Vec<Cell>> {
        let current_grid = &self.grid;
        let rows = current_grid.len();
        let cols = if rows > 0 { current_grid[0].len() } else { 0 };

        let mut next_grid = vec![vec![Cell::default(); cols]; rows];

        next_grid
            .iter_mut()
            .enumerate()
            .take(rows)
            .for_each(|(x, row)| {
                row.iter_mut().enumerate().take(cols).for_each(|(y, cell)| {
                    cell.set_state(Self::tick(rows, cols, current_grid, x, y));
                })
            });

        next_grid
    }

    fn tick(rows: usize, cols: usize, current_grid: &[Vec<Cell>], x: usize, y: usize) -> bool {
        let mut alive_neighbors = 0;
        let cell = &current_grid[x][y];

        for dx in -1..=1 {
            for dy in -1..=1 {
                // Skip the current cell
                if dx == 0 && dy == 0 {
                    continue;
                }

                let neighbor_x = x as i32 + dx;
                let neighbor_y = y as i32 + dy;

                // Check if neighbor is within grid bounds
                if neighbor_x >= 0
                    && neighbor_x < rows as i32
                    && neighbor_y >= 0
                    && neighbor_y < cols as i32
                {
                    let neighbor_x = neighbor_x as usize;
                    let neighbor_y = neighbor_y as usize;
                    if current_grid[neighbor_x][neighbor_y].is_alive() {
                        alive_neighbors += 1;
                    }
                }
            }
        }

        // Apply Conway's Game of Life rules
        match (cell.is_alive(), alive_neighbors) {
            (true, 2 | 3) => true, // Survival
            (false, 3) => true,    // Reproduction
            _ => false,            // Death
        }
    }

    pub fn clear_screen(&self) {
        std::process::Command::new("clear").status().unwrap();
    }

    pub fn set_grid(&mut self, grid: Vec<Vec<Cell>>) {
        self.grid = grid;
    }

    pub fn set_alive_char(&mut self, alive_char: String) {
        self.alive_char = alive_char;
    }

    pub fn set_dead_char(&mut self, dead_char: String) {
        self.dead_char = dead_char;
    }
}

impl Display for Universe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let grid_string = self
            .grid
            .iter()
            .map(|row| {
                row.iter()
                    .map(|cell| {
                        if cell.is_alive() {
                            self.alive_char.clone()
                        } else {
                            self.dead_char.clone()
                        }
                    })
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n");

        self.clear_screen();
        write!(f, "{}", grid_string)
    }
}
