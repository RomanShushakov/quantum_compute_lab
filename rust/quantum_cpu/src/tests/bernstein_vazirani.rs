use super::*;

const EPSILON: f64 = 1e-12;

#[test]
fn recovers_secret_1011() {
    let secret = 0b1011;
    let state = prepare_state(secret, 4);
    let probabilities = state.probabilities();

    assert_eq!(state.num_qubits, 5);

    for (index, probability) in probabilities.iter().enumerate() {
        let input_bits = index & 0b1111;

        if input_bits == secret {
            assert!((*probability - 0.5).abs() < EPSILON);
        } else {
            assert!(probability.abs() < EPSILON);
        }
    }
}

#[test]
fn recovers_all_four_bit_secrets() {
    for secret in 0..16 {
        let state = prepare_state(secret, 4);

        for (index, probability) in state.probabilities().iter().enumerate() {
            let input_bits = index & 0b1111;

            if input_bits != secret {
                assert!(probability.abs() < EPSILON);
            }
        }
    }
}
