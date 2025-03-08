use clap::{Parser, arg, command};
use color_eyre::Result;
use std::path::PathBuf;

mod cell;
mod universe;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Dimension of the universe (NxN)
    #[arg(short, long, required = true)]
    dimension: u32,

    /// Seed for the random grid generation
    #[arg(short, long, default_value_t = 1)]
    seed: u64,

    /// Density of the universe in range (0,1]
    #[arg(short, long, default_value_t = 0.5)]
    density: f64,

    /// Path to a text file to initialize the universe
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Alive character or string for the cell
    #[arg(short, long, default_value = "█")]
    alive_char: String,

    /// Dead character or string for the cell
    #[arg(short, long, default_value = " ")]
    dead_char: String,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let dimension = args.dimension;
    let seed = args.seed;
    let density = args.density;
    let alive_char = args.alive_char;
    let dead_char = args.dead_char;

    let mut universe = match args.path {
        Some(_) => universe::Universe::from_plaintext_file(dimension, args.path),
        None => universe::Universe::new(dimension, seed, density),
    };

    universe.set_alive_char(alive_char);
    universe.set_dead_char(dead_char);

    loop {
        let updated_grid = universe.compute_next_generation();
        universe.set_grid(updated_grid);
        println!("{}", universe);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
