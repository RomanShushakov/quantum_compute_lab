# Experiment 08 - Bathe eigenproblem to two-qubit Hamiltonian

Step 1: deterministic classical preparation for a future quantum VQE.

K = [[6,-1,0],[-1,4,-1],[0,-1,2]]
M = [[2,0,0],[0,2,1],[0,1,1]]

Define S = M^(-1/2) and H = S K S. This maps the generalized
symmetric eigenproblem K phi = lambda M phi into H psi = lambda psi,
with psi = M^(1/2) phi.

For this M:
S = [[1/sqrt(2),0,0],
     [0,2/sqrt(5),-1/sqrt(5)],
     [0,-1/sqrt(5),3/sqrt(5)]]

H = [[3,-2/sqrt(10),1/sqrt(10)],
     [-2/sqrt(10),4.4,-4.2],
     [1/sqrt(10),-4.2,5.6]]

Its eigenvalues are 0.724456494, 2.965179863, 9.310363643.

Pad H into a 4x4 block diagonal matrix with fourth diagonal 12.0.
This represents a two-qubit Hermitian operator and ensures that
|11> is not the ground state. A later ansatz must still be expressive
enough to reach the physical ground state.

Pauli strings use the tensor-product order P_left (x) P_right,
where the left factor acts on the most significant computational bit.
The real symmetric matrix has 10 potentially nonzero coefficients:

II  6.25
IX -0.316227766017
IZ -1.95
XI  0.158113883008
XX -2.1
XZ  0.158113883008
YY -2.1
ZI -2.55
ZX -0.316227766017
ZZ  1.25

Coefficients are c_P = Tr(P H)/4. Their reconstruction is checked
against the original 4x4 matrix in regression tests.

No quantum circuit is executed in this step. It prepares a verified
Hamiltonian for the subsequent Rust statevector VQE experiment.
