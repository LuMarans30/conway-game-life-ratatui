use std::path::PathBuf;

use color_eyre::eyre::Result;
use ratatui::{layout::Size, symbols::Marker};

use crate::universe::Universe;

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
    pub fn new(size: Size, speed: Option<u32>, seed: Option<u64>, density: Option<f64>) -> Self {
        Self {
            size,
            speed: speed.unwrap_or(30),
            initialization: UniverseInitialization::Random {
                seed: seed.unwrap_or(1),
                density: density.unwrap_or(0.5).clamp(0.0, 1.0),
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
        let mut universe = Universe::new(self.size, self.speed, vec![], false, Marker::Block);

        match self.initialization {
            UniverseInitialization::Random { seed, density } => {
                universe.initialize_random(seed, density)
            }
            UniverseInitialization::File(path) => universe.parse_text_file(path)?,
        }

        Ok(universe)
    }
}
