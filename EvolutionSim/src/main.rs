#![allow(
    non_snake_case,
    non_upper_case_globals,
    unused,
    temporary_cstring_as_ptr
)]
#![feature(trace_macros)]

use std::{
    fmt::Write,
    fs::File,
    io::Read,
    ops::{Add, Deref, DerefMut},
    path::Path,
    process::exit,
    slice::Chunks,
    sync::mpsc::Sender,
    thread::sleep,
};

extern crate rand;

mod windowed;

use rand::Rng;
use windowed::window::Window;

mod grid;
use grid::Grid;

mod population;
use population::Population;

mod cell;
use cell::Cell;

mod gene;
use gene::{Gene, NodeID, NodeID_COUNT, INNER_NODE_COUNT, INPUT_NODE_COUNT, OUTPUT_NODE_COUNT};

use crate::windowed::window::wait;
mod neuron;

extern crate threadpool;
use threadpool::ThreadPool;

extern crate ProcEvolutionSim;

//Settings
static mut population_size: u32 = 4000;
static mut genome_length: u32 = 10;
static mut grid_width: u32 = 200;
static mut grid_height: u32 = 200;
static mut mutation_rate: f32 = 0.1;
static mut steps_per_gen: u32 = 500;

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
static mut generation: u32 = 0;
static mut steps: u32 = 0;
static mut should_reset: bool = false;
static mut pause: bool = false;

