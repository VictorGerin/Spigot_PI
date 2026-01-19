use std::{cell::Cell, iter};

fn den(i: i32) -> i32 {
    match i {
        0 => (i * 2) + 1,
        _ => (i * 2) + 1
        
    }
}

fn num(i: i32) -> i32 {
    match i {
        0 => 1,
        _ => i
    }
}



fn main() {


    let arr = iter::once(None)
    .chain(iter::repeat(Some(Cell::new(2))).take(13))
    // .chain(iter::once(None))
    .collect::<Vec<Option<Cell<i32>>>>();

    for _ in 0..4 {
        println!("{:?}", arr);

        arr.iter().for_each(|x| {
            match x.as_ref() {
                None => {},
                Some(cell) =>  {
                    cell.set(cell.get() * 10);
                }
            }
        });

        for (i, window) in arr.windows(2).enumerate().rev() {
            // Helper para extrair valor de &Option<Cell<i32>>
            // as_ref() converte &Option<Cell> para Option<&Cell>
            // map(|c| c.get()) extrai o valor do Cell se ele existir
            //let prev = get_val(&window[2]);
            let curr_cell = window[1].as_ref(); // Precisamos da referência ao Cell para escrever
            let next = window[0].as_ref();

            let curr_cell = match curr_cell {
                Some(cell   ) => cell,
                None => {continue;}
            };


            let resto = curr_cell.get() % den(i as i32);
            let div = curr_cell.get() / den(i as i32);
            curr_cell.set(resto);

            if let Some(next) = next {
                next.set(next.get() + num(i as i32) * div);
            } else {
                println!("{:?}", div / 10);
            }

        }
    }



    // for chunk in arr2.chunks_mut(2) {
    //     println!("{:?}", chunk);
    //     *chunk[0] = 10;
    // }
    // println!("{:?}", arr);
    // println!("{:?}", arr2);
}
