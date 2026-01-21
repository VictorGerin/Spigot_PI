pub mod balanced_chunks_mut;
pub mod pi_digits_iter;

use std::{cell::Cell, iter, sync::mpsc::{channel, sync_channel}, thread};
use balanced_chunks_mut::BalancedChunksMut;
use pi_digits_iter::PiDigitsIter;

/// Calcula o denominador para o índice i no algoritmo Spigot
#[inline]
fn den(i: i32) -> i32 {
    match i {
        0 => 10,
        _ => i.checked_mul(2).and_then(|x| x.checked_add(1)).unwrap()
    }
}

/// Calcula os dígitos de PI usando o algoritmo Spigot de forma sequencial
/// 
/// # Parâmetros
/// - `n_digits`: Número de dígitos de PI a serem calculados
/// 
/// # Retorna
/// Um iterador sobre os dígitos de PI (u8)
pub fn calculate_pi_sequential(n_digits: usize) -> impl Iterator<Item = u8> {
    // Inicializa o array principal de onde os números de PI serão calculados
    // A única diferença aqui foi a adição de um None no início do Vec pois assim poderemos usar 
    // a função windows de forma e melhorar a organização do código.
    // Observação: O array deveria ser inicializado com 2 porém como a primeira etapa é multiplicar
    // por 10 então irei inicializar todos em 20 e deixar a multiplicação por 10 no final de cada
    // etapa.
    let arr = iter::once(None)
        .chain(iter::repeat(Some(Cell::new(20))).take(n_digits * 10 / 3))
        .collect::<Vec<Option<Cell<i32>>>>();

    // Loop para cada dígito de PI
    let pi_digits_raw = (0..n_digits).map(move |_| {
        let mut digit = -1;

        // A junção do None no início e do janelamento e a iteração reversa podemos considerar que window[1]
        // sempre represente o item atual do array a ser processado e window[0] o próximo elemento do array
        // Aqui complica um pouco as coisas pois precisamos ir de trás para frente.
        // O enumerate aqui também cai como uma luva pois nos permite sabermos o index atual do array, informação
        // que usaremos para calcular num(i) e den(i)
        for (i, window) in arr.windows(2).enumerate().rev() {
            let curr_cell = window[1].as_ref();
            let next = window[0].as_ref();

            // curr_cell é garantido ser Some() pela forma que o array foi inicializado
            let curr_cell = match curr_cell {
                Some(cell) => cell,
                None => { continue; }
            };

            // Com isso podemos fazer a divisão pelo denominador.
            let resto = curr_cell.get() % den(i as i32);
            let div = curr_cell.get() / den(i as i32);

            // e podemos ajustar o valor atual do array
            curr_cell.set(resto);

            if let Some(next) = next {
                // Caso ainda exista um "próximo" elemento, temos que atualizar ele também
                next.set(next.get() + (i as i32) * div);
            } else {
                // Caso não exista um próximo elemento significa que chegamos ao fim e como den(0) == 10
                // div já irá conter o resultado do dígito de PI dividido por 10
                digit = div;
                // break nesse caso não é necessário, pois quando next == None isso só ocorrer
                // no último loop
                break;
            }
        }

        // Existe várias formas de multiplicar cada elemento desse array por 10
        // Essa é só uma que achei para isso.
        arr.iter().for_each(|x| {
            match x.as_ref() {
                None => {},
                Some(cell) => {
                    cell.set(cell.get() * 10);
                }
            }
        });

        digit
    });

    // Usa o PiDigitsIter para processar os dígitos brutos e fazer a propagação de carry automaticamente
    PiDigitsIter::new(pi_digits_raw)
}

/// Calcula os dígitos de PI usando o algoritmo Spigot de forma paralela
/// 
/// # Parâmetros
/// - `n_digits`: Número de dígitos de PI a serem calculados
/// - `num_threads`: Número de threads para processamento paralelo
/// - `channel_bound`: Tamanho do buffer dos canais de comunicação entre threads
/// 
/// # Retorna
/// Um iterador sobre os dígitos de PI (u8) que não bloqueia a thread principal
pub fn calculate_pi_parallel(n_digits: usize, num_threads: usize, channel_bound: usize) -> impl Iterator<Item = u8> {
    // Cálculo exato do tamanho do array
    let total_len = (n_digits * 10) / 3;
    let mut big_array = vec![2; total_len];

    // Cria um canal síncrono para disparar o processamento de cada dígito
    let (trigger_tx, trigger_rx) = sync_channel::<i32>(channel_bound);

    // Cria um canal para enviar os dados brutos (i32) para a thread principal
    let (raw_tx, raw_rx) = sync_channel::<i32>(channel_bound);

    // Thread que dispara o processamento de cada dígito
    thread::spawn(move || {
        for _ in 0..n_digits {
            if trigger_tx.send(0).is_err() {
                break;
            }
        }
    });

    // Thread que processa os dados e envia os dados brutos para o canal
    thread::spawn(move || {
        thread::scope(|scope| {
            let mut input_source = trigger_rx;

            let base_size = total_len / num_threads;
            let remainder = total_len % num_threads;

            // Divide o array em chunks balanceados e processa cada chunk em uma thread separada
            for (i, chunk) in BalancedChunksMut::new(&mut big_array, num_threads).enumerate().rev() {
                let start_global_index = (base_size * i) + i.min(remainder);

                let (tx, rx) = channel();
                let rx_in = input_source;
                input_source = rx;

                scope.spawn(move || {
                    for carry_in in rx_in {
                        // fold consome o iterador reverso do chunk (processando da direita para a esquerda)
                        let carry_out = chunk
                            .iter_mut()
                            .enumerate()
                            .rev()
                            .fold(carry_in, |carry, (idx, cell)| {
                                let global_idx: i32 = start_global_index
                                    .checked_add(idx)
                                    .and_then(|x: usize| x.try_into().ok())
                                    .expect("Overflow ao calcular global_idx");
                                
                                let cell_value: i32 = *cell;
                                let cell_x10: i32 = cell_value
                                    .checked_mul(10i32)
                                    .expect("Overflow ao multiplicar cell por 10");
                                
                                let global_idx_plus_one: i32 = global_idx
                                    .checked_add(1i32)
                                    .expect("Overflow ao adicionar 1 a global_idx");
                                
                                let carry_x_idx: i32 = carry
                                    .checked_mul(global_idx_plus_one)
                                    .expect("Overflow ao multiplicar carry por (global_idx + 1)");
                                
                                let current = cell_x10
                                    .checked_add(carry_x_idx)
                                    .expect("Overflow ao calcular current");
                                
                                let denominator = den(global_idx);
                                
                                *cell = current
                                    .checked_rem(denominator)
                                    .expect("Overflow ou divisão por zero no resto");
                                
                                current
                                    .checked_div(denominator)
                                    .expect("Overflow ou divisão por zero na divisão")
                            });

                        if tx.send(carry_out).is_err() {
                            break;
                        }
                    }
                });
            }

            // Envia os dados brutos para o canal usando try_for_each
            let _ = input_source.into_iter().try_for_each(|digit_raw| {
                raw_tx.send(digit_raw)
            });
        });
    });

    // Cria o PiDigitsIter na thread principal com os dados brutos do canal
    PiDigitsIter::new(raw_rx.into_iter())
}