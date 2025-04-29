use color_eyre::eyre::Error;
use rletxtconv::universe::Universe;
use std::path::PathBuf;

use crate::cell::Cell;

pub struct Parser {
    width: usize,
    height: usize,
}

pub trait ParseInput {
    fn parse_input(self) -> Result<Universe, Error>;
}

impl ParseInput for &str {
    fn parse_input(self) -> Result<Universe, Error> {
        let (_, universe) = rletxtconv::parse_text(self)?;
        Ok(universe)
    }
}

impl ParseInput for PathBuf {
    fn parse_input(self) -> Result<Universe, Error> {
        let (_, universe) = rletxtconv::parse_file(&self)?;
        Ok(universe)
    }
}

impl Parser {
    pub fn new(width: usize, height: usize) -> Self {
        Parser { width, height }
    }

    /// Single parse method handling both String and PathBuf inputs
    pub fn parse<T: ParseInput>(&mut self, input: T) -> Result<Vec<Vec<Cell>>, Error> {
        let universe = input.parse_input()?;
        padding_grid(universe, self.width, self.height)
    }
}

fn padding_grid(
    universe: Universe,
    grid_width: usize,
    grid_height: usize,
) -> Result<Vec<Vec<Cell>>, Error> {
    let pattern_width = universe.width;
    let pattern_height = universe.height;

    if pattern_width > grid_width || pattern_height > grid_height {
        return Err(Error::msg("Grid too small for pattern"));
    }

    // Calculate centering offsets
    let top_pad = (grid_height - pattern_height) / 2;
    let left_pad = (grid_width - pattern_width) / 2;

    let mut grid = vec![vec![Cell::default(); grid_width]; grid_height];

    universe
        .cells
        .chunks(pattern_width)
        .enumerate()
        .for_each(|(row_idx, pattern_row)| {
            let target_row = top_pad + row_idx;
            pattern_row.iter().enumerate().for_each(|(col_idx, &cell)| {
                let target_col = left_pad + col_idx;
                grid[target_row][target_col] = Cell::new(cell);
            });
        });

    Ok(grid)
}
