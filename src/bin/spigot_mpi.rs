use spigot_pi::calculate_pi_mpi;

fn main() {
    let n_digits = 100000;

    // Calcula os dígitos de PI usando a implementação MPI distribuída
    // Nota: Esta função deve ser executada com mpirun/mpiexec
    // Exemplo: mpirun -np 4 target/release/spigot_pi --bin spigot_mpi
    if let Some(digits) = calculate_pi_mpi(n_digits) {
        println!("Calculando PI com MPI:");
        for d in digits {
            print!("{}", d);
        }
        println!();
    }
}
