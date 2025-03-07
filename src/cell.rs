use std::fmt::Display;

#[derive(Clone)]
pub struct Cell {
    is_alive: bool,
}

impl Cell {
    pub fn default() -> Self {
        Self { is_alive: false }
    }

    pub fn new(is_alive: bool) -> Self {
        Self { is_alive }
    }

    pub fn is_alive(&self) -> bool {
        self.is_alive
    }

    pub fn set_state(&mut self, is_alive: bool) {
        self.is_alive = is_alive;
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        /* let char = match self.is_alive {
            true => "🙂",
            false => "☠️",
        }; */

        let char = match self.is_alive {
            true => "◻️",
            false => " ",
        };

        write!(f, "{char}")
    }
}
