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
    pub outputs: Vec<TxOutput>,
    pub witnesses: Vec<Witness>,
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

#[derive(Debug, Deserialize, Serialize)]
pub enum Tx {
    Transaction(Transaction),
    TxInput(TxInput),
    TxOutput(TxOutput)
}
enum TxKind {
    Transaction,
    TxInput
    // TxOuput
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Witness {
    pub items: Vec<WitnessItem>
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct WitnessItem {
    pub script: String
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
            }
            TxKind::TxInput => {
                let (txn, off) = parse_tx_input(&temp_bytes)?;
                txns.push(Tx::TxInput(txn));
                off
            }
        };
    };
    // eprintln!("off {}", offset);
    Ok((txns, offset))
}

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

    // // tx_in count
    // temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
    // let (input_count, off) = get_compact_int(&temp_bytes);
    // offset += off;
    //
    // // parsing tx_in
    // let mut inputs = Vec::new();
    // for _ in 0..input_count {
    //     temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
    //     let (data, off) = parse_tx_input(&temp_bytes)?;
    //     inputs.push(data);
    //     offset += off;
    // };
    // txn.inputs = inputs;
    let off;
    (txn.inputs, off) = get_transactions(payload.get(offset..).ok_or(ParsingError)?, TxKind::TxInput)?;
    offset += off;

    // tx_out count
    temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
    let (output_count, off) = get_compact_int(&temp_bytes);
    offset += off;

    // parsing tx_out
    let mut outputs = Vec::new();
    for _ in 0..output_count {
        temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
        let (data, off) = parse_tx_output(&temp_bytes)?;
        outputs.push(data);
        offset += off;
    };
    txn.outputs = outputs;

    // parsing segregated witnesses if any
    if txn.is_segwit {
        raw_txn.extend_from_slice(&payload[6..offset]);

        let mut witnesses = Vec::new();
        for _ in 0..txn.inputs.len() {
            temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
            let (data, off) = parse_witness(&temp_bytes)?;
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
    let mut tx_input = TxInput::default();
    let mut offset = 0;
    let mut temp_bytes;

    let mut prev_output = OutPoint::default();

    // previous transaction hash
    temp_bytes = payload.get(..32).ok_or(ParsingError)?;
    prev_output.hash = hex::encode(temp_bytes);
    offset += 32;

    // previous transaction output index
    temp_bytes = payload.get(offset..offset+4).ok_or(ParsingError)?;
    prev_output.idx = u32::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 4;

    tx_input.prev_output = prev_output;

    // script length in bytes
    temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
    let (script_length, off) = get_compact_int(&temp_bytes);
    offset += off;

    // signature script
    temp_bytes = payload.get(offset..offset + (script_length as usize)).ok_or(ParsingError)?;
    tx_input.signature_script = hex::encode(temp_bytes);
    offset += script_length as usize;

    // sequence number
    temp_bytes = payload.get(offset..offset+4).ok_or(ParsingError)?;
    tx_input.sequence = u32::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 4;

    Ok((tx_input, offset))
}

fn parse_tx_output(payload: &[u8]) -> Result<(TxOutput, usize), ParsingError> {
    let mut tx_output = TxOutput::default();
    let mut offset = 0;
    let mut temp_bytes;

    // value in satoshis
    temp_bytes = payload.get(..8).ok_or(ParsingError)?;
    tx_output.value = i64::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 8;

    // pubkey script length
    temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
    let (script_length, off) = get_compact_int(&temp_bytes);
    offset += off;

    // pubkey script
    temp_bytes = payload.get(offset..offset + (script_length as usize)).ok_or(ParsingError)?;
    tx_output.pub_key_script = hex::encode(temp_bytes);
    offset += script_length as usize;

    Ok((tx_output, offset))
}

fn parse_witness(payload: &[u8]) -> Result<(Witness, usize), ParsingError> {
    let mut witness = Witness::default();
    let mut offset = 0;
    let mut temp_bytes;

    // witness item count
    temp_bytes = payload.get(..).ok_or(ParsingError)?;
    let (item_count, off) = get_compact_int(&temp_bytes);
    offset += off;

    // parsing items
    let mut items = Vec::new();
    for _ in 0..item_count {
        temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
        let (txn, off) = parse_witness_item(&temp_bytes)?;
        items.push(txn);
        offset += off;
    };
    witness.items = items;

    Ok((witness, offset))
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
