use spigot_pi::calculate_pi_parallel;

fn main() {
    let n_digits = 100000;
    let num_threads = 8;
    let channel_bound = 8;

    // Calcula os dígitos de PI usando a implementação paralela
    let pi_iterator = calculate_pi_parallel(n_digits, num_threads, channel_bound);

    print!("PI: 3.");
    // Pula o primeiro dígito (que é o 3) e imprime os demais
    for digit in pi_iterator.skip(1) {
        print!("{}", digit);
    }
    println!();
}