use std::convert::TryInto;
use bitcoin_hashes::{Hash, sha256d};
use serde::{Deserialize, Serialize};
use crate::bcnet::bcmessage::get_compact_int;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Block {
    pub hash: String,
    pub version: i32,
    pub prev_hash: String,
    pub merkle_root: String,
    pub timestamp: u32,
    pub bits: u32,
    pub nonce: u32,
    pub txn_count: u64,
    pub txns: Vec<Transaction>
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Transaction {
    pub hash: String,
    pub version: i32,
    pub segwit_flag: bool,
    pub tx_in_count: u64,
    pub tx_in: Vec<TxIn>,
    pub tx_out_count: u64,
    pub tx_out: Vec<TxOut>,
    pub tx_witnesses: Vec<Witness>,
    pub lock_time: u32
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TxIn {
    pub prev_hash: String,
    pub prev_tx_out_index: u32,
    pub script_length: u64,
    pub signature_script: String,
    pub sequence: u32,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TxOut {
    pub value: i64,
    pub pk_script_length: u64,
    pub pk_script: String
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Witness {
    pub item_count: u64,
    pub items: Vec<WitnessItem>
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct WitnessItem {
    pub length: u64,
    pub script: String
}

#[derive(Debug)]
pub struct ParsingError;

pub fn parse_block(payload: &Vec<u8>) -> Result<Block, ParsingError> {
    let mut block = Block::default();
    let mut offset = 0;
    let mut temp_bytes;

    // header hash
    temp_bytes = payload.get(..80).ok_or(ParsingError)?;
    block.hash = hex::encode(sha256d::Hash::hash(temp_bytes).to_vec());

    // version
    temp_bytes = &payload[..4];
    block.version = i32::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 4;

    // previous block hash
    temp_bytes = &payload[offset..offset+32];
    block.prev_hash = hex::encode(temp_bytes);
    offset += 32;

    // merkle root hash
    temp_bytes = &payload[offset..offset+32];
    block.merkle_root = hex::encode(temp_bytes);
    offset += 32;

    // timestamp
    temp_bytes = &payload[offset..offset+4];
    block.timestamp = u32::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 4;

    // bits
    temp_bytes = &payload[offset..offset+4];
    block.bits = u32::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 4;

    // nonce
    temp_bytes = &payload[offset..offset+4];
    block.nonce = u32::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 4;

    // transaction count
    temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
    let (txn_count, off) = get_compact_int(&temp_bytes.to_vec());
    block.txn_count = txn_count;
    offset += off;

    // parsing transactions
    let mut txns = Vec::new();
    for _ in 0..block.txn_count {
        temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
        let (txn, off) = parse_transaction(&temp_bytes.to_vec())?;
        txns.push(txn);
        offset += off;
    };
    block.txns = txns;

    Ok(block)
}

pub fn parse_transaction(payload: &Vec<u8>) -> Result<(Transaction, usize), ParsingError> {
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
        txn.segwit_flag = true;
        offset += 2;

        raw_txn.extend_from_slice(&payload[..4]); // if segwit, we create a clean txn for the hash
    }

    // tx_in count
    temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
    let (tx_in_count, off) = get_compact_int(&temp_bytes.to_vec());
    txn.tx_in_count = tx_in_count;
    offset += off;

    // parsing tx_in
    let mut tx_in = Vec::new();
    for _ in 0..txn.tx_in_count {
        temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
        let (data, off) = parse_tx_in(&temp_bytes.to_vec())?;
        tx_in.push(data);
        offset += off;
    };
    txn.tx_in = tx_in;

    // tx_out count
    temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
    let (tx_out_count, off) = get_compact_int(&temp_bytes.to_vec());
    txn.tx_out_count = tx_out_count;
    offset += off;

    // parsing tx_out
    let mut tx_out = Vec::new();
    for _ in 0..txn.tx_out_count {
        temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
        let (data, off) = parse_tx_out(&temp_bytes.to_vec())?;
        tx_out.push(data);
        offset += off;
    };
    txn.tx_out = tx_out;

    // parsing segregated witnesses if any
    if txn.segwit_flag {
        raw_txn.extend_from_slice(&payload[6..offset]);

        let mut tx_witnesses = Vec::new();
        for _ in 0..txn.tx_in_count {
            temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
            let (data, off) = parse_witness(&temp_bytes.to_vec())?;
            tx_witnesses.push(data);
            offset += off;
        };
        txn.tx_witnesses = tx_witnesses;
    }

    // lock time
    temp_bytes = payload.get(offset..offset+4).ok_or(ParsingError)?;
    txn.lock_time = u32::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 4;

    // hash
    if txn.segwit_flag {
        raw_txn.extend_from_slice(temp_bytes);
        txn.hash = hex::encode(sha256d::Hash::hash(&raw_txn));
    } else {
        txn.hash = hex::encode(sha256d::Hash::hash(&payload[..offset]));
    }

    Ok((txn, offset))
}

fn parse_tx_in(payload: &Vec<u8>) -> Result<(TxIn, usize), ParsingError> {
    let mut tx_in = TxIn::default();
    let mut offset = 0;
    let mut temp_bytes;

    // previous transaction hash
    temp_bytes = payload.get(..32).ok_or(ParsingError)?;
    tx_in.prev_hash = hex::encode(temp_bytes);
    offset += 32;

    // previous transaction output index
    temp_bytes = payload.get(offset..offset+4).ok_or(ParsingError)?;
    tx_in.prev_tx_out_index = u32::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 4;

    // script length in bytes
    temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
    let (script_length, off) = get_compact_int(&temp_bytes.to_vec());
    tx_in.script_length = script_length;
    offset += off;

    // signature script
    temp_bytes = payload.get(offset..offset + (tx_in.script_length as usize)).ok_or(ParsingError)?;
    tx_in.signature_script = hex::encode(temp_bytes);
    offset += tx_in.script_length as usize;

    // sequence number
    temp_bytes = payload.get(offset..offset+4).ok_or(ParsingError)?;
    tx_in.sequence = u32::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 4;

    Ok((tx_in, offset))
}

fn parse_tx_out(payload: &Vec<u8>) -> Result<(TxOut, usize), ParsingError> {
    let mut tx_out = TxOut::default();
    let mut offset = 0;
    let mut temp_bytes;

    // value in satoshis
    temp_bytes = payload.get(..8).ok_or(ParsingError)?;
    tx_out.value = i64::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 8;

    // pubkey script length
    temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
    let (script_length, off) = get_compact_int(&temp_bytes.to_vec());
    tx_out.pk_script_length = script_length;
    offset += off;

    // pubkey script
    temp_bytes = payload.get(offset..offset + (tx_out.pk_script_length as usize)).ok_or(ParsingError)?;
    tx_out.pk_script = hex::encode(temp_bytes);
    offset += tx_out.pk_script_length as usize;

    Ok((tx_out, offset))
}

fn parse_witness(payload: &Vec<u8>) -> Result<(Witness, usize), ParsingError> {
    let mut witness = Witness::default();
    let mut offset = 0;
    let mut temp_bytes;

    // witness item count
    temp_bytes = payload.get(..).ok_or(ParsingError)?;
    let (item_count, off) = get_compact_int(&temp_bytes.to_vec());
    witness.item_count = item_count;
    offset += off;

    // parsing items
    let mut items = Vec::new();
    for _ in 0..witness.item_count {
        temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
        let (txn, off) = parse_witness_item(&temp_bytes.to_vec())?;
        items.push(txn);
        offset += off;
    };
    witness.items = items;

    Ok((witness, offset))
}

fn parse_witness_item(payload: &Vec<u8>) -> Result<(WitnessItem, usize), ParsingError> {
    let mut witness_item = WitnessItem::default();
    let mut offset = 0;
    let mut temp_bytes;

    // item script length
    temp_bytes = payload.get(..).ok_or(ParsingError)?;
    let (length, off) = get_compact_int(&temp_bytes.to_vec());
    witness_item.length = length;
    offset += off;

    // item script
    temp_bytes = payload.get(offset..offset + (witness_item.length as usize)).ok_or(ParsingError)?;
    witness_item.script = hex::encode(temp_bytes);
    offset += witness_item.length as usize;

    Ok((witness_item, offset))
}