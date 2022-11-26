#![allow(unused_imports)]
extern crate test;
use std::process::exit;
use std::rc::Rc;

use self::test::Bencher;
use custom_dst::MaybeUninitDstArray;
use scoped_threadpool::Pool;

use crate::cell::HeritableData;
use crate::computeMovements;
use crate::determine_deaths;
use crate::determine_reproducers;
use crate::gene::Gene;
use crate::grid::Grid;
use crate::population::Population;
use crate::DebugCell::DebugRefCell;

use super::Config;
use super::ConfigBase;
#[allow(dead_code)]
fn normal_setup() -> (Config, Rc<DebugRefCell<Grid>>, Rc<DebugRefCell<Population>>) {
    let config: Config = Rc::new(ConfigBase::default());

    let grid = Rc::new(DebugRefCell::new(Grid::new(
        config.get_grid_width(),
        config.get_grid_height(),
    )));

    let population = Rc::new(DebugRefCell::new(Population::new(
        &config,
        &mut grid.borrowMut(),
    )));

    (config, grid, population)
}

#[bench]
fn assignGrid(b: &mut Bencher) {
    let (_config, grid, population) = normal_setup();

    b.iter(|| {
        population.borrowMut().assignRandom(&mut grid.borrowMut());
        grid.borrowMut().reset();
    });
}

#[bench]
fn genRandom(b: &mut Bencher) {
    let (config, grid, population) = normal_setup();

    b.iter(|| {
        population
            .borrowMut()
            .genRandom(&config, &mut grid.borrowMut());
        grid.borrowMut().reset();
    });
}

#[bench]
fn computeMovementsBench(b: &mut Bencher) {
    let (config, _grid, population) = normal_setup();

    let mut threadpool = Pool::new(std::thread::available_parallelism().unwrap().get() as u32);

    b.iter(|| computeMovements(&config, &mut threadpool, &mut population.borrowMut(), 0));
}

#[bench]
fn reproduceAsexually(b: &mut Bencher) {
    let (config, grid, population) = normal_setup();

    let scratch = MaybeUninitDstArray::<HeritableData, Gene>::new(
        config.get_genome_size(),
        config.get_pop_size(),
    );

    //Safety: this will be written to before it is ever read from
    let mut scratch = { unsafe { scratch.assume_init() } };

    b.iter(|| {
        let reproducing_cells = population.borrow().getLivingIndices();
        grid.borrowMut().reset();
        population.borrowMut().reproduceAsexually(
            &mut scratch,
            &config,
            reproducing_cells,
            &mut grid.borrowMut(),
        )
    });
}
