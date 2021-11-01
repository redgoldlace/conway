#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Dead,
    Alive,
}

impl Cell {
    pub fn alive(&self) -> bool {
        matches!(self, Cell::Alive)
    }

    pub fn flipped(&self) -> Self {
        match self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        }
    }

    pub fn flip(&mut self) {
        *self = self.flipped();
    }

    pub fn block(&self) -> char {
        match self {
            Cell::Dead => '.',
            Cell::Alive => '@',
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Dead
    }
}

pub struct LocatedCell {
    pub(crate) position: (usize, usize),
    pub(crate) state: Cell,
}

#[derive(Debug, Clone, Copy)]
pub enum Position {
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl Position {
    pub fn offset(&self) -> (isize, isize) {
        match self {
            Position::TopLeft => (-1, -1),
            Position::Top => (0, -1),
            Position::TopRight => (1, -1),
            Position::Left => (-1, 0),
            Position::Right => (1, 0),
            Position::BottomLeft => (-1, 1),
            Position::Bottom => (0, 1),
            Position::BottomRight => (1, 1),
        }
    }

    pub fn all() -> [Position; 8] {
        [
            Position::TopLeft,
            Position::Top,
            Position::TopRight,
            Position::Left,
            Position::Right,
            Position::BottomLeft,
            Position::Bottom,
            Position::BottomRight,
        ]
    }
}
