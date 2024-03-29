use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::str::FromStr;
use std::vec;

use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::disjset::DisjSet;
use crate::disjset::Roots::DisJoint;
use crate::maze::Direction::{Down, Left, Right, Up};

#[derive(Error, Debug)]
pub enum MazeError {
    #[error("wall index out of bound: {:?}", .0)]
    WallOutOfBounds((u16, u16)),
    #[error("invalid difficulty setting")]
    DifficultyParseError,
    #[error("invalid size setting")]
    CellDrawSizeParseError,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

static DIRECTIONS: [Direction; 4] = [Left, Right, Up, Down];

pub struct CellBox {
    pub top: usize,
    pub left: usize,
    pub bottom: usize,
    pub right: usize,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Position {
    pub y: u16,
    pub x: u16,
}

impl Into<(usize, usize)> for Position {
    fn into(self) -> (usize, usize) {
        (self.x as usize, self.y as usize)
    }
}

impl Position {
    pub fn mv(&self, d: &Direction, amount: u16) -> Position {
        match d {
            Left => Position {
                x: self.x - amount,
                y: self.y,
            },
            Right => Position {
                x: self.x + amount,
                y: self.y,
            },
            Up => Position {
                x: self.x,
                y: self.y - amount,
            },
            Down => Position {
                x: self.x,
                y: self.y + amount,
            },
        }
    }
}

pub struct Joystick<'a> {
    pub pos: Position,
    pub maze: &'a Maze,
    pub history: Vec<(Position, Option<Direction>)>,
}

impl Joystick<'_> {
    fn create(maze: &Maze) -> Joystick {
        let pos = maze.cell_to_pos(maze.enter);
        Joystick {
            maze,
            pos,
            history: vec![(pos, None)],
        }
    }

    pub fn left(&mut self) -> &Joystick {
        self.mv(&Left);
        self
    }

    pub fn right(&mut self) -> &Joystick {
        self.mv(&Right);
        self
    }

    pub fn up(&mut self) -> &Joystick {
        self.mv(&Up);
        self
    }

    pub fn down(&mut self) -> &Joystick {
        self.mv(&Down);
        self
    }

    /// Apply as many moves are possible. Returns the completed moves.
    pub fn moves<'a, I>(&mut self, moves: I) -> Vec<&'a Direction>
    where
        I: Iterator<Item = &'a Direction>,
    {
        let mut completed = Vec::new();
        for m in moves {
            if self.mv(m) {
                completed.push(m);
            } else {
                break;
            }
        }
        completed
    }

    /// Attempt the given movement
    pub fn mv(&mut self, d: &Direction) -> bool {
        if let Some(p) = self.maze.move_pos(self.pos, d) {
            self.pos = p;
            self.history.push((p, Some(*d)));
            return true;
        }
        false
    }

    /// Reset the position to starting position
    pub fn reset(&mut self) -> &Joystick {
        self.pos = self.maze.cell_to_pos(self.maze.enter);
        self.history.clear();
        self
    }

    /// Check if current position is exit position
    pub fn is_exit(&self) -> bool {
        self.maze.pos_to_cell(self.pos) == self.maze.exit
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Difficulty {
    Normal,
    Hard,
}

impl FromStr for Difficulty {
    type Err = MazeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Hard" | "hard" | "h" => Ok(Difficulty::Hard),
            "Normal" | "normal" | "norm" | "n" => Ok(Difficulty::Normal),
            _ => Err(MazeError::DifficultyParseError),
        }
    }
}

pub struct Opts {
    pub difficulty: Difficulty,
}

