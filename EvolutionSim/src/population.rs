use rand::Rng;

use crate::{
    cell::{self, MovementData, NeuronData, OtherData, DIR},
    grid::{Grid, GridValueT},
    Config,
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
