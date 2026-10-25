// external imports
use num_complex::Complex64;
use rand::SeedableRng;
use rand::rngs::StdRng;

// internal import
use crate::Qubit;
use crate::StateVector;

const EPSILON: f64 = 1e-12;

fn approx_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON
}

#[test]
fn initial_state_is_zero() {
    let qubit = Qubit::zero();

    assert_eq!(qubit.probabilities(), [1.0, 0.0]);
}

#[test]
fn hadamard_creates_equal_probabilities() {
    let mut qubit = Qubit::zero();

    qubit.hadamard();

    let probabilities = qubit.probabilities();

    assert!(approx_equal(probabilities[0], 0.5));
    assert!(approx_equal(probabilities[1], 0.5));
}

#[test]
fn hadamard_twice_is_identity() {
    let mut qubit = Qubit::zero();

    qubit.hadamard();
    qubit.hadamard();

    assert!(approx_equal(qubit.amplitudes[0].re, 1.0));
    assert!(approx_equal(qubit.amplitudes[0].im, 0.0));
    assert!(approx_equal(qubit.amplitudes[1].re, 0.0));
    assert!(approx_equal(qubit.amplitudes[1].im, 0.0));
}

#[test]
fn state_remains_normalized() {
    let mut qubit = Qubit::zero();

    for _ in 0..100 {
        qubit.hadamard();

        let total: f64 = qubit.probabilities().iter().sum();

        assert!(approx_equal(total, 1.0));
    }
}

#[test]
fn two_qubit_initial_state() {
    let state = StateVector::zero(2);

    assert_eq!(state.num_qubits, 2);
    assert_eq!(state.amplitudes.len(), 4);

    assert_eq!(state.amplitudes[0], Complex64::new(1.0, 0.0));

    for amplitude in &state.amplitudes[1..] {
        assert_eq!(*amplitude, Complex64::new(0.0, 0.0));
    }
}

#[test]
fn three_qubit_initial_state() {
    let state = StateVector::zero(3);

    assert_eq!(state.amplitudes.len(), 8);
    assert_eq!(
        state.probabilities(),
        vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    );
}

#[test]
fn initial_state_is_normalized() {
    for num_qubits in 1..=5 {
        let state = StateVector::zero(num_qubits);
        let total_probability: f64 = state.probabilities().iter().sum();

        assert!((total_probability - 1.0).abs() < 1e-12);
    }
}

#[test]
fn hadamard_on_qubit_zero() {
    let mut state = StateVector::zero(2);
    state.hadamard(0);

    let probabilities = state.probabilities();

    assert!((probabilities[0] - 0.5).abs() < 1e-12);
    assert!((probabilities[1] - 0.5).abs() < 1e-12);
    assert!(probabilities[2].abs() < 1e-12);
    assert!(probabilities[3].abs() < 1e-12);
}

#[test]
fn hadamard_on_qubit_one() {
    let mut state = StateVector::zero(2);
    state.hadamard(1);

    let probabilities = state.probabilities();

    assert!((probabilities[0] - 0.5).abs() < 1e-12);
    assert!(probabilities[1].abs() < 1e-12);
    assert!((probabilities[2] - 0.5).abs() < 1e-12);
    assert!(probabilities[3].abs() < 1e-12);
}

#[test]
fn multi_qubit_hadamard_twice_is_identity() {
    for target in 0..3 {
        let mut state = StateVector::zero(3);

        state.hadamard(target);
        state.hadamard(target);

        for (index, amplitude) in state.amplitudes.iter().enumerate() {
            let expected = if index == 0 { 1.0 } else { 0.0 };

            assert!((amplitude.re - expected).abs() < 1e-12);
            assert!(amplitude.im.abs() < 1e-12);
        }
    }
}

#[test]
fn cnot_does_not_flip_when_control_is_zero() {
    let mut state = StateVector::zero(2);

    state.cnot(0, 1);

    assert_eq!(state.probabilities(), vec![1.0, 0.0, 0.0, 0.0]);
}

