// external imports
use std::f64::consts::PI;

// internal imports
use quantum_core::StateVector;

pub fn energy(theta_0: f64, theta_1: f64) -> f64 {
    let mut state = StateVector::zero(2);

    state.rotation_y(0, theta_0);
    state.rotation_y(1, theta_1);

    state.expectation_z(0) + state.expectation_z(1) + state.expectation_zz(0, 1)
}

pub struct VqeTwoQubitResult {
    pub theta_0: f64,
    pub theta_1: f64,
    pub energy: f64,
}

pub fn optimize(steps: usize) -> VqeTwoQubitResult {
    assert!(steps > 0, "steps must be positive");

    let mut best = VqeTwoQubitResult {
        theta_0: 0.0,
        theta_1: 0.0,
        energy: f64::INFINITY,
    };

    for step_0 in 0..=steps {
        let theta_0 = 2.0 * PI * step_0 as f64 / steps as f64;

        for step_1 in 0..=steps {
            let theta_1 = 2.0 * PI * step_1 as f64 / steps as f64;

            let current_energy = energy(theta_0, theta_1);

            if current_energy < best.energy {
                best = VqeTwoQubitResult {
                    theta_0,
                    theta_1,
                    energy: current_energy,
                };
            }
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-12;

    #[test]
    fn zero_angles_give_energy_three() {
        assert!((energy(0.0, 0.0) - 3.0).abs() < EPSILON);
    }

    #[test]
    fn pi_rotation_gives_ground_state_energy() {
        assert!((energy(PI, 0.0) + 1.0).abs() < EPSILON);
    }

    #[test]
    fn both_pi_rotations_give_ground_state_energy() {
        assert!((energy(PI, PI) + 1.0).abs() < EPSILON);
    }

    #[test]
    fn optimizer_finds_ground_state_energy() {
        let result = optimize(20);

        assert!((result.energy + 1.0).abs() < EPSILON);
    }
}
