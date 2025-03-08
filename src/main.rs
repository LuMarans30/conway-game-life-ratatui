use clap::{Parser, arg, command};
use color_eyre::Result;
use crossterm::{
    ExecutableCommand,
    event::{DisableMouseCapture, EnableMouseCapture},
};
use std::{io::stdout, path::PathBuf};

mod cell;
mod universe;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Seed for the random grid generation
    #[arg(short, long, default_value_t = 1)]
    seed: u64,

    /// Density of the universe in range (0,1]
    #[arg(short = 'D', long, default_value_t = 0.5)]
    density: f64,

    /// Path to a text file to initialize the universe
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// speed (frames per second) for the simulation
    #[arg(short = 'S', long, default_value_t = 30)]
    speed: u32,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let Args {
        seed,
        density,
        path,
        speed,
        ..
    } = args;

    let terminal = ratatui::init();

    let mut universe = match path {
        Some(path) => {
            universe::Universe::from_plaintext_file(path, speed, terminal.size().unwrap())
        }
        None => universe::Universe::new(seed, density, speed, terminal.size().unwrap()),
    };

    stdout().execute(EnableMouseCapture)?;
    let app_result = universe.run(terminal);
    stdout().execute(DisableMouseCapture)?;
    ratatui::restore();
    app_result
}
