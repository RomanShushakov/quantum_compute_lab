use quantum_core::Qubit;

fn main() {
    let mut qubit = Qubit::zero();

    println!("Initial state:");
    print_state(&qubit);

    qubit.hadamard();

    println!("\nAfter Hadamard:");
    print_state(&qubit);

    qubit.hadamard();

    println!("\nAfter second Hadamard:");
    print_state(&qubit);
}

fn print_state(qubit: &Qubit) {
    for (index, amplitude) in qubit.amplitudes.iter().enumerate() {
        println!("|{index}>: {amplitude}");
    }

    let probabilities = qubit.probabilities();

    println!("P(0) = {:.6}", probabilities[0]);
    println!("P(1) = {:.6}", probabilities[1]);
}
