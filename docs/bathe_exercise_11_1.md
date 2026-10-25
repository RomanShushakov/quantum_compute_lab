# Bathe Exercise 11.1 - reproducible iteration reference

Source: K. J. Bathe, *Finite Element Procedures*, 2nd edition,
Chapter 11, Section 11.2.7, Exercise 11.1, printed page 910.

The textbook asks for two inverse iterations and two forward iterations,
starting with x1 = [1, 1, 1], then a generalized Rayleigh quotient.

K = [[6, -1, 0], [-1, 4, -1], [0, -1, 2]]
M = [[2, 0, 0], [0, 2, 1], [0, 1, 1]]

We implement:
- Inverse: solve K*x_next = M*x_current.
- Forward: solve M*x_next = K*x_current.
- Normalize each iterate by its largest absolute component.
- Rayleigh quotient: (x^T*K*x) / (x^T*M*x).

Independent numerical reference:

| Method | First iterate (max-abs normalized) | Second iterate | Rayleigh quotient |
| --- | --- | --- | ---: |
| Inverse | [0.333333333, 0.787878788, 1] | [0.202752779, 0.750661726, 1] | 0.726285881696 |
| Forward | [1, 0.4, 0] | [1, 0.357142857, -0.5] | 3.097387173397 |

Exact generalized eigenvalues (independently checked):
0.724456494, 2.965179863, 9.310363643.

The inverse quotient is already close to the smallest eigenvalue after
two steps. The forward quotient is not close to the largest eigenvalue
after two steps for this particular starting vector; that is not a
failure of the method.

The original book supplies the exercise and matrices, not the decimal
values above. The numerical values are independently computed references.

The existing generalized Jacobi implementation remains unchanged and
continues to provide a separate all-eigenpair solution.

Run from rust/:

    cargo test --workspace
    cargo run -p quantum_cpu

No QPU access is needed.
