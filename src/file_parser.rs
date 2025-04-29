use color_eyre::eyre::Error;
use std::path::PathBuf;

use crate::cell::Cell;

pub struct FileParser {
    width: usize,
    height: usize,
}

impl FileParser {
    pub fn new(width: usize, height: usize) -> Self {
        FileParser { width, height }
    }

    /// Initialize universe from plaintext or RLE file
    pub fn parse_text_file(&mut self, path: PathBuf) -> Result<Vec<Vec<Cell>>, Error> {
        let (_, universe) = rletxtconv::parse_file(&path)?;

        let pattern_width = universe.width;
        let pattern_height = universe.height;

        if pattern_width > self.width || pattern_height > self.height {
            return Err(Error::msg("Grid too small for pattern"));
        }

        // Calculate centering offsets
        let top_pad = (self.height - pattern_height) / 2;
        let left_pad = (self.width - pattern_width) / 2;

        let mut grid = vec![vec![Cell::default(); self.width]; self.height];

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
}
