use std::convert::TryInto;

const UNIT_16: u8 = 0xFD;
const UNIT_32: u8 = 0xFE;
const UNIT_64: u8 = 0xFF;
const UNIT_8_END: usize = 1;
const UNIT_16_END: usize = 3;
const UNIT_32_END: usize = 5;
const UNIT_64_END: usize = 9;

pub fn get_compact_int(payload: &Vec<u8>) -> (u64, usize) {
    let storage_length: u8 = payload[0];

    if storage_length == UNIT_16 {
        return (u16::from_le_bytes((&payload[1..UNIT_16_END]).try_into().unwrap()) as u64, UNIT_16_END);
    }
    if storage_length == UNIT_32 {
        return (u32::from_le_bytes((&payload[1..UNIT_32_END]).try_into().unwrap()) as u64, UNIT_32_END);
    }
    if storage_length == UNIT_64 {
        return (u64::from_le_bytes((&payload[1..UNIT_64_END]).try_into().unwrap()) as u64, UNIT_64_END);
    }
    return (storage_length as u64, UNIT_8_END);
}

pub fn reverse_hash(hash: &str) -> String {
    let mut bytes = hex::decode(hash).unwrap();
    bytes.reverse();
    hex::encode(bytes)
}