// mod bcmessage;
mod bcblocks;
mod bcfile;
mod bcnet;
mod bcpeers;
mod bcparse;

use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::ResolverConfig;
use trust_dns_resolver::config::ResolverOpts;

use std::sync::mpsc;
use std::sync::atomic::Ordering;

use std::thread;
use std::process;

use std::time::{Duration, SystemTime};

const CHECK_TERMINATION_TIMEOUT:Duration = Duration::from_secs(5);
const THREADS: u64 = 100;
const MESSAGE_CHANNEL_SIZE: usize = 100000;
const DNS_START: &str = "seed.btc.petertodd.org";
const PORT_START: &str = "8333";
const LOG_FILE: &str = "./file.txt";

fn main() {
    bcfile::open_logfile(LOG_FILE);
    bcfile::load_headers_at_startup();
    bcblocks::create_block_message_payload(&bcblocks::BLOCKS_MUTEX.lock().unwrap().blocks_id);
    bcblocks::create_getdata_message_payload(&bcblocks::BLOCKS_MUTEX.lock().unwrap().blocks_id);

    // eprintln!("{}", hex::encode(bcblocks::get_getblock_message_payload()));
    // eprintln!("{}", hex::encode(bcblocks::get_getheaders_message_payload()));
    // std::process::exit(1);

    // eprintln!("{:?}", known_block);
    // eprintln!("{:?}", bcblocks::BLOCKS_ID.lock().unwrap());
    // std::process::exit(1);

    let (address_channel_sender, address_channel_receiver) = mpsc::channel();
    let (connecting_start_channel_sender, connecting_start_channel_receiver) = chan::sync(MESSAGE_CHANNEL_SIZE);

    thread::spawn(move || { check_pool_size(SystemTime::now()); });

    for i in 0..THREADS {
        let sender = address_channel_sender.clone();
        let recv = connecting_start_channel_receiver.clone();
        thread::spawn(move || { bcnet::handle_one_peer(recv, sender, i);});
    }

    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
    let mut initial_addresses: Vec<String>= Vec::new();
    for node_addr in resolver.lookup_ip(DNS_START).unwrap() {
        initial_addresses.push(format!("{}:{}", node_addr, PORT_START));
    }
    bcpeers::check_addr_messages(initial_addresses, &address_channel_sender);

    loop {
        let new_peer: String = address_channel_receiver.recv().unwrap();
        bcpeers::NB_ADDR_TO_TEST.fetch_add(1, Ordering::Relaxed);
        connecting_start_channel_sender.send(new_peer);
    }
}

fn check_pool_size(start_time: SystemTime ){
    loop {
        thread::sleep(CHECK_TERMINATION_TIMEOUT);
        bcpeers::get_peers_status();
        if bcpeers::NB_ADDR_TO_TEST.load(Ordering::Relaxed) < 1 {
            let time_spent = SystemTime::now().duration_since(start_time).unwrap_or_default();
            println!("POOL Crawling ends in {:?} ", time_spent);
            process::exit(0);
        }
    }
}
