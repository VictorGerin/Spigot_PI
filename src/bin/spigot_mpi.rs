// Descrição: Binário MPI para cálculo distribuído de PI - compilado apenas quando feature "mpi" está habilitada
// Gerado por: Cursor AI
// Versão: mpi 0.8.1
// AI_GENERATED_CODE_START
#[cfg(feature = "mpi")]
use spigot_pi::calculate_pi_mpi;

#[cfg(feature = "mpi")]
fn main() {
    let n_digits = 1000000;

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

#[cfg(not(feature = "mpi"))]
fn main() {
    eprintln!("Erro: Este binário requer a feature 'mpi' para ser compilado.");
    eprintln!("Compile com: cargo build --features mpi --bin spigot_mpi");
    std::process::exit(1);
}
// AI_GENERATED_CODE_END
