use std::fmt::{Debug, write};
use std::ops::Deref;

use crate::gene::{Gene, NodeID, OUTPUT_NODE_COUNT, INPUT_NODE_COUNT, INNER_NODE_COUNT};
use crate::{genome_length, neuron_presence};

use rand::{thread_rng, Rng};

pub struct NeuralNet {
    neurons: Box<[Neuron]>,
    connections: Box<[Connection]>
}

impl NeuralNet {
    pub fn new(genome: &Box<[Gene]>) -> NeuralNet {

        let mut neurons: Vec<Neuron> = Vec::new();
        
        let mut connections = Vec::new();
        
        for gene in genome.deref() {
            let mut head_already_added = false;
            let mut tail_already_added = false;
            let mut tail_index = 0;
            let mut head_index = 0;
            for neuron_index in 0..neurons.len() {
                if gene.get_head_node_id() == neurons[neuron_index].variant {
                    head_already_added = true;
                    head_index = neuron_index;
                }
                if gene.get_tail_node_id() == neurons[neuron_index].variant {
                    tail_already_added = true;
                    tail_index = neuron_index;
                }
            }

            if gene.get_head_node_id() == gene.get_tail_node_id() {
                tail_already_added = true;
            } 

            if !head_already_added {
                neurons.push(Neuron { variant: gene.get_head_node_id(), value: 0.0 });
                head_index = neurons.len() - 1;
            }
            if !tail_already_added {
                neurons.push(Neuron { variant: gene.get_tail_node_id(), value: 0.0 });
                tail_index = neurons.len() - 1;
            }

            connections.push( Connection { input: head_index, output: tail_index, weight: gene.get_weight() } );

        }

        for index in 0 .. neurons.len() {
            unsafe {
                neuron_presence[neurons[index].variant.get_index()] += 1;
            }
        }

        let connections = {
            let mut non_duplicated_connections: Vec<Connection> = Vec::new();

            for connection_index in 0..connections.len() {
                let mut duplicate_location = None;
                for search_index in 0..non_duplicated_connections.len() {
                    if  (
                        connections[connection_index].input == non_duplicated_connections[search_index].input &&
                        connections[connection_index].output == non_duplicated_connections[search_index].output
                     ) {
                        
                        duplicate_location = Some(search_index);
                    }
                }

                if duplicate_location.is_some() {
                    non_duplicated_connections[duplicate_location.unwrap()].weight += connections[connection_index].weight;
                } else {
                    non_duplicated_connections.push(connections[connection_index]);
                }
            }

            non_duplicated_connections
        };

        let mut sorted_connections = Vec::new();

        for index in 0..connections.len() {
            if neurons[connections[index].input].variant.is_input() && neurons[connections[index].output as usize].variant.is_output() {
                sorted_connections.push(connections[index]);
            }
        }

        for index in 0..connections.len() {
            if neurons[connections[index].input as usize].variant.is_input() && neurons[connections[index].output as usize].variant.is_inner() {
                sorted_connections.push(connections[index]);
            }
        }

        for index in 0..connections.len() {
            if neurons[connections[index].input as usize].variant.is_inner() && neurons[connections[index].output as usize].variant.is_inner() {
                sorted_connections.push(connections[index]);
            }
        }

        for index in 0..connections.len() {
            if neurons[connections[index].input].variant.is_inner() && neurons[connections[index].output as usize].variant.is_output() {
                sorted_connections.push(connections[index]);
            }
        }

        NeuralNet { neurons: neurons.into_boxed_slice(), connections: sorted_connections.into_boxed_slice() }
    }

    //Sensor Values
    //Index 0: X value
    //Index 1: Y value
    //Index 2: Age
    pub fn feed_forward(&mut self, sensor_values: &Vec<f32>) {
        for neuron in self.neurons.as_mut() {
            match neuron.variant {
                NodeID::DistX => { neuron.value = sensor_values[NodeID::DistX.get_index()] },
                NodeID::DistY => { neuron.value = sensor_values[NodeID::DistY.get_index()] },
                NodeID::Age => { neuron.value = sensor_values[NodeID::Age.get_index()] },
                NodeID::Oscillator => { neuron.value = sensor_values[NodeID::Oscillator.get_index()] }
                _ => {  }
            }
        }

        let mut last_index: usize = 0;

        for index in last_index..self.connections.len() {
            if self.neurons[self.connections[index].input as usize].variant.is_input() && self.neurons[self.connections[index].output as usize].variant.is_inner() {
                self.neurons[self.connections[index].output as usize].value += self.neurons[self.connections[index].input as usize].value * self.connections[index].weight;
            } else {
                last_index = index;
                break;
            }
        }

        for index in last_index..self.connections.len() {
            if self.neurons[self.connections[index].input as usize].variant.is_inner() && self.neurons[self.connections[index].output as usize].variant.is_inner() {
                self.neurons[self.connections[index].output as usize].value += self.neurons[self.connections[index].input as usize].value * self.connections[index].weight;
            } else {
                last_index = index;
                break;
            }
        }

        for index in 0..self.neurons.len() {
            if self.neurons[index].variant.is_inner() {
                self.neurons[index].value = activation(self.neurons[index].value);
            }
        }

        for index in last_index..self.connections.len() {
            self.neurons[self.connections[index].output as usize].value += self.neurons[self.connections[index].input as usize].value * self.connections[index].weight;
        }
    }

    pub fn get_outputs(&self) -> Vec<f32> {
        let mut outputs = vec![0.0; OUTPUT_NODE_COUNT as usize];

        for neuron in self.neurons.deref() {
            if neuron.variant.is_output() {
                outputs[NodeID::get_index(&neuron.variant) - INPUT_NODE_COUNT - INNER_NODE_COUNT] = neuron.value;
            }
        }

        outputs
    }

    pub fn clear(&mut self) {
        for index in 0..self.neurons.len() {
            self.neurons[index].value = 0.0;
        }
    }
}

fn activation(value: f32) -> f32 {
    value.tanh()
}

#[derive(Debug, Copy, Clone)]
struct Neuron {
    variant: NodeID,
    value: f32
}

#[derive(Clone, Copy, Debug)]
struct Connection {
    //Input and Output contain indexes to nodes to take values from
    input: usize,
    output: usize,
    weight: f32
}

impl Debug for NeuralNet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n\tNeurons: [\n" );
        for neuron in self.neurons.deref() {
            write!(f, "\t  {:10}: {}\n", neuron.variant, neuron.value);
        }
        write!(f, "\t],");
        write!(f, "\n\tConnections: [\n");
        for (index, connection) in self.connections.deref().into_iter().enumerate() {
            write!(f, "\t  Connection {:3}: {:10} -> {:10} Weight: {}\n", index, self.neurons[connection.input].variant, self.neurons[connection.output].variant, connection.weight);
        }
        write!(f, "\t]");
        Ok( () )
    }
}