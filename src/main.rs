#![allow(non_snake_case, non_upper_case_globals, unused)]

extern crate rand;

mod windowed;
use std::{thread::{sleep}, ops::{Deref, DerefMut}, fmt::Write, io::Read};

use rand::Rng;
use windowed::window::Window;

mod grid;
use grid::Grid;

mod population;
use population::Population;

mod cell;
use cell::Cell;

mod gene;
use gene::{Gene, UNIQUE_INNER_NODES, UNIQUE_INPUT_NODES, UNIQUE_OUTPUT_NODES};

use crate::{gene::NodeID, windowed::window::wait};
mod neuron;

const GRID_WIDTH: u32 = 200;
const GRID_HEIGHT: u32 = 200;

const MUTATION_RATE: f32 = 0.01;
static mut steps_per_gen: u32 = 250;


static mut steps: u32 = 0;
static mut should_reset: bool = false;
static mut pause: bool = false;

const NEURON_COUNT: usize = (UNIQUE_INNER_NODES + UNIQUE_INPUT_NODES + UNIQUE_OUTPUT_NODES) as usize;
static mut neuron_presence: [u32; NEURON_COUNT] = [0; NEURON_COUNT];

static mut framebuffer_width: u32 = 0;
static mut framebuffer_height: u32 = 0;
static mut grid_display_width: u32 = 0;

static mut grid_ptr: *mut Grid = 0 as *mut Grid;
static mut pop_ptr: *mut Population = 0 as *mut Population;

static mut accounted_time: f64 = 0.0;
static mut generation: u32 = 0;

static mut population_size: u32 = 4000;
static mut genome_length: u32 = 10;

fn main() {
    const windowing: bool = true;
    if windowing {
        let window = Window::createWindow(512, 512).expect("Window failed to be created");
        unsafe {
            framebuffer_width = 512;
            framebuffer_height = 512;
            grid_display_width = 512;
        }
        window.make_current();

        let mut grid = Grid::new(GRID_WIDTH, GRID_HEIGHT);

        unsafe { grid_ptr = &mut grid; }

        let mut current_population = Population::new(unsafe { population_size });

        unsafe { pop_ptr = &mut current_population; }

        for argument in std::env::args() {
            if argument.find("file=") != None {
                println!("{}", &argument[5..]);
                load_from_file(&argument[5..]);
            } 
        }

        current_population.assign_random(&mut grid);

        let mut gen: u32 = 0;
        
        window.render(current_population.get_living_cells());

        unsafe { accounted_time = glfw::ffi::glfwGetTime(); }

        while !window.shouldClose() {
            window.poll();
            if unsafe { should_reset } {
                unsafe { accounted_time = glfw::ffi::glfwGetTime(); }
                unsafe { steps = 0; }
                unsafe { generation = 0; }

                current_population.gen_random();

                unsafe { should_reset = false; }
            }

            if unsafe { steps == 0 } {
                grid.reset();
                current_population.assign_random(&mut grid);

                window.render(current_population.get_living_cells());

                println!("Generation {}:", unsafe { generation });
            }

            if unsafe { glfw::ffi::glfwGetTime() - accounted_time > 0.016 && !pause } {
                unsafe { accounted_time += 0.016; }

                unsafe { steps += 1; };

                let living = current_population.get_living_indices();

                for index in &living {
                    let (x, y) = current_population.get_mut_cell(*index).one_step();
                    current_population.add_to_move_queue(*index, x, y);
                }

                determine_deaths(&mut current_population);

                current_population.resolve_dead(&mut grid);
                current_population.resolve_movements(&mut grid);
                
                window.render(current_population.get_living_cells());
            }

            if unsafe { steps == steps_per_gen } {

                let reproducers = determine_reproducers(&current_population);
                if reproducers.len() == 0 {
                    println!("Failed to produce viable offspring");
                    loop {
                        window.poll();
                        if window.shouldClose() || unsafe { should_reset } {
                            unsafe { accounted_time =  glfw::ffi::glfwGetTime() };
                            break;
                        }
                    }
                    continue;
                }

                println!("Dead: {:3}\tReproducing: {:3}\tLiving Non-reproducing: {:3}", 
                    unsafe { population_size } - current_population.get_living_indices().len() as u32, 
                    reproducers.len(),
                    current_population.get_living_indices().len() - reproducers.len(),
                );

                wait(&window, 2.0);

                current_population = {
                    let mut reproducing_cells = Vec::new();
                    for index in reproducers {
                        reproducing_cells.push(&*current_population.get_cell(index));
                    }

                    Population::new_asexually(unsafe { population_size }, &reproducing_cells)
                };
                
                unsafe { steps = 0 };
                unsafe { generation += 1 };
            }
        } 
    }
}

