#![allow(dead_code)]

use std::fmt;
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub struct Opcode {
    pub code: u8
}
// Data Push
pub const OP_0: Opcode = Opcode {code: 0x00};
pub const OP_PUSH_BYTES_1: Opcode = Opcode {code: 0x01};
pub const OP_PUSH_BYTES_2: Opcode = Opcode {code: 0x02};
pub const OP_PUSH_BYTES_3: Opcode = Opcode {code: 0x03};
pub const OP_PUSH_BYTES_4: Opcode = Opcode {code: 0x04};
pub const OP_PUSH_BYTES_5: Opcode = Opcode {code: 0x05};
pub const OP_PUSH_BYTES_6: Opcode = Opcode {code: 0x06};
pub const OP_PUSH_BYTES_7: Opcode = Opcode {code: 0x07};
pub const OP_PUSH_BYTES_8: Opcode = Opcode {code: 0x08};
pub const OP_PUSH_BYTES_9: Opcode = Opcode {code: 0x09};
pub const OP_PUSH_BYTES_10: Opcode = Opcode {code: 0x0a};
pub const OP_PUSH_BYTES_11: Opcode = Opcode {code: 0x0b};
pub const OP_PUSH_BYTES_12: Opcode = Opcode {code: 0x0c};
pub const OP_PUSH_BYTES_13: Opcode = Opcode {code: 0x0d};
pub const OP_PUSH_BYTES_14: Opcode = Opcode {code: 0x0e};
pub const OP_PUSH_BYTES_15: Opcode = Opcode {code: 0x0f};
pub const OP_PUSH_BYTES_16: Opcode = Opcode {code: 0x10};
pub const OP_PUSH_BYTES_17: Opcode = Opcode {code: 0x11};
pub const OP_PUSH_BYTES_18: Opcode = Opcode {code: 0x12};
pub const OP_PUSH_BYTES_19: Opcode = Opcode {code: 0x13};
pub const OP_PUSH_BYTES_20: Opcode = Opcode {code: 0x14};
pub const OP_PUSH_BYTES_21: Opcode = Opcode {code: 0x15};
pub const OP_PUSH_BYTES_22: Opcode = Opcode {code: 0x16};
pub const OP_PUSH_BYTES_23: Opcode = Opcode {code: 0x17};
pub const OP_PUSH_BYTES_24: Opcode = Opcode {code: 0x18};
pub const OP_PUSH_BYTES_25: Opcode = Opcode {code: 0x19};
pub const OP_PUSH_BYTES_26: Opcode = Opcode {code: 0x1a};
pub const OP_PUSH_BYTES_27: Opcode = Opcode {code: 0x1b};
pub const OP_PUSH_BYTES_28: Opcode = Opcode {code: 0x1c};
pub const OP_PUSH_BYTES_29: Opcode = Opcode {code: 0x1d};
pub const OP_PUSH_BYTES_30: Opcode = Opcode {code: 0x1e};
pub const OP_PUSH_BYTES_31: Opcode = Opcode {code: 0x1f};
pub const OP_PUSH_BYTES_32: Opcode = Opcode {code: 0x20};
pub const OP_PUSH_BYTES_33: Opcode = Opcode {code: 0x21};
pub const OP_PUSH_BYTES_34: Opcode = Opcode {code: 0x22};
pub const OP_PUSH_BYTES_35: Opcode = Opcode {code: 0x23};
pub const OP_PUSH_BYTES_36: Opcode = Opcode {code: 0x24};
pub const OP_PUSH_BYTES_37: Opcode = Opcode {code: 0x25};
pub const OP_PUSH_BYTES_38: Opcode = Opcode {code: 0x26};
pub const OP_PUSH_BYTES_39: Opcode = Opcode {code: 0x27};
pub const OP_PUSH_BYTES_40: Opcode = Opcode {code: 0x28};
pub const OP_PUSH_BYTES_41: Opcode = Opcode {code: 0x29};
pub const OP_PUSH_BYTES_42: Opcode = Opcode {code: 0x2a};
pub const OP_PUSH_BYTES_43: Opcode = Opcode {code: 0x2b};
pub const OP_PUSH_BYTES_44: Opcode = Opcode {code: 0x2c};
pub const OP_PUSH_BYTES_45: Opcode = Opcode {code: 0x2d};
pub const OP_PUSH_BYTES_46: Opcode = Opcode {code: 0x2e};
pub const OP_PUSH_BYTES_47: Opcode = Opcode {code: 0x2f};
pub const OP_PUSH_BYTES_48: Opcode = Opcode {code: 0x30};
pub const OP_PUSH_BYTES_49: Opcode = Opcode {code: 0x31};
pub const OP_PUSH_BYTES_50: Opcode = Opcode {code: 0x32};
pub const OP_PUSH_BYTES_51: Opcode = Opcode {code: 0x33};
pub const OP_PUSH_BYTES_52: Opcode = Opcode {code: 0x34};
pub const OP_PUSH_BYTES_53: Opcode = Opcode {code: 0x35};
pub const OP_PUSH_BYTES_54: Opcode = Opcode {code: 0x36};
pub const OP_PUSH_BYTES_55: Opcode = Opcode {code: 0x37};
pub const OP_PUSH_BYTES_56: Opcode = Opcode {code: 0x38};
pub const OP_PUSH_BYTES_57: Opcode = Opcode {code: 0x39};
pub const OP_PUSH_BYTES_58: Opcode = Opcode {code: 0x3a};
pub const OP_PUSH_BYTES_59: Opcode = Opcode {code: 0x3b};
pub const OP_PUSH_BYTES_60: Opcode = Opcode {code: 0x3c};
pub const OP_PUSH_BYTES_61: Opcode = Opcode {code: 0x3d};
pub const OP_PUSH_BYTES_62: Opcode = Opcode {code: 0x3e};
pub const OP_PUSH_BYTES_63: Opcode = Opcode {code: 0x3f};
pub const OP_PUSH_BYTES_64: Opcode = Opcode {code: 0x40};
pub const OP_PUSH_BYTES_65: Opcode = Opcode {code: 0x41};
pub const OP_PUSH_BYTES_66: Opcode = Opcode {code: 0x42};
pub const OP_PUSH_BYTES_67: Opcode = Opcode {code: 0x43};
pub const OP_PUSH_BYTES_68: Opcode = Opcode {code: 0x44};
pub const OP_PUSH_BYTES_69: Opcode = Opcode {code: 0x45};
pub const OP_PUSH_BYTES_70: Opcode = Opcode {code: 0x46};
pub const OP_PUSH_BYTES_71: Opcode = Opcode {code: 0x47};
pub const OP_PUSH_BYTES_72: Opcode = Opcode {code: 0x48};
pub const OP_PUSH_BYTES_73: Opcode = Opcode {code: 0x49};
pub const OP_PUSH_BYTES_74: Opcode = Opcode {code: 0x4a};
pub const OP_PUSH_BYTES_75: Opcode = Opcode {code: 0x4b};
pub const OP_PUSH_DATA_1: Opcode = Opcode {code: 0x4c};
pub const OP_PUSH_DATA_2: Opcode = Opcode {code: 0x4d};
pub const OP_PUSH_DATA_4: Opcode = Opcode {code: 0x4e};
pub const OP_1NEGATE: Opcode = Opcode {code: 0x4f};
pub const OP_RESERVED: Opcode = Opcode {code: 0x50};
pub const OP_1: Opcode = Opcode {code: 0x51};
pub const OP_2: Opcode = Opcode {code: 0x52};
pub const OP_3: Opcode = Opcode {code: 0x53};
pub const OP_4: Opcode = Opcode {code: 0x54};
pub const OP_5: Opcode = Opcode {code: 0x55};
pub const OP_6: Opcode = Opcode {code: 0x56};
pub const OP_7: Opcode = Opcode {code: 0x57};
pub const OP_8: Opcode = Opcode {code: 0x58};
pub const OP_9: Opcode = Opcode {code: 0x59};
pub const OP_10: Opcode = Opcode {code: 0x5a};
pub const OP_11: Opcode = Opcode {code: 0x5b};
pub const OP_12: Opcode = Opcode {code: 0x5c};
pub const OP_13: Opcode = Opcode {code: 0x5d};
pub const OP_14: Opcode = Opcode {code: 0x5e};
pub const OP_15: Opcode = Opcode {code: 0x5f};
pub const OP_16: Opcode = Opcode {code: 0x60};