fn main() {
    let mut tally: [u32; NodeID_COUNT] = [0; NodeID_COUNT];
    for i in 0..100000 {
        let id = NodeID::from_index(
            rand::thread_rng().gen_range(0..(INPUT_NODE_COUNT + INNER_NODE_COUNT)),
        );
        tally[id.get_index()] += 1;
    }

    println!("The argument file=\"path\" will load the save");

    unsafe {
        let grid_layout = std::alloc::Layout::new::<Grid>();
        grid_ptr = std::alloc::alloc(grid_layout) as *mut Grid;

        let pop_layout = std::alloc::Layout::new::<Population>();
        pop_ptr = std::alloc::alloc(pop_layout) as *mut Population;

        let pop = Population::new(1);
        std::ptr::write(pop_ptr, pop);

        let grid = Grid::new(1, 1);
        std::ptr::write(grid_ptr, grid);
    }

    const windowing: bool = true;

    if windowing {
        println!("Press R to reset simulation\nPress SPACE to pause and restart simulation\nPress E to print current neuron frequencies\nPress Escape to close window\nPress S to save current generation's genes");

        let window = Window::createWindow(512, 512).expect("Window failed to be created");
        unsafe {
            framebuffer_width = 512;
            framebuffer_height = 512;
            grid_display_side_length = 512;
        }
        window.make_current();

        let mut config_file = None;

        for argument in std::env::args() {
            if (&argument).find("file=") != None {
                config_file = Some(load_from_file(&argument["file=".len()..]).to_owned());
                break;
            } else if (&argument).find("config=") != None {
                config_file = Some(argument["config".len() + 1..].to_owned());
            }
        }

        if config_file == None {
            let mut file = File::open("config.ini").unwrap();
            let mut string = String::new();

            file.read_to_string(&mut string);
            load_config(string.as_str());

            unsafe {
                (*grid_ptr) = Grid::new(grid_width, grid_height);
                (*pop_ptr) = Population::new(population_size);
            };
        }

        unsafe { (*pop_ptr).assign_random(&mut *grid_ptr) };

        let mut gen: u32 = 0;

        window.render(unsafe { &*pop_ptr }.get_living_cells());

        unsafe {
            accounted_time = glfw::ffi::glfwGetTime();
        }

        let mut outputted = false;

        let mut threadpool =
            ScopedThreadPool::new(std::thread::available_parallelism().unwrap().get());

        while !window.shouldClose() {
            window.poll();
            if unsafe { should_reset } {
                unsafe {
                    accounted_time = glfw::ffi::glfwGetTime();
                    steps = 0;
                    generation = 0;
                    should_reset = false;
                }

                unsafe { &mut *pop_ptr }.gen_random();
            }

            if unsafe { steps == 0 } && !outputted {
                unsafe {
                    (*grid_ptr).reset();
                }
                unsafe { &mut *pop_ptr }.assign_random(unsafe { &mut *grid_ptr });

                window.render(unsafe { &*pop_ptr }.get_living_cells());

                println!("Generation {}:", unsafe { generation });

                outputted = true;
            }

            if unsafe { glfw::ffi::glfwGetTime() - accounted_time > 0.016 && !pause } {
                outputted = false;
                unsafe {
                    accounted_time += 0.016;
                    steps += 1;
                };

                let living = unsafe { &*pop_ptr }.get_living_indices();

                let chunks: Chunks<usize> = {
                    let threads = threadpool.getThreadCount();
                    let (num, rem) = (living.len() / threads, living.len() % threads);

                    let x = living.as_slice();
                    x.chunks(if rem != 0 { num + 1 } else { num })
                };

                let (sender, reciever) = std::sync::mpsc::channel();

                for chunk in chunks {
                    #[derive(Debug)]
                    struct Arg {
                        ptr: *const usize,
                        len: usize,
                        sender: Sender<(usize, (u32, u32))>,
                    }
                    let ptr = unsafe {
                        let lay = std::alloc::Layout::new::<Arg>();
                        let ptr = std::alloc::alloc(lay) as *mut Arg;
                        if ptr.is_null() {
                            println!("Failed to allocate");
                            exit(1);
                        }

                        (*ptr).ptr = chunk.as_ptr();
                        (*ptr).len = chunk.len();

                        (*ptr).sender = sender.clone();

                        std::mem::forget(std::mem::replace(&mut (*ptr).sender, sender.clone()));

                        ptr
                    };

                    /*

                    threadpool.addWork(
                        |arg| {

                            /*
                            let arg = unsafe { &*(arg as *const Arg) };

                            for offset in 0..arg.len {
                                let index = unsafe { *arg.ptr.add(offset) };

                                let coords =
                                    unsafe { &mut *pop_ptr }.get_mut_cell(index).one_step();
                            }

                            unsafe {
                                std::alloc::dealloc(
                                    std::mem::transmute(arg),
                                    std::alloc::Layout::array::<usize>(2).unwrap(),
                                )
                            };

                            */
                        },
                        ptr as *const std::ffi::c_void,
                    );
                    */
                }

                std::thread::scope(|s| {
                    let sender2 = sender.clone();

                    let (v1, v2) = living.split_at(living.len() / 2);

                    let thread1 = s.spawn(move || {
                        for index in v1 {
                            let coords = unsafe { &mut *pop_ptr }.get_mut_cell(*index).one_step();
                            sender.send((*index, coords));
                        }
                    });

                    let thread2 = s.spawn(move || {
                        for index in v2 {
                            let coords = unsafe { &mut *pop_ptr }.get_mut_cell(*index).one_step();
                            sender2.send((*index, coords));
                        }
                    });

                    let mut reced = 0;

                    while reced < living.len() {
                        match reciever.recv() {
                            Ok((index, (x, y))) => {
                                unsafe { &mut *pop_ptr }.add_to_move_queue(index, x, y);
                                reced += 1;
                            }
                            Err(e) => {
                                println!("{}", e);
                                break;
                            }
                        }
                    }

                    thread1.join();
                    thread2.join();
                });

                determine_deaths(unsafe { &mut *pop_ptr });

                unsafe { &mut *pop_ptr }.resolve_dead(unsafe { &mut *grid_ptr });
                unsafe { &mut *pop_ptr }.resolve_movements(unsafe { &mut *grid_ptr });

                window.render(unsafe { &*pop_ptr }.get_living_cells());
            }

            if unsafe { steps == steps_per_gen } {
                let reproducers = determine_reproducers(unsafe { &mut *pop_ptr });
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
                    unsafe { population_size }
                        - unsafe { &*pop_ptr }.get_living_indices().len() as u32,
                    reproducers.len(),
                    unsafe { &*pop_ptr }.get_living_indices().len() - reproducers.len(),
                );

                wait(&window, 2.0);

                unsafe {
                    *pop_ptr = {
                        let mut reproducing_cells = Vec::new();
                        for index in reproducers {
                            reproducing_cells.push((&*pop_ptr).get_cell(index));
                        }

                        Population::new_asexually(unsafe { population_size }, &reproducing_cells)
                    };
                }

                unsafe { steps = 0 };
                unsafe { generation += 1 };
            }
        }
    } else {
        unimplemented!();
    }
}

pub fn determine_reproducers(pop: &Population) -> Vec<usize> {
    let mut reproducers = Vec::new();
    for cell in pop.get_living_cells() {
        if cell.get_coords().0 < unsafe { unsafe { grid_width } } / 4
            || cell.get_coords().0 > 3 * unsafe { unsafe { grid_width } } / 4
        {
            reproducers.push(cell.get_index());
        }
    }

    reproducers
}

