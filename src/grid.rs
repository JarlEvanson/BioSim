use rand::Rng;

use crate::cell::Cell;

pub struct Grid {
    width: u32,
    height: u32,
    grid: Box<[Option<u32>]>,
} 

impl Grid {
    pub fn new(width: u32, height: u32) -> Grid {
        let grid = vec![None; (width * height) as usize];
        Grid { width, height, grid: grid.into_boxed_slice() }
    }

    pub fn get_occupant(&self, x: u32, y: u32) -> Option<u32> {
        return self.grid[(x + y * self.width) as usize]
    }

    pub fn set_occupant(&mut self, x: u32, y: u32, cell: Option<u32>) {
        self.grid[(x + y * self.width) as usize] = cell;
    }

    pub fn reset(&mut self) {
        self.grid.fill_with(|| None);
    }

    pub fn find_random_unoccupied(&self) -> (u32, u32) {
        let mut x = 0;
        let mut y = 0;

        let mut rng = rand::thread_rng();

        #[allow(while_true)]
        while true {
            x = rng.gen_range(0..self.width);
            y = rng.gen_range(0..self.height);

            if self.get_occupant(x, y) == None {
                break;
            }
        }

        (x, y)
    }
}