// Flow Control
pub const OP_NOP: Opcode = Opcode {code: 0x61};
pub const OP_VER: Opcode = Opcode {code: 0x62};
pub const OP_IF: Opcode = Opcode {code: 0x63};
pub const OP_NOTIF: Opcode = Opcode {code: 0x64};
pub const OP_VERIF: Opcode = Opcode {code: 0x65};
pub const OP_VERNOTIF: Opcode = Opcode {code: 0x66};
pub const OP_ELSE: Opcode = Opcode {code: 0x67};
pub const OP_ENDIF: Opcode = Opcode {code: 0x68};
pub const OP_VERIFY: Opcode = Opcode {code: 0x69};
pub const OP_RETURN: Opcode = Opcode {code: 0x6a};

// Stack
pub const OP_TOALTSTACK: Opcode = Opcode {code: 0x6b};
pub const OP_FROMALTSTACK: Opcode = Opcode {code: 0x6c};
pub const OP_2DROP: Opcode = Opcode {code: 0x6d};
pub const OP_2DUP: Opcode = Opcode {code: 0x6e};
pub const OP_3DUP: Opcode = Opcode {code: 0x6f};
pub const OP_2OVER: Opcode = Opcode {code: 0x70};
pub const OP_2ROT: Opcode = Opcode {code: 0x71};
pub const OP_2SWAP: Opcode = Opcode {code: 0x72};
pub const OP_IFDUP: Opcode = Opcode {code: 0x73};
pub const OP_DEPTH: Opcode = Opcode {code: 0x74};
pub const OP_DROP: Opcode = Opcode {code: 0x75};
pub const OP_DUP: Opcode = Opcode {code: 0x76};
pub const OP_NIP: Opcode = Opcode {code: 0x77};
pub const OP_OVER: Opcode = Opcode {code: 0x78};
pub const OP_PICK: Opcode = Opcode {code: 0x79};
pub const OP_ROLL: Opcode = Opcode {code: 0x7a};
pub const OP_ROT: Opcode = Opcode {code: 0x7b};
pub const OP_SWAP: Opcode = Opcode {code: 0x7c};
pub const OP_TUCK: Opcode = Opcode {code: 0x7d};