pub fn determine_deaths(pop: &mut Population) {
    if unsafe { steps == steps_per_gen / 4 } {
        for index in &pop.get_living_indices() {
            let (x, y) = pop.get_cell(*index).get_coords();

            if x < unsafe { grid_width } / 4 || x > 3 * unsafe { grid_width } / 4 {
                pop.add_to_death_queue(*index as u32)
            }
        }
    } else if unsafe { steps == steps_per_gen / 2 } {
        for index in &pop.get_living_indices() {
            let (x, y) = pop.get_cell(*index).get_coords();

            if x > unsafe { grid_width } / 4 && x < 3 * unsafe { grid_width } / 4 {
                pop.add_to_death_queue(*index as u32)
            }
        }
    } else if unsafe { steps == 3 * steps_per_gen / 4 } {
        for index in &pop.get_living_indices() {
            let (x, y) = pop.get_cell(*index).get_coords();

            if x < unsafe { grid_width } / 4 || x > 3 * unsafe { grid_width } / 4 {
                pop.add_to_death_queue(*index as u32)
            }
        }
    }
    if pop.get_death_queue_len() > 0 {
        println!("Killed: {}", pop.get_death_queue_len());
    }
}

pub fn save_to_file(path: &str) {
    let mut file = std::fs::File::create(path).expect("Failed to create file");

    let mut string = String::new();

    unsafe {
        write!(string, "[Config]\nGridWidth={}\nGridHeight={}\nPopulationSize={}\nGenomeLength={}\nStepsPerGen={}\nMutationRate={}\n[Save]\nGeneration={}", grid_width, grid_height, population_size, genome_length, steps_per_gen, mutation_rate, generation);
    }

    for cell in (unsafe { (*pop_ptr).get_all_cells() }).deref() {
        write!(string, "{} ", cell.get_oscillator_period());
        for gene in cell.get_genome().deref() {
            write!(string, "{:08x} ", gene.gene);
        }
        write!(string, "\n");
    }

    std::io::Write::write(&mut file, string.as_bytes());
}

//Returns file for config purposes
pub fn load_from_file(path: &str) -> &str {
    let mut file = std::fs::File::open(path).expect("Failed to open file");

    let string = unsafe {
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer);

        String::from_utf8_unchecked(buffer)
    };

    let (config, save) = {
        let mut splits = string.split("[Save]\n");
        let config = splits.next().unwrap();
        (config, splits.next().unwrap())
    };

    load_config(config);

    let mut lines = save.lines();

    unsafe {
        let gen_line = lines.next().unwrap();
        generation = read_ini_entry("Generation", gen_line).unwrap();
    }

    let mut cells = Vec::new();

    let mut genome = vec![Gene { gene: 0 }; unsafe { genome_length } as usize].into_boxed_slice();

    for (index, line) in lines.enumerate() {
        let mut oscillator = 0;

        for (genome_index, gene) in line.split_ascii_whitespace().enumerate() {
            if genome_index == 0 {
                oscillator = u32::from_str_radix(gene, 10).unwrap();
            } else {
                genome[genome_index - 1] = Gene {
                    gene: u32::from_str_radix(gene, 16).unwrap(),
                };
            }
        }

        cells.push(Cell::new(genome.clone(), oscillator, index));
    }

    unsafe {
        (*pop_ptr) =
            Population::new_with_cells(unsafe { population_size }, cells.into_boxed_slice());
        (*grid_ptr) = Grid::new(grid_width, grid_height);
    }

    path
}

pub fn load_config(string: &str) {
    let mut lines = string.lines();

    if lines.next().unwrap() != "[Config]" {
        panic!("Invalid Config File");
    }

    unsafe {
        grid_width = read_ini_entry("GridWidth", lines.next().unwrap()).unwrap();
        grid_height = read_ini_entry("GridHeight", lines.next().unwrap()).unwrap();
        population_size = read_ini_entry("PopulationSize", lines.next().unwrap()).unwrap();
        genome_length = read_ini_entry("GenomeLength", lines.next().unwrap()).unwrap();
        steps_per_gen = read_ini_entry("StepsPerGen", lines.next().unwrap()).unwrap();
        mutation_rate = read_ini_entry("MutationRate", lines.next().unwrap()).unwrap();
    }
}

pub fn printConfig() {
    unsafe {
        println!("GridWidth={}\nGridHeight={}\nPopulationSize={}\nGenomeLength={}\nStepsPerGen={}\nMutationRate={}", crate::grid_width, crate::grid_height, crate::population_size, crate::genome_length, crate::steps_per_gen, crate::mutation_rate);
    }
}

pub fn read_ini_entry<T: std::str::FromStr>(
    key: &'static str,
    str: &str,
) -> Result<T, <T as std::str::FromStr>::Err> {
    (&str[key.len() + 1..]).parse::<T>()
}

pub fn output_ini_entry<T: std::fmt::Display>(string: &mut String, key: &'static str, obj: T) {
    write!(string, "{}={}", key, obj);
}
