// internal imports
use quantum_core::StateVector;

pub fn prepare_state(secret: usize, num_input_qubits: usize) -> StateVector {
    assert!(num_input_qubits > 0);
    assert!(num_input_qubits < usize::BITS as usize - 1);
    assert!(secret < (1usize << num_input_qubits));

    let auxiliary = num_input_qubits;
    let mut state = StateVector::zero(num_input_qubits + 1);

    // Prepare the auxiliary qubit in |1>.
    state.pauli_x(auxiliary);

    // Prepare the input qubits in |+> and auxiliary in |->.
    for qubit in 0..=auxiliary {
        state.hadamard(qubit);
    }

    // Oracle: f_s(x) = s . x mod 2.
    for qubit in 0..num_input_qubits {
        if secret & (1usize << qubit) != 0 {
            state.cnot(qubit, auxiliary);
        }
    }

    // Decode the secret using interference.
    for qubit in 0..num_input_qubits {
        state.hadamard(qubit);
    }

    state
}

#[cfg(test)]
#[path = "tests/bernstein_vazirani.rs"]
mod tests;
