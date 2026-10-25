use super::*;

#[test]
fn initial_state_is_zero() {
    let qubit = Qubit::zero();

    assert_eq!(qubit.probabilities(), [1.0, 0.0]);
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
