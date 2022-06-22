use std::sync::Mutex;
use std::sync::MutexGuard;
use lazy_static::lazy_static;
use hex::FromHex;
use crate::bcnet::bcmessage::{VERSION, VERSION_END};
use std::collections::HashMap;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct BlockDesc {
    pub idx: usize,
    pub previous: String
}

pub struct BlocksMutex {
    pub blocks_id: Vec<(String, bool, bool)>,
    pub known_blocks: HashMap<String, BlockDesc>
}

lazy_static! {
    // static ref TEMPLATE_MESSAGE_PAYLOAD: Mutex<Vec<u8>> = Mutex::new(Vec::with_capacity(105));
    static ref TEMPLATE_GETBLOCK_PAYLOAD: Mutex<Vec<u8>> = Mutex::new(Vec::with_capacity(197));

    pub static ref BLOCKS_MUTEX: Mutex<BlocksMutex> = {
        let mut m = Vec::with_capacity(5);
        m.push((String::from("0000000000000000000000000000000000000000000000000000000000000000"), false, false));

        let s = BlocksMutex {
            blocks_id: m,
            known_blocks: HashMap::new()
        };

        Mutex::new(s)
    };
}

pub fn get_getblock_message_payload() -> Vec<u8> {
    TEMPLATE_GETBLOCK_PAYLOAD.lock().unwrap().clone()
}

pub fn get_getheaders_message_payload() -> Vec<u8> {
    get_getblock_message_payload()
}

/*
pub fn get_getdata_message_payload(search_block: &str) -> Vec<u8> {
    let mut block_message = Vec::with_capacity(37);
    block_message.extend([0x01]); //Number of Inventory vectors
    block_message.extend([0x02, 0x00, 0x00, 0x00]);
    let mut block = Vec::from_hex(search_block).unwrap();
    block.reverse();
    block_message.extend(block);
    block_message
}
*/

pub fn create_getdata_message_payload() -> Vec<u8>{
    let blocks_id = &BLOCKS_MUTEX.lock().unwrap().blocks_id;

    let mut block_message = Vec::with_capacity(37);
    block_message.extend([0x01]); // Number of Inventory vectors
    block_message.extend([0x02, 0x00, 0x00, 0x40]); // Type of inventory entry (2 = block) (40 for witness)
    let mut search_block:&str = "";

    let mut rng = rand::thread_rng();
    let mut idx = rng.gen_range(0..200);

    for i in 1..blocks_id.len() {
        let (bloc, _, downloaded) = &blocks_id[i];
        if !downloaded && idx < 0{
            search_block = bloc;
            break;
        }
        idx = idx-1;
    }

    let mut block = Vec::from_hex(search_block).unwrap();
    block.reverse();
    // eprintln!("new getdata -> {:02x?}", block);
    // std::process:exit(1)
    block_message.extend(block);
    block_message
}

pub fn create_block_message_payload(blocks_id: &Vec<(String, bool, bool)>) {
    let mut block_message = TEMPLATE_GETBLOCK_PAYLOAD.lock().unwrap();
    *block_message = Vec::with_capacity(block_message.len()+32);
    block_message.extend(VERSION.to_le_bytes());
    block_message.extend([blocks_id.len() as u8-1]); // Fake value replaced later
    let size = blocks_id.len()-1;
    let mut nb = 0;
    for i in 0..blocks_id.len() {
        let (bloc, next, _) = &blocks_id[size-i];
        if !next {
            let mut val = Vec::from_hex(bloc).unwrap();
            val.reverse();
            block_message.extend(val);
            nb+=1;
        }
    }
    block_message[VERSION_END] = nb-1; //Vector size
    // drop(block_message);
    // eprintln!("{}",hex::encode(&get_getheaders_message_payload()));
    // std::process::exit(1);
}

pub fn is_new(blocks_mutex_guard: &mut MutexGuard<BlocksMutex>, block: String, previous: String ) -> Result<usize, ()> {

    let search_block =  blocks_mutex_guard.known_blocks.get(&block).cloned();
    let search_previous = blocks_mutex_guard.known_blocks.get(&previous).cloned();

    match search_previous {
        Some(previous_block) => {
            match search_block {
                None => {
                    let (val, _, downloaded) = blocks_mutex_guard.blocks_id.get(previous_block.idx).unwrap();
                    blocks_mutex_guard.blocks_id[previous_block.idx] =  (val.to_string(), true, *downloaded);
                    blocks_mutex_guard.blocks_id.insert((previous_block.idx+1) as usize, (block.clone(), false, false));

                    let idx = previous_block.idx + 1;
                    blocks_mutex_guard.known_blocks.insert(block.clone(), BlockDesc{idx, previous});
                    // eprintln!("Trouvé previous, Pas trouvé block");
                    // eprintln!("{:?}", blocks_id);
                    // eprintln!("{:?}", known_block);
                    // std::process::exit(1);
                    Ok(idx)
                }
                _ => {
                    Ok(0)
                }
            }
        }
        _ => {
            match search_block {
                Some(found_block) => {
                    // eprintln!("Previous {} non trouvé, Block trouvé {}", &previous, &block);
                    let idx = found_block.idx;
                    let val = BlockDesc{idx, previous: previous.clone()};
                    blocks_mutex_guard.known_blocks.insert(block.clone(), val);

                    blocks_mutex_guard.blocks_id.insert(idx, (previous.clone(), true, false));
                    eprintln!("Previous non {}, Block oui {}", &previous, &block);

                    // eprintln!("{:?}", blocks_id);
                    // eprintln!("{:?}", known_block);
                    std::process::exit(1);
                }
                _ => {
                    eprintln!("Previous non {}, Block non {}", &previous, &block);
                    // eprintln!("{:?}", blocks_id);
                    // eprintln!("{:?}", known_block);
                    Err(())
                }
            }
        }
    }
}
