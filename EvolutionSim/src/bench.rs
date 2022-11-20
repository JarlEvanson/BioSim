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

use super::Config;
use super::ConfigBase;

#[bench]
fn benchStep(b: &mut Bencher) {}
