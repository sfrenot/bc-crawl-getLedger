use lazy_static::lazy_static;
use std::time::SystemTime;
use chrono::{DateTime, NaiveDateTime, Utc};
use sha2::{Sha256, Digest};
use std::net::{TcpStream, IpAddr};
use std::io::{Read, Error, ErrorKind};
use std::convert::TryInto;

use hex::{FromHex};
use crate::bcblocks;
use bitcoin_hashes::{sha256d, Hash};
use crate::bcparse::{Block, parse_block, parse_block_sfr, ParsingError};
use crate::bcnet::bcmessage::ProcessBlockMessageError::Parsing;
use crate::bcutils::get_compact_int;

pub const VERSION:u32 = 70015;
const PORT:u16 = 8333;

// services
const NODE_NETWORK: u64 = 1;
const NODE_BLOOM: u64 = 4;
const NODE_WITNESS: u64 = 8;
const NODE_NETWORK_LIMITED: u64 = 1024;

// offset for addr cmd
pub const ADDRESS_LEN: usize = 30;
pub const TIME_FIELD_END: usize = 4;
pub const SERVICES_END: usize = 12;
pub const IP_FIELD_END: usize = 28;
pub const PORT_FIELD_END: usize = 30;

// offset for version cmd
pub const VERSION_END: usize =4;

const USER_AGENT: usize = 80;
const TIMESTAMP_END: usize = 20;

// payload struct
lazy_static! {
    static ref TEMPLATE_MESSAGE_PAYLOAD: Vec<u8> = create_init_message_payload();

    pub static ref MSG_VERSION:String = String::from("version");
    pub static ref MSG_VERSION_ACK:String = String::from("verack");
    pub static ref MSG_GETADDR:String = String::from("getaddr");
    pub static ref MSG_ADDR:String = String::from("addr");
    pub static ref INV:String = String::from("inv");
    pub static ref CONN_CLOSE:String = String::from("CONNCLOSED");
    pub static ref GET_HEADERS:String = String::from("getheaders");
    pub static ref HEADERS:String = String::from("headers");
    // pub static ref GET_BLOCKS:String = String::from("getblocks");
    pub static ref GET_DATA:String = String::from("getdata");
    pub static ref BLOCK:String = String::from("block");
}

const START_DATE:usize = 12;
const END_DATE:usize= 20;

// HEADER STRUCT
const HEADER_SIZE:usize = 24;
const MAGIC:&[u8;4]  =  &[0xF9, 0xBE, 0xB4, 0xD9];

const START_MAGIC:usize = 0;
const END_MAGIC:usize = 4;
const START_CMD:usize = 4;
const END_CMD:usize = 16;
const START_PAYLOAD_LENGTH :usize= 16;
const END_PAYLOAD_LENGTH :usize= 20;
const START_CHECKSUM:usize = 20;
const END_CHECKSUM:usize = 24;

fn create_init_message_payload() -> Vec<u8> {
    let services:u64 = NODE_NETWORK | NODE_BLOOM | NODE_WITNESS | NODE_NETWORK_LIMITED;
    let date_buffer:u64 = 0;
    let address_buffer:u64 = 0;

    let binary_ip  = [127, 0, 0, 1];

    let mut address_from = Vec::from_hex("00000000000000000000ffff").unwrap();
    address_from.extend(binary_ip);
    address_from.extend(PORT.swap_bytes().to_be_bytes());

    let node_id = Vec::from_hex("1414141414141412").unwrap();
    let user_agent:&[u8] = "\x0C/bcpc:0.0.1/".as_bytes();
    let height:u32 = 708998;

    let mut message_payload = Vec::with_capacity(105);

    message_payload.extend(VERSION.to_le_bytes());
    message_payload.extend(services.to_le_bytes());
    message_payload.extend(date_buffer.to_le_bytes());
    message_payload.extend(address_buffer.to_le_bytes());
    message_payload.extend(services.to_le_bytes());
    message_payload.extend(&address_from);
    message_payload.extend(services.to_le_bytes());
    message_payload.extend(&address_from);
    message_payload.extend(node_id);
    message_payload.extend(user_agent);
    message_payload.extend(height.to_le_bytes());

    message_payload

    // eprintln!("{:02x?}", hex::encode(TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().to_vec()));
    // std::process::exit(1);
}

