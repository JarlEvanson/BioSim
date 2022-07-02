use std::{f32::consts::E, convert::TryInto};

use rand::{thread_rng, Rng};

use crate::{gene::{Gene, NodeID, NodeType, INPUT_NODE_COUNT, INNER_NODE_COUNT}, genome_length, mutation_rate, neuron::NeuralNet, grid_height, grid_width, steps, steps_per_gen};


#[derive(Debug)]
pub struct Cell {
    pub x: u32,
    pub y: u32,
    index: usize,
    last_move_dir: DIR,
    oscillator_period: u32,
    color: (u8, u8, u8),
    genome: Box<[Gene]>,
    neural_net: NeuralNet,
    is_dead: bool,
}

impl Cell {
    pub fn random_new(index: usize) -> Cell {
        let mut genome = Vec::new();

        for gene in 0 .. unsafe { genome_length } {
            genome.push(Gene::new_random());
        }   

        let genome = genome.into_boxed_slice();
        Cell::new(genome, thread_rng().gen::<u32>(), index)
    }

    pub fn new(genome: Box<[Gene]>, oscillator: u32, index: usize) -> Cell {
        let neural_net = NeuralNet::new(&genome);
        let color = Cell::create_color(&genome);
        Cell { x: 0, y: 0, genome, neural_net, is_dead: false, index, last_move_dir: DIR::get_random(), oscillator_period: oscillator & unsafe { steps_per_gen }, color }
    }

    pub fn sexually_reproduce(cell1: &Cell, cell2: &Cell, index: usize) -> Cell {
        let mut new_genes = Vec::with_capacity(unsafe { genome_length }.try_into().unwrap());


        for i in 0 .. unsafe { genome_length } {
            //If true, then first cell contributes
            //If false, then second cell contributes
            if thread_rng().gen_bool(0.5) {
                if thread_rng().gen_range(0.0 as f32 .. 100.0) < unsafe { mutation_rate } {
                    let bit = thread_rng().gen_range(0 .. 32 as u32);
                    unsafe {
                        (*new_genes)[i as usize] = (*cell1.genome)[i as usize] ^ (1 << (bit & 31));
                    }
                }
            } else {
                if thread_rng().gen_range(0.0 as f32 .. 100.0) < unsafe { mutation_rate } {
                    let bit = thread_rng().gen_range(0 .. 32 as u32);
                    unsafe {
                        (*new_genes)[i as usize] = (*cell2.genome)[i as usize] ^ (1 << (bit & 31));
                    }
                }
            }
        }

        let mut oscillator = 0;

        if thread_rng().gen_bool(0.5) {
            oscillator = cell1.oscillator_period;
        } else {
            oscillator = cell2.oscillator_period;
        }

        if thread_rng().gen_range(0.0 as f32 .. 100.0) < unsafe { mutation_rate } {
            let bit = thread_rng().gen_range(0 .. 32 as u32);
            unsafe {
                oscillator = oscillator ^ (1 << (bit & 31));
            }
        }



        let new_genes = new_genes.into_boxed_slice();


        Cell::new(new_genes, oscillator, index)
    }

    pub fn asexually_reproduce(cell: &Cell, index: usize) -> Cell {

        let mut new_genes = cell.genome.clone();

        for i in 0 .. unsafe { genome_length } {
            if thread_rng().gen_range(0.0 as f32 .. 100.0) < unsafe { mutation_rate } {
                let bit = thread_rng().gen_range(0 .. 32 as u32);
                unsafe {
                    *new_genes.as_mut_ptr().add(i as usize) = *new_genes.as_ptr().add(i as usize) ^ (1 << (bit & 31));
                }
            }
        }

        let mut oscillator = cell.oscillator_period;

        if thread_rng().gen_range(0.0 as f32 .. 100.0) < unsafe { mutation_rate } {
            let bit = thread_rng().gen_range(0 .. 32 as u32);
            unsafe {
                oscillator = oscillator ^ (1 << (bit & 31));
            }
        }

        Cell::new(new_genes, oscillator, index)
    }

