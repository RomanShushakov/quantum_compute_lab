# Vulkan GPU quantum experiment

The original `quantum_gpu` binary is the Vulkan device / Hadamard smoke test.
The `bathe_gpu` binary compares the same 2-qubit Bathe VQE ansatz with the
`quantum_cpu` reference and runs exploratory Hadamard timings.

Compile WGSL to SPIR-V with Naga CLI (v30 used during development):

```sh
cd rust
for gate in transform hadamard ry cnot; do
  naga quantum_gpu/shaders/$gate.wgsl quantum_gpu/shaders/$gate.spv
  naga quantum_gpu/shaders/$gate.wgsl
done
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p quantum_gpu --bin quantum_gpu
cargo run --release -p quantum_gpu --bin bathe_gpu
```

GPU state is interleaved complex f32, whereas CPU state is Complex64.
The Bathe reference Hamiltonian and VQE optimizer are imported from quantum_cpu.
The GPU executes the CPU-optimized angles, not a separate GPU optimizer.
The benchmark includes pipeline creation, command submission and queue wait;
it does not measure isolated shader execution or QPU hardware runtime.
The GPU path currently synchronizes after every gate; future performance
work should reuse pipelines and batch command submission.

Do not report timings or energy values as validated until the commands above
run successfully on the target Vulkan device.