// Read message from a peer return command, payload, err
pub fn read_message(mut connection: &TcpStream) -> Result<(String, Vec<u8>), Error> {
    let mut header_buffer = [0 as u8;HEADER_SIZE];

    return match connection.read(&mut header_buffer) {
        Ok(_) => {
            // println!("Lecture faite {:02X?}", header_buffer);
            if header_buffer[START_MAGIC..END_MAGIC] != MAGIC[..] {
                //println!("Error in Magic message header: {:?}", &header_buffer[START_MAGIC..END_MAGIC]);
                return Err(Error::new(ErrorKind::Other, "Magic error"));
            }

            let cmd = String::from_utf8_lossy(&header_buffer[START_CMD..END_CMD]);
            let command = cmd.trim_matches(char::from(0)).to_string();

            let payload_size = u32::from_le_bytes((&header_buffer[START_PAYLOAD_LENGTH..END_PAYLOAD_LENGTH]).try_into().unwrap());
            if payload_size <= 0 {
                return Ok((command, vec![0]));
            };

            let mut payload_buffer = vec![0u8; payload_size as usize];
            match connection.read_exact(&mut payload_buffer) {
                Ok(_) => Ok((command, payload_buffer)),
                Err(e) => Err(e)
            }
        },
        Err(e) => Err(e)
    }
}

pub fn build_request(message : &str) -> Vec<u8>{
    let mut payload_bytes: Vec<u8> = Vec::new();
    // eprintln!("->MSG_VERSION : {:02X?}", payload_bytes);

    if message == *MSG_VERSION {
        payload_bytes = get_payload_with_current_date();
    } else if message == *GET_HEADERS {
        payload_bytes = bcblocks::get_getheaders_message_payload();
    } else if message == *GET_DATA {
        payload_bytes = bcblocks::create_getdata_message_payload();
    }

    let mut header :Vec<u8> = vec![0; HEADER_SIZE];
    build_request_message_header(& mut header, message, &payload_bytes);
    let mut request = vec![];
    request.extend(header);
    request.extend(payload_bytes);

    return request;
}

fn get_payload_with_current_date() -> Vec<u8> {
    let mut payload :Vec<u8>  = TEMPLATE_MESSAGE_PAYLOAD.clone();
    let mut date :Vec<u8> = Vec::new();
    let unix_timestamp:u64 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    date.extend(unix_timestamp.swap_bytes().to_be_bytes());
    payload.splice(START_DATE..END_DATE, date.iter().cloned());
    return payload;
}

fn build_request_message_header(header: & mut Vec<u8>, command_name :&str, payload : &Vec<u8>){

    header.splice(START_MAGIC..END_MAGIC, MAGIC.iter().cloned());
    let end_cmd = command_name.as_bytes().len() +START_CMD;
    if end_cmd > END_CMD { panic!("wrong command") }
    header.splice(START_CMD..end_cmd, command_name.as_bytes().iter().cloned());

    let payload_len :u32 = payload.len() as u32;
    let mut payload_len_buffer = Vec::new();
    payload_len_buffer.extend(payload_len.swap_bytes().to_be_bytes());
    let slice:&[u8] =&payload_len_buffer[..];
    header.splice(START_PAYLOAD_LENGTH..END_PAYLOAD_LENGTH, slice.iter().cloned());

    let checksum = compute_checksum(payload);
    header.splice(START_CHECKSUM..END_CHECKSUM, checksum.iter().cloned());
}

pub fn process_version_message(payload: &Vec<u8>) -> (u32, Vec<u8>, DateTime<Utc>, String) {

    let version_number = u32::from_le_bytes((&payload[..VERSION_END]).try_into().unwrap());
    let services = payload[VERSION_END..SERVICES_END].to_vec();
    let peer_time = get_date_time(payload[SERVICES_END..TIMESTAMP_END].to_vec());

    let (tmp, start_byte) = get_compact_int(&payload[USER_AGENT..].to_vec());
    let useragent_size = tmp as usize;

    let mut user_agent = String::new();
    if USER_AGENT + start_byte + useragent_size < payload.len() {
        if useragent_size > 0 {
            let user_agent_slice = &payload[(USER_AGENT + start_byte)..(USER_AGENT + start_byte + useragent_size)];
            user_agent.push_str(String::from_utf8(user_agent_slice.to_vec()).unwrap().as_str() );

        }
    }
    (version_number, services, peer_time, user_agent)
}

