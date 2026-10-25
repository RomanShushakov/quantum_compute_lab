// external imports
use rand::{SeedableRng, rngs::StdRng};

// mods
mod bernstein_vazirani;

// internal imports
use quantum_core::StateVector;

fn print_state(label: &str, state: &StateVector) {
    println!("\n{label}");

    let probabilities = state.probabilities();

    for (index, amplitude) in state.amplitudes.iter().enumerate() {
        println!(
            "|{:0width$b}>: amplitude = {:.6} {:+.6}i, probability = {:.6}",
            index,
            amplitude.re,
            amplitude.im,
            probabilities[index],
            width = state.num_qubits,
        );
    }
}

fn print_measurements(state: &StateVector, shots: usize) {
    let mut rng = StdRng::seed_from_u64(42);
    let mut counts = vec![0usize; state.amplitudes.len()];

    for _ in 0..shots {
        let outcome = state.sample(&mut rng);
        counts[outcome] += 1;
    }

    println!("\nMeasurement results ({shots} shots):");

    for (index, count) in counts.iter().enumerate() {
        println!("|{:0width$b}>: {}", index, count, width = state.num_qubits,);
    }
}

fn main() {
    let mut state = StateVector::zero(2);

    print_state("Initial state", &state);

    state.hadamard(0);
    print_state("After Hadamard on qubit 0", &state);

    state.cnot(0, 1);
    print_state("After CNOT (control=0, target=1)", &state);

    print_measurements(&state, 1000);

    let secret = 0b1011;
    let num_input_qubits = 4;

    let bv_state = bernstein_vazirani::prepare_state(secret, num_input_qubits);

    let mut rng = StdRng::seed_from_u64(42);
    let mut counts = vec![0usize; 1 << num_input_qubits];

    for _ in 0..1000 {
        let outcome = bv_state.sample(&mut rng);

        // Discard the auxiliary qubit from the measurement result.
        let measured_secret = outcome & ((1 << num_input_qubits) - 1);

        counts[measured_secret] += 1;
    }

    println!("\nBernstein-Vazirani experiment");
    println!("Secret: {secret:04b}");

    for (measured_secret, count) in counts.iter().enumerate() {
        if *count > 0 {
            println!("{measured_secret:04b}: {count}");
        }
    }
}
