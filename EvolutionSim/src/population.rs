use custom_dst::{DstArray, DstData, DstSliceMut, MaybeUninitDstArray};
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
        let mut movement_data = std::boxed::Box::new_uninit_slice(config.get_pop_size());
        let mut neuron_data = std::boxed::Box::new_uninit_slice(config.get_pop_size());
        let mut misc_data = Box::new_uninit_slice(config.get_pop_size());

        let mut other_data =
            MaybeUninitDstArray::new(config.get_genome_size(), config.get_pop_size());

        let mut rng = thread_rng();

        for index in 0..config.get_pop_size() {
            let movement = {
                let (x, y) = grid.find_random_unoccupied();

                MovementData::new(x, y, Direction::get_random(&mut rng))
            };
            movement_data[index].write(movement);

            unsafe {
                write_random_other_init(
                    &mut other_data,
                    index,
                    &mut rng,
                    config.get_genome_size(),
                    config.get_steps_per_gen(),
                );

                //SAFETY Safe because we initialized heritable data above
                let genome = &*other_data.get_footer_ptr(index);

                neuron_data[index].write(NeuronData::new(NeuralNet::new(genome)));

                misc_data[index].write(MiscData::new(genome));
            }
        }

        let (movement_data, neuron_data, heritable_data, misc_data) = unsafe {
            (
                movement_data.assume_init(),
                neuron_data.assume_init(),
                other_data.assume_init(),
                misc_data.assume_init(),
            )
        };

        Population {
            size: config.get_pop_size(),
            movement_data,
            neuron_data,
            heritable_data,
            deathQueue: unsafe {
                std::boxed::Box::new_zeroed_slice(config.get_pop_size()).assume_init()
            },
            deathSize: 0,
            moveQueue: unsafe {
                std::boxed::Box::new_zeroed_slice(config.get_pop_size()).assume_init()
            },
            misc_data,
        }
    }

    pub fn genRandom(&mut self, config: &Config, grid: &mut Grid) {
        let mut rng = thread_rng();

        let heritable = &mut self.heritable_data.get_mut_slice(0, self.size);

        for index in 0..config.get_pop_size() {
            self.movement_data[index] = {
                let (x, y) = grid.find_random_unoccupied();

                grid.set_occupant(x, y, Some(index));

                MovementData {
                    x,
                    y,
                    lastMoveDir: Direction::get_random(&mut rng),
                }
            };

            gen_random_other(&mut heritable[index], &mut rng, config.get_steps_per_gen());

            self.neuron_data[index] =
                NeuronData::new(NeuralNet::new(heritable[index].get_footer()));

            self.misc_data[index] = MiscData::new(heritable[index].get_footer());
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

        let mutationRate = config.get_mutation_rate();
        let stepsPerGen = config.get_steps_per_gen();

        let mut rng = rand::thread_rng();

        let mut new_heritable_data = self.heritable_data.get_mut_slice(0, config.get_pop_size());

        for index in 0..config.get_pop_size() {
            let selectedCell = reproducingCells[rng.gen_range(0..reproducingCells.len())];

            self.movement_data[index] = {
                let (x, y) = grid.find_random_unoccupied();

                grid.set_occupant(x, y, Some(index));

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
        &mut self.moveQueue
    }

    //size is the amount of entries to process
    pub fn resolveMoveQueue(&mut self, size: usize, grid: &mut Grid) {
        for index in 0..size {
            let moverIndex = self.moveQueue[index].0;
            if self.misc_data[moverIndex].isAlive {
                let moverMovementData = &mut self.movement_data[moverIndex];
                let (mut newX, mut newY) = (self.moveQueue[index].1 .0, self.moveQueue[index].1 .1);

                grid.set_occupant(moverMovementData.x, moverMovementData.y, None);

                if grid.get_occupant(newX, newY).is_none() {
                } else if grid.get_occupant(newX, moverMovementData.y).is_none() {
                    //Changes X, but not Y pos
                    newY = moverMovementData.y;
                } else if grid.get_occupant(moverMovementData.x, newY).is_none() {
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

    pub fn get_data_mut(
        &mut self,
    ) -> (
        &mut [MovementData],
        &mut [NeuronData],
        DstSliceMut<HeritableData, Gene>,
        &mut [MiscData],
        &mut [(usize, (GridValueT, GridValueT))],
    ) {
        (
            &mut self.movement_data,
            &mut self.neuron_data,
            self.heritable_data.get_mut_slice(0, self.size),
            &mut self.misc_data,
            &mut self.moveQueue,
        )
    }

    pub fn get_movement_data(&self) -> &[MovementData] {
        &self.movement_data
    }

    pub fn get_mut_movement_data(&mut self) -> &mut [MovementData] {
        &mut self.movement_data
    }

    pub fn get_misc_data(&self) -> &[MiscData] {
        &self.misc_data
    }

    pub fn get_mut_misc_data(&mut self) -> &mut [MiscData] {
        &mut self.misc_data
    }

    pub fn get_neuron_data(&self) -> &[NeuronData] {
        &self.neuron_data
    }

    pub fn get_mut_neuron_data(&mut self) -> &mut [NeuronData] {
        &mut self.neuron_data
    }

    pub fn get_mut_heritable_data(&mut self) -> DstSliceMut<HeritableData, Gene> {
        self.heritable_data.get_mut_slice(0, self.size)
    }

    pub fn getCellMovementData(&self, index: usize) -> &MovementData {
        &self.movement_data[index]
    }

    pub fn getCellHeritableData(&self, index: usize) -> &DstData<HeritableData, Gene> {
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