pub fn process_addr_message(payload: &Vec<u8>) -> Vec<String>{
    let (addr_number, start_byte) = get_compact_int(&payload);
    if addr_number < 2 {
        return vec![];
    }

    let mut addr = vec![];
    let mut read_addr = 0 ;
    // let mut new_addr = 0;
    while read_addr < addr_number {

        let addr_begins_at = start_byte + (ADDRESS_LEN * read_addr as usize);
        let _date_time = get_date_time(payload[addr_begins_at..addr_begins_at+ TIME_FIELD_END].to_vec());
        let _services = payload[addr_begins_at+ TIME_FIELD_END..addr_begins_at+ SERVICES_END].to_vec();
        let ip_addr_field = payload[addr_begins_at+ SERVICES_END..addr_begins_at+ IP_FIELD_END].to_vec();

        let mut array_v6 = [0; 16];
        array_v6.copy_from_slice(&ip_addr_field[..]);
        let _ip_v6 = IpAddr::from(array_v6);

        let mut array_v4 = [0; 4];
        array_v4.copy_from_slice(&ip_addr_field[12..]);
        let ip_v4 = IpAddr::from(array_v4);

        let port = u16::from_be_bytes((&payload[addr_begins_at+ IP_FIELD_END..addr_begins_at+ PORT_FIELD_END]).try_into().unwrap());
        let new_peer: String = format!("{}:{:?}", ip_v4, port);

        addr.push(new_peer);
        read_addr = read_addr +1;
    }
    // eprintln!("--> Ajout {} noeuds", new_addr);
    addr
}

#[derive(Debug)]
pub enum ProcessHeadersMessageError {
    UnkownBlocks,
    NoNewBlocks
}
pub fn process_headers_message(payload: &Vec<u8>) -> Result<Vec<String>, ProcessHeadersMessageError> {
    let mut new_blocks = vec![];

    let mut highest_index = 0;
    let (nb_headers, mut offset) = get_compact_int(&payload);
    let header_length = 80;
    for _i in 0..nb_headers {
        let mut previous_block = [0;32];
        previous_block.clone_from_slice(&payload[offset+4..offset+4+32]);
        previous_block.reverse();
        let current_block = sha256d::Hash::hash(&payload[offset..offset+header_length]);
        // eprintln!("Gen -> {} --> {}", hex::encode(previous_block), current_block.to_string());
        match bcblocks::is_new(current_block.to_string(), hex::encode(previous_block)) {
            Ok(idx) if idx > highest_index => {
                highest_index = idx;
                new_blocks.push(current_block.to_string());
            },
            Ok(_) => {new_blocks.push(current_block.to_string());},
            Err(()) => return Err(ProcessHeadersMessageError::UnkownBlocks)
        };
        offset+=header_length+1
    }

    match highest_index {
        0 => Err(ProcessHeadersMessageError::NoNewBlocks),
        _ => Ok(new_blocks)
    }
}

#[derive(Debug)]
pub enum ProcessBlockMessageError {
    UnkownBlock,
    BlockAlreadyDownloaded,
    Parsing(ParsingError)
}

impl From<ParsingError> for ProcessBlockMessageError {
    fn from(_: ParsingError) -> ProcessBlockMessageError {
        Parsing(ParsingError)
    }
}

pub fn process_block_message(payload: &Vec<u8>) -> Result<Block, ProcessBlockMessageError>{
    // Perf test
    //
    // let now = std::time::SystemTime::now();
    // let parsed = parse_block_sfr(&payload.clone())?;
    // let duree_sfr = now.elapsed().unwrap().as_millis();
    //
    // let now = std::time::SystemTime::now();
    // let parsed = parse_block(&payload.clone())?;
    // let duree_ori = now.elapsed().unwrap().as_millis();
    //
    // eprintln!("-> ori{}, sfr{}", duree_ori, duree_sfr);

    let parsed = parse_block(&payload)?;
    let mut blocks_mutex_guard = bcblocks::BLOCKS_MUTEX.lock().unwrap();
    let search_block = blocks_mutex_guard.known_blocks.get(&parsed.hash).cloned();
    match search_block {
        Some(found_block) => {
            let (hash, next, downloaded, _) = blocks_mutex_guard.blocks_id.get(found_block.idx).unwrap();
            if !downloaded {
                blocks_mutex_guard.blocks_id[found_block.idx] = (hash.to_string(), *next, true, false);
                return Ok(parsed)
            }
            Err(ProcessBlockMessageError::BlockAlreadyDownloaded)
        }
        _ => {
            Err(ProcessBlockMessageError::UnkownBlock)
        }
    }
}

//// COMMON SERVICES
fn get_date_time(mut time_vec: Vec<u8>) -> DateTime<Utc>{
    if time_vec.len() == 4 {
        /* La taille du champ varie dans le protocole de 4 à 8 octets */
        time_vec.append(&mut vec![0,0,0,0]);
    }
    return DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(i64::from_le_bytes(time_vec.try_into().unwrap()), 0), Utc);
}

fn compute_checksum(payload : &Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.input(payload);
    let sum = hasher.result();
    let mut hasher2 = Sha256::new();
    hasher2.input(sum);
    let result = hasher2.result();
    return result[0..4].to_vec();
}
