
pub struct BalancedChunksMut<'a, T> {
    remaining: Option<&'a mut [T]>,
    chunks_left: usize,
    base_size: usize,
    remainder: usize
}

impl<'a, T> BalancedChunksMut<'a, T> {
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
// 2. DoubleEndedIterator: Permite andar para trás (.rev)
// Necessário para fazer o loop reverso na main
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

// 3. ExactSizeIterator: O iterador sabe seu tamanho exato
// Necessário para o .enumerate() funcionar junto com .rev()
impl<'a, T> ExactSizeIterator for BalancedChunksMut<'a, T> {
    fn len(&self) -> usize {
        self.chunks_left
    }
}