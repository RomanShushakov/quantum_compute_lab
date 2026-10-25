use super::*;

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
        PI,
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
