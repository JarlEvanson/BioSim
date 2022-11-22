use custom_dst::{DstData, MaybeUninitDstArray};
use rand::{rngs::ThreadRng, thread_rng, Rng};

use crate::{
    config::MutR,
    gene::{Gene, NodeID},
    grid::GridValueT,
    neuron::NeuralNet,
    steps, TimeT,
};

pub struct MovementData {
    pub x: GridValueT,
    pub y: GridValueT,
    pub lastMoveDir: Direction,
}

impl MovementData {
    pub fn new(x: GridValueT, y: GridValueT, dir: Direction) -> MovementData {
        MovementData {
            x,
            y,
            lastMoveDir: dir,
        }
    }

    pub fn getCoords(&self) -> (GridValueT, GridValueT) {
        (self.x, self.y)
    }

    pub fn setCoords(&mut self, coords: (GridValueT, GridValueT)) {
        self.x = coords.0;
        self.y = coords.1;
    }
}

pub struct NeuronData {
    neural_net: NeuralNet,
}

impl NeuronData {
    pub fn new(neural_net: NeuralNet) -> NeuronData {
        NeuronData { neural_net }
    }
}

pub struct MiscData {
    pub color: (u8, u8, u8),
    pub isAlive: bool,
}

impl MiscData {
    pub fn new(genome: &[Gene]) -> MiscData {
        MiscData {
            color: createColor(genome),
            isAlive: true,
        }
    }
}

#[derive(Clone, Copy)]
pub struct HeritableData {
    oscillatorPeriod: TimeT,
}

impl HeritableData {
    pub fn get_oscillator(&self) -> usize {
        self.oscillatorPeriod
    }
}

impl Default for HeritableData {
    fn default() -> Self {
        Self {
            oscillatorPeriod: Default::default(),
        }
    }
}

fn normalize_oscillator(period: TimeT, steps_per_gen: TimeT) -> TimeT {
    period % steps_per_gen
}

pub fn gen_random_other(
    other_data: &mut DstData<HeritableData, Gene>,
    rng: &mut ThreadRng,
    steps_per_gen: TimeT,
) {
    for gene in other_data.get_mut_footer() {
        *gene = Gene::new_random(rng);
    }

    *other_data.get_header_mut() = HeritableData {
        oscillatorPeriod: normalize_oscillator(rng.gen(), steps_per_gen),
    }
}

pub unsafe fn write_random_other_init(
    array: &mut MaybeUninitDstArray<HeritableData, Gene>,
    arr_index: usize,
    rng: &mut ThreadRng,
    genome_length: usize,
    steps_per_gen: TimeT,
) {
    let mut gene_ptr = array.get_footer_element_ptr_mut(arr_index, 0);

    //SAFETY we know the size of the footer and we know the arr index, so this is safe
    for _ in 0..genome_length {
        unsafe {
            *gene_ptr = Gene::new_random(rng);
            gene_ptr = gene_ptr.add(1);
        }
    }

    drop(gene_ptr);

    array.write_header(
        arr_index,
        HeritableData {
            oscillatorPeriod: normalize_oscillator(rng.gen(), steps_per_gen),
        },
    );
}

#[allow(unused)]
pub fn sexuallyReproduce(
    heritable_data_1: &DstData<HeritableData, Gene>,
    heritable_data_2: &DstData<HeritableData, Gene>,
    cell_loc: &mut DstData<HeritableData, Gene>,
    stepsPerGen: TimeT,
    mutationRate: MutR,
) {
    let mut rng = thread_rng();

    for (index, gene) in cell_loc.get_mut_footer().iter_mut().enumerate() {
        if rng.gen_bool(0.5) {
            if rng.gen_range(0.0 as f32..100.0) < mutationRate {
                let bit = rng.gen_range(0..32 as u32);

                *gene = heritable_data_1.get_footer()[index] ^ (1 << (bit & 31));
            }
        } else {
            if rng.gen_range(0.0 as f32..100.0) < mutationRate {
                let bit = rng.gen_range(0..32 as u32);
                *gene = heritable_data_2.get_footer()[index] ^ (1 << (bit & 31));
            }
        }
    }

    let oscillator = &mut cell_loc.get_header_mut().oscillatorPeriod;

    if thread_rng().gen_bool(0.5) {
        *oscillator = heritable_data_1.get_header().oscillatorPeriod;
    } else {
        *oscillator = heritable_data_2.get_header().oscillatorPeriod;
    }

    if rng.gen_range(0.0 as f32..100.0) < mutationRate {
        let bit = thread_rng().gen_range(0..32 as u32);
        *oscillator = *oscillator ^ (1 << (bit & 31));
    }

    *oscillator = normalize_oscillator(*oscillator, stepsPerGen);
}

