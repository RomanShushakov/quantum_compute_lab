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
mod tests {
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

        let result =
            generalized_jacobi(&stiffness, &mass, 1e-12, 20).expect("Jacobi solver failed");

        assert!(result.converged);

        let ground_state: Vec<f64> = result.eigenvectors.iter().map(|row| row[0]).collect();

        let theta = std::f64::consts::PI + 0.25_f64.atan();

        let fidelity = ground_state_fidelity(theta, &ground_state);

        assert!(
            (fidelity - 1.0).abs() < 1e-10,
            "Ground-state fidelity too low: {fidelity}"
        );
    }
}
