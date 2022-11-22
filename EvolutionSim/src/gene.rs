use std::{
    fmt::{Debug, Display, Write},
    ops::BitXor,
};

use rand::{prelude::ThreadRng, RngCore};

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
        let tail = ((self.gene >> 16) & 0xFF) % (INNER_NODE_COUNT + OUTPUT_NODE_COUNT);
        let head = ((self.gene >> 24) & 0xFF) % (INPUT_NODE_COUNT + INNER_NODE_COUNT);
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

pub const INPUT_NODE_COUNT: u32 = 4;
pub const INNER_NODE_COUNT: u32 = 3;
pub const OUTPUT_NODE_COUNT: u32 = 10;
pub const TOTAL_NODE_COUNT: u32 = INPUT_NODE_COUNT + INNER_NODE_COUNT + OUTPUT_NODE_COUNT;

#[derive(Clone, Copy, Debug)]
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
}

impl Display for NodeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        write!(string, "{:?}", *self)?;
        f.pad(&string)
    }
}

impl PartialEq for NodeID {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl NodeID {
    pub fn get_index(&self) -> usize {
        (*self) as i32 as usize
    }

    pub fn from_index(index: usize) -> NodeID {
        debug_assert!(index < TOTAL_NODE_COUNT as usize);

        unsafe { return std::mem::transmute::<u8, NodeID>(index as u8) }
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

    pub fn get_output_id(&self) -> usize {
        self.get_index() - (INNER_NODE_COUNT as usize + INPUT_NODE_COUNT as usize)
    }

    pub fn is_inner(&self) -> bool {
        if *self == NodeID::Inner1 || *self == NodeID::Inner2 || *self == NodeID::Inner3 {
            return true;
        }
        return false;
    }

    pub fn is_input(&self) -> bool {
        if *self == NodeID::DistX
            || *self == NodeID::DistY
            || *self == NodeID::Age
            || *self == NodeID::Oscillator
        {
            return true;
        }
        return false;
    }

    pub fn is_output(&self) -> bool {
        if *self == NodeID::MoveNorth
            || *self == NodeID::MoveSouth
            || *self == NodeID::MoveEast
            || *self == NodeID::MoveWest
            || *self == NodeID::MoveRandom
            || *self == NodeID::MoveForward
            || *self == NodeID::MoveRight
            || *self == NodeID::MoveLeft
            || *self == NodeID::MoveReverse
            || *self == NodeID::KillForward
        {
            return true;
        }
        return false;
    }
}
