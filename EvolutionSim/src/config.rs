use std::{
    fmt::Display,
    io::{Read, Write},
    mem::size_of,
    process::exit,
};

use crate::{grid::GridValueT, TimeT};

pub type MutR = f32;

pub struct Config {
    pop_size: usize,
    genome_length: usize,
    grid_width: usize,
    grid_height: usize,
    mutation_rate: MutR,
    steps_per_gen: TimeT,
    is_windowing: bool,
}

impl Config {
    pub fn new(
        pop_size: usize,
        genome_length: usize,
        grid_width: GridValueT,
        grid_height: GridValueT,
        mutation_rate: MutR,
        steps_per_gen: usize,
        is_windowing: bool,
    ) -> Self {
        assert!(pop_size <= (grid_width * grid_height));

        Config {
            pop_size,
            genome_length,
            grid_width,
            grid_height,
            mutation_rate,
            steps_per_gen,
            is_windowing,
        }
    }

    pub fn initFromArgs() -> Self {
        let mut config = Config::default();

        #[derive(PartialEq, Clone, Copy)]
        enum Next {
            PopSize,
            GenomeLength,
            GridWidth,
            GridHeight,
            MutationRate,
            StepsPerGen,
        }

        let mut next = None;

        let mut args = std::env::args();
        //Drops naming, useless right now
        args.next();

        for argument in args {
            match next {
                Some(opt) => {
                    match opt {
                        Next::PopSize => config.set_pop_size(argument.parse::<usize>().unwrap()),
                        Next::GenomeLength => {
                            config.set_genome_length(argument.parse::<usize>().unwrap())
                        }

                        Next::GridWidth => {
                            config.set_grid_width(argument.parse::<GridValueT>().unwrap())
                        }
                        Next::GridHeight => {
                            config.set_grid_height(argument.parse::<GridValueT>().unwrap())
                        }
                        Next::MutationRate => {
                            config.set_mutation_rate(argument.parse::<MutR>().unwrap())
                        }
                        Next::StepsPerGen => {
                            config.set_steps_per_gen(argument.parse::<TimeT>().unwrap())
                        }
                    }
                    next = None;
                }
                None => {
                    if argument.eq("-w") {
                        config.is_windowing = true;
                    } else if argument.eq("--population-size") || argument.eq("-p") {
                        next = Some(Next::PopSize)
                    } else if argument.eq("--width") {
                        next = Some(Next::GridWidth);
                    } else if argument.eq("--height") {
                        next = Some(Next::GridHeight);
                    } else if argument.eq("-g") || argument.eq("--genome-length") {
                        next = Some(Next::GenomeLength);
                    } else if argument.eq("-m") || argument.eq("--mutatation-rate") {
                        next = Some(Next::MutationRate);
                    } else if argument.eq("-s") || argument.eq("--steps-per-gen") {
                        next = Some(Next::StepsPerGen);
                    } else {
                        panic!("Invalid Option");
                    }
                }
            }
        }

        if next.is_some() {
            println!("Invalid Options\n");
            exit(1);
        }

        assert!(config.pop_size <= (config.grid_width * config.grid_height));

        config
    }

    pub fn get_pop_size(&self) -> usize {
        self.pop_size
    }

    pub fn get_genome_size(&self) -> usize {
        self.genome_length
    }

    pub fn get_grid_width(&self) -> GridValueT {
        self.grid_width
    }

    pub fn get_grid_height(&self) -> GridValueT {
        self.grid_height
    }

    pub fn get_mutation_rate(&self) -> MutR {
        self.mutation_rate
    }

    pub fn get_steps_per_gen(&self) -> TimeT {
        self.steps_per_gen
    }

    pub fn get_is_windowing(&self) -> bool {
        self.is_windowing
    }

    pub fn set_pop_size(&mut self, popSize: usize) {
        debug_assert_ne!(popSize, 0);

        self.pop_size = popSize;
    }

    pub fn set_genome_length(&mut self, genomeLength: usize) {
        debug_assert_ne!(genomeLength, 0);

        self.genome_length = genomeLength;
    }

    pub fn set_grid_width(&mut self, gridWidth: usize) {
        debug_assert_ne!(gridWidth, 0);

        self.grid_width = gridWidth;
    }

    pub fn set_grid_height(&mut self, gridHeight: usize) {
        debug_assert_ne!(gridHeight, 0);

        self.grid_height = gridHeight;
    }

    pub fn set_steps_per_gen(&mut self, stepsPerGen: TimeT) {
        debug_assert_ne!(stepsPerGen, 0);

        self.steps_per_gen = stepsPerGen;
    }

    pub fn set_mutation_rate(&mut self, mutationRate: MutR) {
        debug_assert!(mutationRate >= 0.0);

        self.mutation_rate = mutationRate;
    }

    pub fn serialize<T: Write>(&self, writer: &mut T) {
        writer
            .write(&(self.pop_size as u64).to_le_bytes())
            .expect("Error: Failed to write config");
        writer
            .write(&(self.genome_length as u64).to_le_bytes())
            .expect("Error: Failed to write config");
        writer
            .write(&(self.grid_width as u64).to_le_bytes())
            .expect("Error: Failed to write config");
        writer
            .write(&(self.grid_height as u64).to_le_bytes())
            .expect("Error: Failed to write config");
        writer
            .write(&(self.mutation_rate).to_le_bytes())
            .expect("Error: Failed to write config");
        writer
            .write(&(self.steps_per_gen as u64).to_le_bytes())
            .expect("Error: Failed to write config");
    }

    pub fn deserialize<T: Read>(reader: &mut T) -> Self {
        let mut buf8 = [0; size_of::<usize>()];
        let mut buf4 = [0; size_of::<f32>()];
        reader
            .read_exact(&mut buf8)
            .expect("Error: Failed to read config");
        let pop_size = usize::from_le_bytes(buf8);
        reader
            .read_exact(&mut buf8)
            .expect("Error: Failed to read config");
        let genome_length = usize::from_le_bytes(buf8);
        reader
            .read_exact(&mut buf8)
            .expect("Error: Failed to read config");
        let grid_width = usize::from_le_bytes(buf8);
        reader
            .read_exact(&mut buf8)
            .expect("Error: Failed to read config");
        let grid_height = usize::from_le_bytes(buf8);
        reader
            .read_exact(&mut buf4)
            .expect("Error: Failed to read config");
        let mutation_rate = MutR::from_le_bytes(buf4);
        reader
            .read_exact(&mut buf8)
            .expect("Error: Failed to read config");
        let steps_per_gen = usize::from_le_bytes(buf8);
        Config {
            pop_size,
            genome_length,
            grid_width,
            grid_height,
            mutation_rate,
            steps_per_gen,
            is_windowing: false,
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Population Size: {}", self.pop_size)?;
        writeln!(f, "Steps Per Gen: {}", self.steps_per_gen)?;
        writeln!(
            f,
            "Width: {}\nHeight: {}",
            self.grid_width, self.grid_height
        )?;
        writeln!(f, "Genome Length: {}", self.genome_length)?;
        writeln!(f, "Mutation Rate: {}%", self.mutation_rate)?;
        writeln!(f, "Windowing: {}", self.is_windowing)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pop_size: 4000,
            genome_length: 20,
            grid_width: 200,
            grid_height: 200,
            mutation_rate: 0.1,
            steps_per_gen: 250,
            is_windowing: false,
        }
    }
}
