use spigot_pi::calculate_pi_sequential;

fn main() {
    let n_digits = 10000;

    // Calcula os dígitos de PI usando a implementação sequencial
    let pi_iterator = calculate_pi_sequential(n_digits);

    print!("PI: 3.");
    // Pula o primeiro dígito (que é o 3) e imprime os demais
    for digit in pi_iterator.skip(1) {
        print!("{}", digit);
    }
    println!();
}
