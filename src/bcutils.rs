use std::convert::TryInto;

const UNIT_16: u8 = 0xFD;
const UNIT_32: u8 = 0xFE;
const UNIT_64: u8 = 0xFF;

pub fn get_compact_int(payload: &Vec<u8>) -> (u64, usize) {
    let storage_length: u8 = payload[0];

    if storage_length == UNIT_16 {
        return (u16::from_le_bytes((&payload[1..3]).try_into().unwrap()) as u64, 3);
    }
    if storage_length == UNIT_32 {
        return (u32::from_le_bytes((&payload[1..5]).try_into().unwrap()) as u64, 5);
    }
    if storage_length == UNIT_64 {
        return (u64::from_le_bytes((&payload[1..9]).try_into().unwrap()) as u64, 9);
    }
    return (storage_length as u64, 1);
}

pub fn reverse_hash(hash: &str) -> String {
    let mut bytes = hex::decode(hash).unwrap();
    bytes.reverse();
    hex::encode(bytes)
}
