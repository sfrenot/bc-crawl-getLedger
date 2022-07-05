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
    let mut txn = Transaction::default();
    let mut offset = 0;
    let mut temp_bytes;
    let mut raw_txn = Vec::new();

    // version
    temp_bytes = payload.get(..4).ok_or(ParsingError)?;
    txn.version = i32::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 4;

    // segwit flag
    temp_bytes = payload.get(offset..offset+2).ok_or(ParsingError)?;
    if temp_bytes == &[0x00, 0x01] {
        txn.is_segwit = true;
        offset += 2;
        raw_txn.extend_from_slice(&payload[..4]); // if segwit, we create a clean txn for the hash
    }

    // tx_in
    let off;
    (txn.inputs, off) = get_transactions(payload.get(offset..).ok_or(ParsingError)?, TxKind::TxInput)?;
    offset += off;

    // tx_out
    let off;
    (txn.outputs, off) = get_transactions(payload.get(offset..).ok_or(ParsingError)?, TxKind::TxOutput)?;
    offset += off;

    // parsing segregated witnesses if any
    if txn.is_segwit {
        raw_txn.extend_from_slice(&payload[6..offset]);

        // eprintln!("{:02X?}", &payload[offset..]);
        let mut witnesses = Vec::new();
        for _ in 0..txn.inputs.len() {
            let (data, off) = get_transactions(&payload[offset..], TxKind::WitnessItem)?;
            witnesses.push(data);
            offset += off;
        };
        txn.witnesses = witnesses;
    }

    // lock time
    temp_bytes = payload.get(offset..offset+4).ok_or(ParsingError)?;
    txn.lock_time = u32::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 4;

    // hash
    txn.hash = match txn.is_segwit {
        true => {
            raw_txn.extend_from_slice(temp_bytes);
            hex::encode(sha256d::Hash::hash(&raw_txn))
        },
        false => hex::encode(sha256d::Hash::hash(&payload[..offset]))
    };

    Ok((txn, offset))
}

fn parse_tx_input(payload: &[u8]) -> Result<(TxInput, usize), ParsingError> {
    let (script_length, off) = get_compact_int(&payload[36..]);
    let script_length = script_length as usize;
    let start_sig = 36 + off;
    let start_seq = start_sig + script_length;

    return Ok((TxInput {
        prev_output: OutPoint {
            hash: hex::encode(&payload[..32]),
            idx: (&payload[..32+4]).read_u32::<LittleEndian>().unwrap()
        },
        signature_script: hex::encode(&payload[start_sig..start_sig + script_length]),
        sequence: (&payload[start_seq..start_seq+4]).read_u32::<LittleEndian>().unwrap()
    }, start_seq + 4));
}

fn parse_tx_output(payload: &[u8]) -> Result<(TxOutput, usize), ParsingError> {
    let (script_length, off) = get_compact_int(&payload[8..]);
    let script_length = script_length as usize;
    let start_pub_key_script = 8+off;

    return Ok((TxOutput{
        value:(&payload[..8]).read_i64::<LittleEndian>().unwrap(),
        pub_key_script: hex::encode(&payload[start_pub_key_script..start_pub_key_script+script_length])
    }, start_pub_key_script+script_length));
}

fn parse_witness_item(payload: &[u8]) -> Result<(WitnessItem, usize), ParsingError> {
    let mut witness_item = WitnessItem::default();
    let mut offset = 0;
    let mut temp_bytes;

    // item script length
    temp_bytes = payload.get(..).ok_or(ParsingError)?;
    let (length, off) = get_compact_int(&temp_bytes);
    offset += off;

    // item script
    temp_bytes = payload.get(offset..offset + (length as usize)).ok_or(ParsingError)?;
    witness_item.script = hex::encode(temp_bytes);
    offset += length as usize;

    Ok((witness_item, offset))
}

//Public
pub fn parse_block(payload: &[u8]) -> Result<Block, ParsingError> {
    return Ok(Block {
        hash: hex::encode(sha256d::Hash::hash(payload.get(..80).ok_or(ParsingError)?)),
        version: (&payload[..4]).read_i32::<LittleEndian>().unwrap(),
        prev_hash: hex::encode(&payload[4..4+32]),
        merkle_root: hex::encode(&payload[36..36+32]),
        timestamp: (&payload[68..68+4]).read_u32::<LittleEndian>().unwrap(),
        bits: (&payload[72..72+4]).read_u32::<LittleEndian>().unwrap(),
        nonce: (&payload[76..76+4]).read_u32::<LittleEndian>().unwrap(),
        txns: {
            let (tx, _) = get_transactions(payload.get(80..).ok_or(ParsingError)?, TxKind::Transaction)?;
            tx
        }
    })
}
