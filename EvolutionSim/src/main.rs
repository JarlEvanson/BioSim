#![allow(non_snake_case, non_upper_case_globals, temporary_cstring_as_ptr)]
#![feature(trace_macros, new_uninit)]
#![feature(test)]

use std::slice::{Chunks, ChunksMut};
use std::{process::exit, rc::Rc};

extern crate ProcEvolutionSim;

extern crate rand;

extern crate scoped_threadpool;
use scoped_threadpool::Pool;

mod windowed;
use windowed::window::Window;

mod grid;
use grid::{Grid, GridValueT};

mod population;
use population::Population;

mod cell;

mod gene;
use gene::NodeID_COUNT;

use crate::windowed::window::wait;
mod neuron;

//Statistics
static mut neuron_presence: [u32; NodeID_COUNT] = [0; NodeID_COUNT];

//Pointers
static mut grid_ptr: *mut Grid = 0 as *mut Grid;
static mut pop_ptr: *mut Population = 0 as *mut Population;

//Display
static mut grid_display_side_length: u32 = 0;
static mut framebuffer_width: u32 = 0;
static mut framebuffer_height: u32 = 0;

//Changes during runtime
static mut accounted_time: f64 = 0.0;
static mut generation: TimeT = 0;
static mut steps: TimeT = 0;
static mut should_reset: bool = false;
static mut pause: bool = false;

mod config;
use config::Config as ConfigBase;

type Config = Rc<ConfigBase>;
type TimeT = usize;

