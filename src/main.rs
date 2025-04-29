use clap::{Args, Parser, Subcommand, ValueHint};
use color_eyre::{
    Result,
    eyre::{Error, eyre},
};
use crossterm::{
    ExecutableCommand,
    event::{DisableMouseCapture, EnableMouseCapture},
};
use std::{
    io::{BufRead, IsTerminal, Read, stdout},
    path::PathBuf,
};

mod cell;
mod parser;
mod universe;
mod universe_builder;

use universe_builder::UniverseBuilder;

#[derive(Debug, Parser)]
#[clap(version, about, long_about = None)]
pub struct App {
    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[clap(subcommand)]
    command: Option<Command>,
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
        #[clap(short, long, required = true, value_hint = ValueHint::FilePath)]
        path: Option<PathBuf>,
    },
}

#[derive(Debug, Args)]
struct GlobalOpts {
    /// speed (frames per second) for the simulation
    #[clap(short = 'S', long, default_value_t = 30)]
    speed: u32,
    /// cell color in RGB format (e.g. RRR,GGG,BBB)
    #[clap(short, long, default_value = "255,255,255")]
    color: String,
}

fn main() -> Result<()> {
    let app_result = run();
    stdout().execute(DisableMouseCapture)?;
    ratatui::restore();
    app_result
}

fn run() -> Result<()> {
    color_eyre::install()?;
    let args = App::parse();

    let App {
        global_opts,
        command,
    } = args;

    let terminal = ratatui::init();
    let size = terminal
        .size()
        .map_err(|_| eyre!("Failed to get terminal size"))?;

    let universe_builder = UniverseBuilder::new(size, None, None, None, None)
        .speed(global_opts.speed)
        .color(global_opts.color);

    let mut universe = {
        match command {
            Some(Command::File { path }) => universe_builder.with_file(path.unwrap()).build(),
            Some(Command::Random { seed, density }) => {
                universe_builder.random(seed, density).build()
            }
            None => match get_stdin_input() {
                Ok(input) => universe_builder.with_stdin(input).build(),
                Err(_) => universe_builder.random(1, 0.5).build(),
            },
        }?
    };

    stdout().execute(EnableMouseCapture)?;
    universe.run(terminal)
}

fn get_stdin_input() -> Result<String, Error> {
    if std::io::stdin().is_terminal() {
        return Err(eyre!("No stdin input provided"));
    }

    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| eyre!("Failed to read stdin: {}", e))?;

    if input.trim().is_empty() {
        Err(eyre!("stdin input is empty"))
    } else {
        Ok(input)
    }
}