// Splice
pub const OP_CAT: Opcode = Opcode {code: 0x7e};
pub const OP_SUBSTR: Opcode = Opcode {code: 0x7f};
pub const OP_LEFT: Opcode = Opcode {code: 0x80};
pub const OP_RIGHT: Opcode = Opcode {code: 0x81};
pub const OP_SIZE: Opcode = Opcode {code: 0x82};

// Bitwise Logic
pub const OP_INVERT: Opcode = Opcode {code: 0x83};
pub const OP_AND: Opcode = Opcode {code: 0x84};
pub const OP_OR: Opcode = Opcode {code: 0x85};
pub const OP_XOR: Opcode = Opcode {code: 0x86};
pub const OP_EQUAL: Opcode = Opcode {code: 0x87};
pub const OP_EQUALVERIFY: Opcode = Opcode {code: 0x88};

pub const OP_RESERVED1: Opcode = Opcode {code: 0x89};
pub const OP_RESERVED2: Opcode = Opcode {code: 0x8a};

// Arithmetic
pub const OP_1ADD: Opcode = Opcode {code: 0x8b};
pub const OP_1SUB: Opcode = Opcode {code: 0x8c};
pub const OP_2MUL: Opcode = Opcode {code: 0x8d};
pub const OP_2DIV: Opcode = Opcode {code: 0x8e};
pub const OP_NEGATE: Opcode = Opcode {code: 0x8f};
pub const OP_ABS: Opcode = Opcode {code: 0x90};
pub const OP_NOT: Opcode = Opcode {code: 0x91};
pub const OP_0NOTEQUAL: Opcode = Opcode {code: 0x92};
pub const OP_ADD: Opcode = Opcode {code: 0x93};
pub const OP_SUB: Opcode = Opcode {code: 0x94};
pub const OP_MUL: Opcode = Opcode {code: 0x95};
pub const OP_DIV: Opcode = Opcode {code: 0x96};
pub const OP_MOD: Opcode = Opcode {code: 0x97};
pub const OP_LSHIFT: Opcode = Opcode {code: 0x98};
pub const OP_RSHIFT: Opcode = Opcode {code: 0x99};
pub const OP_BOOLAND: Opcode = Opcode {code: 0x9a};
pub const OP_BOOLOR: Opcode = Opcode {code: 0x9b};
pub const OP_NUMEQUAL: Opcode = Opcode {code: 0x9c};
pub const OP_NUMEQUALVERIFY: Opcode = Opcode {code: 0x9d};
pub const OP_NUMNOTEQUAL: Opcode = Opcode {code: 0x9e};
pub const OP_LESSTHAN: Opcode = Opcode {code: 0x9f};
pub const OP_GREATERTHAN: Opcode = Opcode {code: 0xa0};
pub const OP_LESSTHANOREQUAL: Opcode = Opcode {code: 0xa1};
pub const OP_GREATERTHANOREQUAL: Opcode = Opcode {code: 0xa2};
pub const OP_MIN: Opcode = Opcode {code: 0xa3};
pub const OP_MAX: Opcode = Opcode {code: 0xa4};
pub const OP_WITHIN: Opcode = Opcode {code: 0xa5};

