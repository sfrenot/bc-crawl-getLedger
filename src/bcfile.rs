use std::io::{BufRead, BufReader};
use serde::Deserialize;
use std::fs::{self, OpenOptions, File};
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

const BLOCKS_DIR: &str = "./blocks";
const BLOCKS_FILE: &str = "./blocks.json";
const BLOCKS_TMP_FILE: &str = "./blocks.tmp.json";

const BLOCKS_MARKS: usize  = 10000;
// const FLUSH_SIZE: u64 = 3200;
const UPDATED_BLOCKS_FROM_GETBLOCK : &str = "./blocks_to_update_from_getblocks.lst";
const UPDATED_BLOCKS_FROM_GETHEADERS : &str = "./blocks_to_update_from_getheaders.lst";

lazy_static! {
    pub static ref LOGGER: Mutex<LineWriter<Box<dyn Write + Send>>> = Mutex::new(LineWriter::new(Box::new(stdout())));
    // pub static ref BLOCKS: Mutex<LineWriter<Box<dyn Write + Send>>> = Mutex::new(LineWriter::new(Box::new(File::create("./blocks.raw").unwrap())));
    // pub static ref SORTIE:LineWriter<File> = LineWriter::new(File::create("./blocks.raw").unwrap());
    // pub static ref TO_UPDATE_COUNT: Mutex<usize> = Mutex::new(0);
    // pub static ref SORTIE:LineWriter<File> = LineWriter::new(File::create(UPDATED_BLOCKS_FROM_GETBLOCK).unwrap());
    pub static ref HEADERS_FROM_DOWNLOADEDBLOCKS: Mutex<File> = Mutex::new(File::options().append(true).create(true).open(UPDATED_BLOCKS_FROM_GETBLOCK).unwrap());
    pub static ref HEADERS_FROM_GETHEADERS: Mutex<File> = Mutex::new(File::options().append(true).create(true).open(UPDATED_BLOCKS_FROM_GETHEADERS).unwrap());
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
        blocks_mutex_guard.blocks_id.push((item.elem.clone(), item.next, item.downloaded, false));
        blocks_mutex_guard.known_blocks.insert(item.elem.clone(), bcblocks::BlockDesc{idx, previous});
        if item.next {
            previous = item.elem;
        } else {
            previous = "".to_string();
        }
        idx+=1;
    }
}

fn inject_new_headers_from_previous_run_at_startup() {
    eprintln!("\nLecture fichier temporaire des headers chargés");
    let mut blocks_mutex_guard = bcblocks::BLOCKS_MUTEX.lock().unwrap();
    let file = OpenOptions::new().append(true).read(true).create(true).open(Path::new(UPDATED_BLOCKS_FROM_GETHEADERS)).unwrap();
    if file.metadata().unwrap().len() > 0 {
        let reader = BufReader::new(file);

        let (previous, next, _, _) = blocks_mutex_guard.blocks_id.last_mut().unwrap();
        *next = true;
        let mut prev = previous.clone();

        for line in reader.lines() {
            let next_block = line.unwrap();
            // eprintln!("->{}", next_block);
            // std::process::exit(1);
            blocks_mutex_guard.blocks_id.push((next_block.clone(), true, false, false));
            let size = blocks_mutex_guard.blocks_id.len()-1;
            let elem = bcblocks::BlockDesc{idx: size, previous: prev.clone()};
            blocks_mutex_guard.known_blocks.insert(next_block.clone(), elem);
            prev = next_block;
        }

        let (_, next, _, _) = blocks_mutex_guard.blocks_id.last_mut().unwrap();
        *next = false;

    }
    // eprintln!("{:?}",blocks_mutex_guard.blocks_id );
    // // eprintln!("********");
    // // eprintln!("{:?}", blocks_mutex_guard.known_blocks);
    //
    // std::process::exit(1);


}

fn inject_downloaded_headers_from_previous_run_at_startup() {
    eprintln!("Lecture fichier temporaire des blocks chargés");
    let mut blocks_mutex_guard = bcblocks::BLOCKS_MUTEX.lock().unwrap();
    let reader = BufReader::new(OpenOptions::new().append(true).read(true).create(true).open(Path::new(UPDATED_BLOCKS_FROM_GETBLOCK)).unwrap());

    for line in reader.lines() {
        let l = line.unwrap();
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
    store_headers(&blocks_mutex_guard.blocks_id);
}

pub fn load_headers_at_startup() {
    if Path::new(BLOCKS_TMP_FILE).exists() {fs::remove_file(BLOCKS_TMP_FILE).unwrap();}
    create_internal_struct_at_startup(read_block_file_at_startup());
    inject_new_headers_from_previous_run_at_startup();
    inject_downloaded_headers_from_previous_run_at_startup();
}

fn store_headers(headers: &Vec<(String, bool, bool, bool)>) {
    let mut file = LineWriter::new(File::create(BLOCKS_TMP_FILE).unwrap());
    file.write_all(b"[\n").unwrap();
    let mut idx = 0;
    for (block, next, downloaded, _) in headers {
        if idx == 0 {idx +=1; continue;} // First record is 00000...
        if !downloaded || !next {
            file.write_all(format!("\t {{\"elem\": \"{}\", \"next\": {}, \"downloaded\": {}, \"donwloading\": false}}", block, next, downloaded).as_ref()).unwrap();
            if idx < headers.len()-1 { // Last record has no ,
                file.write_all(b",\n").unwrap();
            }
        }
        idx += 1;
    }
    file.write_all(b"\n]").unwrap();
    file.flush();
    fs::rename(BLOCKS_TMP_FILE, BLOCKS_FILE).unwrap();
    HEADERS_FROM_DOWNLOADEDBLOCKS.lock().unwrap().set_len(0).unwrap();
    HEADERS_FROM_GETHEADERS.lock().unwrap().set_len(0).unwrap();
}

pub fn store_headers2(headers: Vec<String>) {
    let mut out = HEADERS_FROM_GETHEADERS.lock().unwrap();
    for header in headers {
        out.write_all(header.as_bytes()).unwrap();
        out.write_all(b"\n").unwrap();
    }
    out.flush();
}

pub fn store_block(block_channel: Receiver<Block>) {
    for block in block_channel.iter() {

        // eprintln!("Storing {}",block.hash);
        let dir_path = format!("./{}/{}", BLOCKS_DIR, &block.hash[block.hash.len()-3..]);
        fs::create_dir_all(&dir_path).unwrap();

        let file = File::create(format!("{}/{}.json.gz", dir_path, &block.hash[..block.hash.len()-3])).unwrap();
        let mut gz = GzBuilder::new()
                    .write(file, Compression::default());
        gz.write_all(serde_json::to_string_pretty(&block).unwrap().as_bytes()).unwrap();
        gz.finish().unwrap();

        let mut out = HEADERS_FROM_DOWNLOADEDBLOCKS.lock().unwrap();
        out.write_all(block.hash.as_bytes()).unwrap();
        out.write_all(b"\n").unwrap();
        out.flush().unwrap();
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
    guard.flush();
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

pub fn get_vols() -> (usize, usize, usize){
    (count_lines(File::open(BLOCKS_FILE).unwrap()).unwrap(), count_lines(File::open(UPDATED_BLOCKS_FROM_GETHEADERS).unwrap()).unwrap(), get_dir_content(BLOCKS_DIR).unwrap().files.len())
}
