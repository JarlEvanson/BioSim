use std::{
    fmt::{Debug, Display, Write},
    ops::BitXor,
};

use rand::{prelude::ThreadRng, Rng, RngCore};

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
    pub fn newRandom(rng: &mut ThreadRng) -> Gene {
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

    pub fn getHeadNodeID(&self) -> NodeID {
        self
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
            self.getHeadNodeID(),
            self.getTailNodeID(),
            self.getWeight()
        )
    }
}

impl Debug for Gene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08x}", self.gene)
    }
}

#[derive(Debug)]
pub enum NodeType {
    INPUT,
    INNER,
    OUTPUT,
}

impl PartialEq for NodeType {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

pub const INPUT_NODE_COUNT: u32 = 4;
pub const INNER_NODE_COUNT: u32 = 3;
pub const OUTPUT_NODE_COUNT: u32 = 10;
pub const TOTAL_NODE_COUNT: u32 = INPUT_NODE_COUNT + INNER_NODE_COUNT + OUTPUT_NODE_COUNT;

enum NodeID {
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
    fn getIndex(&self) -> usize {
        (*self) as usize
    }

    fn fromIndex(index: usize) -> NodeID {
        if index < TOTAL_NODE_COUNT {
            unsafe { return std::mem::transmute::<u8, NodeID>(index as u8) }
        } else {
            panic!("Invalid index");
        }
    }

    fn fromIndexUnchecked(index: usize) -> NodeID {
        unsafe { std::mem::transmute::<u8, NodeID>(index as u8) }
    }

    fn asInput(value: u8) -> NodeID {
        match (value & 0b01111111) as u8 {
            0 => NodeID::DistX,
            1 => NodeID::DistY,
            2 => NodeID::Age,
            3 => NodeID::Oscillator,
            _ => unreachable!(),
        }
    }

    fn asInner(value: u8) -> NodeID {
        match (value & 0b01111111) as u8 {
            0 => NodeID::Inner1,
            1 => NodeID::Inner2,
            2 => NodeID::Inner3,
            _ => unreachable!(),
        }
    }

    fn asOutput(value: u8) -> NodeID {
        match (value & 0b01111111) as u8 {
            0 => NodeID::MoveNorth,
            1 => NodeID::MoveSouth,
            2 => NodeID::MoveEast,
            3 => NodeID::MoveWest,
            4 => NodeID::MoveRandom,
            5 => NodeID::MoveForward,
            6 => NodeID::MoveRight,
            7 => NodeID::MoveLeft,
            8 => NodeID::MoveReverse,
            9 => NodeID::KillForward,
            _ => unreachable!(),
        }
    }

    fn isInner(&self) -> bool {
        if *self == NodeID::Inner1 || *self == NodeID::Inner2 || *self == NodeID::Inner3 {
            return true;
        }
        return false;
    }

    fn isInput(&self) -> bool {
        if *self == NodeID::DistX
            || *self == NodeID::DistY
            || *self == NodeID::Age
            || *self == NodeID::Oscillator
        {
            return true;
        }
        return false;
    }

    fn isOutput(&self) -> bool {
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