// Crypto
pub const OP_RIPEMD160: Opcode = Opcode {code: 0xa6};
pub const OP_SHA1: Opcode = Opcode {code: 0xa7};
pub const OP_SHA256: Opcode = Opcode {code: 0xa8};
pub const OP_HASH160: Opcode = Opcode {code: 0xa9};
pub const OP_HASH256: Opcode = Opcode {code: 0xaa};
pub const OP_CODESEPARATOR: Opcode = Opcode {code: 0xab};
pub const OP_CHECKSIG: Opcode = Opcode {code: 0xac};
pub const OP_CHECKSIGVERIFY: Opcode = Opcode {code: 0xad};
pub const OP_CHECKMULTISIG: Opcode = Opcode {code: 0xae};
pub const OP_CHECKMULTISIGVERIFY: Opcode = Opcode {code: 0xaf};

// Expansion
pub const OP_NOP1: Opcode = Opcode {code: 0xb0};
pub const OP_CHECKLOCKTIMEVERIFY: Opcode = Opcode {code: 0xb1};
pub const OP_CHECKSEQUENCEVERIFY: Opcode = Opcode {code: 0xb2};
pub const OP_NOP4: Opcode = Opcode {code: 0xb3};
pub const OP_NOP5: Opcode = Opcode {code: 0xb4};
pub const OP_NOP6: Opcode = Opcode {code: 0xb5};
pub const OP_NOP7: Opcode = Opcode {code: 0xb6};
pub const OP_NOP8: Opcode = Opcode {code: 0xb7};
pub const OP_NOP9: Opcode = Opcode {code: 0xb8};
pub const OP_NOP10: Opcode = Opcode {code: 0xb9};

