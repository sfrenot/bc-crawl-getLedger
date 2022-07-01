pub mod bcmessage;

use std::sync::Mutex;
use lazy_static::lazy_static;
use std::time::Duration;
use std::sync::mpsc::Sender;
use std::sync::atomic::Ordering;
use std::io::Write;
use std::io::Error;
use std::io::ErrorKind;
use pad::{PadStr, Alignment};
use crate::bcparse::Block;

use std::net::{SocketAddr, TcpStream};
use chan::Receiver;

// use crate::bcmessage::{ReadResult, INV, MSG_VERSION, MSG_VERSION_ACK, MSG_GETADDR, CONN_CLOSE, MSG_ADDR, HEADERS, GET_BLOCKS, BLOCK, GET_DATA};
use bcmessage::{MSG_VERSION, MSG_VERSION_ACK, MSG_GETADDR, CONN_CLOSE, MSG_ADDR, GET_HEADERS, HEADERS, BLOCK, GET_DATA};
use crate::bcfile as bcfile;
use crate::bcblocks as bcblocks;
use crate::bcpeers as bcpeers;
use std::collections::HashMap;

const CONNECTION_TIMEOUT:Duration = Duration::from_secs(10);
// const READ_MESSAGE_TIMEOUT:Duration = Duration::from_secs(2);
const MIN_ADDRESSES_RECEIVED_THRESHOLD: usize = 5;
const NB_MAX_READ_ON_SOCKET:usize = 20;

// Debugger
static mut NODES_STATUS:[([u8; 15], u64, u64, u64, u64, u64); crate::THREADS as usize]= [([0; 15], 0, 0, 0, 0, 0); crate::THREADS as usize];
lazy_static! {
    pub static ref NB_NOEUDS_CONNECTES:Mutex<HashMap<u8, usize>> = Mutex::new(HashMap::new());
}

pub fn handle_one_peer(connection_start_channel: Receiver<String>, address_channel_tx: Sender<String>, block_sender: Sender<Block>, num: u8){
    loop{ //Node Management
        let target_address = connection_start_channel.recv().unwrap();
        let mut status: &String = &MSG_VERSION; // Start from this status

        // eprintln!("Connexion {}, {}", _num, target_address);
        let socket: SocketAddr = target_address.parse().unwrap();
        match TcpStream::connect_timeout(&socket, CONNECTION_TIMEOUT) {
            Err(_) => bcpeers::fail(target_address.clone()),
            Ok(connection) => {
                // connection.set_read_timeout(Some(READ_MESSAGE_TIMEOUT)).unwrap();
                loop {
                   // eprintln!("Avant Activation {}, {}", target_address.clone(), status);
                   status = match activate_peer(&num, &connection, &status, &address_channel_tx, &block_sender, &target_address) {
                       Err(e) => {
                           match e.kind() {
                               ErrorKind::Other => {
                                   // eprintln!("Fin du noeud: {}: {}", e, target_address);
                                   bcpeers::done(target_address.clone());
                               },
                               _ => {
                                   // eprintln!("Error sending request: {}: {}", e, target_address);
                                   bcpeers::fail(target_address.clone());
                               }
                           }
                           break;
                       },
                       Ok(new_status) =>{ &new_status }
                   }
                } // loop for node
            }
        };
        // eprintln!("Connecté {}, {}", num, target_address);
        // eprintln!("Fin gestion {}", target_address);
        bcpeers::NB_ADDR_TO_TEST.fetch_sub(1, Ordering::Relaxed);
    }
}

fn handle_incoming_message<'a>(_num: &u8, connection:& TcpStream, sender: &Sender<String>, block_sender: &Sender<Block>, target_address: &String) -> &'a String  {
    let mut lecture:usize = 0; // Garde pour éviter connection infinie inutile
    loop {

        // println!("Lecture de {} thread {}", target_address, num);
        match bcmessage::read_message(&connection) {
            Err(_error) => return &CONN_CLOSE,
            Ok((command, payload)) => {
                //eprintln!("Command From : {} --> {}, payload : {}", &target_address, &command, payload.len());
                // if payload.len() <= 0 { panic!("Payload nul");}
                match command {
                    cmd if cmd == *MSG_VERSION  => {
                        handle_incoming_cmd_version(&target_address, &payload);
                        return &MSG_VERSION;
                    },
                    cmd if cmd == *MSG_VERSION_ACK
                        => return &MSG_VERSION_ACK,
                    cmd if cmd == *MSG_ADDR && handle_incoming_cmd_msg_addr(&payload, &sender)
                        => return &MSG_GETADDR,
                    cmd if cmd == *HEADERS
                        => return match handle_incoming_cmd_msg_header(&payload, &mut lecture) {
                            true  => &GET_HEADERS,
                            false => &CONN_CLOSE
                        },
                    cmd if cmd == *BLOCK
                        => return match handle_incoming_cmd_msg_block(&payload, &mut lecture, &block_sender) {
                        true => &GET_DATA,
                        false => &CONN_CLOSE
                    },
                    _ => {}
                };
            }
        };
        lecture+=1;
        if lecture > NB_MAX_READ_ON_SOCKET {
            eprintln!("Sortie du noeud : trop de lectures inutiles");
            return &CONN_CLOSE;
        }
    }
    // eprintln!("Fermeture {}", target_address);
}