    pub fn one_step(&mut self) -> (u32, u32) {
        unsafe {
            self.neural_net.feed_forward(&vec![
                (2 * self.x) as f32 / (grid_width as f32) - 1.0, 
                (2 * self.y) as f32 / (grid_height as f32) - 1.0, 
                steps as f32 / (steps_per_gen as f32),
                ((((steps as f32 / (self.oscillator_period as f32)) as i32 % 2) * 2) - 1) as f32
            ]);
        }

        let outputs = self.neural_net.get_outputs();

        let offset = DIR::get_random().get_move_offset();
        
        let mut x = outputs[NodeID::get_index(&NodeID::MoveEast)- INPUT_NODE_COUNT - INNER_NODE_COUNT] -
            outputs[NodeID::get_index(&NodeID::MoveWest)- INPUT_NODE_COUNT - INNER_NODE_COUNT] +
            outputs[NodeID::get_index(&NodeID::MoveRandom)- INPUT_NODE_COUNT - INNER_NODE_COUNT] * offset.0 +
            outputs[NodeID::get_index(&NodeID::MoveForward)- INPUT_NODE_COUNT - INNER_NODE_COUNT] * self.last_move_dir.get_move_offset().0 +
            outputs[NodeID::get_index(&NodeID::MoveReverse)- INPUT_NODE_COUNT - INNER_NODE_COUNT] * self.last_move_dir.rotate180().get_move_offset().0 +
            outputs[NodeID::get_index(&NodeID::MoveLeft)- INPUT_NODE_COUNT - INNER_NODE_COUNT] * self.last_move_dir.rotateCCW90().get_move_offset().0 + 
            outputs[NodeID::get_index(&NodeID::MoveRight)- INPUT_NODE_COUNT - INNER_NODE_COUNT] * self.last_move_dir.rotateCW90().get_move_offset().0; 

        let mut y = outputs[NodeID::get_index(&NodeID::MoveNorth)- INPUT_NODE_COUNT - INNER_NODE_COUNT] -
            outputs[NodeID::get_index(&NodeID::MoveSouth)- INPUT_NODE_COUNT - INNER_NODE_COUNT] + 
            outputs[NodeID::get_index(&NodeID::MoveRandom)- INPUT_NODE_COUNT - INNER_NODE_COUNT] * offset.1 +
            outputs[NodeID::get_index(&NodeID::MoveForward)- INPUT_NODE_COUNT - INNER_NODE_COUNT] * self.last_move_dir.get_move_offset().1 +
            outputs[NodeID::get_index(&NodeID::MoveReverse)- INPUT_NODE_COUNT - INNER_NODE_COUNT] * self.last_move_dir.get_move_offset().1 +
            outputs[NodeID::get_index(&NodeID::MoveLeft)- INPUT_NODE_COUNT - INNER_NODE_COUNT] * self.last_move_dir.rotateCCW90().get_move_offset().1 + 
            outputs[NodeID::get_index(&NodeID::MoveRight)- INPUT_NODE_COUNT - INNER_NODE_COUNT] * self.last_move_dir.rotateCW90().get_move_offset().1; ;

        

        x.tanh();
        y.tanh();

        let mut coords = (self.x, self.y);

        if (thread_rng().gen_range(0..i32::MAX) as f32) / (i32::MAX as f32) < x.abs() {
            if x > 0.0 {
                coords.0 = coords.0 + 1;
            } else {
                coords.0 = coords.0.saturating_sub(1);
            }
        }

        if coords.0 >= unsafe { grid_width } {
            coords.0 = unsafe { grid_width } - 1;
        }

        if (thread_rng().gen_range(0..i32::MAX) as f32) / (i32::MAX as f32) < y.abs() {
            if y > 0.0 {
                coords.1 = coords.1 + 1;
            } else {
                coords.1 = coords.1.saturating_sub(1);
            }
        }

        if coords.1 >= unsafe { grid_height } {
            coords.1 = unsafe { grid_height } - 1;
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

    pub fn set_last_dir(&mut self, direction: DIR) {
        self.last_move_dir = direction;
    } 

    pub fn create_color(genome: &Box<[Gene]>) -> (u8, u8, u8) {
        const maxColorVal: u32 = 0xb0;
        const maxLumaVal: u32 = 0xb0;

        let mut color = unsafe {
            let c: u32 = u32::from((genome.first().unwrap().get_head_type() == NodeType::INPUT)) |
                (u32::from((genome.last().unwrap().get_head_type() == NodeType::INPUT)) << 1) |
                (u32::from((genome.first().unwrap().get_tail_type() == NodeType::INNER)) << 2) |
                (u32::from((genome.last().unwrap().get_tail_type() == NodeType::INNER)) << 3) |
                (((genome.first().unwrap().get_head_node_id().get_index() & 1) as u32) << 4) |
                (((genome.first().unwrap().get_tail_node_id().get_index() & 1) as u32) << 5) |
                (((genome.last().unwrap().get_head_node_id().get_index() & 1) as u32) << 6) |
                (((genome.last().unwrap().get_tail_node_id().get_index() & 1) as u32) << 7);
                
            (c, ((c & 0x1f) << 3), ((c & 7) << 5))
        };

        if (color.0 * 3 + color.1 + color.2 * 4) / 8 > maxLumaVal {
            if color.0 > maxColorVal { color.0 %= maxColorVal };
            if color.1 > maxColorVal { color.1 %= maxColorVal };
            if color.2 > maxColorVal { color.2 %= maxColorVal };
        }

        (color.0 as u8, color.1 as u8, color.2 as u8)
    }

    pub fn get_color(&self) -> (u8, u8, u8) {
        self.color
    }

    pub fn get_genome(&self) -> &Box<[Gene]> {
        &self.genome
    }

    pub fn get_oscillator_period(&self) -> u32{
        self.oscillator_period
    }
}

#[derive(Debug)]
pub enum DIR {
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

    pub fn get_dir_from_offset(offset: (i32, i32)) -> DIR {
        match offset {
            (0, 1) => DIR::North, 
            (1, 1) => DIR::NorthEast, 
            (1, 0) => DIR::East, 
            (1, -1) => DIR::SouthEast, 
            (0, -1) => DIR::South, 
            (-1, -1) => DIR::SouthWest, 
            (-1, 0) => DIR::West, 
            (-1, 1) => DIR::NorthWest, 
            (_, _) => unimplemented!()
        }
    }

    pub fn rotateCCW90(&self) -> DIR {
        match *self {
            DIR::North => DIR::West,
            DIR::NorthEast => DIR::NorthWest,
            DIR::East => DIR::North,
            DIR::SouthEast => DIR::NorthEast,
            DIR::South => DIR::East,
            DIR::SouthWest => DIR::SouthEast,
            DIR::West => DIR::South,
            DIR::NorthWest => DIR::SouthWest
        }
    }

    pub fn rotateCW90(&self) -> DIR {
        match *self {
            DIR::West => DIR::North,
            DIR::NorthWest => DIR::NorthEast,
            DIR::North => DIR::East,
            DIR::NorthEast => DIR::SouthEast,
            DIR::East => DIR::South,
            DIR::SouthEast => DIR::SouthWest,
            DIR::South => DIR::West,
            DIR::SouthWest => DIR::NorthWest
        }
    }

    pub fn rotate180(&self) -> DIR {
        match *self {
            DIR::North => DIR::South,
            DIR::NorthEast => DIR::SouthWest,
            DIR::East => DIR::West,
            DIR::SouthEast => DIR::NorthWest,
            DIR::South => DIR::North,
            DIR::SouthWest => DIR::NorthEast,
            DIR::West => DIR::East,
            DIR::NorthWest => DIR::SouthEast
        }
    }
}