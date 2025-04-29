use std::time::{Duration, Instant};

use color_eyre::{
    Result,
    eyre::{Error, eyre},
};
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

use crate::{
    cell::Cell,
    parser::{ParseInput, Parser},
};

pub struct Universe {
    speed: u32,
    grid: Vec<Vec<Cell>>,
    marker: Marker,
    color: String,
    exit: bool,
    size: Size,
}

impl Universe {
    pub fn new(
        size: Size,
        speed: u32,
        grid: Vec<Vec<Cell>>,
        exit: bool,
        marker: Marker,
        color: String,
    ) -> Self {
        Self {
            speed,
            grid,
            marker,
            color,
            exit,
            size,
        }
    }

    pub fn init_random(&mut self, seed: u64, density: f64) {
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

    pub fn parse<T: ParseInput>(&mut self, input: T) -> Result<(), Error> {
        let mut parser = Parser::new(self.size.width as usize, self.size.height as usize);
        let grid = parser.parse(input)?;
        self.set_grid(grid);
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
                let color = match Self::parse_color(&self.color) {
                    Ok(color) => color,
                    Err(e) => {
                        eprintln!("Error parsing color ({}): {:?}", self.color, e);
                        Color::White // Default color on error
                    }
                };

                ctx.draw(&Points {
                    coords: &points,
                    color,
                });
            })
    }

    fn parse_color(color: &str) -> Result<Color, Error> {
        let tokens: Vec<&str> = color
            .split(|c: char| !c.is_ascii_hexdigit())
            .filter(|s| !s.is_empty())
            .collect();

        if tokens.len() != 3 {
            return Err(eyre!("Invalid RGB format. Expected 3 components"));
        }

        let mut components = [0u8; 3];
        for (i, token) in tokens.iter().enumerate() {
            components[i] = token
                .parse::<u8>()
                .map_err(|_| eyre!("Invalid hex value: {}", token))?;
        }

        Ok(Color::Rgb(components[0], components[1], components[2]))
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
