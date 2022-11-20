#![allow(non_snake_case, non_upper_case_globals, temporary_cstring_as_ptr)]
#![feature(trace_macros, new_uninit)]
#![feature(test)]

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::slice::{Chunks, ChunksMut};
use std::{process::exit, rc::Rc};

extern crate ProcEvolutionSim;

extern crate rand;

extern crate scoped_threadpool;
use scoped_threadpool::Pool;

mod windowed;
use windowed::window::Window;

mod grid;
use grid::{Grid, GridValueT};

mod population;
use population::Population;

mod cell;

mod gene;
use gene::NodeID_COUNT;

use crate::windowed::window::wait;
mod neuron;

mod bench;

use DebugCell::DebugRefCell;

//Statistics
static mut neuron_presence: [u32; NodeID_COUNT] = [0; NodeID_COUNT];

//Display
static mut grid_display_side_length: u32 = 0;
static mut framebuffer_width: u32 = 0;
static mut framebuffer_height: u32 = 0;

//Changes during runtime
static mut accounted_time: f64 = 0.0;
static mut generation: TimeT = 0;
static mut steps: TimeT = 0;
static mut should_reset: bool = false;
static mut pause: bool = false;

mod config;
use config::Config as ConfigBase;

type Config = Rc<ConfigBase>;
type TimeT = usize;

fn main() {
    println!("The argument file=\"path\" will load the save");

    let config: Config = Rc::new(ConfigBase::initFromArgs());

    println!("{}", config);

    let grid = Rc::new(DebugRefCell::new(Grid::new(
        config.getGridWidth(),
        config.getGridHeight(),
    )));

    let population = Rc::new(DebugRefCell::new(Population::new(&config)));

    if config.getIsWindowing() {
        println!("Press R to reset simulation\nPress SPACE to pause and restart simulation\nPress E to print current neuron frequencies\nPress Escape to close window\nPress S to save current generation's genes");

        let window = Window::createWindow(&config, 512, 512).expect("Window failed to be created");
        unsafe {
            framebuffer_width = 512;
            framebuffer_height = 512;
            grid_display_side_length = 512;
        }
        window.make_current();

        population.borrowMut().assignRandom(&mut grid.borrowMut());

        window.render(&config, &population.borrow());

        unsafe {
            accounted_time = glfw::ffi::glfwGetTime();
        }

        let mut outputted = false;

        let mut threadpool = Pool::new(1); //std::thread::available_parallelism().unwrap().get() as u32);

        while !window.shouldClose() {
            window.poll();
            if unsafe { should_reset } {
                unsafe {
                    accounted_time = glfw::ffi::glfwGetTime();
                    steps = 0;
                    generation = 0;
                    should_reset = false;
                }

                population.borrowMut().genRandom(&config);
            }

            if unsafe { steps == 0 } && !outputted {
                grid.borrowMut().reset();

                population.borrowMut().assignRandom(&mut grid.borrowMut());

                window.render(&config, &population.borrow());

                println!("Generation {}:", unsafe { generation });

                outputted = true;
            }

            if unsafe { glfw::ffi::glfwGetTime() - accounted_time > 0.016 && !pause } {
                outputted = false;
                unsafe {
                    accounted_time += 0.016;
                    steps += 1;
                };

                let size = computeMovements(&config, &mut threadpool, &mut population.borrowMut());
                population
                    .borrowMut()
                    .resolveMoveQueue(size, &mut grid.borrowMut());

                determine_deaths(&config, &mut population.borrowMut());
                population.borrowMut().resolveDead(&mut grid.borrowMut());

                if population.borrow().getLivingIndices().len() == 0 {
                    println!("Everyone Died");
                    loop {
                        window.poll();
                        if window.shouldClose() || unsafe { should_reset } {
                            unsafe { accounted_time = glfw::ffi::glfwGetTime() };
                            break;
                        }
                    }
                    continue;
                }

                window.render(&config, &population.borrow());
            }

            if unsafe { steps } == config.getStepsPerGen() {
                let reproducers = determine_reproducers(&config, &mut population.borrowMut());
                if reproducers.len() == 0 {
                    println!("Failed to produce viable offspring");
                    loop {
                        window.poll();
                        if window.shouldClose() || unsafe { should_reset } {
                            unsafe { accounted_time = glfw::ffi::glfwGetTime() };
                            break;
                        }
                    }
                    continue;
                }

                println!(
                    "Dead: {:3}\tReproducing: {:3}\tLiving Non-reproducing: {:3}",
                    config.getPopSize() - &population.borrow().getLivingIndices().len(),
                    reproducers.len(),
                    &population.borrow().getLivingIndices().len() - reproducers.len(),
                );

                population
                    .borrowMut()
                    .reproduceAsexually(&config, reproducers);

                //wait(&window, 2.0);

                unsafe { steps = 0 };
                unsafe { generation += 1 };
            }
        }
    } else {
        population.borrowMut().assignRandom(&mut grid.borrowMut());

        let mut threadpool = Pool::new(std::thread::available_parallelism().unwrap().get() as u32);

        loop {
            if unsafe { should_reset } {
                unsafe {
                    steps = 0;
                    generation = 0;
                    should_reset = false;
                }

                population.borrowMut().genRandom(&config);
            }

            if unsafe { steps == 0 } {
                grid.borrowMut().reset();
                population.borrowMut().assignRandom(&mut grid.borrowMut());

                println!("Generation {}:", unsafe { generation });
            }

            {
                unsafe {
                    steps += 1;
                };

                let size = computeMovements(&config, &mut threadpool, &mut population.borrowMut());
                population
                    .borrowMut()
                    .resolveMoveQueue(size, &mut grid.borrowMut());

                determine_deaths(&config, &mut population.borrowMut());
                population.borrowMut().resolveDead(&mut grid.borrowMut());
            }

            if unsafe { steps } == config.getStepsPerGen() {
                if unsafe { generation } % 5 == 0 {}

                let reproducers = determine_reproducers(&config, &mut population.borrowMut());
                if reproducers.len() == 0 {
                    println!("Failed to produce viable offspring");
                    exit(1);
                }

                println!(
                    "Dead: {:3}\tReproducing: {:3}\tLiving Non-reproducing: {:3}",
                    config.getPopSize() - &population.borrow().getLivingIndices().len(),
                    reproducers.len(),
                    &population.borrow().getLivingIndices().len() - reproducers.len(),
                );

                population
                    .borrowMut()
                    .reproduceAsexually(&config, reproducers);

                unsafe { steps = 0 };
                unsafe { generation += 1 };
            }
        }
    }
}

