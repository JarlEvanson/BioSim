use rand::Rng;

use crate::{
    cell::{self, Cell, MovementData, NeuronData, OtherData, DIR},
    gene::NodeID_COUNT,
    grid::{Grid, GridValueT},
    neuron_presence, Config,
};

pub struct Population {
    size: usize,
    movementData: Box<[MovementData]>,
    neuronData: Box<[NeuronData]>,
    otherData: Box<[OtherData]>,
    deathQueue: Box<[usize]>,
    deathSize: usize,
    moveQueue: Box<[(usize, (GridValueT, GridValueT))]>,
}

impl Population {
    pub fn new(config: &Config) -> Population {
        let mut movementData = std::boxed::Box::new_uninit_slice(config.getPopSize());
        let mut neuronData = std::boxed::Box::new_uninit_slice(config.getPopSize());
        let mut otherData = std::boxed::Box::new_uninit_slice(config.getPopSize());
        for index in 0..config.getPopSize() {
            let v = cell::newRandom(config.getGenomeSize(), config.getStepsPerGen());

            movementData[index].write(v.0);
            neuronData[index].write(v.1);
            otherData[index].write(v.2);
        }

        let (movementData, neuronData, otherData) = unsafe {
            (
                movementData.assume_init(),
                neuronData.assume_init(),
                otherData.assume_init(),
            )
        };

        Population {
            size: config.getPopSize(),
            movementData,
            neuronData,
            otherData,
            deathQueue: unsafe {
                std::boxed::Box::new_zeroed_slice(config.getPopSize()).assume_init()
            },
            deathSize: 0,
            moveQueue: unsafe {
                std::boxed::Box::new_zeroed_slice(config.getPopSize()).assume_init()
            },
        }
    }

    pub fn genRandom(&mut self, config: &Config) {
        for index in 0..config.getPopSize() {
            let v = cell::newRandom(config.getGenomeSize(), config.getStepsPerGen());

            self.movementData[index] = v.0;
            self.neuronData[index] = v.1;
            self.otherData[index] = v.2;
        }
    }

    pub fn reproduceAsexually(&mut self, config: &Config, reproducingCells: Vec<usize>) {
        let oldOther = std::mem::take(&mut self.otherData);
        let mut oldNeurons = Vec::with_capacity(self.size);
        for index in 0..self.size {
            oldNeurons.push(self.neuronData[index].getOscillatorPeriod());
        }

        let genomeLength = config.getGenomeSize();
        let mutationRate = config.getMutationRate();
        let stepsPerGen = config.getStepsPerGen();

        if reproducingCells.len() > 1 {
            let mut otherData = std::boxed::Box::new_uninit_slice(config.getPopSize());
            for index in 0..config.getPopSize() {
                let selectedCell =
                    reproducingCells[rand::thread_rng().gen_range(0..reproducingCells.len())];
                let cellData = (&oldOther[selectedCell], oldNeurons[selectedCell]);
                let newData =
                    cell::asexuallyReproduce(cellData, genomeLength, stepsPerGen, mutationRate);

                self.movementData[index] = newData.0;
                self.neuronData[index] = newData.1;
                otherData[index].write(newData.2);
            }

            unsafe {
                self.otherData = otherData.assume_init();
            }
        }
    }

    pub fn resolveDead(&mut self, grid: &mut Grid) {
        for i in 0..self.deathSize {
            let cellIndex = self.deathQueue[i];
            grid.set_occupant(
                self.movementData[cellIndex].x,
                self.movementData[cellIndex].y,
                None,
            );
            self.otherData[cellIndex].isAlive = false;
        }
        self.deathSize = 0;
    }

    pub fn addToDeathQueue(&mut self, cell: usize) {
        self.deathQueue[self.deathSize] = cell;
        self.deathSize += 1;
    }

    pub fn getDeathQueueLen(&self) -> usize {
        self.deathSize
    }

    pub fn getMutMoveQueue(&mut self) -> &mut [(usize, (GridValueT, GridValueT))] {
        &mut *self.moveQueue
    }

