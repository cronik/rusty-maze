use std::fmt;
use std::fs::File;
use std::io::Write;

use serde::{Deserialize, Serialize};
use termion::{clear, color, cursor, style};
use termion::cursor::Goto;
use termion::event::Key;

use crate::game::GameCommand::{NewGame, Quit};
use crate::maze::{Difficulty, Direction, Joystick, Locate, Maze, MazeUI, Opts, Position};

enum GameCommand {
    Quit,
    NewGame,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameState {
    maze: Maze,
    difficulty: Difficulty,
    pos: Position,
    moves: Vec<(Position, Option<Direction>)>
}

/// The game state.
pub struct Game<R, W: Write> {
    /// Standard output.
    stdout: W,
    /// Standard input.
    stdin: R,
    width: u16,
    height: u16,
    difficulty: Difficulty,
    show_path: bool,
    path_visible: bool
}

impl<R, W: Write> Drop for Game<R, W> {
    fn drop(&mut self) {
        // When done, restore the defaults to avoid messing with the terminal.
        write!(self.stdout, "{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1)).unwrap();
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Goto(self.x + 1, self.y + 1))
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Difficulty::Normal =>  write!(f, "NORMAL"),
            Difficulty::Hard =>  write!(f, "HARD"),
        }
    }
}

impl<R: Iterator<Item=Result<Key, std::io::Error>>, W: Write> Game<R, W> {

    pub fn init(mut stdout: W, stdin: R, width: u16, height: u16, difficulty: Difficulty) {
        write!(stdout, "{}", clear::All).unwrap();
        println!("generating {}x{} maze...", width, height);
        let mut game = Game {
            stdin,
            stdout,
            width,
            height,
            difficulty,
            show_path: false,
            path_visible: false
        };

        // Start the event loop.
        loop {
            match game.start(None) {
                Quit => return,
                NewGame => continue
            };
        }
    }

    pub fn restore(mut stdout: W, stdin: R, gs: &GameState) {
        write!(stdout, "{}", clear::All).unwrap();
        println!("restoring maze...");
        let mut game = Game {
            stdin,
            stdout,
            width: gs.maze.width,
            height: gs.maze.height,
            difficulty: gs.difficulty,
            show_path: false,
            path_visible: false
        };

        // Start the event loop.
        let mut state = Some(gs);
        loop {
            match game.start(state) {
                Quit => return,
                NewGame => { state = None; }
            };
        }
    }

    fn draw_maze(&mut self, maze: &MazeUI) {
        // Reset the cursor.
        write!(self.stdout, "{}", cursor::Goto(1, 1)).unwrap();

        for r in maze.draw() {
            for c in r {
                self.stdout.write(c.to_string().as_bytes()).unwrap();
            }
            self.stdout.write(b"\n\r").unwrap();
        }

        let exit = maze.exit().mv(&Direction::Left, 2);
        write!(self.stdout, "{}{}Exit{}", exit, color::Fg(color::Green), style::Reset).unwrap();
        write!(self.stdout, "{}n: new, p: path, q: exit, e: save | {}{}", Goto(1, maze.dimensions().1 + 2), self.difficulty, style::Reset).unwrap();
        self.stdout.flush().unwrap();
    }

    fn draw_path(&mut self, ui: &MazeUI, j: &Joystick, show: bool) {
        if !self.path_visible && !show {
            return;
        }
        let mut last: Option<Position> = None;
        let pad = " ".repeat(ui.cell_width as usize);
        for p in j.history.iter() {
            if show {
                write!(self.stdout, "{}{} {}", ui.locate(&p.0), color::Bg(color::Blue), style::Reset).unwrap();
            } else {
                write!(self.stdout, "{} {}", ui.locate(&p.0), style::Reset).unwrap();
            }
            if let Some(l) = last {
                if let Some(d) = p.1 {
                    match d {
                        Direction::Left => {
                            let lp = ui.locate(&l).mv(&d, ui.cell_width);
                            if show {
                                write!(self.stdout, "{}{}{}{}", lp, color::Bg(color::Blue), pad, style::Reset).unwrap();
                            } else {
                                write!(self.stdout, "{}{}{}", lp, pad, style::Reset).unwrap();
                            }
                        },
                        Direction::Right => {
                            let lp = ui.locate(&l);
                            if show {
                                write!(self.stdout, "{}{}{}{}", lp, color::Bg(color::Blue), pad, style::Reset).unwrap();
                            } else {
                                write!(self.stdout, "{}{}{}", lp, pad, style::Reset).unwrap();
                            }
                        },
                        _ => {
                            let lp = ui.locate(&l).mv(&d, 1);
                            if show {
                                write!(self.stdout, "{}{} {}", lp, color::Bg(color::Blue), style::Reset).unwrap();
                            } else {
                                write!(self.stdout, "{} {}", lp, style::Reset).unwrap();
                            }
                        }
                    }
                }
            }
            last = Some(p.0);
        }
        self.path_visible = show;
    }

    fn save(&self, m: &Maze, j: &Joystick) {
        let state = GameState {
            maze: m.clone(),
            difficulty: self.difficulty,
            pos: j.pos,
            moves: j.history.clone()
        };
        let out = File::create("maze.ron").unwrap();
        ron::ser::to_writer(out, &state).unwrap();
    }

    /// generate maze and start game loop
    fn start(&mut self, state: Option<&GameState>) -> GameCommand {
        let maze= match state {
            Some(gs) => gs.maze.clone(),
            None => Maze::generate(self.width, self.height, &Opts { difficulty: self.difficulty })
        };
        let mut joystick= maze.joystick();
        if let Some(gs) = state {
            joystick.pos = gs.pos;
        }
        let ui = maze.ui();
        self.draw_maze(&ui);
        write!(self.stdout, "{}", ui.locate(&joystick)).unwrap();
        self.stdout.flush().unwrap();
        loop {
            // Read a single byte from stdin.
            let b = self.stdin.next().unwrap().unwrap();
            use termion::event::Key::*;
            match b {
                Char('h') | Char('a') | Left => { joystick.left(); }
                Char('j') | Char('s') | Down => { joystick.down(); }
                Char('k') | Char('w') | Up => { joystick.up(); }
                Char('l') | Char('d') | Right => { joystick.right(); }
                Char('r') => { joystick.reset(); }
                Char('e') => { self.save(&maze, &joystick); }
                Char('p') => { self.show_path = !self.show_path; }
                Char('n') => return NewGame,
                Char('q') => return Quit,
                _ => (),
            }

            self.draw_path(&ui, &joystick, joystick.is_exit() || self.show_path);
            // Make sure the cursor is placed on the current position.
            write!(self.stdout, "{}", ui.locate(&joystick)).unwrap();
            self.stdout.flush().unwrap();
        }
    }
}