pub fn asexuallyReproduce(
    heritable_data: &DstData<HeritableData, Gene>,
    cell_loc: &mut DstData<HeritableData, Gene>,
    stepsPerGen: TimeT,
    mutationRate: MutR,
) {
    let mut rng = thread_rng();

    //Bitwise copy of the cell, is currently valid
    *cell_loc.get_header_mut() = *heritable_data.get_header();
    cell_loc
        .get_mut_footer()
        .copy_from_slice(heritable_data.get_footer());

    for i in cell_loc.get_mut_footer() {
        if rng.gen_range(0.0 as f32..100.0) < mutationRate {
            let bit = rng.gen_range(0..32 as u32);

            *i = Gene::new(i.gene ^ (1 << (bit & 31)));
        }
    }

    let oscillator = &mut cell_loc.get_header_mut().oscillatorPeriod;

    if rng.gen_range(0.0 as f32..100.0) < mutationRate {
        let bit = thread_rng().gen_range(0..32 as u32);
        *oscillator = *oscillator ^ (1 << (bit & 31));
    }

    *oscillator = normalize_oscillator(*oscillator, stepsPerGen);
}

pub fn one_step(
    neuron_data: &mut NeuronData,
    movement_data: &MovementData,
    oscillator: TimeT,
    gridWidth: GridValueT,
    gridHeight: GridValueT,
    stepsPerGen: TimeT,
    rng: &mut ThreadRng,
) -> (usize, usize) {
    let values = [
        (2 * movement_data.x) as f32 / (gridWidth as f32) - 1.0,
        (2 * movement_data.y) as f32 / (gridHeight as f32) - 1.0,
        unsafe { steps } as f32 / (stepsPerGen as f32),
        ((((((unsafe { steps } as f32) / (oscillator as f32)) as i32) % 2) * 2) - 1) as f32,
    ];
    neuron_data.neural_net.prepare_net(&values);
    neuron_data.neural_net.feed_forward();

    let outputs = neuron_data.neural_net.get_outputs();

    let (x_rand, y_rand) = Direction::get_random(rng).get_move_offset();

    let dir = movement_data.lastMoveDir;

    let mut prob_x = outputs[NodeID::get_output_index(&NodeID::MoveEast)]
        - outputs[NodeID::get_output_index(&NodeID::MoveWest)]
        + outputs[NodeID::get_output_index(&NodeID::MoveRandom)] * x_rand
        + outputs[NodeID::get_output_index(&NodeID::MoveForward)] * dir.get_move_offset().0
        + outputs[NodeID::get_output_index(&NodeID::MoveReverse)]
            * dir.rotate180().get_move_offset().0
        + outputs[NodeID::get_output_index(&NodeID::MoveLeft)]
            * dir.rotateCCW90().get_move_offset().0
        + outputs[NodeID::get_output_index(&NodeID::MoveRight)]
            * dir.rotateCW90().get_move_offset().0;

    let mut prob_y = outputs[NodeID::get_output_index(&NodeID::MoveNorth)]
        - outputs[NodeID::get_output_index(&NodeID::MoveSouth)]
        + outputs[NodeID::get_output_index(&NodeID::MoveRandom)] * y_rand
        + outputs[NodeID::get_output_index(&NodeID::MoveForward)] * dir.get_move_offset().1
        + outputs[NodeID::get_output_index(&NodeID::MoveReverse)]
            * dir.rotate180().get_move_offset().1
        + outputs[NodeID::get_output_index(&NodeID::MoveLeft)]
            * dir.rotateCCW90().get_move_offset().1
        + outputs[NodeID::get_output_index(&NodeID::MoveRight)]
            * dir.rotateCW90().get_move_offset().1;

    prob_x = prob_x.tanh();
    prob_y = prob_y.tanh();

    let (mut x, mut y) = (movement_data.x, movement_data.y);

    if (rng.gen_range(0..i32::MAX) as f32) / (i32::MAX as f32) < prob_x.abs() {
        if prob_x > 0.0 {
            x += 1;
        } else {
            x = x.saturating_sub(1);
        }
    }

    if x >= gridWidth {
        x = gridWidth - 1;
    }

    if (thread_rng().gen_range(0..i32::MAX) as f32) / (i32::MAX as f32) < prob_y.abs() {
        if prob_y > 0.0 {
            y += 1;
        } else {
            y = y.saturating_sub(1);
        }
    }

    if y >= gridHeight {
        y = gridHeight - 1;
    }

    (x, y)
}