pub fn computeMovements(config: &Config, threadpool: &mut Pool, pop: &mut Population) -> usize {
    let living = pop.getLivingIndices();

    struct Cheat<T> {
        ptr: *mut T,
    }

    unsafe impl Send for Cheat<Population> {}

    impl Clone for Cheat<Population> {
        fn clone(&self) -> Self {
            Self {
                ptr: self.ptr.clone(),
            }
        }
    }

    impl Copy for Cheat<Population> {}

    impl Deref for Cheat<Population> {
        type Target = Population;

        fn deref(&self) -> &Self::Target {
            unsafe { &*self.ptr }
        }
    }

    impl DerefMut for Cheat<Population> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { &mut *self.ptr }
        }
    }

    let pop_ptr: Cheat<Population> = Cheat {
        ptr: unsafe { std::mem::transmute(pop) },
    };

    let mut x = pop_ptr.clone();

    let results = x.getMutMoveQueue();

    let parts = {
        let threads = threadpool.thread_count();
        let (num, rem) = (
            living.len() / ((threads + 1) as usize),
            living.len() % ((threads + 1) as usize),
        );

        if rem != 0 {
            num + 1
        } else {
            num
        }
    };

    let mut chunks: Chunks<usize> = living.as_slice().chunks(parts);

    let mut resChunks: ChunksMut<(usize, (GridValueT, GridValueT))> = results.chunks_mut(parts);

    let gridWidth = config.getGridWidth();
    let gridHeight = config.getGridHeight();
    let stepsPerGen = config.getStepsPerGen();

    threadpool.scoped(|scope| {
        let localChunk = chunks.next().unwrap();
        let localResults = resChunks.next().unwrap();

        for chunk in chunks {
            let resChunk = resChunks.next().unwrap();
            scope.execute(move || {
                for (index, cellIndex) in chunk.into_iter().enumerate() {
                    let movement = unsafe { &mut *pop_ptr.ptr }.getCellMovementData(*cellIndex);
                    let neurons = unsafe { &mut *pop_ptr.ptr }.getCellMutNeuronData(*cellIndex);

                    let coords =
                        cell::oneStep((neurons, movement), gridWidth, gridHeight, stepsPerGen);
                    resChunk[index] = (*cellIndex, coords);
                }
            });
        }

        for (index, cellIndex) in localChunk.into_iter().enumerate() {
            let movement = unsafe { &mut *pop_ptr.ptr }.getCellMovementData(*cellIndex);
            let neurons = unsafe { &mut *pop_ptr.ptr }.getCellMutNeuronData(*cellIndex);

            let coords = cell::oneStep((neurons, movement), gridWidth, gridHeight, stepsPerGen);
            localResults[index] = (*cellIndex, coords);
        }
    });

    living.len()
}

