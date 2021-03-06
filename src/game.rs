use crate::maze::{Maze, Joystick, Position, Direction};
use std::io::{Write};
use termion::{clear, cursor, color, style};
use termion::event::Key;
use termion::cursor::Goto;
use std::fmt;
use crate::game::GameCommand::{NewGame, Quit};

enum GameCommand {
    Quit,
    NewGame,
}

/// The game state.
pub struct Game<R, W: Write> {
    width: u16,
    height: u16,
    /// Standard output.
    stdout: W,
    /// Standard input.
    stdin: R,
    show_path: bool
}

impl<R, W: Write> Drop for Game<R, W> {
    fn drop(&mut self) {
        // When done, restore the defaults to avoid messing with the terminal.
        write!(self.stdout, "{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1)).unwrap();
    }
}

impl fmt::Display for Joystick<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pos = self.board_pos();
        write!(f, "{}", Goto(pos.x + 1, pos.y + 1))
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Goto(self.x + 1, self.y + 1))
    }
}

impl<R: Iterator<Item=Result<Key, std::io::Error>>, W: Write> Game<R, W> {
    pub fn init(mut stdout: W, stdin: R, w: u16, h: u16) {
        write!(stdout, "{}", clear::All).unwrap();
        println!("generating {}x{} maze...", w, h);
        let mut game = Game {
            width: w,
            height: h,
            stdin,
            stdout,
            show_path: false,
        };

        // Start the event loop.
        loop {
            match game.start() {
                Quit => return,
                NewGame => continue
            };
        }
    }

    fn draw_maze(&mut self, maze: &Maze) {
        // Reset the cursor.
        write!(self.stdout, "{}", cursor::Goto(1, 1)).unwrap();

        for r in maze.draw_board() {
            for c in r {
                self.stdout.write(c.to_string().as_bytes()).unwrap();
            }
            self.stdout.write(b"\n\r").unwrap();
        }

        let exit = maze.exit_board_pos().mv(&Direction::Left, 2);
        write!(self.stdout, "{}{}Exit{}", exit, color::Fg(color::Green), style::Reset).unwrap();
        write!(self.stdout, "{}n: new, p: path, q: exit{}", Goto(1, maze.board_size().1 + 2), style::Reset).unwrap();
        self.stdout.flush().unwrap();
    }

    fn draw_path(&mut self, j: &Joystick, show: bool) {
        let mut last: Option<Position> = None;
        for p in j.history.iter() {
            if show {
                write!(self.stdout, "{}{} {}", p.0, color::Bg(color::Blue), style::Reset).unwrap();
                if let Some(l) = last {
                    if let Some(d) = p.1 {
                        let pad = " ".repeat(j.maze.cell_width as usize);
                        match d {
                            Direction::Left => { write!(self.stdout, "{}{}{}{}", l.mv(&d, j.maze.cell_width), color::Bg(color::Blue), pad, style::Reset).unwrap(); }
                            Direction::Right => { write!(self.stdout, "{}{}{}{}", l, color::Bg(color::Blue), pad, style::Reset).unwrap(); }
                            _ => { write!(self.stdout, "{}{} {}", l.mv(&d, 1), color::Bg(color::Blue), style::Reset).unwrap(); }
                        }
                    }
                }
            } else {
                write!(self.stdout, "{} {}", p.0, style::Reset).unwrap();
                if let Some(l) = last {
                    if let Some(d) = p.1 {
                        write!(self.stdout, "{} {}", l, style::Reset).unwrap();
                        let pad = " ".repeat(j.maze.cell_width as usize);
                        match d {
                            Direction::Left => { write!(self.stdout, "{}{}{}", l.mv(&d, j.maze.cell_width), pad, style::Reset).unwrap(); }
                            Direction::Right => { write!(self.stdout, "{}{}{}", l, pad, style::Reset).unwrap(); }
                            _ => {}
                        }
                    }
                }
            }
            last = Some(p.0);
        }
    }

    /// generate maze and start game loop
    fn start(&mut self) -> GameCommand {
        let maze = Maze::generate(self.width, self.height);
        let mut joystick = maze.joystick();
        self.draw_maze(&maze);
        write!(self.stdout, "{}", joystick).unwrap();
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
                Char('p') => { self.show_path = !self.show_path; }
                Char('n') => return NewGame,
                Char('q') => return Quit,
                _ => (),
            }

            self.draw_path(&joystick, joystick.is_exit() || self.show_path);
            // Make sure the cursor is placed on the current position.
            write!(self.stdout, "{}", joystick).unwrap();
            self.stdout.flush().unwrap();
        }
    }
}