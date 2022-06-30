use std::convert::TryInto;
use bitcoin_hashes::{Hash, sha256d};
use serde::{Deserialize, Serialize};
use crate::bcutils::{get_compact_int, reverse_hash};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Block {
    pub hash: String,
    pub version: i32,
    pub prev_hash: String,
    pub merkle_root: String,
    pub timestamp: u32,
    pub bits: u32,
    pub nonce: u32,
    pub txns: Vec<Transaction>
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Transaction {
    pub hash: String,
    pub version: i32,
    pub is_segwit: bool,
    pub inputs: Vec<TxInput>,
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
    pub hash: String,
    pub idx: u32
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TxOutput {
    pub value: i64,
    pub pub_key_script: String
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

pub fn parse_block(payload: &Vec<u8>) -> Result<Block, ParsingError> {
    let mut block = Block::default();
    let mut offset = 0;
    let mut temp_bytes;

    // header hash
    temp_bytes = payload.get(..80).ok_or(ParsingError)?;
    block.hash = sha256d::Hash::hash(temp_bytes).to_string();

    // version
    temp_bytes = &payload[..4];
    block.version = i32::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 4;

    // previous block hash
    temp_bytes = &payload[offset..offset+32];
    let prev_hash = hex::encode(temp_bytes);
    block.prev_hash = reverse_hash(&prev_hash);
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
    offset += off;

    // parsing transactions
    let mut txns = Vec::new();
    for _ in 0..txn_count {
        temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
        let (txn, off) = parse_transaction(&temp_bytes.to_vec())?;
        txns.push(txn);
        offset += off;
    };
    block.txns = txns;

    Ok(block)
}

fn parse_transaction(payload: &Vec<u8>) -> Result<(Transaction, usize), ParsingError> {
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

    // tx_in count
    temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
    let (input_count, off) = get_compact_int(&temp_bytes.to_vec());
    offset += off;

    // parsing tx_in
    let mut inputs = Vec::new();
    for _ in 0..input_count {
        temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
        let (data, off) = parse_tx_input(&temp_bytes.to_vec())?;
        inputs.push(data);
        offset += off;
    };
    txn.inputs = inputs;

    // tx_out count
    temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
    let (output_count, off) = get_compact_int(&temp_bytes.to_vec());
    offset += off;

    // parsing tx_out
    let mut outputs = Vec::new();
    for _ in 0..output_count {
        temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
        let (data, off) = parse_tx_output(&temp_bytes.to_vec())?;
        outputs.push(data);
        offset += off;
    };
    txn.outputs = outputs;

    // parsing segregated witnesses if any
    if txn.is_segwit {
        raw_txn.extend_from_slice(&payload[6..offset]);

        let mut witnesses = Vec::new();
        for _ in 0..input_count {
            temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
            let (data, off) = parse_witness(&temp_bytes.to_vec())?;
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
    let hash;
    if txn.is_segwit {
        raw_txn.extend_from_slice(temp_bytes);
        hash = hex::encode(sha256d::Hash::hash(&raw_txn));
    } else {
        hash = hex::encode(sha256d::Hash::hash(&payload[..offset]));
    }
    txn.hash = reverse_hash(&hash);

    Ok((txn, offset))
}

pub fn parse_block_sfr(payload: &Vec<u8>) -> Result<Block, ParsingError> {

    let mut block = Block::default();
    let mut offset = 0;
    // let mut temp_bytes;
    let mut hash32:[u8; 32] = [0;32];
    let mut hash4:[u8; 4] = [0; 4];

    // header hash
    block.hash = sha256d::Hash::hash(&payload[..80]).to_string();

    // version
    for i in 0..4 {hash4[i] = payload[offset+i];}
    block.version = i32::from_le_bytes(hash4);
    offset += 4;

    // previous block hash
    for i in 0..32 {hash32[i] = payload[offset+(32-i)-1];}
    block.prev_hash = hex::encode(hash32);
    offset += 32;

    // merkle root hash
    for i in 0..32 {hash32[i] = payload[offset+i];}
    block.merkle_root = hex::encode(hash32);
    offset += 32;

    // timestamp
    for i in 0..4 {hash4[i] = payload[offset+i];}
    block.timestamp = u32::from_le_bytes(hash4);
    offset += 4;

    // bits
    for i in 0..4 {hash4[i] = payload[offset+i];}
    block.bits = u32::from_le_bytes(hash4);
    offset += 4;

    // nonce
    for i in 0..4 {hash4[i] = payload[offset+i];}
    block.nonce = u32::from_le_bytes(hash4);
    offset += 4;

    // transaction count
    let (txn_count, off) = get_compact_int(&payload[offset..].to_vec());
    offset += off;

    // parsing transactions
    let mut txns = Vec::new();
    for _ in 0..txn_count {
        let (txn, off) = parse_transaction_sfr(&payload[offset..].to_vec())?;
        txns.push(txn);
        offset += off;
    };
    block.txns = txns;

    // let duree = now.elapsed().unwrap().as_millis();
    // eprintln!("-> {}", duree);
    Ok(block)
}

fn parse_transaction_sfr(payload: &Vec<u8>) -> Result<(Transaction, usize), ParsingError> {
    let mut txn = Transaction::default();
    let mut offset = 0;
    let mut raw_txn = Vec::new();

    // version
    // temp_bytes = payload.get(..4).ok_or(ParsingError)?;
    txn.version = i32::from_le_bytes(payload[..4].try_into().unwrap());
    offset += 4;

    // segwit flag
    if payload[offset..offset+2] == [0x00, 0x01] {
        txn.is_segwit = true;
        offset += 2;
        raw_txn.extend_from_slice(&payload[..4]); // if segwit, we create a clean txn for the hash
    }

    // tx_in count
    let (input_count, off) = get_compact_int(&payload[offset..].to_vec());
    offset += off;

    // parsing tx_in
    for _ in 0..input_count {
        let (data, off) = parse_tx_input(&payload[offset..].to_vec())?;
        txn.inputs.push(data);
        offset += off;
    };

    // tx_out count
    let (output_count, off) = get_compact_int(&payload[offset..].to_vec());
    offset += off;

    // parsing tx_out
    for _ in 0..output_count {
        let (data, off) = parse_tx_output(&payload[offset..].to_vec())?;
        txn.outputs.push(data);
        offset += off;
    };

    // parsing segregated witnesses if any
    if txn.is_segwit {
        raw_txn.extend_from_slice(&payload[6..offset]);

        for _ in 0..input_count {
            let (data, off) = parse_witness(&payload[offset..].to_vec())?;
            txn.witnesses.push(data);
            offset += off;
        };
    }

    let temp_bytes = payload.get(offset..offset+4).ok_or(ParsingError)?;
    // lock time
    txn.lock_time = u32::from_le_bytes(payload[offset..offset+4].try_into().unwrap());
    offset += 4;

    // hash
    let hash;
    if txn.is_segwit {
        raw_txn.extend_from_slice(temp_bytes);
        hash = hex::encode(sha256d::Hash::hash(&raw_txn));
    } else {
        hash = hex::encode(sha256d::Hash::hash(&payload[..offset]));
    }
    txn.hash = reverse_hash(&hash);

    Ok((txn, offset))
}

fn parse_tx_input(payload: &Vec<u8>) -> Result<(TxInput, usize), ParsingError> {
    let mut tx_input = TxInput::default();
    let mut prev_output = OutPoint::default();
    let mut offset = 0;
    let mut temp_bytes;

    // previous transaction hash
    temp_bytes = payload.get(..32).ok_or(ParsingError)?;
    let prev_hash = hex::encode(temp_bytes);
    prev_output.hash = reverse_hash(&prev_hash);
    offset += 32;

    // previous transaction output index
    temp_bytes = payload.get(offset..offset+4).ok_or(ParsingError)?;
    prev_output.idx = u32::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 4;

    // script length in bytes
    temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
    let (script_length, off) = get_compact_int(&temp_bytes.to_vec());
    offset += off;

    // signature script
    temp_bytes = payload.get(offset..offset + (script_length as usize)).ok_or(ParsingError)?;
    tx_input.signature_script = hex::encode(temp_bytes);
    offset += script_length as usize;

    // sequence number
    temp_bytes = payload.get(offset..offset+4).ok_or(ParsingError)?;
    tx_input.sequence = u32::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 4;

    tx_input.prev_output = prev_output;

    Ok((tx_input, offset))
}

fn parse_tx_output(payload: &Vec<u8>) -> Result<(TxOutput, usize), ParsingError> {
    let mut tx_output = TxOutput::default();
    let mut offset = 0;
    let mut temp_bytes;

    // value in satoshis
    temp_bytes = payload.get(..8).ok_or(ParsingError)?;
    tx_output.value = i64::from_le_bytes(temp_bytes.try_into().unwrap());
    offset += 8;

    // pubkey script length
    temp_bytes = payload.get(offset..).ok_or(ParsingError)?;
    let (script_length, off) = get_compact_int(&temp_bytes.to_vec());
    offset += off;

    // pubkey script
    temp_bytes = payload.get(offset..offset + (script_length as usize)).ok_or(ParsingError)?;
    tx_output.pub_key_script = hex::encode(temp_bytes);
    offset += script_length as usize;

    Ok((tx_output, offset))
}

fn parse_witness(payload: &Vec<u8>) -> Result<(Witness, usize), ParsingError> {
    let mut witness = Witness::default();
    let mut offset = 0;
    let mut temp_bytes;

    // witness item count
    temp_bytes = payload.get(..).ok_or(ParsingError)?;
    let (item_count, off) = get_compact_int(&temp_bytes.to_vec());
    offset += off;

    // parsing items
    let mut items = Vec::new();
    for _ in 0..item_count {
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
    offset += off;

    // item script
    temp_bytes = payload.get(offset..offset + (length as usize)).ok_or(ParsingError)?;
    witness_item.script = hex::encode(temp_bytes);
    offset += length as usize;

    Ok((witness_item, offset))
}
