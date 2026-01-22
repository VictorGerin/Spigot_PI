use mpi::traits::*;
use std::sync::mpsc::{channel, Sender, IntoIter};
use std::thread::{self, JoinHandle};
use crate::pi_digits_iter::PiDigitsIter;

/// Tags MPI para comunicação entre ranks
#[derive(Debug, Clone, Copy)]
enum MpiTag {
    Carry = 0,
}

impl MpiTag {
    fn as_i32(self) -> i32 {
        self as i32
    }
}

/// Struct Wrapper (Guard) para garantir que a thread MPI finalize corretamente.
///
/// CORREÇÃO:
/// Como PiDigitsIter é genérico <I>, precisamos especificar o tipo concreto do iterador.
/// O channel::Receiver::into_iter() retorna o tipo 'std::sync::mpsc::IntoIter<i32>'.
pub struct MpiGuardIter {
    inner: PiDigitsIter<IntoIter<i32>>,
    thread_handle: Option<JoinHandle<()>>,
}

impl Iterator for MpiGuardIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl Drop for MpiGuardIter {
    fn drop(&mut self) {
        // Ao destruir o iterador, esperamos a thread do MPI terminar o cleanup.
        // Isso evita que o processo morra antes do MPI_Finalize ser chamado.
        if let Some(handle) = self.thread_handle.take() {
            // Ignoramos erros de panic na thread filha para não sujar a saída
            let _ = handle.join();
        }
    }
}

/// Calcula o denominador para o índice i no algoritmo Spigot
#[inline]
fn den(i: i32) -> i32 {
    match i {
        0 => 10,
        _ => i.checked_mul(2).and_then(|x| x.checked_add(1)).unwrap()
    }
}

/// Função auxiliar para rank 0: coordena o processamento e envia para o channel
/// Recebe o Sender (tx) criado na thread principal para enviar os dados de volta.
fn rank0_coordinator<C: Communicator>(
    world: &C, 
    size: i32, 
    n_digits: usize, 
    tx: Sender<i32>
) {
    if size > 1 {
        let last_rank = size - 1;
        // 1. Pipeline Fill: Enviar TODOS os triggers de uma vez
        for _digit_idx in 0..n_digits {
            world.process_at_rank(last_rank).send_with_tag(&0i32, MpiTag::Carry.as_i32());
        }
        
        // 2. Finalização: Enviar flag (-1) para o último rank
        world.process_at_rank(last_rank).send_with_tag(&(-1i32), MpiTag::Carry.as_i32());
        
        // 3. Coleta: Receber resultados e enviar para a main thread via channel
        for _digit_idx in 0..n_digits {
            let (carry_from_next, _status) = world.process_at_rank(1)
                .receive_with_tag::<i32>(MpiTag::Carry.as_i32());
            
            // Se o receptor (main thread) fechou, paramos de processar
            if tx.send(carry_from_next).is_err() {
                break;
            }
        }
    } else {
        // Modo single-process (apenas rank 0, sem MPI real)
        for _digit_idx in 0..n_digits {
            if tx.send(0).is_err() {
                break;
            }
        }
    }
    // tx é dropado aqui, o que fecha o channel e sinaliza o fim para o iterador na main
}

/// Função auxiliar para ranks 1..N: processam chunks em pipeline
fn rank_worker<C: Communicator>(world: &C, rank: i32, size: i32, n_digits: usize) {
    let size_usize = size as usize;
    let rank_usize = rank as usize;
    
    // Calcular tamanho total do array
    let total_len = (n_digits * 10) / 3;
    
    // Ranks workers são size - 1 (rank 0 não conta)
    let num_workers = size_usize - 1;
    
    // Divisão de trabalho
    let base_size = total_len / num_workers;
    let remainder = total_len % num_workers;
    
    // Ajuste de índice (worker 0 é rank 1)
    let worker_idx = rank_usize - 1; 
    let chunk_size = base_size + if worker_idx < remainder { 1 } else { 0 };
    let start_global_index = (worker_idx * base_size + worker_idx.min(remainder)) as i32;
    
    // Criar array local
    let mut local_array = vec![2i32; chunk_size];
    
    // Loop de processamento
    loop {
        // Receber do rank "acima" (ou 0 se for o último)
        let sender_rank = if rank == size - 1 { 0 } else { rank + 1 };
        let (input_value, _status) = world.process_at_rank(sender_rank)
            .receive_with_tag::<i32>(MpiTag::Carry.as_i32());
        
        // Verificar flag de finalização
        if input_value == -1 {
            // Repassar flag para baixo (exceto se for rank 1)
            if rank > 1 {
                let prev_rank = rank - 1;
                world.process_at_rank(prev_rank).send_with_tag(&(-1i32), MpiTag::Carry.as_i32());
            }
            break;
        }
        
        // Algoritmo Spigot no chunk local
        let carry = local_array
            .iter_mut()
            .enumerate()
            .rev()
            .fold(input_value, |carry, (local_idx, cell)| {
                let global_idx = start_global_index + local_idx as i32;
                
                // Cálculos com verificação de overflow
                let cell_x10 = (*cell as i64) * 10;
                let global_idx_plus_1 = global_idx as i64 + 1;
                let carry_contribution = (carry as i64) * global_idx_plus_1;
                let current = cell_x10 + carry_contribution;
                
                let denominator = den(global_idx) as i64;
                
                *cell = (current % denominator) as i32;
                (current / denominator) as i32
            });
        
        // Enviar resultado para o rank anterior
        let prev_rank = rank - 1;
        world.process_at_rank(prev_rank).send_with_tag(&carry, MpiTag::Carry.as_i32());
    }
}

/// Função Principal
pub fn calculate_pi_mpi(n_digits: usize) -> Option<impl Iterator<Item = u8>> {
    let (data_tx, data_rx) = channel::<i32>();
    
    // Canal de Handshake: para a thread avisar qual é o rank dela
    let (rank_tx, rank_rx) = channel::<i32>();

    let handle = thread::spawn(move || {
        let universe = mpi::initialize().expect("Falha ao inicializar MPI");
        let world = universe.world();
        let rank = world.rank();
        let size = world.size();
        
        // 1. AVISA A MAIN THREAD QUAL É O MEU RANK
        rank_tx.send(rank).expect("Falha ao enviar rank para main thread");

        if rank == 0 {
            rank0_coordinator(&world, size, n_digits, data_tx);
        } else {
            // Workers não usam data_tx
            rank_worker(&world, rank, size, n_digits);
        }
    });

    // 2. MAIN THREAD ESPERA O RANK (Bloqueia brevemente)
    let my_rank = rank_rx.recv().expect("Thread MPI morreu antes de reportar rank");

    if my_rank == 0 {
        // Se sou Rank 0: Retorno o iterador. 
        // A thread continua rodando em background e será limpa quando o iterador sair de escopo (Drop).
        Some(MpiGuardIter {
            inner: PiDigitsIter::new(data_rx.into_iter()),
            thread_handle: Some(handle),
        })
    } else {
        // Se sou Worker:
        // IMPORTANTE: Precisamos esperar a thread terminar o trabalho aqui!
        // Se retornássemos None imediatamente, a função main() acabaria e mataria o processo worker.
        handle.join().expect("Worker thread panicou");
        None
    }
}