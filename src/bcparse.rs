use std::convert::TryInto;
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

fn get_transactions(payload: &[u8], kind: TxKind) -> Result<(Vec<Tx>, usize), ParsingError> {
    let mut offset = 0;
    let (txn_count, off) = get_compact_int(&payload);
    offset += off;

    let mut txns = Vec::new();
    for _ in 0..txn_count {
        let temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
        offset += match kind {
            TxKind::Transaction => {
                let (txn, off) = parse_transaction(&temp_bytes)?;
                txns.push(Tx::Transaction(txn));
                off
            },
            TxKind::TxInput => {
                let (txn, off) = parse_tx_input(&temp_bytes)?;
                txns.push(Tx::TxInput(txn));
                off
            },
            TxKind::TxOutput => {
                let (txn, off) = parse_tx_output(&temp_bytes)?;
                txns.push(Tx::TxOutput(txn));
                off
            },
            TxKind::WitnessItem => {
                let (txn, off) = parse_witness_item(&temp_bytes)?;
                txns.push(Tx::WitnessItem(txn));
                off
            }
        };
    };
    // eprintln!("off {}", offset);
    Ok((txns, offset))
}

fn parse_transaction(payload: &[u8]) -> Result<(Transaction, usize), ParsingError> {
    return match payload.get(4..6).ok_or(ParsingError)? == &[0x00, 0x01] {
        true => {
            parse_segwit_tx(payload)
        },
        false => {
            parse_standard_tx(payload)
        }
    }
}

fn read_i32(payload: &[u8], start: usize) -> i32 {
    payload.get(start..start+4).unwrap().read_i32::<LittleEndian>().unwrap()
}

fn read_u32(payload: &[u8], start: usize) -> u32 {
    payload.get(start..start+4).unwrap().read_u32::<LittleEndian>().unwrap()
}

fn read_i64(payload: &[u8], start: usize) -> i64 {
    payload.get(start..start+8).unwrap().read_i64::<LittleEndian>().unwrap()
}

fn encode_sha256d(payload: &[u8]) -> String {
    hex::encode(sha256d::Hash::hash(payload))
}

fn encode_addr(payload: &[u8], start:usize) -> String {
    hex::encode(payload.get(start..start+32).unwrap())
}

fn encode_string(payload: &[u8], start:usize, stop: usize) -> String {
    hex::encode(payload.get(start..stop).unwrap())
}

fn parse_segwit_tx(payload: &[u8]) -> Result<(Transaction, usize), ParsingError> {
    let mut offset = 4;
    let offset_in_out:usize;
    let len_in:usize;

    return Ok((Transaction{
        is_segwit: true,
        version: read_i32(payload, 0),
        inputs: {
            offset += 2;
            let (txn, offset_in) = get_transactions(payload.get(offset..).ok_or(ParsingError)?, TxKind::TxInput)?;
            len_in = txn.len();
            offset += offset_in;
            txn
        },
        outputs: {
            let (txn, offset_out) = get_transactions(payload.get(offset..).ok_or(ParsingError)?, TxKind::TxOutput)?;
            offset += offset_out;
            offset_in_out = offset;
            txn
        },
        witnesses: {
            let mut witnesses = Vec::new();
            for _ in 0..len_in {
                let (data, offset_witnesses) = get_transactions(&payload[offset..], TxKind::WitnessItem)?;
                witnesses.push(data);
                offset += offset_witnesses;
            };
            witnesses
        },
        lock_time: read_u32(payload, offset),
        hash: encode_sha256d(&[&payload[..4], &payload[6..offset_in_out], &payload[offset..offset+4]].concat())
    }, offset+4));
}

fn parse_standard_tx(payload: &[u8]) -> Result<(Transaction, usize), ParsingError> {
    let mut offset = 4;
    return Ok((Transaction{
        is_segwit: false,
        version: read_i32(payload, 0),
        inputs: {
            let (txn, off) = get_transactions(payload.get(offset..).ok_or(ParsingError)?, TxKind::TxInput)?;
            offset += off;
            txn
        },
        outputs: {
            let (txn, off) = get_transactions(payload.get(offset..).ok_or(ParsingError)?, TxKind::TxOutput)?;
            offset += off;
            txn
        },
        witnesses: vec!(),
        lock_time: read_u32(payload, offset),
        hash: encode_sha256d(&payload[..offset+4])
    }, offset+4));
}

fn parse_tx_input(payload: &[u8]) -> Result<(TxInput, usize), ParsingError> {
    let (script_length, off) = get_compact_int(&payload[36..]);
    let script_length = script_length as usize;
    let start_sig = 36 + off;
    let start_seq = start_sig + script_length;

    return Ok((TxInput {
        prev_output: OutPoint {
            hash: encode_addr(payload, 0),
            idx: read_u32(payload, 32)
        },
        signature_script: encode_string(payload, start_sig, (start_sig + script_length)),
        sequence: read_u32(payload, start_seq)
    }, start_seq + 4));
}

fn parse_tx_output(payload: &[u8]) -> Result<(TxOutput, usize), ParsingError> {
    let (script_length, off) = get_compact_int(&payload[8..]);
    let script_length = script_length as usize;
    let start_pub_key_script = 8+off;

    return Ok((TxOutput{
        value:read_i64(payload, 0),
        pub_key_script: encode_string(payload, start_pub_key_script, start_pub_key_script+script_length)
    }, start_pub_key_script+script_length));
}

fn parse_witness_item(payload: &[u8]) -> Result<(WitnessItem, usize), ParsingError> {
    // item script length
    let (length, offset) = get_compact_int(&payload.get(..).ok_or(ParsingError)?);
    let length = length as usize;
    // item script
    return Ok((WitnessItem{
        script: encode_string(payload, offset, offset + length)
    }
    , offset+length));
}

//Public
pub fn parse_block(payload: &[u8]) -> Result<Block, ParsingError> {
    return Ok(Block {
        hash: encode_sha256d(payload.get(..80).ok_or(ParsingError)?),
        version: read_i32(payload, 0),
        prev_hash: encode_addr(payload, 4),
        merkle_root: encode_addr(payload, 36),
        timestamp: read_u32(payload, 68),
        bits: read_u32(payload, 72),
        nonce: read_u32(payload, 76),
        txns: {
            let (tx, _) = get_transactions(payload.get(80..).ok_or(ParsingError)?, TxKind::Transaction)?;
            tx
        }
    })
}
