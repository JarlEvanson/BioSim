use std::{fmt::Display, process::exit, str};

use crate::{grid::GridValueT, TimeT};

pub type MutR = f32;

pub struct Config {
    popSize: usize,
    genomeLength: usize,
    gridWidth: usize,
    gridHeight: usize,
    mutationRate: MutR,
    stepsPerGen: TimeT,
    isWindowing: bool,
}

impl Config {
    pub fn new(
        popSize: usize,
        genomeLength: usize,
        gridWidth: GridValueT,
        gridHeight: GridValueT,
        mutationRate: MutR,
        stepsPerGen: usize,
        isWindowing: bool,
    ) -> Config {
        assert!(popSize <= (gridWidth * gridHeight));

        Config {
            popSize,
            genomeLength,
            gridWidth,
            gridHeight,
            mutationRate,
            stepsPerGen,
            isWindowing,
        }
    }

    pub fn initFromArgs() -> Config {
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
                        Next::PopSize => config.setPopSize(argument.parse::<usize>().unwrap()),
                        Next::GenomeLength => {
                            config.setGenomeLength(argument.parse::<usize>().unwrap())
                        }

                        Next::GridWidth => {
                            config.setGridWidth(argument.parse::<GridValueT>().unwrap())
                        }
                        Next::GridHeight => {
                            config.setGridHeight(argument.parse::<GridValueT>().unwrap())
                        }
                        Next::MutationRate => {
                            config.setMutationRate(argument.parse::<MutR>().unwrap())
                        }
                        Next::StepsPerGen => {
                            config.setStepsPerGen(argument.parse::<TimeT>().unwrap())
                        }
                    }
                    next = None;
                }
                None => {
                    if argument.eq("-w") {
                        config.isWindowing = true;
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

        if next != None {
            println!("Invalid Options\n");
            exit(1);
        }

        assert!(config.popSize <= (config.gridWidth * config.gridHeight));

        config
    }

    pub fn getPopSize(&self) -> usize {
        self.popSize
    }

    pub fn getGenomeSize(&self) -> usize {
        self.genomeLength
    }

    pub fn getGridWidth(&self) -> GridValueT {
        self.gridWidth
    }

    pub fn getGridHeight(&self) -> GridValueT {
        self.gridHeight
    }

    pub fn getMutationRate(&self) -> MutR {
        self.mutationRate
    }

    pub fn getStepsPerGen(&self) -> TimeT {
        self.stepsPerGen
    }

    pub fn getIsWindowing(&self) -> bool {
        self.isWindowing
    }

    pub fn setPopSize(&mut self, popSize: usize) {
        debug_assert_ne!(popSize, 0);

        self.popSize = popSize;
    }

    pub fn setGenomeLength(&mut self, genomeLength: usize) {
        debug_assert_ne!(genomeLength, 0);

        self.genomeLength = genomeLength;
    }

    pub fn setGridWidth(&mut self, gridWidth: usize) {
        debug_assert_ne!(gridWidth, 0);

        self.gridWidth = gridWidth;
    }

    pub fn setGridHeight(&mut self, gridHeight: usize) {
        debug_assert_ne!(gridHeight, 0);

        self.gridHeight = gridHeight;
    }

    pub fn setStepsPerGen(&mut self, stepsPerGen: TimeT) {
        debug_assert_ne!(stepsPerGen, 0);

        self.stepsPerGen = stepsPerGen;
    }

    pub fn setMutationRate(&mut self, mutationRate: MutR) {
        debug_assert!(mutationRate >= 0.0);

        self.mutationRate = mutationRate;
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
        write!(f, "Mutation Rate: {}%\n", self.mutationRate)?;
        write!(f, "Windowing: {}\n", self.isWindowing)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            popSize: 4000,
            genomeLength: 20,
            gridWidth: 200,
            gridHeight: 200,
            mutationRate: 0.1,
            stepsPerGen: 250,
            isWindowing: false,
        }
    }
}
