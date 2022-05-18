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

    pub fn get_in_radius(&self, coords: (u32, u32), radius: f32) -> Vec<u32> {
        println!("Center: ({}, {})", coords.0, coords.1);
        let top = (coords.1 as f32 + radius) as u32;
        let bottom = (coords.1 as f32 - radius) as u32;

        let mut in_radius = Vec::new();

        let mut cy = bottom;
        while cy <= top {
            let dy = cy.saturating_sub(coords.1);
            let dx = (radius * radius - (dy * dy) as f32).sqrt();

            let left = (coords.0 as f32 - dx).ceil() as u32;
            let right = (coords.0 as f32 + dx).ceil() as u32;

            let mut cx = left;
            while cx <= right {
                let occupant = self.grid[(cx + cy * self.width) as usize];
                if occupant != None {
                    in_radius.push(unsafe { occupant.unwrap_unchecked() });
                }
                cx += 1;
            }


            cy += 1;
        }

        in_radius
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