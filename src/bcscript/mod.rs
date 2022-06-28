pub mod opcodes;
pub mod parse;
pub mod script;
pub mod interpret;
pub mod public_key;


pub(crate) fn main() {
    let sig_script = "483045022100fcb600ea44edb6b3c9408479c9e29d468bf623e5b465be4d594141b0d7611d2a022069d470033446f0b6c8c85ced09b3e3c2b675e21a0f3017a5fac2ad064a4d1d1e012103dd2162aaf74d3f2e0634ad778380eeecea6ac5f2e53411a128a323ad260d0dc7";
    let pk_script = "76a9148b6305816c87626a9ac4b972c5c376663b2f5dc888ac";
    let hex_script = sig_script.to_owned() + pk_script;
    //let hex_script = "515560606b6c05abcdefabcd";
    let bin_script = hex::decode(hex_script).unwrap();
    let script = parse::parse_script(&bin_script).unwrap();
    println!("{:?}", &script);

    let _ = interpret::interpret(&bin_script, true).unwrap();
}