// Unassigned
pub const OP_UNASSIGNED_186: Opcode = Opcode {code: 0xba};
pub const OP_UNASSIGNED_187: Opcode = Opcode {code: 0xbb};
pub const OP_UNASSIGNED_188: Opcode = Opcode {code: 0xbc};
pub const OP_UNASSIGNED_189: Opcode = Opcode {code: 0xbd};
pub const OP_UNASSIGNED_190: Opcode = Opcode {code: 0xbe};
pub const OP_UNASSIGNED_191: Opcode = Opcode {code: 0xbf};
pub const OP_UNASSIGNED_192: Opcode = Opcode {code: 0xc0};
pub const OP_UNASSIGNED_193: Opcode = Opcode {code: 0xc1};
pub const OP_UNASSIGNED_194: Opcode = Opcode {code: 0xc2};
pub const OP_UNASSIGNED_195: Opcode = Opcode {code: 0xc3};
pub const OP_UNASSIGNED_196: Opcode = Opcode {code: 0xc4};
pub const OP_UNASSIGNED_197: Opcode = Opcode {code: 0xc5};
pub const OP_UNASSIGNED_198: Opcode = Opcode {code: 0xc6};
pub const OP_UNASSIGNED_199: Opcode = Opcode {code: 0xc7};
pub const OP_UNASSIGNED_200: Opcode = Opcode {code: 0xc8};
pub const OP_UNASSIGNED_201: Opcode = Opcode {code: 0xc9};
pub const OP_UNASSIGNED_202: Opcode = Opcode {code: 0xca};
pub const OP_UNASSIGNED_203: Opcode = Opcode {code: 0xcb};
pub const OP_UNASSIGNED_204: Opcode = Opcode {code: 0xcc};
pub const OP_UNASSIGNED_205: Opcode = Opcode {code: 0xcd};
pub const OP_UNASSIGNED_206: Opcode = Opcode {code: 0xce};
pub const OP_UNASSIGNED_207: Opcode = Opcode {code: 0xcf};
pub const OP_UNASSIGNED_208: Opcode = Opcode {code: 0xd0};
pub const OP_UNASSIGNED_209: Opcode = Opcode {code: 0xd1};
pub const OP_UNASSIGNED_210: Opcode = Opcode {code: 0xd2};
pub const OP_UNASSIGNED_211: Opcode = Opcode {code: 0xd3};
pub const OP_UNASSIGNED_212: Opcode = Opcode {code: 0xd4};
pub const OP_UNASSIGNED_213: Opcode = Opcode {code: 0xd5};
pub const OP_UNASSIGNED_214: Opcode = Opcode {code: 0xd6};
pub const OP_UNASSIGNED_215: Opcode = Opcode {code: 0xd7};
pub const OP_UNASSIGNED_216: Opcode = Opcode {code: 0xd8};
pub const OP_UNASSIGNED_217: Opcode = Opcode {code: 0xd9};
pub const OP_UNASSIGNED_218: Opcode = Opcode {code: 0xda};
pub const OP_UNASSIGNED_219: Opcode = Opcode {code: 0xdb};
pub const OP_UNASSIGNED_220: Opcode = Opcode {code: 0xdc};
pub const OP_UNASSIGNED_221: Opcode = Opcode {code: 0xdd};
pub const OP_UNASSIGNED_222: Opcode = Opcode {code: 0xde};
pub const OP_UNASSIGNED_223: Opcode = Opcode {code: 0xdf};
pub const OP_UNASSIGNED_224: Opcode = Opcode {code: 0xe0};
pub const OP_UNASSIGNED_225: Opcode = Opcode {code: 0xe1};
pub const OP_UNASSIGNED_226: Opcode = Opcode {code: 0xe2};
pub const OP_UNASSIGNED_227: Opcode = Opcode {code: 0xe3};
pub const OP_UNASSIGNED_228: Opcode = Opcode {code: 0xe4};
pub const OP_UNASSIGNED_229: Opcode = Opcode {code: 0xe5};
pub const OP_UNASSIGNED_230: Opcode = Opcode {code: 0xe6};
pub const OP_UNASSIGNED_231: Opcode = Opcode {code: 0xe7};
pub const OP_UNASSIGNED_232: Opcode = Opcode {code: 0xe8};
pub const OP_UNASSIGNED_233: Opcode = Opcode {code: 0xe9};
pub const OP_UNASSIGNED_234: Opcode = Opcode {code: 0xea};
pub const OP_UNASSIGNED_235: Opcode = Opcode {code: 0xeb};
pub const OP_UNASSIGNED_236: Opcode = Opcode {code: 0xec};
pub const OP_UNASSIGNED_237: Opcode = Opcode {code: 0xed};
pub const OP_UNASSIGNED_238: Opcode = Opcode {code: 0xee};
pub const OP_UNASSIGNED_239: Opcode = Opcode {code: 0xef};
pub const OP_UNASSIGNED_240: Opcode = Opcode {code: 0xf0};
pub const OP_UNASSIGNED_241: Opcode = Opcode {code: 0xf1};
pub const OP_UNASSIGNED_242: Opcode = Opcode {code: 0xf2};
pub const OP_UNASSIGNED_243: Opcode = Opcode {code: 0xf3};
pub const OP_UNASSIGNED_244: Opcode = Opcode {code: 0xf4};
pub const OP_UNASSIGNED_245: Opcode = Opcode {code: 0xf5};
pub const OP_UNASSIGNED_246: Opcode = Opcode {code: 0xf6};
pub const OP_UNASSIGNED_247: Opcode = Opcode {code: 0xf7};
pub const OP_UNASSIGNED_248: Opcode = Opcode {code: 0xf8};
pub const OP_UNASSIGNED_249: Opcode = Opcode {code: 0xf9};
pub const OP_UNASSIGNED_250: Opcode = Opcode {code: 0xfa};
pub const OP_UNASSIGNED_251: Opcode = Opcode {code: 0xfb};
pub const OP_UNASSIGNED_252: Opcode = Opcode {code: 0xfc};
pub const OP_UNASSIGNED_253: Opcode = Opcode {code: 0xfd};
pub const OP_UNASSIGNED_254: Opcode = Opcode {code: 0xfe};
pub const OP_UNASSIGNED_255: Opcode = Opcode {code: 0xff};