#[test]
fn cnot_flips_target_when_control_is_one() {
    let mut state = StateVector::zero(2);

    state.hadamard(0);
    state.cnot(0, 1);

    let probabilities = state.probabilities();

    assert!((probabilities[0] - 0.5).abs() < 1e-12);
    assert!(probabilities[1].abs() < 1e-12);
    assert!(probabilities[2].abs() < 1e-12);
    assert!((probabilities[3] - 0.5).abs() < 1e-12);
}

#[test]
fn cnot_twice_is_identity() {
    let mut state = StateVector::zero(2);

    state.hadamard(0);
    let before = state.amplitudes.clone();

    state.cnot(0, 1);
    state.cnot(0, 1);

    for (actual, expected) in state.amplitudes.iter().zip(before.iter()) {
        assert!((*actual - *expected).norm() < 1e-12);
    }
}

#[test]
fn pauli_x_flips_qubit_zero() {
    let mut state = StateVector::zero(2);

    state.pauli_x(0);

    assert_eq!(state.probabilities(), vec![0.0, 1.0, 0.0, 0.0]);
}

#[test]
fn pauli_x_flips_qubit_one() {
    let mut state = StateVector::zero(2);

    state.pauli_x(1);

    assert_eq!(state.probabilities(), vec![0.0, 0.0, 1.0, 0.0]);
}

#[test]
fn pauli_z_changes_phase_not_probabilities() {
    let mut state = StateVector::zero(1);

    state.hadamard(0);

    let before = state.probabilities();
    let positive_amplitude = state.amplitudes[1];

    state.pauli_z(0);

    assert_eq!(state.probabilities(), before);
    assert_eq!(state.amplitudes[1], -positive_amplitude);
}

#[test]
fn hadamard_reveals_phase_difference() {
    let mut state = StateVector::zero(1);

    state.hadamard(0);
    state.pauli_z(0);
    state.hadamard(0);

    let probabilities = state.probabilities();

    assert!(probabilities[0] < 1e-12);
    assert!((probabilities[1] - 1.0).abs() < 1e-12);
}

#[test]
fn sampling_zero_state_always_returns_zero() {
    let state = StateVector::zero(3);
    let mut rng = StdRng::seed_from_u64(42);

    for _ in 0..1000 {
        assert_eq!(state.sample(&mut rng), 0);
    }
}

#[test]
fn sampling_bell_state_returns_only_valid_outcomes() {
    let mut state = StateVector::zero(2);
    state.hadamard(0);
    state.cnot(0, 1);

    let mut rng = StdRng::seed_from_u64(42);
    let mut counts = [0usize; 4];

    for _ in 0..1000 {
        counts[state.sample(&mut rng)] += 1;
    }

    assert_eq!(counts[1], 0);
    assert_eq!(counts[2], 0);

    assert!((counts[0] as i64 - 500).abs() < 100);
    assert!((counts[3] as i64 - 500).abs() < 100);
}

#[test]
fn rotation_y_zero_is_identity() {
    let mut state = StateVector::zero(1);

    state.rotation_y(0, 0.0);

    assert!((state.probabilities()[0] - 1.0).abs() < EPSILON);
    assert!(state.probabilities()[1].abs() < EPSILON);
}

#[test]
fn rotation_y_pi_flips_zero_to_one() {
    let mut state = StateVector::zero(1);

    state.rotation_y(0, std::f64::consts::PI);

    let probabilities = state.probabilities();

    assert!(probabilities[0].abs() < EPSILON);
    assert!((probabilities[1] - 1.0).abs() < EPSILON);
}

#[test]
fn rotation_y_half_pi_creates_equal_probabilities() {
    let mut state = StateVector::zero(1);

    state.rotation_y(0, std::f64::consts::FRAC_PI_2);

    let probabilities = state.probabilities();

    assert!((probabilities[0] - 0.5).abs() < EPSILON);
    assert!((probabilities[1] - 0.5).abs() < EPSILON);
}

