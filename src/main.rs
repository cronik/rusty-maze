use std::path::PathBuf;

use structopt::StructOpt;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use rusty_maze::game::{Game, GameState};
use rusty_maze::maze::Difficulty;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, StructOpt)]
#[structopt(name = "rusty_maze", about = "Rusty Maze Game")]
struct Opt {
    #[structopt(
        short = "w",
        long = "width",
        help = "Maze width [default: terminal width]"
    )]
    width: Option<u16>,
    #[structopt(
        short = "h",
        long = "height",
        help = "Maze height [default: terminal height]"
    )]
    height: Option<u16>,
    #[structopt(short = "d", long, default_value = "Hard", help = "Maze difficulty")]
    difficulty: Difficulty,
    #[structopt(name = "FILE", parse(from_os_str), help = "Maze data to restore")]
    file: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt: Opt = Opt::from_args();

    // Get and lock the stdios.
    let stdout = std::io::stdout();
    let stdout = stdout.lock();
    let stdin = std::io::stdin();
    let stdin = stdin.lock();

    // We go to raw mode to make the control over the terminal more fine-grained.
    let stdout = stdout.into_raw_mode()?;

    if let Some(path) = opt.file {
        let file = File::open(path)?;
        let state: GameState = ron::de::from_reader(BufReader::new(file))?;
        Game::restore(stdout, stdin.keys(), &state);
    } else {
        let termsize = termion::terminal_size().ok();
        let termwidth = termsize.map(|(w, _)| w / 4);
        let termheight = termsize.map(|(_, h)| (h / 2) - 1);

        let width = opt.width.or(termwidth).unwrap().max(5);
        let height = opt.height.or(termheight).unwrap().max(5);

        Game::init(stdout, stdin.keys(), width, height, opt.difficulty);
    }

    Ok(())
}