impl Default for Opts {
    fn default() -> Self {
        Opts {
            difficulty: Difficulty::Hard,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Maze {
    walls: Vec<(u16, u16)>,
    enter: u16,
    exit: u16,
    size: u16,
    pub width: u16,
    pub height: u16,
}

/// Maze is created by constructing a Disjoint Set for all the cells in the maze grid.
/// Walls are randomly knocked down until a a path is made from the entrance (0,0)
/// to the exit (w,h). When the entrance and exit are part of the same set then we
/// know we have a path to the exit.
impl Maze {
    /// Create a new Maze of the given size
    pub fn generate(width: u16, height: u16, opts: &Opts) -> Maze {
        let size = width * height;
        let mut m = Maze {
            walls: vec![(0, 0); 0],
            enter: 0,
            exit: size - 1,
            size,
            width,
            height,
        };

        for c in 0..size {
            // create right cell wall if not end of a row
            if (c % width) != width - 1 {
                m.walls.push((c, c + 1));
            }

            // create bottom cell wall if not last row
            let b = c + width;
            if b < size {
                m.walls.push((c, b));
            }
        }

        // randomly destroy some walls
        let mut cells = DisjSet::new(size as usize);
        let mut rng = rand::thread_rng();
        match opts.difficulty {
            Difficulty::Hard => {
                // remove walls until every cell in the maze if part of the same set
                loop {
                    let i = rng.gen_range(0..m.walls.len());
                    let w = m.walls[i];
                    // only remove walls of different sets, otherwise the maze will be trivialized
                    if let DisJoint(r1, r2) = cells.find_roots(w.0 as usize, w.1 as usize) {
                        cells.union(r1, r2);
                        m.walls.remove(i);
                    }
                    if cells.distinct_sets() == 1 {
                        break;
                    }
                }
            }
            Difficulty::Normal => {
                // remove walls until enter and exit are of the same set
                while let DisJoint(_, _) = cells.find_roots(m.enter as usize, m.exit as usize) {
                    let i = rng.gen_range(0..m.walls.len());
                    let w = m.walls[i];
                    // only remove walls of different sets, otherwise the maze will be trivialized
                    if let DisJoint(r1, r2) = cells.find_roots(w.0 as usize, w.1 as usize) {
                        cells.union(r1, r2);
                        m.walls.remove(i);
                    }
                }
            }
        }

        return m;
    }

    /// Create new maze of the given size and walls.
    pub fn create(w: u16, h: u16, walls: Vec<(u16, u16)>) -> Result<Maze, MazeError> {
        let size = w * h;
        let m = Maze {
            walls: walls.clone(),
            enter: 0,
            exit: size - 1,
            width: w,
            height: h,
            size,
        };
        for w in walls {
            if w.0 > m.exit || w.1 > m.exit {
                return Err(MazeError::WallOutOfBounds(w));
            }
        }
        return Ok(m);
    }

    /// Compute the available movements for the given position in the grid.
    fn movements(&self, p: Position) -> HashSet<Direction> {
        let mut moves: HashSet<Direction> = HashSet::new();
        for d in DIRECTIONS.iter() {
            match self.move_pos(p, d) {
                Some(_) => {
                    moves.insert(d.clone());
                }
                None => (),
            };
        }
        return moves;
    }

    /// Attempt to move from the given position in the direction. If a wall prevents the move
    /// None is returned otherwise the new position grid position is returned.
    fn move_pos(&self, p: Position, d: &Direction) -> Option<Position> {
        if self.pos_to_cell(p) > self.exit {
            return None;
        }

        let dest = match d {
            Left => {
                if p.x == 0 {
                    None
                } else {
                    Some(Position { x: p.x - 1, y: p.y })
                }
            }
            Right => {
                if p.x == self.width - 1 {
                    None
                } else {
                    Some(Position { x: p.x + 1, y: p.y })
                }
            }
            Up => {
                if p.y == 0 {
                    None
                } else {
                    Some(Position { x: p.x, y: p.y - 1 })
                }
            }
            Down => {
                if p.y == self.height - 1 {
                    None
                } else {
                    Some(Position { x: p.x, y: p.y + 1 })
                }
            }
        };

        return match dest {
            Some(dp) => {
                let i1 = self.pos_to_cell(p);
                let i2 = self.pos_to_cell(dp);
                if self.walls.contains(&(i1, i2)) || self.walls.contains(&(i2, i1)) {
                    None
                } else {
                    Some(dp.clone())
                }
            }
            None => None,
        };
    }

    /// translate position to cell index
    fn pos_to_cell(&self, p: Position) -> u16 {
        p.y * self.width + p.x
    }

    /// translate cell index to a grid position
    fn cell_to_pos(&self, p: u16) -> Position {
        Position {
            x: p % self.width,
            y: p / self.width,
        }
    }

    /// create joystick for moving and tracking.
    pub fn joystick(&self) -> Joystick {
        Joystick::create(self)
    }

    pub fn ui(&self) -> MazeUI {
        MazeUI {
            cell_width: 4,
            cell_height: 2,
            maze: self,
        }
    }
}

pub struct MazeUI<'a> {
    pub cell_width: u16,
    pub cell_height: u16,
    maze: &'a Maze,
}

/// Maze position locator
pub trait Locate<T> {
    /// locate the center of the cell box for the given maze position
    fn locate(&self, t: &T) -> Position;
}

impl Locate<Position> for MazeUI<'_> {
    fn locate(&self, p: &Position) -> Position {
        let pbox = self.cell_box(p);
        Position {
            x: pbox.left as u16 + (self.cell_width / 2),
            y: pbox.top as u16 + (self.cell_height / 2),
        }
    }
}

impl Locate<Joystick<'_>> for MazeUI<'_> {
    fn locate(&self, j: &Joystick) -> Position {
        self.locate(&j.pos)
    }
}

impl MazeUI<'_> {
    /// compute the bounding box for a cell in the maze
    fn cell_box(&self, p: &Position) -> CellBox {
        CellBox {
            top: (p.y * self.cell_height) as usize,
            left: (p.x * self.cell_width) as usize,
            bottom: ((p.y * self.cell_height) + self.cell_height) as usize,
            right: ((p.x * self.cell_width) + self.cell_width) as usize,
        }
    }