pub fn createColor(genome: &[Gene]) -> (u8, u8, u8) {
    const maxColorVal: u32 = 0xb0;
    const maxLumaVal: u32 = 0xb0;

    let mut color = {
        let c: u32 = u32::from(genome.first().unwrap().get_head_node_id().is_input())
            | (u32::from(genome.last().unwrap().get_head_node_id().is_input()) << 1)
            | (u32::from(genome.first().unwrap().get_tail_node_id().is_inner()) << 2)
            | (u32::from(genome.last().unwrap().get_tail_node_id().is_inner()) << 3)
            | (((genome.first().unwrap().get_head_node_id().get_index() & 1) as u32) << 4)
            | (((genome.first().unwrap().get_tail_node_id().get_index() & 1) as u32) << 5)
            | (((genome.last().unwrap().get_head_node_id().get_index() & 1) as u32) << 6)
            | (((genome.last().unwrap().get_tail_node_id().get_index() & 1) as u32) << 7);

        (c, ((c & 0x1f) << 3), ((c & 7) << 5))
    };

    if (color.0 * 3 + color.1 + color.2 * 4) / 8 > maxLumaVal {
        if color.0 > maxColorVal {
            color.0 %= maxColorVal
        };
        if color.1 > maxColorVal {
            color.1 %= maxColorVal
        };
        if color.2 > maxColorVal {
            color.2 %= maxColorVal
        };
    }

    (color.0 as u8, color.1 as u8, color.2 as u8)
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction {
    pub fn get_move_offset(&self) -> (f32, f32) {
        match *self {
            Direction::North => (0.0, 1.0),
            Direction::NorthEast => (1.0, 1.0),
            Direction::East => (1.0, 0.0),
            Direction::SouthEast => (1.0, -1.0),
            Direction::South => (0.0, -1.0),
            Direction::SouthWest => (-1.0, -1.0),
            Direction::West => (-1.0, 0.0),
            Direction::NorthWest => (-1.0, 1.0),
        }
    }

    pub fn get_random(rng: &mut ThreadRng) -> Direction {
        match rng.gen_range(0..8) {
            0 => Direction::North,
            1 => Direction::NorthEast,
            2 => Direction::East,
            3 => Direction::SouthEast,
            4 => Direction::South,
            5 => Direction::SouthWest,
            6 => Direction::West,
            7 => Direction::NorthWest,
            _ => unreachable!(),
        }
    }

    pub fn get_dir_from_offset(offset: (isize, isize)) -> Direction {
        match offset {
            (0, 1) => Direction::North,
            (1, 1) => Direction::NorthEast,
            (1, 0) => Direction::East,
            (1, -1) => Direction::SouthEast,
            (0, -1) => Direction::South,
            (-1, -1) => Direction::SouthWest,
            (-1, 0) => Direction::West,
            (-1, 1) => Direction::NorthWest,
            (_, _) => unimplemented!(),
        }
    }

    pub fn rotateCCW90(&self) -> Direction {
        match *self {
            Direction::North => Direction::West,
            Direction::NorthEast => Direction::NorthWest,
            Direction::East => Direction::North,
            Direction::SouthEast => Direction::NorthEast,
            Direction::South => Direction::East,
            Direction::SouthWest => Direction::SouthEast,
            Direction::West => Direction::South,
            Direction::NorthWest => Direction::SouthWest,
        }
    }

    pub fn rotateCW90(&self) -> Direction {
        match *self {
            Direction::West => Direction::North,
            Direction::NorthWest => Direction::NorthEast,
            Direction::North => Direction::East,
            Direction::NorthEast => Direction::SouthEast,
            Direction::East => Direction::South,
            Direction::SouthEast => Direction::SouthWest,
            Direction::South => Direction::West,
            Direction::SouthWest => Direction::NorthWest,
        }
    }

    pub fn rotate180(&self) -> Direction {
        match *self {
            Direction::North => Direction::South,
            Direction::NorthEast => Direction::SouthWest,
            Direction::East => Direction::West,
            Direction::SouthEast => Direction::NorthWest,
            Direction::South => Direction::North,
            Direction::SouthWest => Direction::NorthEast,
            Direction::West => Direction::East,
            Direction::NorthWest => Direction::SouthEast,
        }
    }
}
