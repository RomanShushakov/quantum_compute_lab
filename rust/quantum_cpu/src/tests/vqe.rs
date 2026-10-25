use std::f64::consts::PI;

use super::*;

const EPSILON: f64 = 1e-12;

#[test]
fn energy_matches_cosine() {
    let angles = [0.0, std::f64::consts::FRAC_PI_2, PI, 1.3];

    for theta in angles {
        assert!((energy(theta) - theta.cos()).abs() < EPSILON);
    }
}

#[test]
fn optimizer_finds_ground_state_energy() {
    let result = optimize(100);

    assert!((result.energy + 1.0).abs() < EPSILON);
    assert!((result.theta - PI).abs() < EPSILON);
}

#[test]
fn optimizer_finds_approximate_minimum_with_coarse_grid() {
    let result = optimize(21);

    assert!(result.energy < -0.98);
}

#[test]
fn parameter_shift_matches_analytical_gradient() {
    let angles = [0.0, 0.3, 1.0, std::f64::consts::FRAC_PI_2, PI, 4.7];

    for theta in angles {
        let expected = -theta.sin();
        let actual = parameter_shift_gradient(theta);

        assert!((actual - expected).abs() < EPSILON);
    }
}

#[test]
fn parameter_shift_gradient_at_minimum_is_zero() {
    let theta = PI;

    let gradient = parameter_shift_gradient(theta);

    assert!(gradient.abs() < EPSILON);
}

#[test]
fn gradient_descent_finds_ground_state_energy() {
    let result = optimize_gradient_descent(1.0, 0.2, 100);

    assert!((result.energy + 1.0).abs() < 1e-10);
    assert!((result.theta - PI).abs() < 1e-5);
}

#[test]
fn gradient_descent_reduces_energy() {
    let initial_energy = energy(1.0);

    let result = optimize_gradient_descent(1.0, 0.2, 10);

    assert!(result.energy < initial_energy);
}

#[test]
fn gradient_descent_matches_grid_search() {
    let grid_result = optimize(100);
    let gradient_result = optimize_gradient_descent(1.0, 0.2, 100);

    assert!((grid_result.energy - gradient_result.energy).abs() < 1e-10);
}
