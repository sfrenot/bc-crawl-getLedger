use std::io::{BufRead, BufReader};
use serde::Deserialize;
use std::fs::{self, File};
use std::io::{LineWriter, stdout, Write};
use lazy_static::lazy_static;
use std::sync::Mutex;
use crate::bcblocks;
use chrono::{DateTime, Utc};
use crate::bcblocks::Block;
use crate::bcnet::bcmessage::reverse_hash;

lazy_static! {
    pub static ref LOGGER: Mutex<LineWriter<Box<dyn Write + Send>>> = Mutex::new(LineWriter::new(Box::new(stdout())));
    // pub static ref BLOCKS: Mutex<LineWriter<Box<dyn Write + Send>>> = Mutex::new(LineWriter::new(Box::new(File::create("./blocks.raw").unwrap())));
    pub static ref SORTIE:LineWriter<File> = LineWriter::new(File::create("./blocks.raw").unwrap());
    pub static ref TO_UPDATE_COUNT: Mutex<usize> = Mutex::new(0);
}

/// Header storage
#[derive(Debug, Deserialize)]
pub struct Header {
    pub elem: String,
    pub next: bool,
    pub downloaded: bool
}

pub fn load_blocks() {
    eprintln!("Début lecture fichier blocks");
    let file = File::open("./blocks.json").unwrap();
    let blocks: Vec<Header> = serde_json::from_reader(BufReader::new(file)).unwrap();

    let mut blocks_mutex_guard = bcblocks::BLOCKS_MUTEX.lock().unwrap();

    let mut idx:usize = 1;
    let mut previous: String = "".to_string();
    for item in blocks {
        // eprintln!("-> {}", item.elem);
        blocks_mutex_guard.blocks_id.push((item.elem.clone(), item.next, item.downloaded));
        blocks_mutex_guard.known_blocks.insert(item.elem.clone(), bcblocks::BlockDesc{idx, previous});
        if item.next {
            previous = item.elem;
        } else {
            previous = "".to_string();
        }
        idx+=1;
    }

    if let Ok(f) = File::open("./to_update.lock") {
        let reader = BufReader::new(f);
        for line in reader.lines() {
            match blocks_mutex_guard.known_blocks.get(&line.unwrap()).cloned(){
                Some(block) => {
                    let (hash, next, _) = blocks_mutex_guard.blocks_id.get(block.idx).unwrap();
                    blocks_mutex_guard.blocks_id[block.idx] = (hash.to_string(), *next, true);
                }
                None => {
                    eprintln!("Unknown hash in lock file");
                    std::process::exit(1);
                }
            }
        }
        store_headers(&blocks_mutex_guard.blocks_id);
    }
    eprintln!("Fin lecture fichier blocks");
}

pub fn store_headers(blocks: &Vec<(String, bool, bool)>) -> bool {
    let mut file = LineWriter::new(File::create("./blocks-found.json").unwrap());
    let mut new_blocks = false;
    file.write_all(b"[\n").unwrap();
    for i in 1..blocks.len() {
        let (block, next, downloaded) = &blocks[i];
        file.write_all(format!("\t {{\"elem\": \"{}\", \"next\": {}, \"downloaded\": {}}}", block, next, downloaded).as_ref()).unwrap();
        if i < blocks.len()-1 {
         file.write_all(b",\n").unwrap();
        } else {
         file.write_all(b"\n").unwrap();
        }
        if !new_blocks && !*next {
            new_blocks = true;
        }
    }
    file.write_all(b"]").unwrap();
    drop(file);
    fs::rename("./blocks-found.json", "./blocks.json").unwrap();
    let _ = fs::remove_file("./to_update.lock");
    *TO_UPDATE_COUNT.lock().unwrap() = 0;
    new_blocks
}

pub fn store_block(block: &Block) {
    let rev_hash = reverse_hash(&block.hash);
    let dir_path = "./blocks/".to_owned() + &rev_hash[rev_hash.len()-2..];
    match fs::create_dir_all(&dir_path) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Error writing block to disk: {}", err);
            std::process::exit(1)
        }
    }
    let mut file = File::create(format!("{}/{}.json", dir_path, rev_hash)).unwrap();
    file.write_all(serde_json::to_string_pretty(&block).unwrap().as_bytes()).unwrap();

    let mut f = File::options().append(true).create(true).open("./to_update.lock").unwrap();
    f.write_all(rev_hash.as_bytes()).unwrap();
    f.write_all(b"\n").unwrap();
    *TO_UPDATE_COUNT.lock().unwrap() += 1;
}

/// Addr storage
pub fn open_logfile(arg_file: Option<&str>) {
    let file: File;
    match arg_file {
        None => panic!("Error parsing file name"),
        Some(f) =>  {
            file = File::create(f).unwrap();
        }
    }
    let mut logger = LOGGER.lock().unwrap();
    *logger = LineWriter::new(Box::new(file));
}

pub fn store_event(msg :&String){
    let mut guard = LOGGER.lock().unwrap();
    guard.write_all(msg.as_ref()).expect("error at logging");
}

pub fn store_version_message(target_address: &String, (_, _, _, _): (u32, Vec<u8>, DateTime<Utc>, String)){
    //TODO: supprimer le &VEc
    let mut msg: String  = String::new();
    msg.push_str(format!("Seed: {} \n", target_address).as_ref());
    // msg.push_str(format!("Seed = {}  ", target_address).as_ref());
    // msg.push_str(format!("version = {}   ", version_number).as_str());
    // msg.push_str(format!("user agent = {}   ", user_agent).as_str());
    // msg.push_str(format!("time = {}  ", peer_time.format("%Y-%m-%d %H:%M:%S")).as_str());
    // msg.push_str(format!("now = {}  ", Into::<DateTime<Utc>>::into(SystemTime::now()).format("%Y-%m-%d %H:%M:%S")).as_str());
    // msg.push_str(format!("since = {:?}  ",SystemTime::now().duration_since(SystemTime::from(peer_time)).unwrap_or_default() ).as_str());
    // msg.push_str(format!("services = {:?}\n", services ).as_str());
    store_event(&msg);
}
