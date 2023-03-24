use std::cmp::{max, min};

use bitcoin_hashes::{hash160, ripemd160, sha1, sha256, sha256d};
use bitcoin_hashes::Hash;
use colored::Colorize;
use tabled::{Alignment, MaxWidth, MinWidth, Modify, Style};
use tabled::builder::Builder;
use tabled::object::Rows;
use crate::bcparse::Transaction;

use super::opcodes::*;
use super::parse::{parse_one_op, parse_script};
use super::script::{as_bool, as_script_nb};
use super::script::*;

#[allow(dead_code)] // SIGHASH_ALL is the default behavior so not directly used
const SIGHASH_ALL: u8 = 1; // sign all inputs and outputs -> default
const SIGHASH_NONE: u8 = 2; // sign all inputs and no outputs
const SIGHASH_SINGLE: u8 = 3; // sign all inputs and one output corresponding to the input
const SIGHASH_ANYONECANPAY: u8 = 0x80; // modifier to be used with other flags, sign only one input

pub struct Stack {
    pub main: Vec<Vec<u8>>,
    pub alt: Vec<Vec<u8>>,
}

impl Stack {
    fn push(&mut self, bytes: Vec<u8>) -> Result<(), ScriptError> {
        if bytes.len() > MAX_SCRIPT_ELEMENT_SIZE {
            return Err(ScriptError::PushSize);
        }
        self.main.push(bytes);

        if self.main.len() + self.alt.len() >= MAX_STACK_SIZE {
            return Err(ScriptError::StackOverflow);
        }
        Ok(())
    }

    fn push_alt(&mut self, bytes: Vec<u8>) -> Result<(), ScriptError> {
        if bytes.len() > MAX_SCRIPT_ELEMENT_SIZE {
            return Err(ScriptError::PushSize);
        }
        self.alt.push(bytes);

        if self.main.len() + self.alt.len() >= MAX_STACK_SIZE {
            return Err(ScriptError::StackOverflow);
        }
        Ok(())
    }

    fn pop(&mut self) -> Result<Vec<u8>, ScriptError> {
        self.main.pop().ok_or(ScriptError::InvalidStackOperation)
    }

    fn pop_alt(&mut self) -> Result<Vec<u8>, ScriptError> {
        self.alt.pop().ok_or(ScriptError::InvalidAltStackOperation)
    }

    // Usage: stack.top(0) to get last element or stack.top(-1) to get 2nd element from the end
    fn top(&self, pos: i64) -> Result<Vec<u8>, ScriptError> {
        if pos > 0 {
            panic!("Wrong index given (positive)")
        }
        let idx = self.main.len() as i64 - 1 + pos;
        if idx < 0 {
            return Err(ScriptError::InvalidStackOperation);
        }
        Ok(self.main.get(idx as usize).ok_or(ScriptError::InvalidStackOperation)?.to_vec())
    }

    fn rm_top(&mut self, pos: i64) -> Result<Vec<u8>, ScriptError> {
        if pos > 0 {
            panic!("Wrong index given (positive)")
        }
        let idx = self.main.len() as i64 - 1 + pos;
        if idx < 0 {
            return Err(ScriptError::InvalidStackOperation);
        }
        Ok(self.main.remove(idx as usize))
    }

    fn swap_top(&mut self, a: i64, b: i64) -> Result<(), ScriptError> {
        if a > 0 || b > 0 {
            panic!("Wrong index given (positive)")
        }
        let idx_a = self.main.len() as i64 - 1 + a;
        let idx_b = self.main.len() as i64 - 1 + b;
        if idx_a < 0 || idx_b < 0 {
            return Err(ScriptError::InvalidStackOperation);
        }
        self.main.swap(idx_a as usize, idx_b as usize);
        Ok(())
    }
}

fn print_stack(stack: &[Vec<u8>], title: &str, min_width: usize, max_width: usize) {
    let mut hex_stack = {
        let mut vec = Vec::new();
        for elem in stack {
            vec.push(format!("0x{}", hex::encode(elem)))
        }
        if stack.is_empty() {
            vec.push(String::from(""));
        }
        vec
    };
    hex_stack.reverse();

    let mut table_builder = Builder::default().set_columns([title]);
    for item in &hex_stack {
        table_builder = table_builder.add_record([item]);
    }

    let table = table_builder.build().with(Style::modern())
        .with(MaxWidth::wrapping(max_width))
        .with(MinWidth::new(min_width))
        .with(Modify::new(Rows::new(1..))
            .with(Alignment::left()));

    print!("{}", &table.to_string());
}

