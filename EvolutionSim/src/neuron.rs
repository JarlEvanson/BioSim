use std::fmt::Debug;

use crate::gene::{
    Gene, NodeID, INNER_NODE_COUNT, INPUT_NODE_COUNT, OUTPUT_NODE_COUNT, TOTAL_NODE_COUNT,
};

pub struct NeuralNet {
    //Input, Inner, Output
    neurons: [Neuron; TOTAL_NODE_COUNT],
    connections: [Connection; NodeID::get_max_connections()],
}

impl NeuralNet {
    pub fn new(genome: &[Gene]) -> NeuralNet {
        let mut net = NeuralNet {
            neurons: [Neuron { value: 0.0 }; TOTAL_NODE_COUNT],
            connections: [Connection { weight: 0.0 }; NodeID::get_max_connections()],
        };

        for gene in genome {
            net.connections[gene.get_connection_index()].weight += gene.get_weight();
        }

        net
    }

    pub fn prepare_net(&mut self, sensor_values: &[f32]) {
        self.clear();

        self.neurons[NodeID::DistX.get_index()].value = sensor_values[NodeID::DistX.get_index()];
        self.neurons[NodeID::DistY.get_index()].value = sensor_values[NodeID::DistY.get_index()];
        self.neurons[NodeID::Age.get_index()].value = sensor_values[NodeID::Age.get_index()];
        self.neurons[NodeID::Oscillator.get_index()].value =
            sensor_values[NodeID::Oscillator.get_index()];
    }

    //Sensor Values
    //Index 0: X value
    //Index 1: Y value
    //Index 2: Age
    pub fn feed_forward(&mut self) {
        //Input to Inner
        for tail in INPUT_NODE_COUNT..(INPUT_NODE_COUNT + INNER_NODE_COUNT) {
            for head in 0..INPUT_NODE_COUNT {
                self.neurons[tail].value +=
                    self.neurons[head].value * self.get_connection(head, tail).weight
            }
            self.neurons[tail].value = activation(self.neurons[tail].value);
        }

        //Input to Output
        for tail in (INPUT_NODE_COUNT + INNER_NODE_COUNT)..TOTAL_NODE_COUNT {
            for head in 0..INPUT_NODE_COUNT {
                self.neurons[tail].value +=
                    self.neurons[head].value * self.get_connection(head, tail).weight
            }
        }

        //Inner to Inner
        for tail in INPUT_NODE_COUNT..(INPUT_NODE_COUNT + INNER_NODE_COUNT) {
            for head in INPUT_NODE_COUNT..(INPUT_NODE_COUNT + INNER_NODE_COUNT) {
                self.neurons[tail].value +=
                    self.neurons[head].value * self.get_connection(head, tail).weight
            }

            self.neurons[tail].value = activation(self.neurons[tail].value);
        }

        //Inner to Output
        for tail in (INPUT_NODE_COUNT + INNER_NODE_COUNT)..TOTAL_NODE_COUNT {
            for head in INPUT_NODE_COUNT..(INPUT_NODE_COUNT + INNER_NODE_COUNT) {
                self.neurons[tail].value +=
                    self.neurons[head].value * self.get_connection(head, tail).weight
            }
        }
    }

    pub fn get_outputs(&self) -> [f32; OUTPUT_NODE_COUNT] {
        let mut outputs = [0.0; OUTPUT_NODE_COUNT];

        for neuron in (INPUT_NODE_COUNT + INNER_NODE_COUNT)..(TOTAL_NODE_COUNT) {
            outputs[neuron - INPUT_NODE_COUNT - INNER_NODE_COUNT] = self.neurons[neuron].value;
        }

        outputs
    }

    pub fn clear(&mut self) {
        for index in 0..self.neurons.len() {
            self.neurons[index].value = 0.0;
        }
    }

    fn get_connection(&self, head: usize, tail: usize) -> Connection {
        self.connections
            [NeuralNet::get_connection_index(NodeID::from_index(head), NodeID::from_index(tail))]
    }

    pub const fn get_connection_index(head: NodeID, tail: NodeID) -> usize {
        if head.is_input() {
            if tail.is_inner() {
                tail.get_inner_index() + head.get_input_index() * INPUT_NODE_COUNT
            } else {
                tail.get_output_index()
                    + head.get_input_index() * INNER_NODE_COUNT
                    + (INPUT_NODE_COUNT * INNER_NODE_COUNT)
            }
        } else {
            if tail.is_inner() {
                tail.get_inner_index()
                    + head.get_inner_index() * INNER_NODE_COUNT
                    + (INPUT_NODE_COUNT * INNER_NODE_COUNT + INPUT_NODE_COUNT * OUTPUT_NODE_COUNT)
            } else {
                tail.get_output_index()
                    + head.get_inner_index() * OUTPUT_NODE_COUNT
                    + (INPUT_NODE_COUNT * INNER_NODE_COUNT
                        + INPUT_NODE_COUNT * OUTPUT_NODE_COUNT
                        + INNER_NODE_COUNT * INNER_NODE_COUNT)
            }
        }
    }
}

fn activation(value: f32) -> f32 {
    if value <= 0.0 {
        0.0
    } else {
        value
    }
}

#[derive(Debug, Copy, Clone)]
struct Neuron {
    value: f32,
}

#[derive(Clone, Copy, Debug)]
struct Connection {
    weight: f32,
}

impl Debug for NeuralNet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n\tNeurons: [\n")?;
        for neuron in 0..TOTAL_NODE_COUNT {
            write!(
                f,
                "\t  {:10}: {}\n",
                NodeID::from_index(neuron),
                self.neurons[neuron].value
            )?;
        }
        Ok(())
    }
}
