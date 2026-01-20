use std::{cell::Cell, iter};

#[inline]
fn den(i: i32) -> i32 {
    match i {
        0 => 10,
        _ => (i * 2) + 1
        
    }
}



/**
 * Essa é minha implementação do algoritimo SPIGOT para calcular o número PI baseado no artigo
 * https://www.cs.williams.edu/~heeringa/classes/cs135/s15/readings/spigot.pdf
 * 
 * A explicação do funcionamento matemático deixo para o artigo, aqui irei explicar como o algoritimo
 * functiona.
 * 
 * Obs:
 * Todas as operações aritiméticas descritas aqui são estritamente aritimetica de números inteiros!
 * 
 * 1º
 * Para calcular os "n" digitos de PI deve ser inicializar um array de tamanho (10n/3) e com todos os
 * index inicializados com o numero "2". Esse array que será usado para computar os digitos de PI.
 * 
 * 2º
 * Para cada dígito de PI devemos computar do final até o inico do array chengado no inicio o resu-
 * tado final no index[0] do array irá guardar um digito de PI multiplicado por 10, por isso deve
 * dividir o index 0 por 10 para isolar o digito.
 * 
 * 3º
 * Cada Index do array representa uma fração seguindo a seguinte lista
 * 
 * 0º  1º  2º  3º  4º (index)
 * 
 * 1   1   2   3   4
 * /   /   /   /   /  (fração)
 * 1   3   5   7   9
 * 
 * 
 * Observe que o denominador segue um simples padrão de todos os numeros impares e podemos calcular
 * simplemente fazendo (i * 2) + 1. Já o numerador por sua vez é mais simples, ele, simplementes é a
 * sequencia de todos os numeros naturáis com exeção do primeiro index 0 que repete o "1"
 * 
 * 4º
 * para cada dígito de PI todos os elementos do array devem ser multiplicados por 10 antes de fazer as contas.
 * 
 * Com esses 4 pontos acima fechado podemos iniciar os calculos para um index i qualquer do array
 * iremos pegar o valor guardado nele arr[i] e iremos computar o denominador de i den(i) e o numerador num(i)
 * com isso podemos fazer o seguinte
 * 
 * arr[i-1] = arr[i-1] + (arr[i] / den(i)) * num(i)
 * arr[i] = arr[i] % den(i)
 * 
 * fazemos isso de (10n/3) até 0 o algoritimo funciona normalmente até o index 1 sendo o ultimo loop
 * o diferente onde o index é o 0.
 * 
 * Para o ultimo loop
 *  iremos extrair o digito atual de PI fazendo:
 * digito = arr[0] / 10
 * e depois iremos atualizar normal como se o denominador fosse "10"
 * arr[0] = arr[0] % 10
 * 
 * 
 * Depois de achamos o digito de PI temos que multiplicar por 10 todo o Arr para podemos repetir esse algoritimo "n"
 * vezes emcima do mesmo arr a cada vez iremos extrair um digito de PI diferente.
 * 
 */
fn main() {


    let n = 10000;


    //Inicializa o array principal de onde os numeros de PI serão calculados
    //A unica diferença aqui foi a Adição de um None no inicio do Vec pois assim poderemos usar 
    //a função windows de forma e melhorar a organização do código.
    //Observação: O array deveria ser inicializado com 2 porem como a primeira etapa é multiplicar
    //por 10 então irei inicializar todos em 20 e deixar a multiplicação por 10 no final de cada
    //etapa.
    let arr = iter::once(None)
    .chain(iter::repeat(Some(Cell::new(20))).take(n * 10 / 3))
    .collect::<Vec<Option<Cell<i32>>>>();


    //Loop para cada digito de PI
    let mut pi_digits = (0..n).map(|_| {
        
        let mut digit = -1;

        //A junção do None no incio e do janelamento e a interação reversa podemos considerar que window[1]
        //sempre represente o item atual do array a ser processado e window[0] o proximo elemento do array
        //Aqui complica um pouco as coisas pois precisamos ir de tras para frente.
        //O enumerate aqui também cai como uma luva pois nos permite sabermos o index atual do array, informação
        //que usaremos para calcular num(i) e den(i)
        for (i, window) in arr.windows(2).enumerate().rev() {

            let curr_cell = window[1].as_ref();
            let next = window[0].as_ref();

            //curr_cell é garantido ser Some() pela forma que o array foi inicializado
            let curr_cell = match curr_cell {
                Some(cell) => cell,
                None => {continue;}
            };

            //Com isso podemos fazer a divisão pelo denominador.
            let resto = curr_cell.get() % den(i as i32);
            let div = curr_cell.get() / den(i as i32);

            //e podemos ajustar o valor atual do array
            curr_cell.set(resto);


            if let Some(next) = next {
                //Caso ainda exista um "proximo" elemento, temos que atualizar ele também
                next.set(next.get() + (i as i32) * div);
            } else {
                //Caso não exista um próximo elemento significa que chegamos ao fim e como den(0) == 10
                //div já irá conter o resultado do digito de PI dividido por 10
                digit = div;
                //break nesse caso não é necessário, pois quando next == None isso só ocorrer
                //no ultimo loop
                break;
            }
        }

        //Existe várias forma de multiplicar cada elemento desse array por 10
        //Essa é só uma que achei para isso.
        arr.iter().for_each(|x| {
            match x.as_ref() {
                None => {},
                Some(cell) =>  {
                    cell.set(cell.get() * 10);
                }
            }
        });

        digit as u8
    })
    .collect::<Vec<u8>>();


    //Em alguns casos é possivel obter um digito de PI >= 10 e isso quer dizer que
    //na verdade o valor do digito é o resto da divisão por 10 e o anterior deve receber o carry
    //A propagação é feita em uma única passagem da direita para a esquerda
    for i in (1..pi_digits.len()).rev() {
        let curr_value = pi_digits[i];
        if curr_value >= 10 {
            let carry = curr_value / 10;
            let remainder = curr_value % 10;
            pi_digits[i] = remainder;
            pi_digits[i - 1] += carry;
        }
    }

    println!("{:?}", pi_digits);

}
