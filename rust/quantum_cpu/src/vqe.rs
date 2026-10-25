// external imports
use std::f64::consts::PI;

// internal imports
use quantum_core::StateVector;

pub struct VqeResult {
    pub theta: f64,
    pub energy: f64,
}

pub fn energy(theta: f64) -> f64 {
    let mut state = StateVector::zero(1);

    state.rotation_y(0, theta);

    state.expectation_z(0)
}

pub fn optimize(steps: usize) -> VqeResult {
    assert!(steps > 0, "steps must be positive");

    let mut best = VqeResult {
        theta: 0.0,
        energy: f64::INFINITY,
    };

    for step in 0..=steps {
        let theta = 2.0 * PI * step as f64 / steps as f64;

        let current_energy = energy(theta);

        if current_energy < best.energy {
            best = VqeResult {
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
) -> VqeResult {
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

    VqeResult {
        theta,
        energy: energy(theta),
    }
}

#[cfg(test)]
#[path = "tests/vqe.rs"]
mod tests;
