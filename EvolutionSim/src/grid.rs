use rand::Rng;

pub type GridValueT = usize;

pub struct Grid {
    width: GridValueT,
    height: GridValueT,
    grid: Box<[Option<usize>]>,
}

impl Grid {
    pub fn new(width: GridValueT, height: GridValueT) -> Grid {
        let grid = vec![None; width * height];
        Grid {
            width,
            height,
            grid: grid.into_boxed_slice(),
        }
    }

    pub fn get_occupant(&self, x: GridValueT, y: GridValueT) -> Option<usize> {
        self.grid[x + y * self.width]
    }

    pub fn set_occupant(&mut self, x: GridValueT, y: GridValueT, cell: Option<usize>) {
        self.grid[x + y * self.width] = cell;
    }

    pub fn get_in_radius(&self, coords: (GridValueT, GridValueT), radius: f32) -> Vec<usize> {
        println!("Center: ({}, {})", coords.0, coords.1);
        let top = (coords.1 as f32 + radius) as GridValueT;
        let bottom = (coords.1 as f32 - radius) as GridValueT;

        let mut in_radius = Vec::new();

        let mut cy = bottom;
        while cy <= top {
            let dy = cy.saturating_sub(coords.1);
            let dx = (radius * radius - (dy * dy) as f32).sqrt();

            let left = (coords.0 as f32 - dx).ceil() as GridValueT;
            let right = (coords.0 as f32 + dx).ceil() as GridValueT;

            let mut cx = left;
            while cx <= right {
                let occupant = self.grid[cx + cy * self.width];
                if let Some(cell_index) = occupant {
                    in_radius.push(cell_index);
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

    pub fn find_random_unoccupied(&self) -> (usize, usize) {
        let mut x;
        let mut y;

        let mut rng = rand::thread_rng();

        loop {
            x = rng.gen_range(0..self.width);
            y = rng.gen_range(0..self.height);

            if self.get_occupant(x, y).is_none() {
                break;
            }
        }

        (x, y)
    }

    pub fn get_dimensions(&self) -> (GridValueT, GridValueT) {
        (self.width, self.height)
    }
}

impl std::fmt::Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Grid")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}