pub fn determine_reproducers(config: &Config, pop: &Population) -> Vec<usize> {
    let mut reproducers = Vec::new();
    for cellIndex in pop.getLivingIndices() {
        let cell = pop.getCellMovementData(cellIndex);
        if cell.getCoords().0 < config.getGridWidth() / 4
            || cell.getCoords().0 > 3 * config.getGridWidth() / 4
        {
            reproducers.push(cellIndex);
        }
    }

    reproducers
}

pub fn determine_deaths(config: &Config, pop: &mut Population) {
    if unsafe { steps } == config.getStepsPerGen() / 4 {
        for index in &pop.getLivingIndices() {
            let (x, _) = pop.getCellMovementData(*index).getCoords();

            if x < config.getGridWidth() / 4 || x > (3 * config.getGridWidth()) / 4 {
                pop.addToDeathQueue(*index)
            }
        }
    } else if unsafe { steps } == config.getStepsPerGen() / 2 {
        for index in &pop.getLivingIndices() {
            let (x, _) = pop.getCellMovementData(*index).getCoords();

            if x > config.getGridWidth() / 4 && x < (3 * config.getGridWidth()) / 4 {
                pop.addToDeathQueue(*index)
            }
        }
    } else if unsafe { steps } == (3 * config.getStepsPerGen()) / 4 {
        for index in &pop.getLivingIndices() {
            let (x, _) = pop.getCellMovementData(*index).getCoords();

            if x < config.getGridWidth() / 4 || x > (3 * config.getGridWidth()) / 4 {
                pop.addToDeathQueue(*index)
            }
        }
    }

    if pop.getDeathQueueLen() > 0 {
        println!(
            "Step {} Killed: {}",
            unsafe { steps },
            pop.getDeathQueueLen()
        );
    }
}

mod DebugCell {
    use std::{
        cell::UnsafeCell,
        mem::transmute,
        ops::{Deref, DerefMut},
    };

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum RefType {
        Mutable,
        Immutable(usize),
        None,
    }