    //size is the amount of entries to process
    pub fn resolveMoveQueue(&mut self, size: usize, grid: &mut Grid) {
        for index in 0..size {
            let moverIndex = self.moveQueue[index].0;
            if self.otherData[moverIndex].isAlive {
                let moverMovementData = &mut self.movementData[moverIndex];
                let (mut newX, mut newY) = (self.moveQueue[index].1 .0, self.moveQueue[index].1 .1);

                grid.set_occupant(moverMovementData.x, moverMovementData.y, None);

                if grid.get_occupant(newX, newY) == None {
                } else if grid.get_occupant(newX, moverMovementData.y) == None {
                    //Changes X, but not Y pos
                    newY = moverMovementData.y;
                } else if grid.get_occupant(moverMovementData.x, newY) == None {
                    newX = moverMovementData.x;
                } else {
                    newX = moverMovementData.x;
                    newY = moverMovementData.y;
                }

                grid.set_occupant(newX, newY, Some(moverIndex));

                if !(newY == moverMovementData.y && newX == moverMovementData.x) {
                    moverMovementData.lastMoveDir = DIR::get_dir_from_offset((
                        newX as isize - moverMovementData.x as isize,
                        newY as isize - moverMovementData.y as isize,
                    ));
                }

                moverMovementData.setCoords((newX, newY));
            }
        }
    }

    pub fn assignRandom(&mut self, grid: &mut Grid) {
        assert!(self.size <= grid.get_dimensions().0 * grid.get_dimensions().1);

        for index in 0..self.size {
            let coords = grid.find_random_unoccupied();

            self.movementData[index].setCoords(coords);

            grid.set_occupant(coords.0, coords.1, Some(index));
        }
    }

    pub fn getLivingIndices(&self) -> Vec<usize> {
        let mut vec = Vec::new();
        for index in 0..self.size {
            if self.otherData[index].isAlive {
                vec.push(index);
            }
        }
        vec
    }

    pub fn getCellMovementData(&self, index: usize) -> &MovementData {
        &self.movementData[index]
    }

    pub fn getCellOtherData(&self, index: usize) -> &OtherData {
        &self.otherData[index]
    }

    pub fn getCellMutNeuronData(&mut self, index: usize) -> &mut NeuronData {
        &mut self.neuronData[index]
    }
}

impl std::fmt::Debug for Population {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Population")
            .field("size", &self.size)
            .field("deathSize", &self.deathSize)
            .finish()
    }
}

pub struct PopulationOld {
    size: usize,
    cells: Box<[Cell]>,
    death_queue: Vec<usize>,
    move_queue: Vec<(usize, GridValueT, GridValueT)>,
}

impl PopulationOld {
    pub fn new_asexually(config: &Config, reproducing_cells: &Vec<&Cell>) -> PopulationOld {
        let mut cells = Vec::with_capacity(config.getPopSize());

        unsafe {
            for index in 0..NodeID_COUNT {
                neuron_presence[index] = 0;
            }
        }

        let genomeLength = config.getGenomeSize();
        let mutationRate = config.getMutationRate();
        let stepsPerGen = config.getStepsPerGen();

        if reproducing_cells.len() > 1 {
            for index in 0..config.getPopSize() {
                cells.push(Cell::asexually_reproduce(
                    reproducing_cells[rand::thread_rng().gen_range(0..reproducing_cells.len())],
                    index,
                    genomeLength,
                    mutationRate,
                    stepsPerGen,
                ));
            }
        } else {
            for index in 0..config.getPopSize() {
                cells.push(Cell::asexually_reproduce(
                    reproducing_cells[0],
                    index,
                    genomeLength,
                    mutationRate,
                    stepsPerGen,
                ));
            }
        }

        PopulationOld {
            size: config.getPopSize(),
            cells: cells.into_boxed_slice(),
            death_queue: Vec::new(),
            move_queue: Vec::new(),
        }
    }

    pub fn new_sexually(config: &Config, reproducing_cells: &Vec<&Cell>) -> PopulationOld {
        let mut cells = Vec::with_capacity(config.getPopSize());

        unsafe {
            for index in 0..NodeID_COUNT {
                neuron_presence[index] = 0;
            }
        }

        let genomeLength = config.getGenomeSize();
        let mutationRate = config.getMutationRate();
        let stepsPerGen = config.getStepsPerGen();

        if reproducing_cells.len() > 1 {
            for index in 0..config.getPopSize() {
                cells.push(Cell::asexually_reproduce(
                    reproducing_cells[rand::thread_rng().gen_range(0..reproducing_cells.len())],
                    index,
                    genomeLength,
                    mutationRate,
                    stepsPerGen,
                ));
            }
        } else {
            for index in 0..config.getPopSize() {
                cells.push(Cell::asexually_reproduce(
                    reproducing_cells[0],
                    index,
                    genomeLength,
                    mutationRate,
                    stepsPerGen,
                ));
            }
        }

        PopulationOld {
            size: config.getPopSize(),
            cells: cells.into_boxed_slice(),
            death_queue: Vec::new(),
            move_queue: Vec::new(),
        }
    }