fn main() {
    println!("The argument file=\"path\" will load the save");

    let config: Config = Rc::new(ConfigBase::initFromArgs());

    println!("{}", config);

    unsafe {
        let grid_layout = std::alloc::Layout::new::<Grid>();
        grid_ptr = std::alloc::alloc(grid_layout) as *mut Grid;

        let pop_layout = std::alloc::Layout::new::<Population>();
        pop_ptr = std::alloc::alloc(pop_layout) as *mut Population;

        std::ptr::write(pop_ptr, Population::new(&config));
        std::ptr::write(
            grid_ptr,
            Grid::new(config.getGridWidth(), config.getGridHeight()),
        );
    }

    if config.getIsWindowing() {
        println!("Press R to reset simulation\nPress SPACE to pause and restart simulation\nPress E to print current neuron frequencies\nPress Escape to close window\nPress S to save current generation's genes");

        let window = Window::createWindow(&config, 512, 512).expect("Window failed to be created");
        unsafe {
            framebuffer_width = 512;
            framebuffer_height = 512;
            grid_display_side_length = 512;
        }
        window.make_current();

        unsafe { (*pop_ptr).assignRandom(&mut *grid_ptr) };

        window.render(&config, unsafe { &*pop_ptr });

        unsafe {
            accounted_time = glfw::ffi::glfwGetTime();
        }

        let mut outputted = false;

        let mut threadpool = Pool::new(1); //std::thread::available_parallelism().unwrap().get() as u32);

        while !window.shouldClose() {
            window.poll();
            if unsafe { should_reset } {
                unsafe {
                    accounted_time = glfw::ffi::glfwGetTime();
                    steps = 0;
                    generation = 0;
                    should_reset = false;
                }

                unsafe { &mut *pop_ptr }.genRandom(&config);
            }

            if unsafe { steps == 0 } && !outputted {
                unsafe {
                    (*grid_ptr).reset();
                }
                unsafe { &mut *pop_ptr }.assignRandom(unsafe { &mut *grid_ptr });

                window.render(&config, unsafe { &*pop_ptr });

                println!("Generation {}:", unsafe { generation });

                outputted = true;
            }

            if unsafe { glfw::ffi::glfwGetTime() - accounted_time > 0.016 && !pause } {
                outputted = false;
                unsafe {
                    accounted_time += 0.016;
                    steps += 1;
                };

                let size = computeMovements(&config, &mut threadpool, unsafe { &mut *pop_ptr });
                unsafe { &mut *pop_ptr }.resolveMoveQueue(size, unsafe { &mut *grid_ptr });

                determine_deaths(&config, unsafe { &mut *pop_ptr });
                unsafe { &mut *pop_ptr }.resolveDead(unsafe { &mut *grid_ptr });

                if unsafe { &mut *pop_ptr }.getLivingIndices().len() == 0 {
                    println!("Everyone Died");
                    loop {
                        window.poll();
                        if window.shouldClose() || unsafe { should_reset } {
                            unsafe { accounted_time = glfw::ffi::glfwGetTime() };
                            break;
                        }
                    }
                    continue;
                }

                window.render(&config, unsafe { &*pop_ptr });
            }

            if unsafe { steps } == config.getStepsPerGen() {
                let reproducers = determine_reproducers(&config, unsafe { &mut *pop_ptr });
                if reproducers.len() == 0 {
                    println!("Failed to produce viable offspring");
                    loop {
                        window.poll();
                        if window.shouldClose() || unsafe { should_reset } {
                            unsafe { accounted_time = glfw::ffi::glfwGetTime() };
                            break;
                        }
                    }
                    continue;
                }

                println!(
                    "Dead: {:3}\tReproducing: {:3}\tLiving Non-reproducing: {:3}",
                    config.getPopSize() - unsafe { &*pop_ptr }.getLivingIndices().len(),
                    reproducers.len(),
                    unsafe { &*pop_ptr }.getLivingIndices().len() - reproducers.len(),
                );

                unsafe {
                    (*pop_ptr).reproduceAsexually(&config, reproducers);
                }

                //wait(&window, 2.0);

                unsafe { steps = 0 };
                unsafe { generation += 1 };
            }
        }
    } else {
        unsafe { (*pop_ptr).assignRandom(&mut *grid_ptr) };

        let mut threadpool = Pool::new(std::thread::available_parallelism().unwrap().get() as u32);

        loop {
            if unsafe { should_reset } {
                unsafe {
                    steps = 0;
                    generation = 0;
                    should_reset = false;
                }

                unsafe { &mut *pop_ptr }.genRandom(&config);
            }

            if unsafe { steps == 0 } {
                unsafe {
                    (*grid_ptr).reset();
                }
                unsafe { &mut *pop_ptr }.assignRandom(unsafe { &mut *grid_ptr });

                println!("Generation {}:", unsafe { generation });
            }

            {
                unsafe {
                    steps += 1;
                };

                println!("{:?}", unsafe { &*pop_ptr });

                let size = computeMovements(&config, &mut threadpool, unsafe { &mut *pop_ptr });
                unsafe { &mut *pop_ptr }.resolveMoveQueue(size, unsafe { &mut *grid_ptr });

                determine_deaths(&config, unsafe { &mut *pop_ptr });
                unsafe { &mut *pop_ptr }.resolveDead(unsafe { &mut *grid_ptr });
            }

            if unsafe { steps } == config.getStepsPerGen() {
                if unsafe { generation } % 5 == 0 {}

                let reproducers = determine_reproducers(&config, unsafe { &mut *pop_ptr });
                if reproducers.len() == 0 {
                    println!("Failed to produce viable offspring");
                    exit(1);
                }

                println!(
                    "Dead: {:3}\tReproducing: {:3}\tLiving Non-reproducing: {:3}",
                    config.getPopSize() - unsafe { &*pop_ptr }.getLivingIndices().len(),
                    reproducers.len(),
                    unsafe { &*pop_ptr }.getLivingIndices().len() - reproducers.len(),
                );

                unsafe {
                    (*pop_ptr).reproduceAsexually(&config, reproducers);
                }

                unsafe { steps = 0 };
                unsafe { generation += 1 };
            }
        }
    }
}

