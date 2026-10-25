use crate::jacobi::generalized_jacobi;

use super::*;

const TOLERANCE: f64 = 1e-10;

#[test]
fn zero_rotation_has_energy_two() {
    assert!((energy(0.0) - 2.0).abs() < TOLERANCE);
}

#[test]
fn pi_rotation_has_energy_minus_two() {
    assert!((energy(PI) + 2.0).abs() < TOLERANCE);
}

#[test]
fn half_pi_rotation_has_energy_half() {
    assert!((energy(PI / 2.0) - 0.5).abs() < TOLERANCE);
}

#[test]
fn energy_matches_analytical_expression() {
    let theta = 1.2345f64;
    let expected = 2.0 * theta.cos() + 0.5 * theta.sin();

    assert!((energy(theta) - expected).abs() < TOLERANCE);
}

#[test]
fn optimal_angle_matches_exact_ground_energy() {
    let theta = PI + 0.25_f64.atan();
    let expected = -(17.0_f64).sqrt() / 2.0;

    assert!((energy(theta) - expected).abs() < TOLERANCE);
}

#[test]
fn grid_search_finds_ground_energy() {
    let result = optimize(10_000);
    let expected = -(17.0_f64).sqrt() / 2.0;

    assert!((result.energy - expected).abs() < 1e-6);
}

#[test]
#[should_panic(expected = "steps must be positive")]
fn grid_search_rejects_zero_steps() {
    optimize(0);
}

#[test]
fn parameter_shift_matches_analytical_gradient() {
    let angles = [0.0, 0.3, 1.0, PI / 2.0, PI, 4.7];

    for theta in angles {
        let expected = -2.0 * theta.sin() + 0.5 * theta.cos();
        let actual = parameter_shift_gradient(theta);

        assert!((actual - expected).abs() < 1e-12);
    }
}

#[test]
fn parameter_shift_gradient_at_minimum_is_zero() {
    let theta = PI + 0.25_f64.atan();
    assert!(parameter_shift_gradient(theta).abs() < 1e-12);
}

#[test]
fn gradient_descent_finds_ground_state_energy() {
    let result = optimize_gradient_descent(3.0, 0.2, 100);
    let expected_theta = PI + 0.25_f64.atan();
    let expected_energy = -(17.0_f64).sqrt() / 2.0;

    assert!((result.theta - expected_theta).abs() < 1e-8);
    assert!((result.energy - expected_energy).abs() < 1e-12);
}

#[test]
fn gradient_descent_reduces_energy() {
    let initial_theta = 3.0;
    let result = optimize_gradient_descent(initial_theta, 0.2, 10);

    assert!(result.energy < energy(initial_theta));
}

#[test]
fn gradient_descent_matches_grid_search() {
    let grid_result = optimize(10_000);
    let gradient_result = optimize_gradient_descent(3.0, 0.2, 100);

    assert!((grid_result.energy - gradient_result.energy).abs() < 1e-7);
}

#[test]
fn vqe_ground_state_matches_jacobi_eigenvector() {
    let stiffness = vec![
        vec![5.0, 0.0, 0.0, 0.5],
        vec![0.0, 3.0, 0.5, 0.0],
        vec![0.0, 0.5, 3.0, 0.0],
        vec![0.5, 0.0, 0.0, 1.0],
    ];

    let mass = vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![0.0, 0.0, 1.0, 0.0],
        vec![0.0, 0.0, 0.0, 1.0],
    ];

    let result = generalized_jacobi(&stiffness, &mass, 1e-12, 20).expect("Jacobi solver failed");

    assert!(result.converged);

    let ground_state: Vec<f64> = result.eigenvectors.iter().map(|row| row[0]).collect();

    let theta = std::f64::consts::PI + 0.25_f64.atan();

    let fidelity = ground_state_fidelity(theta, &ground_state);

    assert!(
        (fidelity - 1.0).abs() < 1e-10,
        "Ground-state fidelity too low: {fidelity}"
    );
}
