use super::*;

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
fn rotation_y_zero_is_identity() {
    let mut state = StateVector::zero(1);

    state.rotation_y(0, 0.0);

    assert!((state.probabilities()[0] - 1.0).abs() < EPSILON);
    assert!(state.probabilities()[1].abs() < EPSILON);
}

#[test]
fn rotation_y_pi_flips_zero_to_one() {
    let mut state = StateVector::zero(1);

    state.rotation_y(0, PI);

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
