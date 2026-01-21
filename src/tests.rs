use crate::{calculate_pi_sequential, calculate_pi_parallel};
use std::fs::File;
use std::io::{BufReader, Read};

/// Lê os primeiros n dígitos do arquivo pi_dec_1m.txt usando stream
/// para não carregar o arquivo inteiro na memória
fn read_expected_digits(n: usize) -> impl Iterator<Item = u8> {
    let file = File::open("pi_dec_1m.txt").expect("Não foi possível abrir o arquivo pi_dec_1m.txt");
    let reader = BufReader::new(file);
    
    reader
        .bytes()
        .map(|b| b.expect("Erro ao ler byte do arquivo"))
        .filter(|&b| b >= b'0' && b <= b'9')
        .map(|b| b - b'0')
        .take(n)
}

/// Verifica se os dígitos calculados correspondem aos dígitos esperados
/// Compara dígito a dígito e dispara panic com assert_eq! em caso de divergência
fn verify_pi_digits<I1, I2>(expected: I1, actual: I2)
where
    I1: Iterator<Item = u8>,
    I2: Iterator<Item = u8>,
{
    for (pos, (expected_digit, actual_digit)) in expected.zip(actual).enumerate() {
        assert_eq!(
            expected_digit,
            actual_digit,
            "Dígito incorreto na posição {}: esperado {}, encontrado {}",
            pos,
            expected_digit,
            actual_digit
        );
    }
}

#[test]
fn test_pi_sequential_verification() {
    let n_digits = 20000;
    let expected = read_expected_digits(n_digits);
    let actual = calculate_pi_sequential(n_digits);
    verify_pi_digits(expected, actual);
}

#[test]
fn test_pi_parallel_verification() {
    let n_digits = 20000;
    let num_threads = std::thread::available_parallelism()
        .expect("Não foi possível determinar o número de threads disponíveis")
        .get();
    let channel_bound = 1000;
    let expected = read_expected_digits(n_digits);
    let actual = calculate_pi_parallel(n_digits, num_threads, channel_bound);
    verify_pi_digits(expected, actual);
}