#[test]
fn rotation_y_preserves_normalization() {
    let mut state = StateVector::zero(2);

    state.hadamard(0);
    state.cnot(0, 1);

    state.rotation_y(1, 0.7);

    let total: f64 = state.probabilities().iter().sum();

    assert!((total - 1.0).abs() < EPSILON);
}

#[test]
fn rotation_y_inverse_restores_state() {
    let mut state = StateVector::zero(1);

    state.rotation_y(0, 0.73);
    state.rotation_y(0, -0.73);

    assert!((state.probabilities()[0] - 1.0).abs() < EPSILON);
    assert!(state.probabilities()[1].abs() < EPSILON);
}

#[test]
fn expectation_z_of_zero_state_is_one() {
    let state = StateVector::zero(1);

    assert!((state.expectation_z(0) - 1.0).abs() < EPSILON);
}

#[test]
fn expectation_z_of_one_state_is_minus_one() {
    let mut state = StateVector::zero(1);
    state.pauli_x(0);

    assert!((state.expectation_z(0) + 1.0).abs() < EPSILON);
}

#[test]
fn expectation_z_after_hadamard_is_zero() {
    let mut state = StateVector::zero(1);
    state.hadamard(0);

    assert!(state.expectation_z(0).abs() < EPSILON);
}

#[test]
fn expectation_z_matches_rotation_y_cosine() {
    let angles = [
        0.0,
        std::f64::consts::FRAC_PI_4,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
        1.7,
    ];

    for theta in angles {
        let mut state = StateVector::zero(1);
        state.rotation_y(0, theta);

        let expected = theta.cos();
        let actual = state.expectation_z(0);

        assert!((actual - expected).abs() < EPSILON);
    }
}

#[test]
fn expectation_zz_of_zero_state_is_one() {
    let state = StateVector::zero(2);

    assert!((state.expectation_zz(0, 1) - 1.0).abs() < EPSILON);
}

#[test]
fn expectation_zz_of_opposite_bits_is_minus_one() {
    let mut state = StateVector::zero(2);
    state.pauli_x(0);

    assert!((state.expectation_zz(0, 1) + 1.0).abs() < EPSILON);
}

#[test]
fn expectation_zz_of_bell_state_is_one() {
    let mut state = StateVector::zero(2);
    state.hadamard(0);
    state.cnot(0, 1);

    assert!((state.expectation_zz(0, 1) - 1.0).abs() < EPSILON);
}

#[test]
fn expectation_zz_of_uniform_superposition_is_zero() {
    let mut state = StateVector::zero(2);
    state.hadamard(0);
    state.hadamard(1);

    assert!(state.expectation_zz(0, 1).abs() < EPSILON);
}

#[test]
fn expectation_xx_of_zero_state_is_zero() {
    let state = StateVector::zero(2);

    assert!(state.expectation_xx(0, 1).abs() < EPSILON);
}

#[test]
fn expectation_xx_of_bell_state_is_one() {
    let mut state = StateVector::zero(2);
    state.hadamard(0);
    state.cnot(0, 1);

    assert!((state.expectation_xx(0, 1) - 1.0).abs() < EPSILON);
}

#[test]
fn expectation_xx_of_negative_bell_state_is_minus_one() {
    let mut state = StateVector::zero(2);
    state.hadamard(0);
    state.cnot(0, 1);
    state.pauli_z(0);

    assert!((state.expectation_xx(0, 1) + 1.0).abs() < EPSILON);
}

#[test]
fn expectation_xx_of_uniform_superposition_is_one() {
    let mut state = StateVector::zero(2);
    state.hadamard(0);
    state.hadamard(1);

    assert!((state.expectation_xx(0, 1) - 1.0).abs() < EPSILON);
}
