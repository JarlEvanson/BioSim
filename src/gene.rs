use std::{fmt::{Display, Debug, Write}, ops::BitXor};

#[derive(Clone, Copy)]
pub struct Gene {
    /*Bit 31 = Head Type -> 1 bit
     *Bits 30-24 = Head NodeID -> 7 bits
     *Bits 23 = Tail Type -> 1 bit
     *Bits 22-16 = Tail NodeID -> 7 bits
     *Bits 15-0 = Weight -> 16 bits
    */
    pub gene: u32
}

impl Gene {
    #[inline(always)]
    pub fn new(gene: u32) -> Gene {
        Gene { gene }
    }

    pub fn get_head_type(&self) -> NodeType {
        if self.gene >> 31 == 1 { NodeType::INPUT } else { NodeType::INNER }
    }

    pub fn get_tail_type(&self) -> NodeType {
        if (self.gene >> 23) & 1 == 1 { NodeType::INNER } else { NodeType::OUTPUT } 
    }

    pub fn get_weight(&self) -> f32 {
        ((self.gene & 0xffff) as i16 as f32) / ((u16::MAX / 8) as f32)
    }

    pub fn get_head_node_id(&self) -> NodeID {
        match self.get_head_type() {
            NodeType::INNER => NodeID::as_inner((self.gene >> 24) as u8),
            NodeType::INPUT => NodeID::as_input((self.gene >> 24) as u8),
            _ => panic!()
        }
    }

    pub fn get_tail_node_id(&self) -> NodeID {
        match self.get_tail_type() {
            NodeType::INNER => NodeID::as_inner((self.gene >> 16) as u8),
            NodeType::OUTPUT => NodeID::as_output((self.gene >> 16) as u8),
            _ => panic!()
        }
    }
}

impl BitXor<u32> for Gene {
    type Output = Gene;

    fn bitxor(self, rhs: u32) -> Self::Output {
        Gene { gene: self.gene ^ rhs }
    }
}

impl Display for Gene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.get_head_node_id(), self.get_tail_node_id(), self.get_weight())
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
    OUTPUT
}

impl PartialEq for NodeType {

    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

pub const UNIQUE_INPUT_NODES: u8 = 4;
pub const UNIQUE_INNER_NODES: u8 = 3;
pub const UNIQUE_OUTPUT_NODES: u8 = 9;

#[derive(Clone, Copy, Debug)]
pub enum NodeID {
    DistX,
    DistY,
    Age,
    Oscillator,
    Inner1,
    Inner2,
    Inner3,
    MoveNorth,
    MoveSouth,
    MoveEast,
    MoveWest,
    MoveRandom,
    MoveForward,
    MoveRight,
    MoveLeft,
    MoveReverse
}

impl Display for NodeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        write!(string, "{:?}", *self);
        f.pad(&string)
    }
}

impl PartialEq for NodeID {

    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl NodeID {
    pub fn get_output_index(&self) -> usize {
        match *self {
            NodeID::MoveNorth => 0,
            NodeID::MoveSouth => 1,
            NodeID::MoveEast => 2,
            NodeID::MoveWest => 3,
            NodeID::MoveForward => 4,
            NodeID::MoveRight => 5,
            NodeID::MoveLeft => 6,
            NodeID::MoveReverse => 7,
            NodeID::MoveRandom => 8,
            _ => unimplemented!()
        }
    }

    pub fn from_output_index(index: usize) -> &'static str {
        match index {
            0  => "MoveNorth",
            1  => "MoveSouth",
            2  => "MoveEast",
            3  => "MoveWest",
            4  => "MoveRandom",
            5 => "MoveForward",
            6 => "MoveRight",
            7 => "MoveLeft",
            8 => "MoveReverse",
            _ => ""
        }
    }

    pub fn get_input_index(&self) -> usize {
        match *self {
            NodeID::DistX => 0,
            NodeID::DistY => 1,
            NodeID::Age => 2,
            NodeID::Oscillator => 3,
            _ => unimplemented!()
        }
    }

    pub fn get_index(&self) -> usize {
        match *self {
            NodeID::Age => 0,
            NodeID::DistX => 1,
            NodeID::DistY => 2,
            NodeID::Inner1 => 3,
            NodeID::Inner2 => 4,
            NodeID::Inner3 => 5,
            NodeID::MoveNorth  => 6,
            NodeID::MoveSouth  => 7,
            NodeID::MoveEast  => 8,
            NodeID::MoveWest => 9,
            NodeID::MoveForward => 10,
            NodeID::MoveRight => 11,
            NodeID::MoveLeft => 12,
            NodeID::MoveReverse => 13,
            NodeID::MoveRandom => 14,
            NodeID::Oscillator => 15,
            _ => unimplemented!()
        }
    }

    pub fn from_index(index: usize) -> &'static str {
        match index {
            0 => "Age",
            1 => "DistX",
            2 => "DistY",
            3 => "Inner1",
            4 => "Inner2",
            5 => "Inner3",
            6 => "MoveNorth",
            7 => "MoveSouth",
            8 => "MoveEast",
            9 => "MoveWest",
            10 => "MoveForward",
            11 => "MoveRight",
            12 => "MoveLeft",
            13 => "MoveReverse",
            14 => "MoveRandom",
            15 => "Oscillator",
            _ => unimplemented!()
        }
    }

    pub fn as_inner(value: u8) -> NodeID {
        match (value & 0b01111111) % UNIQUE_INNER_NODES {
            0 => NodeID::Inner1,
            1 => NodeID::Inner2,
            2 => NodeID::Inner3,
            _ => unreachable!()
        }
    }

    pub fn as_input(value: u8) -> NodeID {
        match (value & 0b01111111) % UNIQUE_INPUT_NODES {
            0 => NodeID::DistX,
            1 => NodeID::DistY,
            2 => NodeID::Age,
            3 => NodeID::Oscillator,
            _ => unreachable!()
        }
    }

    pub fn as_output(value: u8) -> NodeID {
        match (value & 0b01111111) % UNIQUE_OUTPUT_NODES {
            0 => NodeID::MoveNorth,
            1 => NodeID::MoveSouth,
            2 => NodeID::MoveEast,
            3 => NodeID::MoveWest,
            4 => NodeID::MoveRandom,
            5 => NodeID::MoveForward,
            6 => NodeID::MoveRight,
            7 => NodeID::MoveLeft,
            8 => NodeID::MoveReverse,
            _ => unreachable!()
        }
    }

    pub fn is_inner(&self) -> bool {
        if  *self == NodeID::Inner1 || 
            *self == NodeID::Inner2 || 
            *self == NodeID::Inner3 
        {
            return true
        }
        return false
    }

    pub fn is_input(&self) -> bool {
        if  *self == NodeID::DistX || 
            *self == NodeID::DistY ||
            *self == NodeID::Age
        {
            return true
        }
        return false
    }

    pub fn is_output(&self) -> bool {
        if  *self == NodeID::MoveNorth ||
            *self == NodeID::MoveSouth ||
            *self == NodeID::MoveEast ||
            *self == NodeID::MoveWest ||
            *self == NodeID::MoveRandom ||
            *self == NodeID::MoveForward ||
            *self == NodeID::MoveRight ||
            *self == NodeID::MoveLeft ||
            *self == NodeID::MoveReverse
        {
            return true
        }
        return false
    }
}