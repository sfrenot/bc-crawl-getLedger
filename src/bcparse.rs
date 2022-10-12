use std::fmt;
use bitcoin_hashes::{Hash, sha256d};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Visitor};
use crate::bcutils::{get_compact_int, reverse_hash};
use byteorder::{ReadBytesExt, LittleEndian};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Block {
    #[serde(serialize_with = "serialize_hash", deserialize_with = "deserialize_hash")]
    pub hash: String,
    pub version: i32,
    #[serde(serialize_with = "serialize_hash", deserialize_with = "deserialize_hash")]
    pub prev_hash: String,
    #[serde(serialize_with = "serialize_hash", deserialize_with = "deserialize_hash")]
    pub merkle_root: String,
    pub timestamp: u32,
    pub bits: u32,
    pub nonce: u32,
    pub txns: Vec<Tx>
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Transaction {
    #[serde(serialize_with = "serialize_hash", deserialize_with = "deserialize_hash")]
    pub hash: String,
    pub version: i32,
    pub is_segwit: bool,
    pub inputs: Vec<Tx>,
    pub outputs: Vec<Tx>,
    pub witnesses: Vec<Vec<Tx>>,
    pub lock_time: u32
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TxInput {
    pub prev_output: OutPoint,
    pub signature_script: String,
    pub sequence: u32
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct OutPoint {
    #[serde(serialize_with = "serialize_hash", deserialize_with = "deserialize_hash")]
    pub hash: String,
    pub idx: u32
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TxOutput {
    pub value: i64,
    pub pub_key_script: String
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct WitnessItem {
    pub script: String
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Tx {
    Transaction(Transaction),
    TxInput(TxInput),
    TxOutput(TxOutput),
    WitnessItem(WitnessItem)
}

enum TxKind {
    Transaction,
    TxInput,
    TxOutput,
    WitnessItem
}

#[derive(Debug)]
pub struct ParsingError;

// custom serialization and deserialization
fn serialize_hash<S>(hash: &str, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
    s.serialize_str(&reverse_hash(hash))
}

struct HashVisitor;
impl<'de> Visitor<'de> for HashVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a bitcoin hash")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E> where E: Error {
        let res = reverse_hash(s);
        Ok(res)
    }
}

fn deserialize_hash<'de, D>(d: D) -> Result<String, D::Error> where D: Deserializer<'de> {
    d.deserialize_string(HashVisitor)
}

struct Payload<'a> {
    pl: &'a[u8],
    off: usize
}

impl Payload<'_> {
    fn read_u32(&mut self) -> u32 {
        self.off+=4;
        self.pl.get(self.off-4..self.off).unwrap().read_u32::<LittleEndian>().unwrap()
    }
    fn read_i32(&mut self) -> i32 {
        self.off+=4;
        self.pl.get(self.off-4..self.off).unwrap().read_i32::<LittleEndian>().unwrap()
    }
    fn read_i64(&mut self) -> i64 {
        self.off+=8;
        self.pl.get(self.off-8..self.off).unwrap().read_i64::<LittleEndian>().unwrap()
    }
    fn encode_addr(&mut self) -> String {
        self.off+=32;
        hex::encode(self.pl.get(self.off-32..self.off).unwrap())
    }
    fn encode_string(&mut self, length: usize) -> String {
        self.off+=length;
        hex::encode(self.pl.get(self.off-length..self.off).unwrap())
    }
    fn get_compact_int(&mut self) -> usize {
        let (txn_count, off) = get_compact_int(&self.pl[self.off..]);
        self.off += off;
        txn_count as usize
    }
}

fn block_hash(tx: &Payload) -> String {
    hex::encode(sha256d::Hash::hash(&tx.pl[..80]))
}

fn tx_hash(tx: &Payload, from: usize) -> String {
    hex::encode(sha256d::Hash::hash(&tx.pl[from..tx.off]))
}

fn segwit_hash(tx: &Payload, from: usize, txs_offset: usize) -> String {

    let tmp = &[&tx.pl[from..from+4],
          &tx.pl[from+6..txs_offset],
          &tx.pl[tx.off-4..tx.off]].concat();

    hex::encode(sha256d::Hash::hash(tmp))
}

fn is_segwit(tx: &Payload) -> bool {
    tx.pl.get(tx.off+4..tx.off+6).ok_or(ParsingError).unwrap() == &[0x00, 0x01]
}

fn tx_loop(txns_pl: &mut Payload, txn_count: usize, kind: TxKind) -> Vec<Tx> {
    let mut txns = Vec::new();
    for _ in 0..txn_count {
        match kind {
            TxKind::Transaction => {
                let txn = match is_segwit(txns_pl) {
                    true =>  parse_segwit_tx(txns_pl).unwrap(),
                    false => parse_standard_tx(txns_pl).unwrap()
                };
                txns.push(Tx::Transaction(txn));
            },
            TxKind::TxInput => {
                let txn = parse_tx_input(txns_pl).unwrap();
                txns.push(Tx::TxInput(txn));
            },
            TxKind::TxOutput => {
                let txn = parse_tx_output(txns_pl).unwrap();
                txns.push(Tx::TxOutput(txn));
            },
            TxKind::WitnessItem => {
                let txn = parse_witness_item(txns_pl).unwrap();
                txns.push(Tx::WitnessItem(txn));
            }
        };
    }
    txns
}

fn get_main_transactions(txs: &mut Payload) -> Vec<Tx> {
    let tx_count = txs.get_compact_int();
    tx_loop(txs, tx_count, TxKind::Transaction)
}

fn parse_segwit_tx(raw_tx: &mut Payload) -> Result<Transaction, ParsingError> {
    // let mut offset = 4;
    let offset_in_out:usize;
    let len_in:usize;
    let start = raw_tx.off;

    return Ok(Transaction{
        is_segwit: true,
        version: raw_tx.read_i32(),
        inputs: {
            raw_tx.off += 2;
            let tx_count = raw_tx.get_compact_int();
            let txn = tx_loop(raw_tx, tx_count, TxKind::TxInput);

            // let (txn, offset_in) = get_transactions(payload.get(offset..).ok_or(ParsingError)?, TxKind::TxInput)?;
            len_in = txn.len();
            // offset += offset_in;
            txn
        },
        outputs: {
            let tx_count = raw_tx.get_compact_int();
            let txn = tx_loop(raw_tx, tx_count, TxKind::TxOutput);
            offset_in_out = raw_tx.off;
            txn

        },
        witnesses: {
            let mut witnesses = Vec::new();
            for _ in 0..len_in {
                let tx_count = raw_tx.get_compact_int();
                let data = tx_loop(raw_tx, tx_count, TxKind::WitnessItem);
                witnesses.push(data);
            };
            witnesses
        },
        lock_time: raw_tx.read_u32(),
        hash: segwit_hash(raw_tx, start, offset_in_out)
    });
}

fn parse_standard_tx(raw_tx: &mut Payload) -> Result<Transaction, ParsingError> {
    let from = raw_tx.off;

    return Ok(Transaction{
        is_segwit: false,
        version: raw_tx.read_i32(),
        inputs: {
            let count = raw_tx.get_compact_int();
            tx_loop(raw_tx, count, TxKind::TxInput)
        },
        outputs: {
            let count = raw_tx.get_compact_int();
            tx_loop(raw_tx, count, TxKind::TxOutput)
        },
        witnesses: vec!(),
        lock_time: raw_tx.read_u32(),
        hash: tx_hash(raw_tx, from)
    });
}

fn parse_tx_input(tx_input: &mut Payload) -> Result<TxInput, ParsingError> {

    return Ok(TxInput {
        prev_output: OutPoint {
            hash: tx_input.encode_addr(),
            idx: tx_input.read_u32()
        },
        signature_script: {
            let script_length = tx_input.get_compact_int();
            tx_input.encode_string(script_length)
        },
        sequence: tx_input.read_u32()
    });
}

fn parse_tx_output(tx_output: &mut Payload) -> Result<TxOutput, ParsingError> {
    return Ok(TxOutput{
        value: tx_output.read_i64(),
        pub_key_script: {
            let script_length = tx_output.get_compact_int();
            tx_output.encode_string(script_length)
        }
    });
}

fn parse_witness_item(tx_witness: &mut Payload) -> Result<WitnessItem, ParsingError> {
    let length = tx_witness.get_compact_int();
    return Ok(WitnessItem{
        script: tx_witness.encode_string(length)
    });
}

//Public Entry
pub fn parse_block(payload: &[u8]) -> Result<Block, ParsingError> {
    let mut block = Payload{ pl: payload, off: 0};
    return Ok(Block {
        hash: block_hash(&block),
        version: block.read_i32(),
        prev_hash: block.encode_addr(),
        merkle_root: block.encode_addr(),
        timestamp: block.read_u32(),
        bits: block.read_u32(),
        nonce: block.read_u32(),
        txns: get_main_transactions(&mut block)
    })
}
