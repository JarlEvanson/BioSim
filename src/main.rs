#![allow(non_snake_case, non_upper_case_globals, unused)]

extern crate rand;

mod windowed;
use std::thread::{sleep};

use rand::Rng;
use windowed::window::Window;

mod grid;
use grid::Grid;

mod population;
use population::Population;

mod cell;
use cell::Cell;

mod gene;
use gene::Gene;

use crate::{gene::NodeID, windowed::window::wait};
mod neuron;

const WIDTH: u32 = 100;
const HEIGHT: u32 = 100;
const GENOME_LENGTH: u32 = 6;
const PARENT_VARIATION: f32 = 0.20;
const MUTATION_RATE: f32 = 0.01;
const POPULATION_SIZE: u32 = 1000;
const STEPS_PER_GEN: u32 = 250;

static mut steps: u32 = 0;
static mut should_reset: bool = false;
static mut pause: bool = false;

static mut rand_mutations: u32 = 0;

static mut grid_ptr: *const Grid = 0 as *const Grid;

fn main() {
    const windowing: bool = true;
    if windowing {
        let window = Window::createWindow(512, 512).expect("Window failed to be created");
        window.make_current();

        let mut grid = Grid::new(WIDTH, HEIGHT);

        unsafe { grid_ptr = &grid; }

        let mut current_population = Population::new(POPULATION_SIZE);

        current_population.assign_random(&mut grid);

        let mut gen: u32 = 0;
        
        window.render(current_population.get_living_cells());

        let mut accounted_time = 0.0;

        let mut generation = 0;


        while !window.shouldClose() {
            window.poll();
            if unsafe { should_reset } {
                unsafe { steps = 0; }
                generation = 0;

                current_population.gen_random();

                current_population.assign_random(&mut grid);

                println!("Generation 0:");
                
                unsafe { should_reset = false; }
            }

            if unsafe { steps == STEPS_PER_GEN } {
                unsafe {
                    steps = 0;
                    generation += 1;

                    grid.reset();
                    current_population.assign_random(&mut grid);

                    window.render(current_population.get_living_cells());

                    println!("Generation {}:", generation);

                    //wait(&window, 3.0);

                    //accounted_time += 3.0;
                }
            }

            if unsafe { glfw::ffi::glfwGetTime() - accounted_time > 0.016 && !pause } {
                //accounted_time += 0.016;

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

            if unsafe { steps == STEPS_PER_GEN } {
                let reproducers = determine_reproducers(&current_population);
                if reproducers.len() == 0 {
                    println!("Failed to produce viable offspring");
                    loop {
                        window.poll();
                        if window.shouldClose() || unsafe { should_reset } {
                            accounted_time = unsafe { glfw::ffi::glfwGetTime() };
                            break;
                        }
                    }
                    continue;
                }

                println!("Killed: {}\tReproducing: {}\tLiving Non-reproducing: {} MoveRandom: {}", 
                    POPULATION_SIZE - current_population.get_living_indices().len() as u32, 
                    reproducers.len(),
                    current_population.get_living_indices().len() - reproducers.len(),
                    unsafe { rand_mutations }
                );

                unsafe { rand_mutations = 0; }

                current_population = {
                    let mut reproducing_cells = Vec::new();
                    for index in reproducers {
                        reproducing_cells.push(&*current_population.get_cell(index));
                    }

                    Population::new_asexually(POPULATION_SIZE, &reproducing_cells)
                };

                //wait(&window, 3.0);

                accounted_time += 3.0;
            }
        } 
    }
}

pub fn determine_reproducers(pop: &Population) -> Vec<usize> {
    let mut reproducers = Vec::new();
    for cell in pop.get_living_cells() {
        if cell.get_coords().0 < WIDTH / 4 || cell.get_coords().0 > 3 * WIDTH / 4 {
            reproducers.push(cell.get_index());
        } 
    }

    reproducers
}

pub fn determine_deaths(pop: &mut Population) {
    if unsafe { steps } == STEPS_PER_GEN / 2 {
        for index in &pop.get_living_indices() {
            let (x, y) = pop.get_cell(*index).get_coords();

            if x < WIDTH / 4 || x > 3 * WIDTH / 4 {
                pop.add_to_death_queue(*index as u32)
            }
        }
    }
}