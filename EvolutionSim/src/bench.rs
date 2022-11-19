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
use crate::grid_ptr;
use crate::pop_ptr;
use crate::population::Population;
use crate::should_reset;
use crate::steps;

use super::Config;
use super::ConfigBase;

#[bench]
fn benchStep(b: &mut Bencher) {
    println!("The argument file=\"path\" will load the save");

    let config: Config = Rc::new(ConfigBase::default());

    println!("{}", config);

    unsafe {
        let grid_layout = std::alloc::Layout::new::<Grid>();
        grid_ptr = std::alloc::alloc(grid_layout) as *mut Grid;

        let pop_layout = std::alloc::Layout::new::<Population>();
        pop_ptr = std::alloc::alloc(pop_layout) as *mut Population;

        std::ptr::write(pop_ptr, Population::new(&config));
        std::ptr::write(
            grid_ptr,
            Grid::new(config.getGridWidth(), config.getGridHeight()),
        );
    }
    unsafe { (*pop_ptr).assignRandom(&mut *grid_ptr) };

    let mut threadpool = Pool::new(std::thread::available_parallelism().unwrap().get() as u32);

    loop {
        if unsafe { should_reset } {
            unsafe {
                steps = 0;
                generation = 0;
                should_reset = false;
            }

            unsafe { &mut *pop_ptr }.genRandom(&config);
        }

        if unsafe { steps == 0 } {
            unsafe {
                (*grid_ptr).reset();
            }
            unsafe { &mut *pop_ptr }.assignRandom(unsafe { &mut *grid_ptr });

            println!("Generation {}:", unsafe { generation });
        }

        b.iter(|| {
            let size = computeMovements(&config, &mut threadpool, unsafe { &mut *pop_ptr });
            unsafe { &mut *pop_ptr }.resolveMoveQueue(size, unsafe { &mut *grid_ptr });

            determine_deaths(&config, unsafe { &mut *pop_ptr });
            unsafe { &mut *pop_ptr }.resolveDead(unsafe { &mut *grid_ptr });
        });

        if unsafe { steps } == config.getStepsPerGen() {
            if unsafe { generation } % 5 == 0 {}

            let reproducers = determine_reproducers(&config, unsafe { &mut *pop_ptr });
            if reproducers.len() == 0 {
                println!("Failed to produce viable offspring");
                exit(1);
            }

            println!(
                "Dead: {:3}\tReproducing: {:3}\tLiving Non-reproducing: {:3}",
                config.getPopSize() - unsafe { &*pop_ptr }.getLivingIndices().len(),
                reproducers.len(),
                unsafe { &*pop_ptr }.getLivingIndices().len() - reproducers.len(),
            );

            unsafe {
                (*pop_ptr).reproduceAsexually(&config, reproducers);
            }

            unsafe { steps = 0 };
            unsafe { generation += 1 };
        }
    }
}
