use std::{
    fmt::{write, Display},
    str,
};

pub type MutR = f32;

pub struct Config {
    popSize: usize,
    genomeLength: usize,
    gridWidth: usize,
    gridHeight: usize,
    mutationRate: MutR,
    stepsPerGen: usize,
}

impl Config {
    pub fn new(
        popSize: usize,
        genomeLength: usize,
        gridWidth: usize,
        gridHeight: usize,
        mutationRate: MutR,
        stepsPerGen: usize,
    ) -> Config {
        Config {
            popSize,
            genomeLength,
            gridWidth,
            gridHeight,
            mutationRate,
            stepsPerGen,
        }
    }

    pub fn load<T: ToString>(config: T) -> Config {
        let string = config.to_string();

        todo!()
    }

    pub fn getPopSize(&self) -> usize {
        self.popSize
    }

    pub fn getGenomeSize(&self) -> usize {
        self.genomeLength
    }

    pub fn getGridWidth(&self) -> usize {
        self.gridWidth
    }

    pub fn getGridHeight(&self) -> usize {
        self.gridHeight
    }

    pub fn getMutationRate(&self) -> MutR {
        self.mutationRate
    }

    pub fn getStepsPerGen(&self) -> usize {
        self.stepsPerGen
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Population Size: {}\n", self.popSize)?;
        write!(f, "Steps Per Gen: {}\n", self.stepsPerGen)?;
        write!(
            f,
            "Width: {}\nHeight: {}\n",
            self.gridWidth, self.gridHeight
        )?;
        write!(f, "Genome Length: {}\n", self.genomeLength)?;
        write!(f, "Mutation Rate: {}%", self.mutationRate)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            popSize: 4000,
            genomeLength: 20,
            gridWidth: 100,
            gridHeight: 100,
            mutationRate: 0.5,
            stepsPerGen: 250,
        }
    }
}

pub fn readINIEntry<T: str::FromStr>(
    key: &'static str,
    str: &str,
) -> Result<T, <T as str::FromStr>::Err> {
    (&str[key.len() + 1..]).parse::<T>()
}
