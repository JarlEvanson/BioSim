use std::{
    fmt::{Debug, Display, Write},
    ops::{Add, BitXor, Mul},
};

use rand::{prelude::ThreadRng, RngCore};

use crate::neuron::NeuralNet;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Gene {
    /* Bits 0-15 = Weight -> 16 bits
     * Bits 16-23 = Tail Node -> 8 bits
     * Bits 24-31 = Head Node -> 8 bits
     */
    pub gene: u32,
}

impl Gene {
    #[inline(always)]
    pub fn new(gene: u32) -> Gene {
        Gene { gene }.normalize()
    }

    #[inline(always)]
    pub fn new_random(rng: &mut ThreadRng) -> Gene {
        Gene {
            gene: rng.next_u32(),
        }
        .normalize()
    }

    fn normalize(self) -> Gene {
        let weight = self.gene & 0xFFFF;
        let tail = ((self.gene >> 16) & 0xFF) % (INNER_NODE_COUNT + OUTPUT_NODE_COUNT) as u32;
        let head = ((self.gene >> 24) & 0xFF) % (INPUT_NODE_COUNT + INNER_NODE_COUNT) as u32;
        Gene {
            gene: (head << 24) | (tail << 16) | weight,
        }
    }

    pub fn get_head_node_id(&self) -> NodeID {
        NodeID::as_head_types((self.gene >> 24) as u8)
    }

    pub fn get_tail_node_id(&self) -> NodeID {
        NodeID::as_tail_types(((self.gene >> 16) & 0xFF) as u8)
    }

    pub fn get_weight(&self) -> f32 {
        (self.gene as i16 as f32) / ((u16::MAX / 8) as f32)
    }

    pub fn get_connection_index(&self) -> usize {
        NeuralNet::get_connection_index(self.get_head_node_id(), self.get_tail_node_id())
    }
}

impl BitXor<u32> for Gene {
    type Output = Gene;

    fn bitxor(self, rhs: u32) -> Self::Output {
        Gene {
            gene: self.gene ^ rhs,
        }
    }
}

impl Display for Gene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.get_head_node_id(),
            self.get_tail_node_id(),
            self.get_weight()
        )
    }
}

impl Debug for Gene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08x}", self.gene)
    }
}

pub const INPUT_NODE_COUNT: usize = 4;
pub const INNER_NODE_COUNT: usize = 3;
pub const OUTPUT_NODE_COUNT: usize = 10;
pub const TOTAL_NODE_COUNT: usize = INPUT_NODE_COUNT + INNER_NODE_COUNT + OUTPUT_NODE_COUNT;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum NodeID {
    //Input Nodes
    DistX = 0,
    DistY,
    Age,
    Oscillator,
    //Inner Nodes
    Inner1,
    Inner2,
    Inner3,
    //Output Nodes
    MoveNorth,
    MoveEast,
    MoveSouth,
    MoveWest,
    MoveRandom,
    MoveForward,
    MoveRight,
    MoveLeft,
    MoveReverse,
    KillForward,
    End,
}

impl Add<usize> for NodeID {
    type Output = usize;

    fn add(self, rhs: usize) -> Self::Output {
        self.get_index() + rhs
    }
}

impl Mul<usize> for NodeID {
    type Output = usize;

    fn mul(self, rhs: usize) -> Self::Output {
        self.get_index() * rhs
    }
}

impl Display for NodeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        write!(string, "{:?}", *self)?;
        f.pad(&string)
    }
}

use self::NodeID::*;

impl NodeID {
    const fn to_int(self) -> usize {
        match self {
            DistX => 0,
            DistY => 1,
            Age => 2,
            Oscillator => 3,
            Inner1 => 4,
            Inner2 => 5,
            Inner3 => 6,
            MoveNorth => 7,
            MoveEast => 8,
            MoveSouth => 9,
            MoveWest => 10,
            MoveRandom => 11,
            MoveForward => 12,
            MoveRight => 13,
            MoveLeft => 14,
            MoveReverse => 15,
            KillForward => 16,
            End => unimplemented!(),
        }
    }

    pub const fn get_max_connections() -> usize {
        INPUT_NODE_COUNT * INNER_NODE_COUNT
            + INPUT_NODE_COUNT * OUTPUT_NODE_COUNT
            + INNER_NODE_COUNT * INNER_NODE_COUNT
            + INNER_NODE_COUNT * OUTPUT_NODE_COUNT
    }

    pub const fn get_index(&self) -> usize {
        (*self) as i32 as usize
    }

    pub fn from_index(index: usize) -> NodeID {
        assert!(index < TOTAL_NODE_COUNT);

        unsafe { NodeID::from_index_unchecked(index) }
    }

    #[allow(unused)]
    unsafe fn from_index_unchecked(index: usize) -> NodeID {
        unsafe { std::mem::transmute::<u8, NodeID>(index as u8) }
    }

    fn as_head_types(value: u8) -> NodeID {
        unsafe { std::mem::transmute(value) }
    }

    fn as_tail_types(value: u8) -> NodeID {
        unsafe { std::mem::transmute(value + (INPUT_NODE_COUNT as u8)) }
    }

    pub const fn get_output_index(&self) -> usize {
        self.get_index() - (INNER_NODE_COUNT + INPUT_NODE_COUNT)
    }

    pub const fn get_input_index(&self) -> usize {
        self.get_index()
    }

    pub const fn get_inner_index(&self) -> usize {
        self.get_index() - INPUT_NODE_COUNT
    }

    pub fn as_input(value: usize) -> NodeID {
        NodeID::from_index(value)
    }

    pub fn as_inner(value: usize) -> NodeID {
        NodeID::from_index(value + INPUT_NODE_COUNT)
    }

    pub fn as_output(value: usize) -> NodeID {
        NodeID::from_index(value + INPUT_NODE_COUNT + INNER_NODE_COUNT)
    }

    pub const fn is_input(&self) -> bool {
        self.to_int() < NodeID::Inner1.to_int()
    }

    pub const fn is_inner(&self) -> bool {
        self.to_int() > NodeID::Oscillator.to_int() && self.to_int() < NodeID::MoveNorth.to_int()
    }

    pub const fn is_output(&self) -> bool {
        self.to_int() < NodeID::End.to_int() && self.to_int() > NodeID::Inner3.to_int()
    }
}

#[cfg(test)]
mod test {
    use super::NodeID::*;

    #[test]
    fn compare_const_and_runtime() {
        assert!(DistX.to_int() == DistX.get_index());
        assert!(DistY.to_int() == DistY.get_index());
        assert!(Age.to_int() == Age.get_index());
        assert!(Oscillator.to_int() == Oscillator.get_index());
        assert!(Inner1.to_int() == Inner1.get_index());
        assert!(Inner2.to_int() == Inner2.get_index());
        assert!(Inner3.to_int() == Inner3.get_index());
        assert!(MoveNorth.to_int() == MoveNorth.get_index());
        assert!(MoveEast.to_int() == MoveEast.get_index());
        assert!(MoveSouth.to_int() == MoveSouth.get_index());
        assert!(MoveWest.to_int() == MoveWest.get_index());
        assert!(MoveRandom.to_int() == MoveRandom.get_index());
        assert!(MoveForward.to_int() == MoveForward.get_index());
        assert!(MoveRight.to_int() == MoveRight.get_index());
        assert!(MoveLeft.to_int() == MoveLeft.get_index());
        assert!(MoveReverse.to_int() == MoveReverse.get_index());
        assert!(KillForward.to_int() == KillForward.get_index());
    }
}