pub fn determine_reproducers(pop: &Population) -> Vec<usize> {
    let mut reproducers = Vec::new();
    for cell in pop.get_living_cells() {
        if cell.get_coords().0 < GRID_WIDTH / 4 || cell.get_coords().0 > 3 * GRID_WIDTH / 4 {
            reproducers.push(cell.get_index());
        } 
    }

    reproducers
}

pub fn determine_deaths(pop: &mut Population) {
    if unsafe { steps == steps_per_gen / 2 } {
        for index in &pop.get_living_indices() {
            let (x, y) = pop.get_cell(*index).get_coords();

            if x < GRID_WIDTH / 4 || x > 3 * GRID_WIDTH / 4 {
                pop.add_to_death_queue(*index as u32)
            }
        }
    } /* else if unsafe { steps } == steps_per_gen / 2 {
        for index in &pop.get_living_indices() {
            let (x, y) = pop.get_cell(*index).get_coords();

            if x > GRID_WIDTH / 4 && x < 3 * GRID_WIDTH / 4 {
                pop.add_to_death_queue(*index as u32)
            }
        }
    } else if unsafe { steps } == 3 * steps_per_gen / 4 {
        for index in &pop.get_living_indices() {
            let (x, y) = pop.get_cell(*index).get_coords();

            if x < GRID_WIDTH / 4 || x > 3 * GRID_WIDTH / 4 {
                pop.add_to_death_queue(*index as u32)
            }
        }
    }
    if pop.get_death_queue_len() > 0 {
        println!("Killed: {}", pop.get_death_queue_len());
    } */
}

pub fn save_to_file(path: &str) {
    let mut file = std::fs::File::create(path).expect("Failed to create file");

    let mut string = String::new();

    write!(string, "{}\n{}\n{}\n", unsafe { population_size}, unsafe { genome_length }, unsafe { generation });


    for cell in (unsafe { (*pop_ptr).get_all_cells() }).deref() {
        for gene in cell.get_genome().deref() {
            write!(string, "{:08x} ", gene.gene);
        }
        write!(string, "\n");
    }

    std::io::Write::write(&mut file, string.as_bytes());
}

pub fn load_from_file(path: &str) {
    let mut file = std::fs::File::open(path).expect("Failed to open file");

    let string = unsafe {
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer);

        String::from_utf8_unchecked(buffer)
    };

    let mut lines = string.lines();

    let size =  lines.next().unwrap().parse::<u32>().unwrap();
    let length = lines.next().unwrap().parse::<u32>().unwrap();
    let gen = lines.next().unwrap().parse::<u32>().unwrap();

    unsafe { 
        population_size = size;
        generation = gen; 
        genome_length = length;
    }

    let mut cells = Vec::new();

    let mut genome = vec![Gene { gene: 0}; length as usize].into_boxed_slice();

    for (index, line) in lines.enumerate() {
        
        for (genome_index, gene) in line.split_ascii_whitespace().enumerate() {
            genome[genome_index] = Gene { gene: u32::from_str_radix(gene, 16).unwrap() };
            
        }

        cells.push(Cell::new(genome.clone(), index));
    }

    unsafe { 
        (*pop_ptr) = Population::new_with_cells(size, cells.into_boxed_slice());
    }
}