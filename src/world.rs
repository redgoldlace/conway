use std::{
    alloc::Layout,
    fmt::{Display, Write},
    ops::{Index, IndexMut},
};

use crate::cell::{Cell, LocatedCell, Position};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct World {
    width: usize,
    height: usize,
    cells: Box<[Cell]>,
}

impl World {
    /// Constructs a new `World` with the specified width and height. This does not allocate if the world would contain
    /// 0 cells.
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        let cells = if size == 0 {
            Default::default()
        } else {
            let layout = Layout::array::<Cell>(size).unwrap();
            // SAFETY: `size_of::<Cell>()` is greater than 0, so this is not undefined behaviour.
            let ptr = unsafe { std::alloc::alloc(layout) } as *mut Cell;

            for n in 0..size {
                // SAFETY: We allocated sufficient memory for this operation above. We're writing into a buffer that we
                // have control over.
                unsafe {
                    std::ptr::write(ptr.wrapping_add(n), Cell::Dead);
                }
            }

            // SAFETY: We safely initialized this memory above. The buffer is correctly sized and the pointer is not
            // dangling.
            unsafe {
                let slice = std::slice::from_raw_parts_mut(ptr, size);
                Box::from_raw(slice)
            }
        };

        World {
            width,
            height,
            cells,
        }
    }

    pub fn iter(&self) -> <&World as IntoIterator>::IntoIter {
        self.into_iter()
    }

    pub fn get(&self, position: impl WorldIndex) -> Option<Cell> {
        let index = position.to_index(self)?;
        // SAFETY: `to_index` has already checked that this is a valid index.
        unsafe { Some(*self.cells.get_unchecked(index)) }
    }

    pub fn get_mut(&mut self, position: impl WorldIndex) -> Option<&mut Cell> {
        let index = position.to_index(self)?;
        // SAFETY: `to_index` has already checked that this is a valid index.
        unsafe { Some(self.cells.get_unchecked_mut(index)) }
    }

    pub fn has_live_neighbor(&self, (x, y): (usize, usize), position: Position) -> bool {
        let (x_offset, y_offset) = position.offset();
        let (new_x, new_y) = ((x as isize + x_offset), (y as isize + y_offset));

        if new_x < 0 || new_y < 0 {
            false
        } else {
            self.get((new_x as usize, new_y as usize))
                .map_or(false, |cell| cell.alive())
        }
    }

    pub fn live_neighbors(&self, (x, y): (usize, usize)) -> usize {
        Position::all()
            .into_iter()
            .map(|position| self.has_live_neighbor((x, y), position) as usize)
            .sum()
    }

    pub fn tick(self) -> Self {
        let mut new = self.clone();

        for LocatedCell { position, state } in self.iter() {
            let neighbors = self.live_neighbors(position);
            let new_state = match state {
                Cell::Alive if (2..=3).contains(&neighbors) => Cell::Alive,
                Cell::Alive => Cell::Dead,
                Cell::Dead if neighbors == 3 => Cell::Alive,
                Cell::Dead => Cell::Dead,
            };

            // SAFETY: `LocatedCell` guarantees that `position` is a valid position in the world.
            *new.get_mut(position).unwrap() = new_state;
        }

        new
    }

    /// Get a reference to the world's width.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get a reference to the world's height.
    pub fn height(&self) -> usize {
        self.height
    }
}

impl Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (row_index, row) in self.cells.chunks(self.width).enumerate() {
            for cell in row {
                f.write_char(cell.block())?;
            }

            // We don't want to leave a trailing newline.
            if row_index + 1 < self.height {
                f.write_char('\n')?;
            }
        }

        Ok(())
    }
}

/// Represents an indexable type that can be converted to a 1D coordinate
pub trait WorldIndex {
    fn to_index(&self, world: &World) -> Option<usize>;
}

impl WorldIndex for (usize, usize) {
    fn to_index(&self, world: &World) -> Option<usize> {
        let (x, y) = *self;
        (y * world.width + x).to_index(world)
    }
}

impl WorldIndex for usize {
    fn to_index(&self, world: &World) -> Option<usize> {
        let index = *self;
        (index < world.width * world.height).then(|| index)
    }
}

impl<T: WorldIndex> Index<T> for World {
    type Output = Cell;

    fn index(&self, index: T) -> &Self::Output {
        let index = index.to_index(self).expect("index out of bounds");
        // SAFETY: We have checked the index is valid above
        unsafe { self.cells.get_unchecked(index) }
    }
}

impl<T: WorldIndex> IndexMut<T> for World {
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        let index = index.to_index(self).expect("index out of bounds");
        // SAFETY: We have checked the index is valid above
        unsafe { self.cells.get_unchecked_mut(index) }
    }
}

pub struct WorldIterator<'a> {
    world: &'a World,
    index: usize,
}

impl<'a> WorldIterator<'a> {
    fn new(world: &'a World) -> Self {
        Self { world, index: 0 }
    }
}

impl<'a> Iterator for WorldIterator<'a> {
    type Item = LocatedCell;

    fn next(&mut self) -> Option<Self::Item> {
        let position = (self.index % self.world.width, self.index / self.world.width);

        let result = self
            .world
            .get(self.index)
            .map(|state| LocatedCell { position, state })?;

        self.index += 1;

        Some(result)
    }
}

impl<'a> IntoIterator for &'a World {
    type Item = LocatedCell;

    type IntoIter = WorldIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        WorldIterator::new(self)
    }
}
