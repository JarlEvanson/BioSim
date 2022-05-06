use std::{fmt::{Display, Debug, Write}, ops::BitXor};

#[derive(Clone, Copy)]
pub struct Gene {
    /*Bit 31 = Head Type -> 1 bit
     *Bits 30-24 = Head NodeID -> 7 bits
     *Bits 23 = Tail Type -> 1 bit
     *Bits 22-16 = Tail NodeID -> 7 bits
     *Bits 15-0 = Weight -> 16 bits
    */
    gene: u32
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

#[derive(Clone, Copy, Debug)]
pub enum NodeID {
    DistX,
    DistY,
    Age,
    Inner1,
    Inner2,
    Inner3,
    MoveX,
    MoveY,
    MoveNorth,
    MoveSouth,
    MoveEast,
    MoveWest,
    MoveRandom
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
    pub fn as_inner(value: u8) -> NodeID {
        const UNIQUE_INNER_NODES: u8 = 3;
        match (value & 0b01111111) % UNIQUE_INNER_NODES {
            0 => NodeID::Inner1,
            1 => NodeID::Inner2,
            2 => NodeID::Inner3,
            _ => unreachable!()
        }
    }

    pub fn as_input(value: u8) -> NodeID {
        const UNIQUE_INPUT_NODES: u8 = 3;
        match (value & 0b01111111) % UNIQUE_INPUT_NODES {
            0 => NodeID::DistX,
            1 => NodeID::DistY,
            2 => NodeID::Age,
            _ => unreachable!()
        }
    }

    pub fn as_output(value: u8) -> NodeID {
        const UNIQUE_OUTPUT_NODES: u8 = 7;
        match (value & 0b01111111) % UNIQUE_OUTPUT_NODES {
            0 => NodeID::MoveX,
            1 => NodeID::MoveY,
            2 => NodeID::MoveNorth,
            3 => NodeID::MoveSouth,
            4 => NodeID::MoveEast,
            5 => NodeID::MoveWest,
            6 => NodeID::MoveRandom,
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
        if  *self == NodeID::MoveX || 
            *self == NodeID::MoveY ||
            *self == NodeID::MoveNorth ||
            *self == NodeID::MoveSouth ||
            *self == NodeID::MoveEast ||
            *self == NodeID::MoveWest ||
            *self == NodeID::MoveRandom
        {
            return true
        }
        return false
    }

    pub fn get_input_pos(name: &str) -> Option<u8> {
        match name {
            "DistX" => Some(0),
            "DistY" => Some(1),
            "Age" => Some(2),
            _ => None
        }
    }
}