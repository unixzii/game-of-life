use std::cmp::PartialEq;

/// A storage with two buffers for arbitary type.
/// 
/// This is useful for generating next generation world while referring to the
/// current generation world, and make this process done without allocating
/// new memory.
struct DoubleBuffer<T> {
    back: Option<T>,
    front: Option<T>,
}

impl<T> DoubleBuffer<T> {
    /// Create a new instance with the given buffer factory.
    fn new<F>(mut f: F) -> DoubleBuffer<T>
        where F: FnMut() -> T {
        return DoubleBuffer {
            back: Some(f()),
            front: Some(f()),
        };
    }

    /// Swap the buffer.
    fn swap(&mut self) {
        let tmp = self.back.take().unwrap();
        self.back = self.front.take();
        self.front = Some(tmp);
    }

    /// Returns the mutable reference to the back buffer.
    fn back_mut(&mut self) -> &mut T {
        return self.back.as_mut().unwrap();
    }

    /// Returns the mutable reference to the front buffer.
    fn front_mut(&mut self) -> &mut T {
        return self.front.as_mut().unwrap();
    }

    /// Returns the reference to the front buffer.
    fn front_ref(&self) -> &T {
        return self.front.as_ref().unwrap();
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum Cell {
    Alive,
    Dead,
}

/// The Game of Life world.
pub struct World {
    stride: i32,
    cells: DoubleBuffer<Vec<Cell>>,
}

impl World {
    /// Create a new world with given size.
    pub fn new(width: i32, height: i32) -> World {
        let cells: DoubleBuffer<Vec<Cell>> = DoubleBuffer::new(|| {
            let length = (width * height) as usize; 
            let mut vec = Vec::with_capacity(length);
            for _ in 0..length {
                vec.push(Cell::Dead);
            }
            return vec;
        });

        return World {
            stride: width,
            cells: cells,
        };
    }

    pub fn width(&self) -> i32 {
        return self.stride;
    }

    pub fn height(&self) -> i32 {
        return (self.cells.front_ref().len() / (self.stride as usize)) as i32;
    }

    /// Modifies the cell at the given point.
    pub fn set_cell(&mut self, x: i32, y: i32, cell: Cell) {
        let index = self.index(x, y);
        self.cells.front_mut()[index] = cell;
    }

    /// Returns a copy of the cell at the given point. 
    pub fn cell_at(&self, x: i32, y: i32) -> Cell {
        let index = self.index(x, y);
        return self.cells.front_ref()[index];
    }

    /// Returns the count of neighbours of the cell at the given point.
    pub fn neighbour_count(&self, x: i32, y: i32) -> i32 {
        let dir = [-1, 0, 1];
        let mut count = 0;
        for x_dir in dir.iter() {
            for y_dir in dir.iter() {
                let dest_x = x + x_dir;
                let dest_y = y + y_dir;
                if dest_x < 0 || dest_y < 0 || (dest_x == x && dest_y == y) {
                    continue;
                }

                let dest_index = self.index(dest_x, dest_y);
                if dest_index >= self.cells.front_ref().len() {
                    continue;
                }
                if self.cells.front_ref()[dest_index] == Cell::Alive {
                    count += 1;
                }
            }
        }
        return count;
    }

    /// Generate the next generation.
    pub fn next_gen(&mut self) {
        for col in 0..(self.height()) {
            for row in 0..(self.width()) {
                let index = self.index(row, col);
                let neighbour_count = self.neighbour_count(row, col);

                // The rules of "Conway's Game of Life":
                // https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life
                let next_gen_cell =
                if self.cells.front_ref()[index] == Cell::Alive {
                    match neighbour_count {
                        0..2  => Cell::Dead,
                        2..=3 => Cell::Alive,
                        _     => Cell::Dead,
                    }
                } else {
                    if neighbour_count == 3 {
                        Cell::Alive
                    } else {
                        Cell::Dead
                    }
                };

                // We fill the next generation cells to the back store.
                let back = self.cells.back_mut();
                back[index] = next_gen_cell;
            }
        }

        // Swap the buffer, make next generation cells current.
        self.cells.swap();
    }

    /// Returns the index of cell vector for the cell at the given point.
    fn index(&self, x: i32, y: i32) -> usize {
        return (self.stride * y + x) as usize;
    }
}