// Constants to configure step-by-step script execution display
const MAX_SCRIPT_DISPLAY_WIDTH: usize = 80;
const MIN_SCRIPT_DISPLAY_WIDTH: usize = 30;

fn print_state(stack: &Stack, script: &Script, step_nb: usize) {
    let mut display_max_width = MAX_SCRIPT_DISPLAY_WIDTH;
    let display_min_width = MIN_SCRIPT_DISPLAY_WIDTH;

    if let Some((w, _)) = term_size::dimensions() {
        if w > MIN_SCRIPT_DISPLAY_WIDTH && w < MAX_SCRIPT_DISPLAY_WIDTH {
            display_max_width = w;
        }
    }

    println!("\n\n");

    // Print remaining script instructions
    if !script.is_empty() {
        println!("{} (Step {})\n", "Script".bold(), step_nb);

        let colors = ["green", "yellow", "magenta", "cyan", "white"];
        let mut line_len = 0;
        for i in 0..script.len() {
            let mut item_str = format!("{:?}", script[i]);
            if line_len + item_str.len() > display_max_width && line_len > 0 {
                println!("\n");
                line_len = 0;
            }

            let color = String::from("bright ") + match script[i] {
                ScriptItem::ByteArray(..) => "blue",
                ScriptItem::Opcode(op) => {
                    colors[(op.code % colors.len() as u8) as usize]
                }
            };

            if item_str.len() > display_max_width {
                while item_str.len() > display_max_width {
                    let sub_str = &item_str[..display_max_width];
                    println!("{}", sub_str.bold().black().on_color(color.as_str()));
                    item_str = String::from(&item_str[display_max_width..])
                }
                println!("{}\n", item_str.bold().black().on_color(color));
                line_len = 0;
            } else {
                print!("{}", item_str.bold().black().on_color(color));
                line_len += item_str.len();

                if i != script.len() - 1 {
                    print!(" ");
                    line_len += 1;
                }
            }
        }
        println!();
    } else {
        println!("{}", "Final state".bold());
    }

    print_stack(&stack.main, "Main Stack", display_min_width, display_max_width);

    if !&stack.alt.is_empty() {
        print_stack(&stack.alt, "Alt Stack", display_min_width, display_max_width);
    }
}

fn compute_tx_hash(tx: &Transaction, input_idx: usize, subscript: &[u8], sig_type: u8) -> sha256d::Hash {
    let mut ntx = tx.clone();

    let anyone_can_pay = sig_type & SIGHASH_ANYONECANPAY != 0;
    let hash_single = sig_type & 0x1f == SIGHASH_SINGLE;
    let hash_none = sig_type & 0x1f == SIGHASH_NONE;

    // we check if input idx is in range in normal and SIGHASH_SINGLE cases
    if input_idx >= tx.inputs.len() || (hash_single && input_idx >= tx.outputs.len()) {
        let mut result = vec![0, 32];
        result[0] = 1;
        // Here we mimick the Bitcoin Core behaviour and
        // return the hash of 100...00 if we have an index error
        return sha256d::Hash::from_slice(&result).unwrap()
    }

    if hash_none {
        ntx.outputs.clear();
    }

    if hash_single {
        ntx.outputs.truncate(input_idx + 1);
        for i in 0..ntx.outputs.len() - 1 {
            ntx.outputs[i].value = -1;
            ntx.outputs[i].pub_key_script.clear();
        }
    }

    if anyone_can_pay {
        ntx.inputs = ntx.inputs[input_idx..input_idx+1].to_vec();
    }

    // We set all the sig scripts to empty scripts except the
    // one for the current input which is set to subscript
    for i in 0..ntx.inputs.len() {
        if i != input_idx {
            ntx.inputs[i].signature_script.clear();

            if hash_single || hash_none {
                ntx.inputs[i].sequence = 0;
            }
        } else {
            ntx.inputs[i].signature_script = hex::encode(subscript);
        }
    }

    let mut bytes = ntx.to_bytes(false);
    bytes.extend((sig_type as u32).to_le_bytes());
    sha256d::Hash::hash(&bytes)
}

