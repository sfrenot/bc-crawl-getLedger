use crate::bcparse::Transaction;

pub mod opcodes;
pub mod parse;
pub mod script;
pub mod interpret;


pub(crate) fn main() {
    let sig_script = "47304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901";
    let pk_script = "410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac";
    // We add a code separator "ab" to separate sig and pk script as they are theorically supposed to be executed one after another and not together
    let hex_script = sig_script.to_owned() + "ab" + pk_script;
    let bin_script = hex::decode(hex_script).unwrap();

    let transaction: Transaction = serde_json::from_str(r#"{
      "hash": "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
      "version": 1,
      "is_segwit": false,
      "inputs": [
        {
          "prev_output": {
            "hash": "0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9",
            "idx": 0
          },
          "signature_script": "47304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901",
          "sequence": 4294967295
        }
      ],
      "outputs": [
        {
          "value": 1000000000,
          "pub_key_script": "4104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac"
        },
        {
          "value": 4000000000,
          "pub_key_script": "410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac"
        }
      ],
      "witnesses": [],
      "lock_time": 0
    }"#).unwrap();
    //println!("{:#?}", &transaction);
    //let _script = parse::parse_script(&bin_script).unwrap();
    //println!("{:?}", &_script);

    let _ = interpret::interpret(&bin_script, &transaction, 0, true).unwrap();
}
