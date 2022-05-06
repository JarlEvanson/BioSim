use rand::{thread_rng, Rng};

use crate::{gene::Gene, GENOME_LENGTH, PARENT_VARIATION, MUTATION_RATE, neuron::NeuralNet, HEIGHT, WIDTH, steps, STEPS_PER_GEN};


#[derive(Debug)]
pub struct Cell {
    pub x: u32,
    pub y: u32,
    index: usize,
    last_move_dir: DIR,
    genome: Box<[Gene]>,
    neural_net: NeuralNet,
    is_dead: bool,
}

impl Cell {
    pub fn random_new(coords: (u32, u32), index: usize) -> Cell {
        let mut genome = Vec::new();

        for gene in 0 .. GENOME_LENGTH {
            genome.push(Gene::new(thread_rng().gen()));
        }   

        let genome = genome.into_boxed_slice();
        let neural_net = NeuralNet::new(&genome);
        Cell { x: coords.0, y: coords.1, genome: genome, neural_net, is_dead: false, index, last_move_dir: DIR::get_random() }
    }

    pub fn new(coords: (u32, u32), genome: Box<[Gene]>, index: usize) -> Cell {
        let neural_net = NeuralNet::new(&genome);
        Cell { x: coords.0, y: coords.1, genome, neural_net, is_dead: false, index, last_move_dir: DIR::get_random() }
    }

    pub fn sexually_reproduce(cell1: &Cell, cell2: &Cell) -> Box<[Gene]> {
        let genome_spread = unsafe { (GENOME_LENGTH as f32 * PARENT_VARIATION) as i32 };
        let mut genomes_per_parent = {
            if genome_spread != 0 {
                thread_rng().gen_range(-genome_spread..genome_spread)
            } else {
                0 as i32
            }
        } as i32;

        let genomes_per_parent: u32 = (genomes_per_parent + (GENOME_LENGTH as i32 / 2 as i32)) as u32;

        let vec1 = Vec::from(cell1.genome.clone());
        let vec2 = Vec::from(cell2.genome.clone());
       
        let i: u8 = thread_rng().gen_range(0..1);

        let genes1;
        let genes2;

        if i == 0 {
            genes1 = vec1.split_at(genomes_per_parent as usize).0;
            genes2 = vec2.split_at(genomes_per_parent as usize).1;
        } else {
            genes1 = vec1.split_at(genomes_per_parent as usize).1;
            genes2 = vec2.split_at(genomes_per_parent as usize).0;
        }

        let final_genes =  Vec::from([genes1, genes2].concat());

        let mut new_genes = final_genes.into_boxed_slice();

        if thread_rng().gen_range(0.0 as f32 .. 100.0) < unsafe { MUTATION_RATE } {
            for i in 0 .. GENOME_LENGTH {
                let bit = thread_rng().gen_range(0 .. 32 as u32);

                unsafe {
                    *new_genes.as_mut_ptr().add(i as usize) = *new_genes.as_ptr().add(i as usize) ^ (1 << (bit & 31));
                }
            }
        }

        new_genes
    }

    pub fn asexually_reproduce(cell: &Cell, index: usize) -> Cell {

        let mut new_genes = cell.genome.clone();

        for i in 0 .. GENOME_LENGTH {
            if thread_rng().gen_range(0.0 as f32 .. 100.0) < unsafe { MUTATION_RATE } {
                let bit = thread_rng().gen_range(0 .. 32 as u32);
                unsafe {
                    *new_genes.as_mut_ptr().add(i as usize) = *new_genes.as_ptr().add(i as usize) ^ (1 << (bit & 31));
                }
            }
        }

        let neural_net = NeuralNet::new(&new_genes);
        Cell { x: 0, y: 0, genome: new_genes, neural_net, is_dead: false, index, last_move_dir: DIR::get_random() }
    }

    pub fn one_step(&mut self) -> (u32, u32) {
        self.neural_net.feed_forward(&vec![
            (2 * self.x) as f32 / (WIDTH as f32) - 1.0, 
            (2 * self.y) as f32 / (HEIGHT as f32) - 1.0, 
            unsafe { (steps as f32) / (STEPS_PER_GEN as f32) }
         ]);

        let outputs = self.neural_net.get_outputs();

        let offset = DIR::get_random().get_move_offset();

        let x = (outputs[0] + outputs[2] - outputs[3] + offset.0 * outputs[6]).tanh();
        let y = (outputs[1] + outputs[4] - outputs[5] + offset.1 * outputs[6]).tanh();
        
        let mut coords = (self.x, self.y);

        if (thread_rng().gen_range(0..i32::MAX) as f32) / (i32::MAX as f32) < x.abs() {
            if x > 0.0 {
                coords.0 = coords.0 + 1;
            } else {
                coords.0 = coords.0.saturating_sub(1);
            }
        }

        if coords.0 >= WIDTH {
            coords.0 = WIDTH - 1;
        }

        if (thread_rng().gen_range(0..i32::MAX) as f32) / (i32::MAX as f32) < y.abs() {
            if y > 0.0 {
                coords.1 = coords.1 + 1;
            } else {
                coords.1 = coords.1.saturating_sub(1);
            }
        }

        if coords.1 >= HEIGHT {
            coords.1 = HEIGHT - 1;
        }

        coords
    }

    pub fn get_coords(&self) -> (u32, u32) {
        (self.x, self.y)
    }

    pub fn mark_dead(&mut self) {
        self.is_dead = true;
    }

    pub fn set_coords(&mut self, coords: (u32, u32)) {
        self.x = coords.0;
        self.y = coords.1;
    }

    pub fn is_dead(&self) -> bool {
        self.is_dead
    }

    pub fn get_index(&self) -> usize {
        self.index
    }
}

#[derive(Debug)]
enum DIR {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest
}

impl DIR {
    pub fn get_move_offset(&self) -> (f32, f32) {
        match *self {
            DIR::North => (0.0, 1.0),
            DIR::NorthEast => (1.0, 1.0),
            DIR::East => (1.0, 0.0),
            DIR::SouthEast => (1.0, -1.0),
            DIR::South => (0.0, -1.0),
            DIR::SouthWest => (-1.0, -1.0),
            DIR::West => (-1.0, 0.0),
            DIR::NorthWest => (-1.0, 1.0)
        }
    }

    pub fn get_random() -> DIR {
        match rand::thread_rng().gen_range(0..8) {
            0 => DIR::North,
            1 => DIR::NorthEast,
            2 => DIR::East,
            3 => DIR::SouthEast,
            4 => DIR::South,
            5 => DIR::SouthWest,
            6 => DIR::West,
            7 => DIR::NorthWest,
            _ => unreachable!()
        }
    }
}