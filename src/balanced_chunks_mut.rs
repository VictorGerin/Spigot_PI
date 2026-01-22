/// Um iterador que divide um slice mutável em chunks balanceados.
///
/// Este iterador é similar ao `chunks_mut()` padrão do Rust, mas com uma diferença importante:
/// ao invés de colocar todo o resto da divisão no último chunk (deixando-o significativamente
/// maior que os outros), o `BalancedChunksMut` distribui o resto uniformemente entre os primeiros
/// chunks, garantindo que todos os chunks tenham tamanhos muito próximos (diferindo no máximo
/// por 1 elemento).
///
/// # Exemplo
///
/// Para um slice de 10 elementos dividido em 3 chunks:
/// - `chunks_mut(3)`: `[4, 4, 2]` - o último chunk fica muito menor
/// - `BalancedChunksMut::new(3)`: `[4, 3, 3]` - todos os chunks têm tamanhos próximos
///
/// Para um slice de 10 elementos dividido em 4 chunks:
/// - `chunks_mut(4)`: `[3, 3, 3, 1]` - o último chunk fica muito menor
/// - `BalancedChunksMut::new(4)`: `[3, 3, 2, 2]` - os chunks têm tamanhos balanceados
///
/// # Algoritmo
///
/// O algoritmo calcula `base_size = len / num_chunks` e `remainder = len % num_chunks`.
/// Os primeiros `remainder` chunks recebem `base_size + 1` elementos, enquanto os demais
/// recebem `base_size` elementos. Isso garante que a diferença máxima entre qualquer
/// dois chunks seja de apenas 1 elemento.
pub struct BalancedChunksMut<'a, T> {
    remaining: Option<&'a mut [T]>,
    chunks_left: usize,
    base_size: usize,
    remainder: usize
}

impl<'a, T> BalancedChunksMut<'a, T> {
    /// Cria um novo iterador que divide o slice em `num_chunks` chunks balanceados.
    ///
    /// # Parâmetros
    /// - `slice`: O slice mutável a ser dividido
    /// - `num_chunks`: O número de chunks desejados
    ///
    /// # Retorna
    /// Um iterador que produz `num_chunks` slices mutáveis com tamanhos balanceados.
    ///
    /// # Panics
    /// Não causa panic, mas retorna um iterador vazio se `num_chunks == 0`.
    pub fn new(slice: &'a mut[T], num_chunks:usize) -> Self {
        if num_chunks == 0 {
            Self { remaining: None, chunks_left: 0, base_size: 0, remainder: 0 }
        } else {
            let len = slice.len();
            Self {
                remaining: Some(slice),
                chunks_left: num_chunks,
                base_size: len / num_chunks,
                remainder: len % num_chunks
            }
        }
    }
}

impl<'a, T> Iterator for BalancedChunksMut<'a, T> {
    type Item = &'a mut[T];
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.chunks_left {
            0 => None,
            _ => {
                // Pega a referência do slice restante (ownership temporário via take)
                let slice = self.remaining.take()?;

                // Calcula o tamanho DESTE chunk específico.
                // Se ainda houver "resto" (remainder > 0), este chunk ganha 1 item extra.
                let current_size = self.base_size + if self.remainder > 0 { 1 } else { 0 };

                // Atualiza os contadores de estado
                self.chunks_left -= 1;
                if self.remainder > 0 { self.remainder -= 1; }

                if self.chunks_left == 0 {
                    // Se for o último chunk, retorna tudo o que sobrou (evita cálculo de split)
                    Some(slice)
                } else {
                    // Corta o pedaço atual e guarda o resto de volta na struct
                    let (head, tail) = slice.split_at_mut(current_size);
                    self.remaining = Some(tail);

                    Some(head)
                }
            }
        }
    }
    
}
/// Implementação de `DoubleEndedIterator` que permite iterar em ordem reversa.
///
/// Isso é necessário para processar os chunks de trás para frente, o que é útil
/// no algoritmo Spigot onde o processamento precisa começar do final do array.
impl<'a, T> DoubleEndedIterator for BalancedChunksMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.chunks_left == 0 { return None; }

        let slice = self.remaining.take()?;

        // Lógica Inversa: Precisamos saber se o chunk lá do FINAL tem tamanho extra.
        // Os chunks com tamanho extra são os índices [0, 1, ... remainder-1].
        // O índice lógico atual do fim é (chunks_left - 1).
        let is_bonus_chunk = (self.chunks_left - 1) < self.remainder;
        let current_size = self.base_size + if is_bonus_chunk { 1 } else { 0 };

        self.chunks_left -= 1;
        // Se pegamos um chunk do final e ele ERA um chunk de bonus (o que só acontece
        // se remainder == chunks_left), precisamos decrementar o remainder.
        if is_bonus_chunk { self.remainder -= 1; }

        if self.chunks_left == 0 {
            return Some(slice);
        }

        // Como estamos pegando do final, cortamos em (len - size)
        // head fica no remaining, tail é retornado
        let split_idx = slice.len() - current_size;
        let (head, tail) = slice.split_at_mut(split_idx);
        
        self.remaining = Some(head);
        Some(tail)
    }
}

/// Implementação de `ExactSizeIterator` que permite saber o número exato de chunks restantes.
///
/// Isso é necessário para que `.enumerate()` funcione corretamente quando combinado
/// com `.rev()`, permitindo obter o índice correto de cada chunk durante a iteração reversa.
impl<'a, T> ExactSizeIterator for BalancedChunksMut<'a, T> {
    fn len(&self) -> usize {
        self.chunks_left
    }
}