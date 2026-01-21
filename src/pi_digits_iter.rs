use std::collections::VecDeque;

/**
 * Iterador que corrige dígitos de PI quando o algoritmo spigot gera valores >= 10.
 * 
 * O algoritmo spigot pode ocasionalmente gerar dígitos com valor 10 ou maior. Quando isso
 * acontece, é necessário fazer a propagação de carry (vai-um) para os dígitos anteriores.
 * 
 * **Problema**: Se um dígito é >= 10, ele deve ser reduzido módulo 10 e o dígito anterior
 * deve ser incrementado. Se o dígito anterior for 9, ele se torna 10 e o processo se repete,
 * propagando o overflow até encontrar um dígito < 9.
 * 
 * **Solução**: Este iterador processa os dígitos brutos (i32) e:
 * 1. Quando encontra um dígito >= 10: incrementa o dígito anterior e mantém apenas o resto (d % 10)
 * 2. Quando encontra uma sequência de 9's seguida de um dígito >= 10: todos os 9's viram 0's
 *    e o dígito anterior ao primeiro 9 é incrementado
 * 3. Quando encontra um dígito < 9 após 9's: libera todos os 9's acumulados normalmente
 * 
 * **Exemplo**:
 * - Entrada: [3, 1, 4, 9, 9, 12, 5, ...]
 * - Processamento:
 *   - 3, 1, 4 → liberados normalmente
 *   - 9, 9 → acumulados (não liberados ainda)
 *   - 12 → detecta carry: incrementa o dígito anterior (4 → 5), converte 9's em 0's, mantém 2
 *   - Resultado: [3, 1, 5, 0, 0, 2, 5, ...]
 */
pub struct PiDigitsIter<I> {
    /// O iterador de entrada (fonte dos dados brutos de PI)
    iter: I,
    /// Estado da Máquina: dígito anterior que ainda não foi confirmado/liberado
    /// Este dígito aguarda para ver se o próximo dígito causará carry
    predigit: Option<i32>,
    /// Contador de quantos dígitos "9" consecutivos foram encontrados
    /// Estes 9's são mantidos em espera pois podem virar 0's se o próximo dígito for >= 10
    nines: usize,
    /// Buffer de saída: guarda dígitos prontos para serem entregues no next()
    buffer: VecDeque<u8>,
    /// Flag para saber se a fonte acabou e já fizemos o flush final
    done: bool,
}

impl<I> PiDigitsIter<I> 
where I: Iterator<Item = i32> 
{
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            predigit: None,
            nines: 0,
            buffer: VecDeque::new(),
            done: false,
        }
    }

    // Função auxiliar para enfileirar dados no buffer
    fn queue_digit(&mut self, digit: i32) {
        self.buffer.push_back(digit as u8);
    }
}

impl<I> Iterator for PiDigitsIter<I> 
where I: Iterator<Item = i32> 
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        // 1. Se já tem coisa no buffer, entrega imediatamente.
        if let Some(digit) = self.buffer.pop_front() {
            return Some(digit);
        }

        // 2. Se a fonte já acabou e o buffer está vazio, encerra.
        if self.done {
            return None;
        }

        // 3. Loop de processamento para tentar encher o buffer
        // Processa cada dígito bruto do algoritmo spigot e aplica a correção de carry
        while let Some(d) = self.iter.next() {
            if d == 9 {
                // Dígito 9: acumula para verificar se o próximo dígito causará carry
                // Se o próximo for >= 10, estes 9's viram 0's
                // Se o próximo for < 9, estes 9's são liberados normalmente
                self.nines += 1;
            } else if d < 9 {
                // Caso seguro (d < 9): não há carry, libera tudo que estava em espera
                // Libera o predigit anterior (se existir)
                if let Some(p) = self.predigit {
                    self.queue_digit(p);
                }
                // Libera todos os 9's acumulados (não houve carry, então são 9's válidos)
                for _ in 0..self.nines {
                    self.queue_digit(9);
                }
                
                // O dígito atual vira o novo predigit (aguardando o próximo)
                self.predigit = Some(d);
                self.nines = 0;
                
                // Se geramos saída, paramos o loop para entregar o primeiro item
                if !self.buffer.is_empty() {
                    return self.buffer.pop_front();
                }
            } else {
                // Caso Carry (d >= 10): CORREÇÃO DE OVERFLOW
                // O dígito atual é >= 10, então precisa propagar o carry para trás
                
                // Incrementa o dígito anterior (propagação de carry)
                // Se predigit era 4 e d é 12, então 4 vira 5
                if let Some(p) = self.predigit {
                    self.queue_digit(p + 1);
                }
                
                // Todos os 9's acumulados viram 0's devido ao carry
                // Exemplo: se tínhamos [..., 4, 9, 9, 12], vira [..., 5, 0, 0, 2]
                for _ in 0..self.nines {
                    self.queue_digit(0);
                }
                
                // O dígito atual (>= 10) é reduzido módulo 10 e vira o novo predigit
                // Exemplo: 12 % 10 = 2
                self.predigit = Some(d % 10);
                self.nines = 0;

                if !self.buffer.is_empty() {
                    return self.buffer.pop_front();
                }
            }
        }

        // 4. Se o loop acabou (iter retornou None), faz o FLUSH FINAL
        // Libera qualquer dígito que ainda estava em espera
        self.done = true; // Marca como finalizado para não entrar aqui de novo
        
        // Libera o último predigit
        if let Some(p) = self.predigit {
            self.queue_digit(p);
        }
        // Libera os últimos 9's acumulados (se houver)
        for _ in 0..self.nines {
            self.queue_digit(9);
        }

        // Retorna o que tiver sobrado (ou None se não sobrou nada)
        self.buffer.pop_front()
    }
}