    pub struct Ref<'a, T> {
        reference: &'a T,
        #[cfg(debug_assertions)]
        counter: &'a mut RefType,
    }

    impl<'a, T> Deref for Ref<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.reference
        }
    }

    impl<'a, T> Drop for Ref<'a, T> {
        fn drop(&mut self) {
            #[cfg(debug_assertions)]
            if let RefType::Immutable(count) = self.counter {
                if *count == 0 {
                    panic!("Leaked memory!, invalid");
                } else if *count == 1 {
                    std::mem::drop(count);
                    *self.counter = RefType::None;
                    return;
                } else {
                    *count -= 1;
                    return;
                }
            }
            #[cfg(debug_assertions)]
            panic!("Invalid type");
        }
    }

    pub struct RefMut<'a, T> {
        reference: &'a mut T,
        #[cfg(debug_assertions)]
        counter: &'a mut RefType,
    }

    impl<'a, T> Deref for RefMut<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.reference
        }
    }

    impl<'a, T> DerefMut for RefMut<'a, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.reference
        }
    }

    impl<'a, T> Drop for RefMut<'a, T> {
        fn drop(&mut self) {
            #[cfg(debug_assertions)]
            if let RefType::Mutable = self.counter {
                *self.counter = RefType::None;
                return;
            }
            #[cfg(debug_assertions)]
            panic!("Invalid type");
        }
    }

    pub struct DebugRefCell<T> {
        interior: UnsafeCell<T>,
        #[cfg(debug_assertions)]
        refType: UnsafeCell<RefType>,
    }

    impl<T> DebugRefCell<T> {
        pub fn new(value: T) -> DebugRefCell<T> {
            #[cfg(debug_assertions)]
            return DebugRefCell {
                interior: UnsafeCell::new(value),
                refType: UnsafeCell::new(RefType::None),
            };
            #[cfg(not(debug_assertions))]
            return DebugRefCell {
                interior: UnsafeCell::new(value),
            };
        }

        pub fn borrow<'a, 'b>(&'b self) -> Ref<'a, T>
        where
            'b: 'a,
        {
            #[cfg(debug_assertions)]
            let kind = unsafe { *self.refType.get() };
            #[cfg(debug_assertions)]
            match kind {
                RefType::None => unsafe {
                    (*&mut *self.refType.get()) = RefType::Immutable(1);
                    let reference = transmute::<*mut T, &'a T>(self.interior.get());
                    Ref {
                        reference,
                        counter: &mut *self.refType.get(),
                    }
                },
                RefType::Immutable(count) => unsafe {
                    (*&mut *self.refType.get()) = RefType::Immutable(count.checked_add(1).unwrap());
                    let reference = transmute::<*mut T, &'a T>(self.interior.get());
                    Ref {
                        reference,
                        counter: &mut *self.refType.get(),
                    }
                },
                RefType::Mutable => {
                    panic!("Cannot borrow a immutable reference and a mutable reference at the same time");
                }
            }
            #[cfg(not(debug_assertions))]
            unsafe {
                Ref {
                    reference: transmute::<*mut T, &'a T>(self.interior.get()),
                }
            }
        }

        pub fn borrowMut<'a, 'b>(&'b self) -> RefMut<'a, T>
        where
            'b: 'a,
        {
            #[cfg(debug_assertions)]
            let kind = unsafe { *self.refType.get() };
            #[cfg(debug_assertions)]
            match kind {
                RefType::None => unsafe {
                    (*&mut *self.refType.get()) = RefType::Mutable;
                    let reference = transmute::<*mut T, &'a mut T>(self.interior.get());
                    RefMut {
                        reference,
                        counter: &mut *self.refType.get(),
                    }
                },
                RefType::Immutable(_) => panic!(
                    "Cannot borrow a immutable reference and a mutable reference at the same time"
                ),
                RefType::Mutable => panic!("Cannot borrow multiple mutable references"),
            }
            #[cfg(not(debug_assertions))]
            unsafe {
                RefMut {
                    reference: transmute::<*const T, &'a mut T>(self.interior.get()),
                }
            }
        }

        #[cfg(debug_assertions)]
        fn getRefType(&self) -> RefType {
            unsafe { *self.refType.get() }
        }
    }

    #[test]
    #[cfg(debug_assertions)]
    fn refDropTest() {
        let cell = DebugRefCell::new(0 as usize);

        let x = cell.borrow();
        let y = cell.borrow();
        let z = cell.borrow();

        assert_eq!(cell.getRefType(), RefType::Immutable(3));

        std::mem::drop(x);

        assert_eq!(cell.getRefType(), RefType::Immutable(2));

        std::mem::drop(z);

        assert_eq!(cell.getRefType(), RefType::Immutable(1));

        std::mem::drop(y);

        assert_eq!(cell.getRefType(), RefType::None);
    }

    #[test]
    #[cfg(debug_assertions)]
    fn refMutDropTest() {
        let cell = DebugRefCell::new(0 as usize);

        let x = cell.borrowMut();

        assert_eq!(cell.getRefType(), RefType::Mutable);

        std::mem::drop(x);
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn refImmAliasTest() {
        let cell = DebugRefCell::new(0 as usize);

        let x = cell.borrow();
        let y = cell.borrow();

        cell.borrowMut();
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn refMutAliasTest() {
        let cell = DebugRefCell::new(0 as usize);

        let z = cell.borrowMut();

        let x = cell.borrow();
        let y = cell.borrow();
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn refMultiMut() {
        let cell = DebugRefCell::new(0 as usize);

        let z = cell.borrowMut();

        let x = cell.borrowMut();
    }
}
