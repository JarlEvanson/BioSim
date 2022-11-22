#![allow(unused_imports)]
extern crate test;
use std::process::exit;
use std::rc::Rc;

use self::test::Bencher;
use scoped_threadpool::Pool;

use crate::computeMovements;
use crate::determine_deaths;
use crate::determine_reproducers;
use crate::generation;
use crate::grid::Grid;
use crate::population::Population;
use crate::should_reset;
use crate::steps;
use crate::DebugCell::DebugRefCell;

use super::Config;
use super::ConfigBase;

#[bench]
fn assignGrid(b: &mut Bencher) {
    let config: Config = Rc::new(ConfigBase::default());

    let grid = Rc::new(DebugRefCell::new(Grid::new(
        config.getGridWidth(),
        config.getGridHeight(),
    )));

    let population = Rc::new(DebugRefCell::new(Population::new(
        &config,
        &mut grid.borrowMut(),
    )));

    b.iter(|| {
        population.borrowMut().assignRandom(&mut grid.borrowMut());
        grid.borrowMut().reset();
    });
}

#[bench]
fn genRandom(b: &mut Bencher) {
    let config: Config = Rc::new(ConfigBase::default());

    let grid = Rc::new(DebugRefCell::new(Grid::new(
        config.getGridWidth(),
        config.getGridHeight(),
    )));

    let population = Rc::new(DebugRefCell::new(Population::new(
        &config,
        &mut grid.borrowMut(),
    )));

    b.iter(|| {
        population
            .borrowMut()
            .genRandom(&config, &mut grid.borrowMut());
        grid.borrowMut().reset();
    });
}

#[bench]
fn computeMovementsBench(b: &mut Bencher) {
    let config: Config = Rc::new(ConfigBase::default());

    let grid = Rc::new(DebugRefCell::new(Grid::new(
        config.getGridWidth(),
        config.getGridHeight(),
    )));

    let population = Rc::new(DebugRefCell::new(Population::new(
        &config,
        &mut grid.borrowMut(),
    )));

    let mut threadpool = Pool::new(std::thread::available_parallelism().unwrap().get() as u32);

    b.iter(|| computeMovements(&config, &mut threadpool, &mut population.borrowMut()));
}