    pub fn new(config: &Config) -> PopulationOld {
        let mut cells = Vec::with_capacity(config.getPopSize());

        unsafe {
            for index in 0..NodeID_COUNT {
                neuron_presence[index] = 0;
            }
        }

        let genomeLength = config.getGenomeSize();
        let stepsPerGen = config.getStepsPerGen();

        for index in 0..config.getPopSize() {
            cells.push(Cell::random_new(index, genomeLength, stepsPerGen));
        }

        PopulationOld {
            size: config.getPopSize(),
            cells: cells.into_boxed_slice(),
            death_queue: Vec::new(),
            move_queue: Vec::new(),
        }
    }

    pub fn new_with_cells(size: usize, cells: Box<[Cell]>) -> PopulationOld {
        PopulationOld {
            size,
            cells,
            death_queue: Vec::new(),
            move_queue: Vec::new(),
        }
    }

    pub fn gen_random(&mut self, config: &Config) {
        unsafe {
            for index in 0..NodeID_COUNT {
                neuron_presence[index] = 0;
            }
        }

        let genomeLength = config.getGenomeSize();
        let stepsPerGen = config.getStepsPerGen();

        for index in 0..self.size {
            self.cells[index as usize] = Cell::random_new(index, genomeLength, stepsPerGen);
        }
    }

    pub fn get_all_cells(&self) -> &Box<[Cell]> {
        &self.cells
    }

    pub fn resolve_dead(&mut self, grid: &mut Grid) {
        for i in self.death_queue.iter_mut() {
            self.cells[*i as usize].mark_dead();
            grid.set_occupant(
                self.cells[*i as usize].get_coords().0,
                self.cells[*i as usize].get_coords().1,
                None,
            );
        }

        self.death_queue.clear();
    }

    pub fn add_to_death_queue(&mut self, cell: usize) {
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
        for cell in (*self.cells).iter() {
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
        for index in 0..self.size {
            let coords = grid.find_random_unoccupied();
            (*self.cells)[index as usize].set_coords(coords);

            grid.set_occupant(coords.0, coords.1, Some(index));
        }
    }

    pub fn add_to_move_queue(&mut self, index: usize, x: GridValueT, y: GridValueT) {
        self.move_queue.push((index, x, y));
    }

    pub fn resolve_movements(&mut self, grid: &mut Grid) {
        for mover in &self.move_queue {
            let cell = &mut self.cells.as_mut()[mover.0];
            if !cell.is_dead() {
                let (x, y) = (mover.1, mover.2);

                if grid.get_occupant(x, y) == None {
                    grid.set_occupant(cell.get_coords().0, cell.get_coords().1, None);
                    grid.set_occupant(x, y, Some(cell.get_index()));

                    cell.set_last_dir(DIR::get_dir_from_offset((
                        x as isize - (cell.x as isize),
                        y as isize - (cell.y as isize),
                    )));

                    cell.set_coords((x, y));
                } else if grid.get_occupant(x, cell.y) == None {
                    grid.set_occupant(cell.get_coords().0, cell.get_coords().1, None);
                    grid.set_occupant(x, cell.y, Some(cell.get_index()));

                    cell.set_last_dir(DIR::get_dir_from_offset((
                        x as isize - (cell.x as isize),
                        0,
                    )));

                    cell.set_coords((x, cell.y));
                } else if grid.get_occupant(cell.x, y) == None {
                    grid.set_occupant(cell.get_coords().0, cell.get_coords().1, None);
                    grid.set_occupant(cell.x, y, Some(cell.get_index()));

                    cell.set_last_dir(DIR::get_dir_from_offset((
                        0,
                        y as isize - (cell.y as isize),
                    )));

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

impl std::fmt::Debug for PopulationOld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PopulationOld")
            .field("size", &self.size)
            .finish()
    }
}
