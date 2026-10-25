// external imports
use rand::{SeedableRng, rngs::StdRng};

// mods
mod bernstein_vazirani;
mod jacobi;
mod vqe;
mod vqe_interacting;
mod vqe_two_qubit;

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

    let grid_result = vqe::optimize(100);
    let gradient_result = vqe::optimize_gradient_descent(1.0, 0.2, 100);

    println!("\nVQE experiment");
    println!("Hamiltonian: Z");

    println!("\nGrid search");
    println!("Optimal theta: {:.6}", grid_result.theta);
    println!("Minimum energy: {:.6}", grid_result.energy);

    println!("\nGradient descent (parameter shift)");
    println!("Optimal theta: {:.6}", gradient_result.theta);
    println!("Minimum energy: {:.6}", gradient_result.energy);

    let result = vqe_two_qubit::optimize(20);

    println!("\nTwo-qubit VQE experiment");
    println!("Hamiltonian: Z0 + Z1 + Z0Z1");
    println!("Optimal theta_0: {:.6}", result.theta_0);
    println!("Optimal theta_1: {:.6}", result.theta_1);
    println!("Minimum energy: {:.6}", result.energy);

    let interacting_result = vqe_interacting::optimize(10_000);

    println!();
    println!("Interacting two-qubit VQE experiment");
    println!("Hamiltonian: Z0 + Z1 + 0.5 X0X1");
    println!("Optimal theta: {:.6}", interacting_result.theta);
    println!("Minimum energy: {:.9}", interacting_result.energy);
    println!("Exact ground energy: {:.9}", -(17.0_f64).sqrt() / 2.0);

    println!();
    println!("Generalized Jacobi eigenvalue experiment");

    // H = Z0 + Z1 + 0.5 X0X1
    // Shift by 3I to obtain a positive-definite stiffness matrix.
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

    let vqe_jacobi_result = jacobi::generalized_jacobi(&stiffness, &mass, 1e-12, 20)
        .expect("Jacobi eigensolver failed");

    println!("Converged: {}", vqe_jacobi_result.converged);
    println!("Sweeps: {}", vqe_jacobi_result.sweeps);

    for (index, eigenvalue) in vqe_jacobi_result.eigenvalues.iter().enumerate() {
        println!(
            "Eigenvalue {}: {:.9} (original H: {:.9})",
            index + 1,
            eigenvalue,
            eigenvalue - 3.0
        );
    }

    println!("Ground-state eigenvector:");
    for row in &vqe_jacobi_result.eigenvectors {
        println!("{:.9}", row[0]);
    }

    println!();

    let ground_state: Vec<f64> = vqe_jacobi_result
        .eigenvectors
        .iter()
        .map(|row| row[0])
        .collect();

    let optimal_theta = std::f64::consts::PI + 0.25_f64.atan();

    let fidelity = vqe_interacting::ground_state_fidelity(optimal_theta, &ground_state);

    println!("VQE/Jacobi ground-state fidelity: {:.12}", fidelity);

    let optimized_fidelity =
        vqe_interacting::ground_state_fidelity(interacting_result.theta, &ground_state);

    println!(
        "Grid-optimized VQE/Jacobi fidelity: {:.12}",
        optimized_fidelity
    );

    println!();

    println!("Bathe Exercise 11.1 - Generalized eigenproblem");

    let stiffness = vec![
        vec![6.0, -1.0, 0.0],
        vec![-1.0, 4.0, -1.0],
        vec![0.0, -1.0, 2.0],
    ];

    let mass = vec![
        vec![2.0, 0.0, 0.0],
        vec![0.0, 2.0, 1.0],
        vec![0.0, 1.0, 1.0],
    ];

    let result = jacobi::generalized_jacobi(&stiffness, &mass, 1e-12, 30)
        .expect("Bathe eigenproblem failed");

    let residuals = jacobi::eigenpair_residuals(&stiffness, &mass, &result);
    let gram = jacobi::mass_gram_matrix(&mass, &result);

    println!("Converged: {}", result.converged);
    println!("Sweeps: {}", result.sweeps);

    for (i, (&lambda, &residual)) in result.eigenvalues.iter().zip(residuals.iter()).enumerate() {
        println!(
            "Eigenpair {}: lambda = {:.9}, residual = {:.3e}",
            i + 1,
            lambda,
            residual
        );
    }

    println!("Mass Gram matrix (Phi^T M Phi):");

    for row in &gram {
        for value in row {
            print!("{value:>14.9} ");
        }
        println!();
    }
}