// TODO: -> has a hashmap
fn next_status(from: &String) -> &String {
    match from {
        elem if *elem == *MSG_VERSION => {&MSG_VERSION_ACK},
        elem if *elem == *MSG_VERSION_ACK => {&MSG_GETADDR},
        elem if *elem == *MSG_GETADDR => {&GET_HEADERS},
        elem if *elem == *GET_HEADERS => {&GET_DATA},
        elem if *elem == *GET_DATA => {&GET_DATA},
        _ => {&CONN_CLOSE}
    }
}

fn trace(num: &u8, target: &String, current: &String) {
    unsafe {
        let (mut node, run, mut ver, mut addr, mut head, mut data) = NODES_STATUS[*num as usize];
        match current {
            current if current == "version" => ver += 1,
            current if current == "getaddr" => addr += 1,
            current if current == "getheaders" => head += 1,
            current if current == "getdata" => data += 1,
            _ => ()
        };

        node.copy_from_slice(&target.pad_to_width_with_alignment(20, Alignment::Right).as_bytes()[0..15]);

        // eprint!("{} -> ", &num);
        NODES_STATUS[*num as usize] = (node, run+1, ver, addr, head, data);
        for (_a, _b, _c, _d, _e, f) in NODES_STATUS {
            if f > 2 {
                NB_NOEUDS_CONNECTES.lock().unwrap().insert(*num, 1);
            }
            // eprint!("({}/{},{},{},{},{})", String::from_utf8_lossy(&a).trim(), b, c, d, e, f);
        }
    }
}

fn activate_peer<'a>(num: &u8, mut connection: &TcpStream, current: &'a String, sender: &Sender<String>, block_sender: &Sender<Block>, target: &String) -> Result<&'a String, Error> {
    // // Trace function
    trace(&num, &target, &current);

    connection.write(bcmessage::build_request(current).as_slice()).unwrap();

    match handle_incoming_message(num, connection, sender, block_sender, target) {
        res if *res == *CONN_CLOSE => Err(Error::new(ErrorKind::Other, format!("Connexion terminée {} <> {}", current, res))),
        res if *res == *current => Ok(next_status(current)),
        // res if *res == *MSG_GETADDR && *current == *GET_HEADERS => Ok(current), // Remote node answers many times the same thing
        res => Err(Error::new(ErrorKind::ConnectionReset, format!("Wrong message {} <> {}", current, res)))
    }
}

// Incoming messages
fn handle_incoming_cmd_version(peer: &String, payload: &Vec<u8>) {
    bcfile::store_version_message(peer, bcmessage::process_version_message(payload));
    bcpeers::register_peer_connection(peer);
}

fn handle_incoming_cmd_msg_addr(payload: &Vec<u8>, sender: &Sender<String>) -> bool {
    bcpeers::check_addr_messages(bcmessage::process_addr_message(&payload), &sender) > MIN_ADDRESSES_RECEIVED_THRESHOLD
}

fn handle_incoming_cmd_msg_header(payload: &Vec<u8>, lecture: &mut usize) -> bool {
    // eprintln!("Status : {} -> {}", idx, block);
    match bcmessage::process_headers_message(payload) {
        Ok(blocks) => {
            // eprintln!("-> {:?}", blocks);
            bcfile::store_headers2(blocks);
            bcblocks::create_block_message_payload();
            // eprintln!("new payload -> {:02x?}", hex::encode(&bcblocks::get_getheaders_message_payload()));
            // eprintln!("new payload");
            *lecture = 0;
            true
        },
        Err(err) => {
            match err {
                bcmessage::ProcessHeadersMessageError::UnkownBlocks => {
                    eprintln!("Sortie du noeud");
                    false
                },
                _ => {
                    // eprintln!("Erreur -> {:?}", err);
                    // std::process::exit(1);
                    true
                }
            }
        }
    }
}

fn handle_incoming_cmd_msg_block(payload: &Vec<u8>, lecture: &mut usize, block_sender: &Sender<Block>) -> bool {
    match bcmessage::process_block_message(payload) {
        Ok(block) => {
            block_sender.send(block).unwrap();
            *lecture = 0;
            // eprintln!("new block stored");
            true
        },
        Err(e) => {
            match e {
                bcmessage::ProcessBlockMessageError::UnkownBlock => {
                    eprintln!("Error processing block message: Unknown Block");
                    false
                },
                bcmessage::ProcessBlockMessageError::Parsing(..) => {
                    eprintln!("Error processing block message: Parsing Error");
                    false
                },

                bcmessage::ProcessBlockMessageError::BlockAlreadyDownloaded => {
                    true
                }
            }
        }
    }
}
