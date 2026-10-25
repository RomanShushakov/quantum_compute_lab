// external imports
use std::f64::consts::PI;

// internal imports
use quantum_core::StateVector;

pub struct VqeInteractingResult {
    pub theta: f64,
    pub energy: f64,
}

pub fn energy(theta: f64) -> f64 {
    let mut state = StateVector::zero(2);

    state.rotation_y(0, theta);
    state.cnot(0, 1);

    state.expectation_z(0) + state.expectation_z(1) + 0.5 * state.expectation_xx(0, 1)
}

pub fn optimize(steps: usize) -> VqeInteractingResult {
    assert!(steps > 0, "steps must be positive");

    let mut best = VqeInteractingResult {
        theta: 0.0,
        energy: energy(0.0),
    };

    for step in 1..=steps {
        let theta = 2.0 * PI * step as f64 / steps as f64;
        let current_energy = energy(theta);

        if current_energy < best.energy {
            best = VqeInteractingResult {
                theta,
                energy: current_energy,
            };
        }
    }

    best
}

pub fn parameter_shift_gradient(theta: f64) -> f64 {
    let shift = std::f64::consts::FRAC_PI_2;

    let energy_plus = energy(theta + shift);
    let energy_minus = energy(theta - shift);

    (energy_plus - energy_minus) / 2.0
}

pub fn optimize_gradient_descent(
    initial_theta: f64,
    learning_rate: f64,
    iterations: usize,
) -> VqeInteractingResult {
    assert!(initial_theta.is_finite(), "initial_theta must be finite");
    assert!(
        learning_rate.is_finite() && learning_rate > 0.0,
        "learning_rate must be finite and positive"
    );
    assert!(iterations > 0, "iterations must be positive");

    let mut theta = initial_theta;

    for _ in 0..iterations {
        let gradient = parameter_shift_gradient(theta);
        theta -= learning_rate * gradient;
    }

    VqeInteractingResult {
        theta,
        energy: energy(theta),
    }
}

pub fn ground_state_fidelity(theta: f64, eigenvector: &[f64]) -> f64 {
    assert_eq!(eigenvector.len(), 4);

    let amplitudes = [(theta / 2.0).cos(), 0.0, 0.0, (theta / 2.0).sin()];

    let overlap: f64 = amplitudes
        .iter()
        .zip(eigenvector.iter())
        .map(|(a, b)| a * b)
        .sum();

    overlap * overlap
}

#[cfg(test)]
#[path = "tests/vqe_interacting.rs"]
mod tests;