pub fn computeMovements(config: &Config, threadpool: &mut Pool, pop: &mut Population) -> usize {
    let living = pop.getLivingIndices();

    let results = pop.getMutMoveQueue();

    let parts = {
        let threads = threadpool.thread_count();
        let (num, rem) = (
            living.len() / ((threads + 1) as usize),
            living.len() % ((threads + 1) as usize),
        );

        if rem != 0 {
            num + 1
        } else {
            num
        }
    };

    let mut chunks: Chunks<usize> = living.as_slice().chunks(parts);

    let mut resChunks: ChunksMut<(usize, (GridValueT, GridValueT))> = results.chunks_mut(parts);

    let gridWidth = config.getGridWidth();
    let gridHeight = config.getGridHeight();
    let stepsPerGen = config.getStepsPerGen();

    threadpool.scoped(|scope| {
        let localChunk = chunks.next().unwrap();
        let localResults = resChunks.next().unwrap();

        for chunk in chunks {
            let resChunk = resChunks.next().unwrap();
            scope.execute(move || {
                for (index, cellIndex) in chunk.into_iter().enumerate() {
                    let movement = unsafe { &mut *pop_ptr }.getCellMovementData(*cellIndex);
                    let neurons = unsafe { &mut *pop_ptr }.getCellMutNeuronData(*cellIndex);

                    let coords =
                        cell::oneStep((neurons, movement), gridWidth, gridHeight, stepsPerGen);
                    resChunk[index] = (*cellIndex, coords);
                }
            });
        }

        for (index, cellIndex) in localChunk.into_iter().enumerate() {
            let movement = unsafe { &mut *pop_ptr }.getCellMovementData(*cellIndex);
            let neurons = unsafe { &mut *pop_ptr }.getCellMutNeuronData(*cellIndex);

            let coords = cell::oneStep((neurons, movement), gridWidth, gridHeight, stepsPerGen);
            localResults[index] = (*cellIndex, coords);
        }
    });

    living.len()
}

pub fn determine_reproducers(config: &Config, pop: &Population) -> Vec<usize> {
    let mut reproducers = Vec::new();
    for cellIndex in pop.getLivingIndices() {
        let cell = pop.getCellMovementData(cellIndex);
        if cell.getCoords().0 < config.getGridWidth() / 4
            || cell.getCoords().0 > 3 * config.getGridWidth() / 4
        {
            reproducers.push(cellIndex);
        }
    }

    reproducers
}

pub fn determine_deaths(config: &Config, pop: &mut Population) {
    if unsafe { steps } == config.getStepsPerGen() / 4 {
        for index in &pop.getLivingIndices() {
            let (x, _) = pop.getCellMovementData(*index).getCoords();

            if x < config.getGridWidth() / 4 || x > (3 * config.getGridWidth()) / 4 {
                pop.addToDeathQueue(*index)
            }
        }
    } else if unsafe { steps } == config.getStepsPerGen() / 2 {
        for index in &pop.getLivingIndices() {
            let (x, _) = pop.getCellMovementData(*index).getCoords();

            if x > config.getGridWidth() / 4 && x < (3 * config.getGridWidth()) / 4 {
                pop.addToDeathQueue(*index)
            }
        }
    } else if unsafe { steps } == (3 * config.getStepsPerGen()) / 4 {
        for index in &pop.getLivingIndices() {
            let (x, _) = pop.getCellMovementData(*index).getCoords();

            if x < config.getGridWidth() / 4 || x > (3 * config.getGridWidth()) / 4 {
                pop.addToDeathQueue(*index)
            }
        }
    }
    /*
    println!(
        "{:?} {}",
        unsafe { &*pop_ptr },
        unsafe { &*pop_ptr }.getLivingIndices().len()
    ); */
    if pop.getDeathQueueLen() > 0 {
        println!(
            "Step {} Killed: {}",
            unsafe { steps },
            pop.getDeathQueueLen()
        );
    }
}
