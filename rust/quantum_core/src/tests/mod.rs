// external imports
use num_complex::Complex64;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::f64::consts::PI;

// internal import
use crate::Qubit;
use crate::StateVector;

const EPSILON: f64 = 1e-12;

fn approx_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON
}

mod gates;
mod measurements;
mod statevector;
