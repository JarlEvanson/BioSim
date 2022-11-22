use custom_dst::{DstArray, DstData, MaybeUninitDstArray};
use rand::{thread_rng, Rng};

use crate::{
    cell::{
        self, gen_random_other, write_random_other_init, Direction, HeritableData, MiscData,
        MovementData, NeuronData,
    },
    gene::Gene,
    grid::{Grid, GridValueT},
    neuron::NeuralNet,
    Config,
};

pub struct Population {
    size: usize,
    movement_data: Box<[MovementData]>,
    neuron_data: Box<[NeuronData]>,
    heritable_data: DstArray<HeritableData, Gene>,
    misc_data: Box<[MiscData]>,
    deathQueue: Box<[usize]>,
    deathSize: usize,
    moveQueue: Box<[(usize, (GridValueT, GridValueT))]>,
}

impl Population {
    pub fn new(config: &Config, grid: &mut Grid) -> Population {
        let mut movementData = std::boxed::Box::new_uninit_slice(config.getPopSize());
        let mut neuronData = std::boxed::Box::new_uninit_slice(config.getPopSize());
        let mut misc_data = Box::new_uninit_slice(config.getPopSize());

        let mut other_data = MaybeUninitDstArray::new(config.getGenomeSize(), config.getPopSize());

        let mut rng = thread_rng();

        for index in 0..config.getPopSize() {
            let movement = {
                let (x, y) = grid.find_random_unoccupied();

                MovementData::new(x, y, Direction::get_random(&mut rng))
            };
            movementData[index].write(movement);

            unsafe {
                write_random_other_init(
                    &mut other_data,
                    index,
                    &mut rng,
                    config.getGenomeSize(),
                    config.getStepsPerGen(),
                );

                //SAFETY Safe because we initialized heritable data above
                let genome = &*other_data.get_footer_ptr(index);

                neuronData[index].write(NeuronData::new(NeuralNet::new(genome)));

                misc_data[index].write(MiscData::new(genome));
            }
        }

        let (movementData, neuronData, otherData, misc_data) = unsafe {
            (
                movementData.assume_init(),
                neuronData.assume_init(),
                other_data.assume_init(),
                misc_data.assume_init(),
            )
        };

        Population {
            size: config.getPopSize(),
            movement_data: movementData,
            neuron_data: neuronData,
            heritable_data: otherData,
            deathQueue: unsafe {
                std::boxed::Box::new_zeroed_slice(config.getPopSize()).assume_init()
            },
            deathSize: 0,
            moveQueue: unsafe {
                std::boxed::Box::new_zeroed_slice(config.getPopSize()).assume_init()
            },
            misc_data: misc_data,
        }
    }

    pub fn genRandom(&mut self, config: &Config, grid: &mut Grid) {
        let mut rng = thread_rng();
        for index in 0..config.getPopSize() {
            let movement = {
                let (x, y) = grid.find_random_unoccupied();

                MovementData {
                    x,
                    y,
                    lastMoveDir: Direction::get_random(&mut rng),
                }
            };
            self.movement_data[index] = movement;

            let other_data = &mut self.heritable_data.get_mut_slice(0, self.size)[index];

            gen_random_other(other_data, &mut rng, config.getStepsPerGen());

            self.misc_data[index] = MiscData::new(other_data.get_footer());

            self.neuron_data[index] = NeuronData::new(NeuralNet::new(other_data.get_footer()));
        }
    }

    pub fn reproduceAsexually(
        &mut self,
        scratch: &mut DstArray<HeritableData, Gene>,
        config: &Config,
        reproducingCells: Vec<usize>,
        grid: &mut Grid,
    ) {
        //Prevents alloc in hot loop
        //Old heritable data is now in scratch
        self.heritable_data.swap(scratch);

        let mutationRate = config.getMutationRate();
        let stepsPerGen = config.getStepsPerGen();

        let mut rng = rand::thread_rng();

        let mut new_heritable_data = self.heritable_data.get_mut_slice(0, config.getPopSize());

        if reproducingCells.len() > 1 {
            for index in 0..config.getPopSize() {
                let selectedCell = reproducingCells[rng.gen_range(0..reproducingCells.len())];

                self.movement_data[index] = {
                    let (x, y) = grid.find_random_unoccupied();

                    MovementData {
                        x,
                        y,
                        lastMoveDir: Direction::get_random(&mut rng),
                    }
                };

                cell::asexuallyReproduce(
                    &scratch[selectedCell],
                    &mut new_heritable_data[index],
                    stepsPerGen,
                    mutationRate,
                );

                self.neuron_data[index] =
                    NeuronData::new(NeuralNet::new(new_heritable_data[index].get_footer()));

                self.misc_data[index] = MiscData::new(new_heritable_data[index].get_footer());
            }
        }
    }

    pub fn resolveDead(&mut self, grid: &mut Grid) {
        for i in 0..self.deathSize {
            let cellIndex = self.deathQueue[i];
            grid.set_occupant(
                self.movement_data[cellIndex].x,
                self.movement_data[cellIndex].y,
                None,
            );
            self.misc_data[cellIndex].isAlive = false;
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
            if self.misc_data[moverIndex].isAlive {
                let moverMovementData = &mut self.movement_data[moverIndex];
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
                    moverMovementData.lastMoveDir = Direction::get_dir_from_offset((
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

            self.movement_data[index].setCoords(coords);

            grid.set_occupant(coords.0, coords.1, Some(index));
        }
    }

    pub fn getLivingIndices(&self) -> Vec<usize> {
        let mut vec = Vec::new();
        for index in 0..self.size {
            if self.misc_data[index].isAlive {
                vec.push(index);
            }
        }
        vec
    }

    pub fn getCellMovementData(&self, index: usize) -> &MovementData {
        &self.movement_data[index]
    }

    pub fn getCellHeritableData<'a>(&'a self, index: usize) -> &'a DstData<HeritableData, Gene> {
        self.heritable_data.get_arr_element(index)
    }

    pub fn getCellMiscData(&self, index: usize) -> &MiscData {
        &self.misc_data[index]
    }

    pub fn getCellMutNeuronData(&mut self, index: usize) -> &mut NeuronData {
        &mut self.neuron_data[index]
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