impl From<u8> for Opcode {
    fn from(c: u8) -> Self {
        Opcode {code: c}
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OP_")?;
        match *self {
            OP_0 => write!(f, "0"),
            Opcode {code: c} if c >= 1 && c <= 75 => write!(f, "PUSH_BYTES_{}", c),
            OP_PUSH_DATA_1 => write!(f, "PUSH_DATA_1"),
            OP_PUSH_DATA_2 => write!(f, "PUSH_DATA_2"),
            OP_PUSH_DATA_4 => write!(f, "PUSH_DATA_4"),
            OP_1NEGATE => write!(f, "1NEGATE"),
            OP_RESERVED => write!(f, "RESERVED"),
            Opcode{code: c} if c >= 0x51 && c <= 0x60 => write!(f, "{}", c - 0x50),
            OP_NOP => write!(f, "NOP"),
            OP_VER => write!(f, "VER"),
            OP_IF => write!(f, "IF"),
            OP_NOTIF => write!(f, "NOTIF"),
            OP_VERIF => write!(f, "VERIF"),
            OP_VERNOTIF => write!(f, "VERNOTIF"),
            OP_ELSE => write!(f, "ELSE"),
            OP_ENDIF => write!(f, "ENDIF"),
            OP_VERIFY => write!(f, "VERIFY"),
            OP_RETURN => write!(f, "RETURN"),
            OP_TOALTSTACK => write!(f, "TOALTSTACK"),
            OP_FROMALTSTACK => write!(f, "FROMALTSTACK"),
            OP_2DROP => write!(f, "2DROP"),
            OP_2DUP => write!(f, "2DUP"),
            OP_3DUP => write!(f, "3DUP"),
            OP_2OVER => write!(f, "2OVER"),
            OP_2ROT => write!(f, "2ROT"),
            OP_2SWAP => write!(f, "2SWAP"),
            OP_IFDUP => write!(f, "IFDUP"),
            OP_DEPTH => write!(f, "DEPTH"),
            OP_DROP => write!(f, "DROP"),
            OP_DUP => write!(f, "DUP"),
            OP_NIP => write!(f, "NIP"),
            OP_OVER => write!(f, "OVER"),
            OP_PICK => write!(f, "PICK"),
            OP_ROLL => write!(f, "ROLL"),
            OP_ROT => write!(f, "ROT"),
            OP_SWAP => write!(f, "SWAP"),
            OP_TUCK => write!(f, "TUCK"),
            OP_CAT => write!(f, "CAT"),
            OP_SUBSTR => write!(f, "SUBSTR"),
            OP_LEFT => write!(f, "LEFT"),
            OP_RIGHT => write!(f, "RIGHT"),
            OP_SIZE => write!(f, "SIZE"),
            OP_INVERT => write!(f, "INVERT"),
            OP_AND => write!(f, "AND"),
            OP_OR => write!(f, "OR"),
            OP_XOR => write!(f, "XOR"),
            OP_EQUAL => write!(f, "EQUAL"),
            OP_EQUALVERIFY => write!(f, "EQUALVERIFY"),
            OP_RESERVED1 => write!(f, "RESERVED1"),
            OP_RESERVED2 => write!(f, "RESERVED2"),
            OP_1ADD => write!(f, "1ADD"),
            OP_1SUB => write!(f, "1SUB"),
            OP_2MUL => write!(f, "2MUL"),
            OP_2DIV => write!(f, "2DIV"),
            OP_NEGATE => write!(f, "NEGATE"),
            OP_ABS => write!(f, "ABS"),
            OP_NOT => write!(f, "NOT"),
            OP_0NOTEQUAL => write!(f, "0NOTEQUAL"),
            OP_ADD => write!(f, "ADD"),
            OP_SUB => write!(f, "SUB"),
            OP_MUL => write!(f, "MUL"),
            OP_DIV => write!(f, "DIV"),
            OP_MOD => write!(f, "MOD"),
            OP_LSHIFT => write!(f, "LSHIFT"),
            OP_RSHIFT => write!(f, "RSHIFT"),
            OP_BOOLAND => write!(f, "BOOLAND"),
            OP_BOOLOR => write!(f, "BOOLOR"),
            OP_NUMEQUAL => write!(f, "NUMEQUAL"),
            OP_NUMEQUALVERIFY => write!(f, "NUMEQUALVERIFY"),
            OP_NUMNOTEQUAL => write!(f, "NUMNOTEQUAL"),
            OP_LESSTHAN => write!(f, "LESSTHAN"),
            OP_GREATERTHAN => write!(f, "GREATERTHAN"),
            OP_LESSTHANOREQUAL => write!(f, "LESSTHANOREQUAL"),
            OP_GREATERTHANOREQUAL => write!(f, "GREATERTHANOREQUAL"),
            OP_MIN => write!(f, "MIN"),
            OP_MAX => write!(f, "MAX"),
            OP_WITHIN => write!(f, "WITHIN"),
            OP_RIPEMD160 => write!(f, "RIPEMD160"),
            OP_SHA1 => write!(f, "SHA1"),
            OP_SHA256 => write!(f, "SHA256"),
            OP_HASH160 => write!(f, "HASH160"),
            OP_HASH256 => write!(f, "HASH256"),
            OP_CODESEPARATOR => write!(f, "CODESEPARATOR"),
            OP_CHECKSIG => write!(f, "CHECKSIG"),
            OP_CHECKSIGVERIFY => write!(f, "CHECKSIGVERIFY"),
            OP_CHECKMULTISIG => write!(f, "CHECKMULTISIG"),
            OP_CHECKMULTISIGVERIFY => write!(f, "CHECKMULTISIGVERIFY"),
            OP_NOP1 => write!(f, "NOP1"),
            OP_CHECKLOCKTIMEVERIFY => write!(f, "CHECKLOCKTIMEVERIFY"),
            OP_CHECKSEQUENCEVERIFY => write!(f, "CHECKSEQUENCEVERIFY"),
            Opcode {code: c} if c >= 0xb3 && c <= 0xb9 => write!(f, "NOP{}", c-0xb3 + 4),
            Opcode {code: c} => write!(f, "UNASSIGNED_{}", c),
        }
    }
}

// All the Opcodes
lazy_static! {
    pub static ref OPCODES: HashMap<u8, Opcode> = {
        let mut map = HashMap::with_capacity(256);
        for i in 0..=255 {
            let opcode = Opcode::from(i);
            map.insert(i, opcode);
        }
        map
    };
    pub static ref DISABLED_OPCODES: HashSet<Opcode> = {
        let opcodes = [OP_CAT, OP_SUBSTR, OP_LEFT, OP_RIGHT, OP_INVERT, OP_AND, OP_OR, OP_XOR,
            OP_2MUL, OP_2DIV, OP_MUL, OP_MOD, OP_DIV, OP_LSHIFT, OP_RSHIFT];
        let mut set = HashSet::with_capacity(15);
        for op in opcodes {
            set.insert(op);
        }
        set
    };
}