fn check_sig(sig: &mut Vec<u8>, pub_key: &[u8], subscript: &[u8], tx: &Transaction, input_idx: usize) -> bool {
    let sig_type = sig.last().copied().unwrap();
    sig.pop();

    let key = libsecp256k1::PublicKey::parse_slice(&pub_key, None).unwrap();
    // TODO: from block version 3, signatures should be parsed with parse_der()
    let sig = libsecp256k1::Signature::parse_der_lax(&sig).unwrap();

    let tx_hash = compute_tx_hash(tx, input_idx, &subscript, sig_type);
    let message = libsecp256k1::Message::parse_slice(&tx_hash).unwrap();

    libsecp256k1::verify(&message, &sig, &key)
}

pub fn interpret(script: &[u8], tx: &Transaction, input_idx: usize, verbose: bool) -> Result<(), ScriptError> {
    let mut stack = Stack { main: Vec::with_capacity(20), alt: Vec::with_capacity(20) };
    let mut condition_stack: Vec<bool> = Vec::with_capacity(10);
    let mut execute: bool;
    let mut op_count: usize = 0;
    let mut pc: usize = 0;
    let mut subscript_start: usize = 0;

    if script.len() > MAX_SCRIPT_SIZE {
        return Err(ScriptError::ScriptSize);
    }

    let mut display_script: Script = Script::new();
    let mut step_nb: usize = 0;
    if verbose {
        display_script = parse_script(script)?;
        print_state(&stack, &display_script, step_nb);
    }

    while let Some(item) = parse_one_op(script, &mut pc)? {
        execute = !condition_stack.contains(&false);

        match item {
            ScriptItem::ByteArray(b) => {
                if b.len() > MAX_SCRIPT_ELEMENT_SIZE {
                    return Err(ScriptError::PushSize);
                }
                if execute {
                    stack.push(b)?
                }
            }
            ScriptItem::Opcode(op) => {
                if DISABLED_OPCODES.contains(&op) {
                    return Err(ScriptError::DisabledOpcode);
                }

                if op.code > OP_16.code {
                    op_count += 1;
                }
                if op_count > MAX_OPS_PER_SCRIPT {
                    return Err(ScriptError::OpCount);
                }

                if execute || (OP_IF.code <= op.code && op.code <= OP_ENDIF.code) {
                    match op {
                        //
                        // Data Push
                        //
                        OP_0 => stack.push(to_script_nb(0))?,
                        OP_1NEGATE => stack.push(to_script_nb(-1))?,
                        Opcode { code: c } if c >= OP_1.code && c <= OP_16.code => stack.push(to_script_nb((c - OP_1.code + 1) as i64))?,

                        //
                        // Flow Control
                        //
                        OP_NOP => {}
                        OP_CHECKSEQUENCEVERIFY => {}
                        OP_CHECKLOCKTIMEVERIFY => {}
                        OP_NOP1 | OP_NOP4 | OP_NOP5 | OP_NOP6 |
                        OP_NOP7 | OP_NOP8 | OP_NOP9 | OP_NOP10 => {}
                        OP_IF | OP_NOTIF => {
                            let mut condition = false;
                            if execute {
                                condition = as_bool(&stack.pop()?);
                                if op == OP_NOTIF {
                                    condition = !condition;
                                }
                            }
                            condition_stack.push(condition);
                        }
                        OP_ELSE => {
                            if condition_stack.is_empty() {
                                return Err(ScriptError::UnbalancedConditional);
                            }
                            let last = condition_stack.last_mut().unwrap();
                            *last = !*last;
                        }
                        OP_ENDIF => {
                            if condition_stack.is_empty() {
                                return Err(ScriptError::UnbalancedConditional);
                            }
                            condition_stack.pop();
                        }
                        OP_VERIFY => {
                            let v = as_bool(&stack.pop()?);
                            if !v {
                                return Err(ScriptError::Verify);
                            }
                        }
                        OP_RETURN => return Err(ScriptError::OpReturn),

                        //
                        // Stack
                        //
                        OP_TOALTSTACK => {
                            let v = stack.pop()?;
                            stack.push_alt(v)?
                        }
                        OP_FROMALTSTACK => {
                            let v = stack.pop_alt()?;
                            stack.push(v)?
                        }
                        OP_2DROP => {
                            stack.pop()?;
                            stack.pop()?;
                        }
                        OP_2DUP => {
                            let v1 = stack.top(-1)?;
                            let v2 = stack.top(0)?;
                            stack.push(v1)?;
                            stack.push(v2)?
                        }
                        OP_3DUP => {
                            let v1 = stack.top(-2)?;
                            let v2 = stack.top(-1)?;
                            let v3 = stack.top(0)?;
                            stack.push(v1)?;
                            stack.push(v2)?;
                            stack.push(v3)?
                        }
                        OP_2OVER => {
                            let v1 = stack.top(-3)?;
                            let v2 = stack.top(-2)?;
                            stack.push(v1)?;
                            stack.push(v2)?
                        }
                        OP_2ROT => {
                            let v1 = stack.rm_top(-5)?;
                            let v2 = stack.rm_top(-4)?;
                            stack.push(v1)?;
                            stack.push(v2)?
                        }
                        OP_2SWAP => {
                            stack.swap_top(0, -2)?;
                            stack.swap_top(-1, -3)?
                        }
                        OP_IFDUP => {
                            let v = stack.top(0)?;
                            if as_bool(&v) {
                                stack.push(v)?
                            }
                        }
                        OP_DEPTH => {
                            let v = to_script_nb(stack.main.len() as i64);
                            stack.push(v)?
                        }
                        OP_DROP => { stack.pop()?; }
                        OP_DUP => {
                            let v = stack.top(0)?;
                            stack.push(v)?
                        }
                        OP_NIP => { stack.rm_top(-1)?; }
                        OP_OVER => {
                            let v = stack.top(-1)?;
                            stack.push(v)?
                        }
                        OP_PICK => {
                            let n = as_script_nb(&stack.pop()?)?;
                            let v = stack.top(-n)?;
                            stack.push(v)?
                        }
                        OP_ROLL => {
                            let n = as_script_nb(&stack.pop()?)?;
                            let v = stack.rm_top(-n)?;
                            stack.push(v)?
                        }
                        OP_ROT => {
                            let v = stack.rm_top(-2)?;
                            stack.push(v)?
                        }
                        OP_SWAP => stack.swap_top(0, -1)?,
                        OP_TUCK => {
                            let v1 = stack.pop()?;
                            let v2 = stack.pop()?;
                            let v3 = v1.clone();
                            stack.push(v1)?;
                            stack.push(v2)?;
                            stack.push(v3)?
                        }

                        //
                        // Splice
                        //
                        OP_SIZE => {
                            let v = to_script_nb(stack.top(0)?.len() as i64);
                            stack.push(v)?
                        }

                        //
                        // Bitwise Logic
                        //
                        OP_EQUAL | OP_EQUALVERIFY => {
                            let v1 = stack.pop()?;
                            let v2 = stack.pop()?;
                            stack.push(to_script_bool(v1 == v2))?;

                            if op == OP_EQUALVERIFY {
                                if v1 == v2 {
                                    stack.pop()?;
                                } else {
                                    return Err(ScriptError::EqualVerify);
                                }
                            }
                        }

                        //
                        // Arithmetic
                        //
                        OP_1ADD | OP_1SUB | OP_NEGATE | OP_ABS | OP_NOT | OP_0NOTEQUAL => {
                            let mut v = as_script_nb(&stack.pop()?)?;
                            match op {
                                OP_1ADD => v += 1,
                                OP_1SUB => v -= 1,
                                OP_NEGATE => v *= -1,
                                OP_ABS => v = v.abs(),
                                OP_NOT => v = (v == 0) as i64,
                                OP_0NOTEQUAL => v = (v != 0) as i64,
                                _ => panic!()
                            }
                            stack.push(to_script_nb(v))?
                        }
                        OP_ADD | OP_SUB | OP_BOOLAND | OP_BOOLOR | OP_NUMEQUAL | OP_NUMEQUALVERIFY |
                        OP_NUMNOTEQUAL | OP_LESSTHAN | OP_GREATERTHAN | OP_LESSTHANOREQUAL |
                        OP_GREATERTHANOREQUAL | OP_MIN | OP_MAX => {
                            let v2 = as_script_nb(&stack.pop()?)?;
                            let v1 = as_script_nb(&stack.pop()?)?;
                            let res = match op {
                                OP_ADD => v1 + v2,
                                OP_SUB => v1 - v2,
                                OP_BOOLAND => (v1 != 0 && v2 != 0) as i64,
                                OP_BOOLOR => (v1 != 0 || v2 != 0) as i64,
                                OP_NUMEQUAL | OP_NUMEQUALVERIFY => (v1 == v2) as i64,
                                OP_NUMNOTEQUAL => (v1 != v2) as i64,
                                OP_LESSTHAN => (v1 < v2) as i64,
                                OP_GREATERTHAN => (v1 > v2) as i64,
                                OP_LESSTHANOREQUAL => (v1 <= v2) as i64,
                                OP_GREATERTHANOREQUAL => (v1 >= v2) as i64,
                                OP_MIN => min(v1, v2),
                                OP_MAX => max(v1, v2),
                                _ => panic!()
                            };
                            stack.push(to_script_nb(res))?;

                            if op == OP_NUMEQUALVERIFY {
                                if v1 == v2 {
                                    stack.pop()?;
                                } else {
                                    return Err(ScriptError::NumEqualVerify);
                                }
                            }
                        }
                        OP_WITHIN => {
                            let max = stack.pop()?;
                            let min = stack.pop()?;
                            let x = stack.pop()?;
                            let res = (min <= x && x < max) as i64;
                            stack.push(to_script_nb(res))?
                        }

                        //
                        // Crypto
                        //
                        OP_RIPEMD160 | OP_SHA1 | OP_SHA256 | OP_HASH160 | OP_HASH256 => {
                            let v = stack.pop()?;
                            let res = match op {
                                OP_RIPEMD160 => ripemd160::Hash::hash(&v).to_vec(),
                                OP_SHA1 => sha1::Hash::hash(&v).to_vec(),
                                OP_SHA256 => sha256::Hash::hash(&v).to_vec(),
                                OP_HASH160 => hash160::Hash::hash(&v).to_vec(),
                                OP_HASH256 => sha256d::Hash::hash(&v).to_vec(),
                                _ => panic!()
                            };
                            stack.push(res)?
                        }
                        OP_CODESEPARATOR => subscript_start = pc,
                        OP_CHECKSIG | OP_CHECKSIGVERIFY => {
                            let pub_key = stack.pop()?;
                            let mut signature = stack.pop()?;

                            // part of the script that will be included in the serialized transaction
                            let mut subscript = script[subscript_start..].to_vec();

                            // we remove the sig if present and separators from the script_code
                            let to_delete = ScriptItem::ByteArray(signature.clone());
                            find_and_delete(&mut subscript, &[to_delete, ScriptItem::Opcode(OP_CODESEPARATOR)])?;

                            let valid = check_sig(&mut signature, &pub_key, &subscript, tx, input_idx);
                            stack.push(to_script_bool(valid))?;

                            if op == OP_CHECKSIGVERIFY {
                                if valid {
                                    stack.pop()?;
                                } else {
                                    return Err(ScriptError::CheckSigVerify)
                                }
                            }
                        }

                        OP_CHECKMULTISIG | OP_CHECKMULTISIGVERIFY => {}

                        _ => return Err(ScriptError::BadOpcode)
                    }
                }
            }
        }

        if verbose {
            display_script.remove(0);
            step_nb += 1;
            print_state(&stack, &display_script, step_nb);
        }
    }

    if !condition_stack.is_empty() {
        return Err(ScriptError::UnbalancedConditional);
    }

    if stack.main.is_empty() {
        return Err(ScriptError::EmptyStack)
    }

    if !as_bool(&stack.top(0)?) {
        return Err(ScriptError::EvalFalse)
    }

    Ok(())
}