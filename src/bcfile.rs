use std::io::{BufRead, BufReader};
use serde::Deserialize;
use std::fs::{self, OpenOptions, File, read_to_string};
use std::path::Path;
use std::io::{self, LineWriter, stdout, Write};
use lazy_static::lazy_static;
use std::sync::Mutex;
use crate::bcblocks;
use chrono::{DateTime, Utc};
use crate::bcparse::Block;
use fs_extra::dir::get_dir_content;
use linecount::count_lines;
use flate2::Compression;
use flate2::GzBuilder;
use std::sync::mpsc::Receiver;
use crate::bcutils::reverse_hash;

const BLOCKS_DIR: &str = "./blocks";
const HEADERS_FILE: &str = "./headers.lst";
const HEADERS_TEMP_FILE: &str = "./headers.tmp.lst";
const HEADERS_GENESIS_FILE: &str = "./headers.genesis.lst";

const BLOCKS_MARKS: usize  = 10000;
const UPDATED_HEADERS_FROM_GETBLOCK: &str = "./headers_to_update_from_getblocks.lst";

lazy_static! {
    pub static ref LOGGER: Mutex<LineWriter<Box<dyn Write + Send>>> = Mutex::new(LineWriter::new(Box::new(stdout())));
    // pub static ref BLOCKS: Mutex<LineWriter<Box<dyn Write + Send>>> = Mutex::new(LineWriter::new(Box::new(File::create("./blocks.raw").unwrap())));
    // pub static ref SORTIE:LineWriter<File> = LineWriter::new(File::create("./blocks.raw").unwrap());
    // pub static ref TO_UPDATE_COUNT: Mutex<usize> = Mutex::new(0);
    // pub static ref SORTIE:LineWriter<File> = LineWriter::new(File::create(UPDATED_BLOCKS_FROM_GETBLOCK).unwrap());
    pub static ref HEADERS_FROM_DOWNLOADED_BLOCKS: Mutex<File> = Mutex::new(File::options().append(true).create(true).open(UPDATED_HEADERS_FROM_GETBLOCK).unwrap());
    pub static ref HEADERS: Mutex<File> = Mutex::new(File::options().append(true).create(true).open(HEADERS_FILE).unwrap());
}

#[derive(Debug, Deserialize)]
pub struct Header {
    pub elem: String,
    pub next: bool,
    pub downloaded: bool
}

fn read_block_file_at_startup() -> String {
    eprintln!("Début lecture fichier headers");
    if !Path::new(HEADERS_FILE).exists() {
        fs::copy(HEADERS_GENESIS_FILE, HEADERS_FILE).unwrap();
    }
    read_to_string(HEADERS_FILE).unwrap()
}

fn create_internal_struct_at_startup(headers: String) {
    eprintln!("Début création structures");
    let mut idx: usize = 1;
    let mut previous: String = "".to_string();
    let mut blocks_mutex_guard = bcblocks::BLOCKS_MUTEX.lock().unwrap();

    let mut it = headers.lines().peekable();
    while let Some(header) = it.next() {
        if idx % BLOCKS_MARKS == 0 {
            eprint!("*");
            io::stderr().flush().unwrap();
        }

        let next = it.peek().is_some();
        let reversed = reverse_hash(header);

        blocks_mutex_guard.blocks_id.push((reversed.clone(), next, false, false));
        blocks_mutex_guard.known_blocks.insert(reversed.clone(), bcblocks::BlockDesc{idx, previous});

        previous = reversed;
        idx += 1;
    }
}

fn inject_downloaded_headers_from_previous_run_at_startup() {
    eprintln!("Lecture fichier temporaire des blocks chargés");
    let mut blocks_mutex_guard = bcblocks::BLOCKS_MUTEX.lock().unwrap();
    let reader = BufReader::new(OpenOptions::new().append(true).read(true).create(true).open(Path::new(UPDATED_HEADERS_FROM_GETBLOCK)).unwrap());

    for line in reader.lines() {
        let l = reverse_hash(&line.unwrap());
        match blocks_mutex_guard.known_blocks.get(&l).cloned() {
            Some(block) => {
                let (hash, next, _, _) = blocks_mutex_guard.blocks_id.get(block.idx).unwrap();
                blocks_mutex_guard.blocks_id[block.idx] = (hash.to_string(), *next, true, false);

            },
            None => {
                eprintln!("Block inconnu {}", &l);
                std::process::exit(1);
            }
        }

    }
    update_headers_file(&blocks_mutex_guard.blocks_id);
    HEADERS_FROM_DOWNLOADED_BLOCKS.lock().unwrap().set_len(0).unwrap();
}

pub fn load_headers_at_startup() {
    if Path::new(HEADERS_TEMP_FILE).exists() { fs::remove_file(HEADERS_TEMP_FILE).unwrap() }
    if !Path::new(BLOCKS_DIR).exists() { fs::create_dir(BLOCKS_DIR).unwrap() }
    create_internal_struct_at_startup(read_block_file_at_startup());
    inject_downloaded_headers_from_previous_run_at_startup();
}

fn update_headers_file(headers: &Vec<(String, bool, bool, bool)>) {
    let mut file = LineWriter::new(File::create(HEADERS_TEMP_FILE).unwrap());
    let mut idx = 0;
    for (hash, next, downloaded, _) in headers {
        if idx == 0 { idx +=1; continue; } // First record is 00000...
        if !downloaded || !next {
            file.write_all(reverse_hash(hash).as_bytes()).unwrap();
            file.write_all(b"\n").unwrap();
        }
        idx += 1;
    }
    file.flush().unwrap();
    fs::rename(HEADERS_TEMP_FILE, HEADERS_FILE).unwrap();
}

pub fn store_headers(headers: Vec<String>) {
    let mut out = HEADERS.lock().unwrap();
    for header in headers {
        out.write_all(reverse_hash(&header).as_bytes()).unwrap();
        out.write_all(b"\n").unwrap();
    }
    out.flush().unwrap();
}

pub fn store_block(block_channel: Receiver<Block>) {
    for block in block_channel.iter() {

        // eprintln!("Storing {}",block.hash);
        let rev_hash = reverse_hash(&block.hash);
        let dir_path = format!("./{}/{}/{}", BLOCKS_DIR, &rev_hash[rev_hash.len()-2..], &rev_hash[rev_hash.len()-4..rev_hash.len()-2]);
        fs::create_dir_all(&dir_path).unwrap();

        let file = File::create(format!("{}/{}.json.gz", dir_path, &rev_hash[..rev_hash.len()-4])).unwrap();
        let mut gz = GzBuilder::new()
                    .write(file, Compression::default());
        gz.write_all(serde_json::to_string_pretty(&block).unwrap().as_bytes()).unwrap();
        gz.finish().unwrap();

        let mut out = HEADERS_FROM_DOWNLOADED_BLOCKS.lock().unwrap();
        out.write_all(rev_hash.as_bytes()).unwrap();
        out.write_all(b"\n").unwrap();
        out.flush().unwrap();
        // std::process::exit(1);
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
    guard.flush().unwrap();
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

pub fn get_vols() -> (usize, usize){
    (count_lines(File::open(HEADERS_FILE).unwrap()).unwrap(), get_dir_content(BLOCKS_DIR).unwrap().files.len())
}
