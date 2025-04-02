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

    /// Initialize universe from plaintext file
    pub fn parse_text_file(&mut self, path: PathBuf) -> Result<Vec<Vec<Cell>>, Error> {
        path.try_exists()?;

        let file_contents = std::fs::read_to_string(path)?;
        let mut grid: Vec<Vec<Cell>> = Vec::new();

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
        if pattern_width > self.width || pattern_height > self.height {
            return Err(Error::msg("Grid too small for pattern"));
        }

        // Calculate centering offsets
        let vert_pad = self.height.saturating_sub(pattern_height);
        let top_pad = vert_pad / 2;
        let left_pad = (self.width.saturating_sub(pattern_width)) / 2;

        // Build vertically centered grid
        grid.clear();
        for _ in 0..top_pad {
            grid.push(vec![Cell::default(); self.width]);
        }

        for line in pattern_lines {
            let mut row = vec![Cell::default(); self.width];
            let start_idx = left_pad;
            let end_idx = (left_pad + line.len()).min(self.width);

            line.chars()
                .take(end_idx - start_idx)
                .enumerate()
                .for_each(|(i, c)| {
                    //start from left_pad to horizontally center the pattern
                    row[start_idx + i] = Cell::new(c != '.');
                });

            grid.push(row);
        }

        // Fill remaining rows for vertical centering
        while grid.len() < self.height {
            grid.push(vec![Cell::default(); self.width]);
        }

        Ok(grid)
    }
}
