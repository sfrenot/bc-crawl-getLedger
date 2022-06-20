use std::io::{BufRead, BufReader};
use serde::Deserialize;
use std::fs::{self, File};
use std::path::Path;
use std::io::{self, LineWriter, stdout, Write};
use lazy_static::lazy_static;
use std::sync::Mutex;
use crate::bcblocks;
use chrono::{DateTime, Utc};
use crate::bcparse::Block;

const BLOCKS_FILE : &str = "./blocks.json";
const BLOCKS_TMP_FILE : &str = "./blocks.tmp.json";

const BLOCKS_MARKS: usize  = 10000;
const UPDATED_BLOCKS_FROM_GETBLOCK : &str = "./blocks_to_update_from_getblocks.lst";

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

fn read_block_file() -> Vec<Header> {
    eprintln!("Début lecture fichier blocks");
    serde_json::from_reader(BufReader::new(File::open(BLOCKS_FILE).unwrap())).unwrap()
}

fn create_internal_struct(blocks: Vec<Header>) {
    eprintln!("Début création structures");
    let mut idx:usize = 1;
    let mut previous: String = "".to_string();
    let mut blocks_mutex_guard = bcblocks::BLOCKS_MUTEX.lock().unwrap();
    for item in blocks {
        // eprintln!("-> {}", item.elem);
        if idx % BLOCKS_MARKS == 0 {
            eprint!("*");
            io::stderr().flush().unwrap();
        }
        blocks_mutex_guard.blocks_id.push((item.elem.clone(), item.next, item.downloaded));
        blocks_mutex_guard.known_blocks.insert(item.elem.clone(), bcblocks::BlockDesc{idx, previous});
        if item.next {
            previous = item.elem;
        } else {
            previous = "".to_string();
        }
        idx+=1;
    }
}

fn update_blocks_from_getblocks() {
    let fichier = Path::new(UPDATED_BLOCKS_FROM_GETBLOCK);
    if fichier.exists() {
        eprintln!("\nLecture fichier temporaire des blocks chargés");
        let mut blocks_mutex_guard = bcblocks::BLOCKS_MUTEX.lock().unwrap();
        let reader = BufReader::new(File::open(fichier).unwrap());
        for line in reader.lines() {
            let block = blocks_mutex_guard.known_blocks.get(&line.unwrap()).cloned().expect("Unknown hash in lock file");
            let (hash, next, _) = blocks_mutex_guard.blocks_id.get(block.idx).unwrap();
            blocks_mutex_guard.blocks_id[block.idx] = (hash.to_string(), *next, true);
        }
        fs::remove_file(fichier).unwrap();
        store_headers(&blocks_mutex_guard.blocks_id);
    }
}

pub fn load_blocks() {
    if Path::new(BLOCKS_TMP_FILE).exists() {fs::remove_file(BLOCKS_TMP_FILE).unwrap();}
    create_internal_struct(read_block_file());
    update_blocks_from_getblocks();
}

pub fn store_headers(blocks: &Vec<(String, bool, bool)>) -> bool {
    let mut file = LineWriter::new(File::create(BLOCKS_TMP_FILE).unwrap());
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
    fs::rename(BLOCKS_TMP_FILE, BLOCKS_FILE).unwrap();
    *TO_UPDATE_COUNT.lock().unwrap() = 0;
    new_blocks
}

pub fn store_block(block: &Block) {
    let dir_path = "./blocks/".to_owned() + &block.hash[block.hash.len()-2..];
    fs::create_dir_all(&dir_path).unwrap();
    let mut file = File::create(format!("{}/{}.json", dir_path, block.hash)).unwrap();
    file.write_all(serde_json::to_string_pretty(&block).unwrap().as_bytes()).unwrap();

    let mut f = File::options().append(true).create(true).open("./to_update.lock").unwrap();
    f.write_all(block.hash.as_bytes()).unwrap();
    f.write_all(b"\n").unwrap();
    *TO_UPDATE_COUNT.lock().unwrap() += 1;
}

pub fn open_logfile(file_name :&str) {
    let file: File = File::create(file_name).unwrap();
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
