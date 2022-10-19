use std::fmt;

use bitcoin_hashes::{Hash, sha256d};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Visitor};

use crate::bcutils::{get_compact_int, reverse_hash};

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
    pub txns: Vec<Transaction>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Transaction {
    #[serde(serialize_with = "serialize_hash", deserialize_with = "deserialize_hash")]
    pub hash: String,
    pub version: i32,
    pub is_segwit: bool,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub witnesses: Vec<Vec<WitnessItem>>,
    pub lock_time: u32,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TxInput {
    pub prev_output: OutPoint,
    pub signature_script: String,
    pub sequence: u32,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct OutPoint {
    #[serde(serialize_with = "serialize_hash", deserialize_with = "deserialize_hash")]
    pub hash: String,
    pub idx: u32,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TxOutput {
    pub value: i64,
    pub pub_key_script: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct WitnessItem {
    pub script: String,
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
    pl: &'a [u8],
    off: usize,
}

impl Payload<'_> {
    fn read_u32(&mut self) -> Result<u32, ParsingError> {
        self.off += 4;
        Ok(self.pl.get(self.off - 4..self.off).ok_or(ParsingError)?.read_u32::<LittleEndian>().unwrap())
    }
    fn read_i32(&mut self) -> Result<i32, ParsingError> {
        self.off += 4;
        Ok(self.pl.get(self.off - 4..self.off).ok_or(ParsingError)?.read_i32::<LittleEndian>().unwrap())
    }
    fn read_i64(&mut self) -> Result<i64, ParsingError> {
        self.off += 8;
        Ok(self.pl.get(self.off - 8..self.off).ok_or(ParsingError)?.read_i64::<LittleEndian>().unwrap())
    }
    fn encode_addr(&mut self) -> Result<String, ParsingError> {
        self.off += 32;
        Ok(hex::encode(self.pl.get(self.off - 32..self.off).ok_or(ParsingError)?))
    }
    fn encode_string(&mut self, length: usize) -> Result<String, ParsingError> {
        self.off += length;
        Ok(hex::encode(self.pl.get(self.off - length..self.off).ok_or(ParsingError)?))
    }
    fn get_compact_int(&mut self) -> Result<usize, ParsingError> {
        let (txn_count, off) = get_compact_int(&self.pl.get(self.off..).ok_or(ParsingError)?);
        self.off += off;
        Ok(txn_count as usize)
    }
}

fn block_hash(block: &Payload) -> Result<String, ParsingError> {
    Ok(hex::encode(sha256d::Hash::hash(&block.pl.get(..80).ok_or(ParsingError)?)))
}

fn tx_hash(tx: &Payload, from: usize) -> String {
    hex::encode(sha256d::Hash::hash(&tx.pl[from..tx.off]))
}

fn segwit_hash(tx: &Payload, from: usize, txs_offset: usize) -> String {
    let tmp = &[&tx.pl[from..from + 4],
        &tx.pl[from + 6..txs_offset],
        &tx.pl[tx.off - 4..tx.off]].concat();

    hex::encode(sha256d::Hash::hash(tmp))
}

fn is_segwit(tx: &Payload) -> Result<bool, ParsingError> {
    Ok(tx.pl.get(tx.off + 4..tx.off + 6).ok_or(ParsingError)? == &[0x00, 0x01])
}

fn tx_loop(pl: &mut Payload, tx_count: usize) -> Result<Vec<Transaction>, ParsingError> {
    let mut txs = Vec::new();
    for _ in 0..tx_count {
        let tx = match is_segwit(pl)? {
            true => parse_segwit_tx(pl)?,
            false => parse_standard_tx(pl)?
        };
        txs.push(tx);
    }
    Ok(txs)
}

fn input_loop(pl: &mut Payload, in_count: usize) -> Result<Vec<TxInput>, ParsingError> {
    let mut inputs = Vec::new();
    for _ in 0..in_count {
        inputs.push(parse_tx_input(pl)?);
    }
    Ok(inputs)
}

fn output_loop(pl: &mut Payload, out_count: usize) -> Result<Vec<TxOutput>, ParsingError> {
    let mut outputs = Vec::new();
    for _ in 0..out_count {
        outputs.push(parse_tx_output(pl)?);
    }
    Ok(outputs)
}

fn witness_loop(pl: &mut Payload, wit_count: usize) -> Result<Vec<Vec<WitnessItem>>, ParsingError> {
    let mut witnesses = Vec::new();
    for _ in 0..wit_count {
        let item_count = pl.get_compact_int()?;
        let mut wit = Vec::new();
        for _ in 0..item_count {
            wit.push(parse_witness_item(pl)?);
        }
        witnesses.push(wit);
    }
    Ok(witnesses)
}

fn parse_segwit_tx(payload: &mut Payload) -> Result<Transaction, ParsingError> {
    // let mut offset = 4;
    let offset_in_out: usize;
    let len_in: usize;
    let start = payload.off;

    Ok(Transaction {
        is_segwit: true,
        version: payload.read_i32()?,
        inputs: {
            payload.off += 2; // we skip segwit flag
            let in_count = payload.get_compact_int()?;
            let inputs = input_loop(payload, in_count)?;
            len_in = inputs.len();
            inputs
        },
        outputs: {
            let out_count = payload.get_compact_int()?;
            let outputs = output_loop(payload, out_count)?;
            offset_in_out = payload.off;
            outputs
        },
        witnesses: witness_loop(payload, len_in)?,
        lock_time: payload.read_u32()?,
        hash: segwit_hash(payload, start, offset_in_out),
    })
}

fn parse_standard_tx(payload: &mut Payload) -> Result<Transaction, ParsingError> {
    let from = payload.off;

    Ok(Transaction {
        is_segwit: false,
        version: payload.read_i32()?,
        inputs: {
            let in_count = payload.get_compact_int()?;
            input_loop(payload, in_count)?
        },
        outputs: {
            let out_count = payload.get_compact_int()?;
            output_loop(payload, out_count)?
        },
        witnesses: vec!(),
        lock_time: payload.read_u32()?,
        hash: tx_hash(payload, from),
    })
}

fn parse_tx_input(tx_input: &mut Payload) -> Result<TxInput, ParsingError> {
    Ok(TxInput {
        prev_output: OutPoint {
            hash: tx_input.encode_addr()?,
            idx: tx_input.read_u32()?,
        },
        signature_script: {
            let script_length = tx_input.get_compact_int()?;
            tx_input.encode_string(script_length)?
        },
        sequence: tx_input.read_u32()?,
    })
}

fn parse_tx_output(tx_output: &mut Payload) -> Result<TxOutput, ParsingError> {
    Ok(TxOutput {
        value: tx_output.read_i64()?,
        pub_key_script: {
            let script_length = tx_output.get_compact_int()?;
            tx_output.encode_string(script_length)?
        },
    })
}

fn parse_witness_item(tx_witness: &mut Payload) -> Result<WitnessItem, ParsingError> {
    let length = tx_witness.get_compact_int()?;
    Ok(WitnessItem {
        script: tx_witness.encode_string(length)?
    })
}

//Public Entry
pub fn parse_block(payload: &[u8]) -> Result<Block, ParsingError> {
    let mut block = Payload { pl: payload, off: 0 };
    Ok(Block {
        hash: block_hash(&block)?,
        version: block.read_i32()?,
        prev_hash: block.encode_addr()?,
        merkle_root: block.encode_addr()?,
        timestamp: block.read_u32()?,
        bits: block.read_u32()?,
        nonce: block.read_u32()?,
        txns: {
            let tx_count = block.get_compact_int()?;
            tx_loop(&mut block, tx_count)?
        },
    })
}
