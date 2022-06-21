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

const BLOCKS_DIR: &str = "./blocks";
const BLOCKS_FILE: &str = "./blocks.json";
const BLOCKS_TMP_FILE: &str = "./blocks.tmp.json";

const BLOCKS_MARKS: usize  = 10000;
const FLUSH_SIZE: u64 = 3200;
const UPDATED_BLOCKS_FROM_GETBLOCK : &str = "./blocks_to_update_from_getblocks.lst";

lazy_static! {
    pub static ref LOGGER: Mutex<LineWriter<Box<dyn Write + Send>>> = Mutex::new(LineWriter::new(Box::new(stdout())));
    // pub static ref BLOCKS: Mutex<LineWriter<Box<dyn Write + Send>>> = Mutex::new(LineWriter::new(Box::new(File::create("./blocks.raw").unwrap())));
    // pub static ref SORTIE:LineWriter<File> = LineWriter::new(File::create("./blocks.raw").unwrap());
    // pub static ref TO_UPDATE_COUNT: Mutex<usize> = Mutex::new(0);
    // pub static ref SORTIE:LineWriter<File> = LineWriter::new(File::create(UPDATED_BLOCKS_FROM_GETBLOCK).unwrap());
    pub static ref HEADERS_FROM_BLOCKS: Mutex<File> = Mutex::new(File::options().append(true).create(true).open(UPDATED_BLOCKS_FROM_GETBLOCK).unwrap());
}

#[derive(Debug, Deserialize)]
pub struct Header {
    pub elem: String,
    pub next: bool,
    pub downloaded: bool
}

fn read_block_file_at_startup() -> Vec<Header> {
    eprintln!("Début lecture fichier blocks");
    serde_json::from_reader(BufReader::new(File::open(BLOCKS_FILE).unwrap())).unwrap()
}

fn create_internal_struct_at_startup(blocks: Vec<Header>) {
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

fn inject_pending_headers_from_previous_run_at_startup() {
    eprintln!("\nLecture fichier temporaire des blocks chargés");
    let mut blocks_mutex_guard = bcblocks::BLOCKS_MUTEX.lock().unwrap();
    let reader = BufReader::new(File::open(Path::new(UPDATED_BLOCKS_FROM_GETBLOCK)).unwrap());

    for line in reader.lines() {
        let block = blocks_mutex_guard.known_blocks.get(&line.unwrap()).cloned().expect("Unknown hash in lock file");
        let (hash, next, _) = blocks_mutex_guard.blocks_id.get(block.idx).unwrap();
        blocks_mutex_guard.blocks_id[block.idx] = (hash.to_string(), *next, true);
    }
    store_headers(&blocks_mutex_guard.blocks_id);
}

pub fn load_headers_at_startup() {
    if Path::new(BLOCKS_TMP_FILE).exists() {fs::remove_file(BLOCKS_TMP_FILE).unwrap();}
    create_internal_struct_at_startup(read_block_file_at_startup());
    inject_pending_headers_from_previous_run_at_startup();
}

pub fn store_headers(headers: &Vec<(String, bool, bool)>) {
    let mut file = LineWriter::new(File::create(BLOCKS_TMP_FILE).unwrap());
    file.write_all(b"[\n").unwrap();
    for i in 1..headers.len() {
        let (block, next, downloaded) = &headers[i];
        file.write_all(format!("\t {{\"elem\": \"{}\", \"next\": {}, \"downloaded\": {}}}", block, next, downloaded).as_ref()).unwrap();
        if i < headers.len()-1 {
         file.write_all(b",\n").unwrap();
        } else {
         file.write_all(b"\n").unwrap();
        }
    }
    file.write_all(b"]").unwrap();
    fs::rename(BLOCKS_TMP_FILE, BLOCKS_FILE).unwrap();

    HEADERS_FROM_BLOCKS.lock().unwrap().set_len(0).unwrap();
}

pub fn store_block(blocks_id: &Vec<(String, bool, bool)>, block: &Block) {
    let dir_path = format!("./{}/{}", BLOCKS_DIR, &block.hash[block.hash.len()-2..]);
    fs::create_dir_all(&dir_path).unwrap();
    let mut file = File::create(format!("{}/{}.json", dir_path, block.hash)).unwrap();
    file.write_all(serde_json::to_string_pretty(&block).unwrap().as_bytes()).unwrap();

    let mut out = HEADERS_FROM_BLOCKS.lock().unwrap();
    out.write_all(block.hash.as_bytes()).unwrap();
    out.write_all(b"\n").unwrap();

    if *&out.metadata().unwrap().len() > FLUSH_SIZE {
        drop(out);
        store_headers(&blocks_id);
    }
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