    /// dimensions of maze board (width, height)
    pub fn dimensions(&self) -> (u16, u16) {
        (
            self.maze.width * self.cell_width,
            self.maze.height * self.cell_height,
        )
    }

    /// get the exit position
    pub fn exit(&self) -> Position {
        self.locate(&self.maze.cell_to_pos(self.maze.exit))
    }

    /// draw maze as a matrix of cell boxes
    pub fn draw(&self) -> Vec<Vec<char>> {
        // init board matrix
        let cp = self.cell_width - 1;
        let bw = ((self.maze.width * cp) + (self.maze.width + 1)) as usize; // board width
        let bh = ((self.maze.height * 2) + 1) as usize; // board height
        let mut board = vec![vec![' '; bw]; bh];

        let row = |r: &mut Vec<char>, st: char, end: char, join: char, pad: char| {
            let mut i = 0;
            r[i] = st;
            for c in 0..self.maze.width {
                for _ in 0..cp {
                    i = i + 1;
                    r[i] = pad;
                }
                if c < self.maze.width - 1 {
                    i = i + 1;
                    r[i] = join;
                }
            }
            i = i + 1;
            r[i] = end;
        };

        // build grid
        row(&mut board[0], '┌', '┐', '┬', '─');
        for i in 0..self.maze.height {
            let r = ((i * 2) + 1) as usize;
            row(&mut board[r], '│', '│', '│', ' ');
            row(&mut board[r + 1], '├', '┤', '┼', '─');
        }
        row(&mut board[bh - 1], '└', '┘', '┴', '─');

        // remove walls
        for i in 0..self.maze.size {
            let p = self.maze.cell_to_pos(i as u16);
            let pbox = self.cell_box(&p);
            let moves = self.maze.movements(p);
            if moves.contains(&Left) {
                for rw in pbox.top..=pbox.bottom {
                    board[rw][pbox.left] = ' ';
                }
            }
            if moves.contains(&Right) {
                for rw in pbox.top..=pbox.bottom {
                    board[rw][pbox.right] = ' ';
                }
            }
            if moves.contains(&Up) {
                for cl in pbox.left..=pbox.right {
                    board[pbox.top][cl] = ' ';
                }
            }
            if moves.contains(&Down) {
                for cl in pbox.left..=pbox.right {
                    board[pbox.bottom][cl] = ' ';
                }
            }
        }

        let mut corners = HashMap::new();
        corners.insert("    ", ' ');
        corners.insert("│   ", '╵');
        corners.insert("  │ ", '╷');
        corners.insert("│ │ ", '│');
        corners.insert(" ─  ", '╴');
        corners.insert("   ─", '╶');
        corners.insert(" ─ ─", '─');
        corners.insert("  │─", '┌');
        corners.insert("│  ─", '└');
        corners.insert(" ─│ ", '┐');
        corners.insert("│─  ", '┘');
        corners.insert("│─│ ", '┤');
        corners.insert("│ │─", '├');
        corners.insert(" ─│─", '┬');
        corners.insert("│─│─", '┼');
        corners.insert("│─ ─", '┴');
        // fix corners
        for i in 0..bh {
            if i % self.cell_height as usize == 0 {
                for j in 0..bw {
                    if j % self.cell_width as usize == 0 && board[i][j] == ' ' {
                        let mut chars = vec![' '; 4];
                        if i > 0 {
                            chars[0] = board[i.checked_sub(1).unwrap_or(0)][j]
                        }
                        if j > 0 {
                            chars[1] = board[i][j.checked_sub(1).unwrap_or(0)]
                        }
                        if i < bh - 1 {
                            chars[2] = board[i + 1][j]
                        }
                        if j < bw - 1 {
                            chars[3] = board[i][j + 1]
                        }
                        let spec = String::from_iter(chars);
                        match corners.get(spec.as_str()) {
                            Some(c) => board[i][j] = c.clone(),
                            None => board[i][j] = '*',
                        }
                    }
                }
            }
        }

        return board;
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_ui_draw() {
        let m = Maze::generate(15, 15, &Default::default());
        let matrix = m.ui().draw();
        for r in matrix {
            println!("{}", String::from_iter(r));
        }
    }

    #[test]
    fn test_save() {
        let walls = vec![
            (0, 5),
            (1, 2),
            (2, 7),
            (3, 8),
            (5, 10),
            (8, 13),
            (10, 11),
            (11, 12),
            (11, 16),
            (13, 14),
            (14, 19),
            (15, 20),
            (16, 17),
            (16, 21),
            (17, 18),
            (17, 22),
            (19, 24),
            (20, 21),
        ];
        let m = Maze::create(5, 5, walls).unwrap();

        assert_eq!(
            m.move_pos(Position { x: 0, y: 0 }, &Right),
            Some(Position { x: 1, y: 0 })
        );
        assert_eq!(m.move_pos(Position { x: 0, y: 0 }, &Left), None);
        assert_eq!(m.move_pos(Position { x: 0, y: 0 }, &Up), None);
        assert_eq!(m.move_pos(Position { x: 0, y: 0 }, &Down), None);
        // second row
        assert_eq!(m.move_pos(Position { x: 0, y: 1 }, &Left), None);

        assert_eq!(
            m.movements(Position { x: 0, y: 1 }),
            HashSet::from_iter(vec![Right])
        );

        // move to exit
        let mut j = m.joystick();
        let completed = j.moves([Right, Down, Right, Down, Right, Down, Down, Right].iter());
        assert_eq!(completed.len(), 8);
        assert_eq!(j.pos, Position { x: 4, y: 4 });
        assert_eq!(j.is_exit(), true);

        // stuck down bad path
        j = m.joystick();
        j.moves([Right, Down, Right, Right, Right, Down, Down, Down].iter());
        assert_eq!(j.pos, Position { x: 4, y: 2 });
    }
}
