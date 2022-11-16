use std::{fmt::{Display, Debug, Write}, ops::BitXor};

use rand::Rng;

use ProcEvolutionSim::mergeEnums;

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

    #[inline(always)]
    pub fn new_random() -> Gene {
        let mut gene = 0;

        let head_node = NodeID::from_index(rand::thread_rng().gen_range(0 .. (INPUT_NODE_COUNT + INNER_NODE_COUNT)));

        if head_node.is_input() {
            gene = gene | (1 << 31);
        }

        gene = gene | ( head_node.get_index() << 24);

        let tail_node = NodeID::from_index(rand::thread_rng().gen_range(0 .. (INNER_NODE_COUNT + OUTPUT_NODE_COUNT)) + FIRST_INNER_NODE_INDEX);

        if tail_node.is_inner() {
            gene = gene | (1 << 23);
        }

        gene = gene | ( (tail_node.get_index() - INPUT_NODE_COUNT + 1) << 16 );

        let gene = (gene | (rand::thread_rng().gen::<u16>() as usize)) as u32;

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

//#[derive(Debug, Clone, Copy)]
mergeEnums!(
    NodeID,
    enum INPUT_NODE {
        DistX,
        DistY,
        Age,
        Oscillator
    },
    enum INNER_NODE {
        Inner1,
        Inner2,
        Inner3
    },
    enum OUTPUT_NODE {
        MoveNorth,
        MoveEast,
        MoveSouth,
        MoveWest,
        MoveRandom,
        MoveForward,
        MoveRight,
        MoveLeft,
        MoveReverse,
        KillForward
    }
);

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
    pub fn get_index(&self) -> usize {
        (*self) as usize
    }

    pub fn from_index(index: usize) -> NodeID {
        if index < NodeID_COUNT {
            unsafe {
                return std::mem::transmute::<u8, NodeID>(index as u8)
            }
        } else {
            panic!("Invalid index");
        }
    }

    pub fn as_inner(value: u8) -> NodeID {
        match (value & 0b01111111) % INNER_NODE_COUNT as u8 {
            0 => NodeID::Inner1,
            1 => NodeID::Inner2,
            2 => NodeID::Inner3,
            _ => unreachable!()
        }
    }

    pub fn as_input(value: u8) -> NodeID {
        match (value & 0b01111111) % INPUT_NODE_COUNT as u8 {
            0 => NodeID::DistX,
            1 => NodeID::DistY,
            2 => NodeID::Age,
            3 => NodeID::Oscillator,
            _ => unreachable!()
        }
    }

    pub fn as_output(value: u8) -> NodeID {
        match (value & 0b01111111) % OUTPUT_NODE_COUNT as u8 {
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
            *self == NodeID::Age ||
            *self == NodeID::Oscillator
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
            *self == NodeID::MoveReverse ||
            *self == NodeID::KillForward
        {
            return true
        }
        return false
    }
}