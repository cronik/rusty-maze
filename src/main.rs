
use structopt::StructOpt;
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use rusty_maze::game::Game;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(short = "w", long = "width")]
    width: Option<u16>,
    #[structopt(short = "h", long = "height")]
    height: Option<u16>
}

fn main() {
    let opt:Opt = Opt::from_args();

    // Get and lock the stdios.
    let stdout = std::io::stdout();
    let stdout = stdout.lock();
    let stdin = std::io::stdin();
    let stdin = stdin.lock();

    // We go to raw mode to make the control over the terminal more fine-grained.
    let stdout = stdout.into_raw_mode().unwrap();

    let termsize = termion::terminal_size().ok();
    let termwidth = termsize.map(|(w,_)| w / 4);
    let termheight = termsize.map(|(_,h)| (h / 2) - 1);

    Game::init(stdout, stdin.keys(), opt.width.or(termwidth).unwrap().max(5), opt.height.or(termheight).unwrap().max(5))
}