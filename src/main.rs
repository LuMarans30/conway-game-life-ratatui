use clap::{Args, Parser, Subcommand};
use color_eyre::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    ExecutableCommand,
};
use std::{io::stdout, path::PathBuf};

mod cell;
mod universe;

#[derive(Debug, Parser)]
#[clap(version, about, long_about = None)]
pub struct App {
    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Generate a random universe
    Random {
        /// Seed for the random grid generation
        #[clap(short, long, default_value_t = 1)]
        seed: u64,

        /// Density of the universe in range (0,1]
        #[clap(short = 'D', long, default_value_t = 0.5)]
        density: f64,
    },
    /// Generate a universe from a text file
    File {
        /// Path to a text file to initialize the universe
        #[clap(short, long)]
        path: Option<PathBuf>,
    },
}

#[derive(Debug, Args)]
struct GlobalOpts {
    /// speed (frames per second) for the simulation
    #[clap(short = 'S', long, default_value_t = 30)]
    speed: u32,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = App::parse();

    let App {
        global_opts,
        command,
    } = args;

    let terminal = ratatui::init();

    let universe_builder =
        universe::UniverseBuilder::new(terminal.size().unwrap()).speed(global_opts.speed);

    let universe = match command {
        Command::File { path } => universe_builder.with_file(path.unwrap()).build(),
        Command::Random { seed, density } => universe_builder.random(seed, density).build(),
    };

    stdout().execute(EnableMouseCapture)?;
    let app_result = universe?.run(terminal);
    stdout().execute(DisableMouseCapture)?;
    ratatui::restore();
    app_result
}
