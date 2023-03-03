use std::any::Any;
use std::mem;

// Com esse metodo a gente define
// cada componente. Não sei se 
// já esta assim no seu codigo, mas
// se n tiver recomendo mto. Evita 
// por completo a necessidade do downcasting de Any.
// eu mesmo usei Any aqui só pra mostrar como ficaria
// em estruturas um pouco mais complexas, que n são
// so um inteiro no lugar do valor.
struct Chunk {
    kind: u8,
    length: u32,
    data: Vec<u8>
}

#[derive(Debug)]
enum ComponentKind {
    Health,
    Stamina
}

#[derive(Debug)]
struct Component {
    kind: ComponentKind,
    value: Box<dyn Any>
}

// O grosso aqui é lógica de serialização.
// Se você tá usando uma lib pra isso
// seu código final deve ficar bem menor
fn kind_to_bytes(kind: &ComponentKind) -> Vec<u8> {
    match kind {
        ComponentKind::Health => vec![0x01],
        ComponentKind::Stamina => vec![0x02],
    }
}

fn chunk_to_bytes(chunk: &Chunk) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::new();
    data.push(chunk.kind);
    data.extend_from_slice(&chunk.length.to_ne_bytes());
    data.extend_from_slice(&chunk.data);
    data
}

fn serialize(components: Vec<&Component>) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::new(); 

    for c in components {
        match c.kind {
            ComponentKind::Health => {
                match c.value.downcast_ref::<i64>() {
                    Some(as_int) =>  {
                        data.extend_from_slice(&kind_to_bytes(&c.kind));
                        let chunk_data = as_int.to_ne_bytes();
                        let length = (mem::size_of_val(&chunk_data) as u32).to_ne_bytes();
                        data.extend_from_slice(&length.to_vec());
                        data.extend_from_slice(&chunk_data.to_vec());
                    },
                    None => println!("badly formatted health component")
                }
            },
            ComponentKind::Stamina => {
                match c.value.downcast_ref::<i64>() {
                    Some(as_int) => {
                        data.extend_from_slice(&kind_to_bytes(&c.kind));
                        let chunk_data = as_int.to_ne_bytes();
                        let length = (mem::size_of_val(&chunk_data) as u32).to_ne_bytes();
                        data.extend_from_slice(&length.to_vec());
                        data.extend_from_slice(&chunk_data.to_vec());
                    },
                    None => println!("badly formatted stamina component")
                }
            }
        }
    }

    data
}

fn to_chunks(input: &Vec<u8>) -> Vec<Chunk> {
    let mut chunks: Vec<Chunk> = Vec::new();
    let mut cursor = 0;
    while cursor < input.len() {
        let kind = &input[cursor];
        cursor += 1;
        let length_as_bytes: [u8;4] = input[cursor..cursor+4].try_into().unwrap();
        let length = u32::from_ne_bytes(length_as_bytes);
        cursor += 4;
        let data = &input[cursor..(cursor + usize::try_from(length).unwrap())];

        chunks.push(Chunk {
            kind: *kind,
            length,
            data: data.to_vec()
        });
        cursor += length as usize;
    }
    chunks
}

fn deserialize(chunks: Vec<Chunk>) -> Vec<Component> {
    let mut components: Vec<Component> = Vec::new();

    for c in chunks {
        match c.kind {
            0x01 => components.push(Component {
                kind: ComponentKind::Health,
                value: Box::<i64>::new(i64::from_ne_bytes(c.data.try_into().unwrap()))}),
            0x02 => components.push(Component {
                kind: ComponentKind::Stamina,
                value: Box::<i64>::new(i64::from_ne_bytes(c.data.try_into().unwrap()))}),
            _ => println!("badly formated chunk")
        }
    }

    components
}

// Aqui "data1" seria a fonte de dado mais
// confiável. A store que tá no servidor.
// "data2" é a store que tá vindo do client.

// Comparando só os bytes de cada componente
// a gente consegue saber onde estão as 
// inconsistências sem nem precisar
// deserializar no lado do servidor.
// E ainda por cima dá pra mandar só
// os componentes que precisam levar rollback.

// Nessa POC as stores têm sempre os mesmos
// componentes na mesma ordem. o que muda é a info dentro 
// deles. Se as stores do client e servidor no seu código
// tiverem um formato diferente uma da outra, aí
// nessa parte aqui você vai ter que usar um Map
// pra poder comparar os componentes ao invés de
// usar o index deles no array.
fn diff(data1: Vec<u8>, data2: Vec<u8>) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let chunks1 = to_chunks(&data1);
    let chunks2 = to_chunks(&data2);

    for i in 0..chunks1.len() {
        if chunks1[i].data != chunks2[i].data {
            result.extend_from_slice(&chunk_to_bytes(&chunks1[i]));
        }
    }
    result
}

fn main() {
    let incorrect_health = Component { kind: ComponentKind::Health, value: Box::<i64>::new(88) };
    let health = Component { kind: ComponentKind::Health, value: Box::<i64>::new(30) };
    let stamina = Component { kind: ComponentKind::Stamina, value: Box::<i64>::new(90) };

    let correct = serialize(vec![&health, &stamina]);
    let incorrect = serialize(vec![&incorrect_health, &stamina]);

    let correction = diff(correct, incorrect);
    let deserialized_correction = deserialize(to_chunks(&correction));

    println!("correction {deserialized_correction:?}"); //um array contendo o componente de vida.
}
