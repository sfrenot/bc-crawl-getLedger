use std::collections::HashMap;
use std::sync::Mutex;

use hex::FromHex;
use lazy_static::lazy_static;

use crate::bcnet::bcmessage::{VERSION, VERSION_END};

#[derive(Debug, Clone)]
pub struct BlockDesc {
    pub idx: usize,
    pub previous: String,
}

pub struct BlocksMutex {
    pub blocks_id: Vec<(String, bool, bool, bool)>,
    pub known_blocks: HashMap<String, BlockDesc>,
}

lazy_static! {
    // static ref TEMPLATE_MESSAGE_PAYLOAD: Mutex<Vec<u8>> = Mutex::new(Vec::with_capacity(105));
    static ref TEMPLATE_GETBLOCK_PAYLOAD: Mutex<Vec<u8>> = Mutex::new(Vec::with_capacity(197));

    pub static ref BLOCKS_MUTEX: Mutex<BlocksMutex> = {
        let mut m = Vec::with_capacity(5);
        m.push((String::from("0000000000000000000000000000000000000000000000000000000000000000"), false, false, false));

        let s = BlocksMutex {
            blocks_id: m,
            known_blocks: HashMap::new()
        };

        Mutex::new(s)
    };
}

pub fn get_getheaders_message_payload() -> Vec<u8> {
    TEMPLATE_GETBLOCK_PAYLOAD.lock().unwrap().clone()
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

pub fn create_getdata_message_payload() -> Vec<u8> {
    let mut block_message = Vec::with_capacity(37);
    block_message.extend([0x01]); // Number of Inventory vectors
    block_message.extend([0x02, 0x00, 0x00, 0x40]); // Type of inventory entry (2 = block) (40 for witness)
    let mut search_block = "".to_owned();

    let blocks_id = &mut BLOCKS_MUTEX.lock().unwrap().blocks_id;
    for i in 1..blocks_id.len() {
        let (bloc, prev, downloaded, downloading) = blocks_id[i].clone();
        if !(downloaded) && !(downloading) {
            blocks_id[i] = (bloc.to_string(), prev, downloaded, true);
            search_block = bloc;
            break;
        }
    }

    let block = Vec::from_hex(search_block).unwrap();
    // eprintln!("new getdata -> {:02x?}", block);
    // std::process:exit(1)
    block_message.extend(block);
    block_message
}

pub fn create_block_message_payload() {
    let blocks_id = &BLOCKS_MUTEX.lock().unwrap().blocks_id;
    let mut block_message = TEMPLATE_GETBLOCK_PAYLOAD.lock().unwrap();
    *block_message = Vec::with_capacity(block_message.len() + 32);
    block_message.extend(VERSION.to_le_bytes());
    block_message.push(1);

    let (lastbloc, _, _, _) = blocks_id.last().unwrap();
    let val = Vec::from_hex(lastbloc).unwrap();
    block_message.extend(val);

    let (firstbloc, _, _, _) = &blocks_id[0];
    let val = Vec::from_hex(firstbloc).unwrap();
    block_message.extend(val);

    block_message[VERSION_END] = 1;


    // drop(block_message);
    // eprintln!("{}",hex::encode(&get_getheaders_message_payload()));
    // std::process::exit(1);
}

pub fn is_new(block: &str, previous: &str) -> Result<usize, ()> {
    let mut blocks_mutex_guard = BLOCKS_MUTEX.lock().unwrap();

    let search_block = blocks_mutex_guard.known_blocks.get(block).cloned();
    let search_previous = blocks_mutex_guard.known_blocks.get(previous).cloned();

    match search_previous {
        Some(previous_block) => {
            match search_block {
                None => {
                    let (val, _, downloaded, _) = blocks_mutex_guard.blocks_id.get(previous_block.idx).unwrap();
                    blocks_mutex_guard.blocks_id[previous_block.idx] = (val.to_string(), true, *downloaded, false);
                    blocks_mutex_guard.blocks_id.insert((previous_block.idx + 1) as usize, (block.to_string(), false, false, false));

                    let idx = previous_block.idx + 1;
                    blocks_mutex_guard.known_blocks.insert(block.to_string(), BlockDesc { idx, previous: previous.to_string() });
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
                    let val = BlockDesc { idx, previous: previous.to_string() };
                    blocks_mutex_guard.known_blocks.insert(block.to_string(), val);
                    blocks_mutex_guard.blocks_id.insert(idx, (previous.to_string(), true, false, false));
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
