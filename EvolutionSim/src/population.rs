use std::{ops::Deref, convert::TryInto};

use rand::Rng;

use crate::{cell::{Cell, DIR}, population_size, grid::Grid, gene::NodeID_COUNT, neuron_presence};


pub struct Population {
    size: u32,
    cells: Box<[Cell]>,
    death_queue: Vec<u32>,
    move_queue: Vec<(usize, u32, u32)>
}

impl Population {
    pub fn new_asexually(size: u32, reproducing_cells: &Vec<&Cell>) -> Population {
        let mut cells = Vec::with_capacity(size.try_into().unwrap());

        unsafe {
            for index in 0 .. NodeID_COUNT {
                neuron_presence[index] = 0;
            }
        }

        if reproducing_cells.len() > 1 {
            for index in 0 .. size {
                cells.push( Cell::asexually_reproduce( reproducing_cells[rand::thread_rng().gen_range(0 .. reproducing_cells.len())], index as usize));
            }
        } else {
            for index in 0 .. size {
                cells.push(Cell::asexually_reproduce( reproducing_cells[0], index as usize));
            }
        }

        Population { size, cells: cells.into_boxed_slice(), death_queue: Vec::new(), move_queue: Vec::new() }
    }

    pub fn new_sexually(size: u32, reproducing_cells: &Vec<&Cell>) -> Population {
        let mut cells = Vec::with_capacity(size.try_into().unwrap());

        unsafe {
            for index in 0 .. NodeID_COUNT {
                neuron_presence[index] = 0;
            }
        }

        if reproducing_cells.len() > 1 {
            for index in 0 .. size {
                cells.push( Cell::asexually_reproduce( reproducing_cells[rand::thread_rng().gen_range(0 .. reproducing_cells.len())], index as usize));
            }
        } else {
            for index in 0 .. size {
                cells.push(Cell::asexually_reproduce( reproducing_cells[0], index as usize));
            }
        }

        Population { size, cells: cells.into_boxed_slice(), death_queue: Vec::new(), move_queue: Vec::new() }
    }

    pub fn new(size: u32) -> Population {
        let mut cells = Vec::with_capacity(size.try_into().unwrap());

        unsafe {
            for index in 0 .. NodeID_COUNT {
                neuron_presence[index] = 0;
            }
        }

        for index in 0 .. size {
            cells.push(Cell::random_new(index as usize));
        }

        Population { size, cells: cells.into_boxed_slice(), death_queue: Vec::new(), move_queue: Vec::new() }
    }

    pub fn new_with_cells(size: u32, cells: Box<[Cell]>) -> Population {
        Population { size, cells, death_queue: Vec::new(), move_queue: Vec::new() }
    }

    pub fn gen_random(&mut self) {
        unsafe { 
            for index in 0 .. NodeID_COUNT {
                neuron_presence[index] = 0;
            }
        }
        for index in 0 .. self.size {
            self.cells[index as usize] = Cell::random_new(index as usize);
        }
    }

    pub fn get_all_cells(&self) -> &Box<[Cell]> {
        &self.cells
    }

    pub fn resolve_dead(&mut self, grid: &mut Grid) {
        for i in self.death_queue.iter_mut() {
            self.cells[*i as usize].mark_dead();
            grid.set_occupant(self.cells[*i as usize].get_coords().0, self.cells[*i as usize].get_coords().1, None);
        }

        self.death_queue.clear();
    }

    pub fn add_to_death_queue(&mut self, cell: u32) {
        self.death_queue.push(cell);
    }

    pub fn get_living_indices(&self) -> Vec<usize> {
        let mut living = Vec::new();
        for (index, cell) in (*self.cells).iter().enumerate() {
            if !(*cell).is_dead() {
                living.push(index);
            }
        }

        living
    }

    pub fn get_living_cells(&self) -> Vec<&Cell> {
        let mut living = Vec::new();
        for (index, cell) in (*self.cells).iter().enumerate() {
            if !(*cell).is_dead() {
                living.push(cell);
            }
        }

        living
    }

    pub fn get_mut_cell(&mut self, index: usize) -> &mut Cell {
        &mut self.cells[index]
    }

    pub fn get_cell(&self, index: usize) -> &Cell {
        &self.cells[index]
    }

    pub fn set_cell(&mut self, cell: Cell, index: usize) {
        (*self.cells)[index] = cell;
    }

    pub fn assign_random(&mut self, grid: &mut Grid) {
        for index in 0 .. self.size {
            let coords = grid.find_random_unoccupied();
            (*self.cells)[index as usize].set_coords(coords);

            grid.set_occupant(coords.0, coords.1, Some(index));
        }
    }

    pub fn add_to_move_queue(&mut self, index: usize, x: u32, y: u32) {
        self.move_queue.push((index, x, y));
    }

    pub fn resolve_movements(&mut self, grid: &mut Grid) {
    
        for mover in &self.move_queue {
            let cell = &mut self.cells.as_mut()[mover.0];
            if !cell.is_dead() {
                let (x, y) = (mover.1, mover.2);

                if (grid.get_occupant(x, y) == None) {
                    grid.set_occupant(cell.get_coords().0, cell.get_coords().1, None);
                    grid.set_occupant(x, y, Some(cell.get_index() as u32));

                    cell.set_last_dir(DIR::get_dir_from_offset((x as i32 - (cell.x as i32), y as i32 - (cell.y as i32))));

                    cell.set_coords((x, y));
                } else if (grid.get_occupant(x, cell.y) == None ) {
                    grid.set_occupant(cell.get_coords().0, cell.get_coords().1, None);
                    grid.set_occupant(x, cell.y, Some(cell.get_index() as u32));

                    cell.set_last_dir(DIR::get_dir_from_offset((x as i32 - (cell.x as i32), 0)));

                    cell.set_coords((x, cell.y));
                } else if (grid.get_occupant(cell.x, y) == None ) {
                    
                    grid.set_occupant(cell.get_coords().0, cell.get_coords().1, None);
                    grid.set_occupant(cell.x, y, Some(cell.get_index() as u32));
                    
                    cell.set_last_dir(DIR::get_dir_from_offset((0, y as i32 - (cell.y as i32))));

                    cell.set_coords((cell.x, y));
                }
            }
        }

        self.move_queue.clear();
    }

    pub fn get_death_queue_len(&self) -> usize {
        self.death_queue.len()
    }
}

impl std::fmt::Debug for Population {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Population")
            .field("size", &self.size)
            .finish()
    }
}