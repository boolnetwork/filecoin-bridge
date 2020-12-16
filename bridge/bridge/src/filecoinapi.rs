use tokio::runtime::Runtime as tokioRuntime;
use lotus_api_forest::api::MpoolApi;
use forest_message;
use forest_cid;
use forest_vm::{self, Serialized};
use forest_encoding::Cbor;
use forest_crypto;
use num_traits::cast::FromPrimitive;
use std::env;

pub fn get_nonce(addr: forest_address::Address) -> u64 {
    let mut rt = tokioRuntime::new().unwrap();
    let http = lotus_api_forest::Http::new("http://127.0.0.1:1234/rpc/v0");
    let ret:u64 = rt.block_on(http.mpool_get_nonce(&addr)).unwrap();
    ret
}

pub fn message_create(from:Vec<u8>,to:Vec<u8>,val:u128,) -> (forest_message::UnsignedMessage ,Vec<u8>){
    let from_addr = forest_address::Address::new_secp256k1(&from).unwrap();
    let to_addr = forest_address::Address::from_bytes(&to).unwrap();
    let nonce = get_nonce(from_addr.clone());

    println!("from {:?}",from);
    println!("to {:?}",to);
    println!("val {:?}",val);
    println!("from_addr {}",from_addr);
    println!("to_addr {}",to_addr);
    println!("nonce {}",nonce);
//    let unsigned_msg = forest_message::UnsignedMessage::builder()
//        .to(from_addr)
//        .sequence(nonce)
//        .from(to_addr)
//        .build()
//        .unwrap();
    let unsignedtx = forest_message::UnsignedMessage {
        version: 0,
        to: to_addr,
        from: from_addr,
        sequence: nonce,
        value: forest_vm::TokenAmount::from_u128(val).unwrap(),
        method_num: 0u64,
        params: Serialized::new(vec![0u8]),
        gas_limit: 600000i64,
        gas_fee_cap:forest_vm::TokenAmount::from_u128(600000u128).unwrap(),
        gas_premium:forest_vm::TokenAmount::from_u128(600000u128).unwrap(),
    };
    (unsignedtx.clone(),unsignedtx.to_signing_bytes())
}

pub fn send_fc_message(message: forest_message::SignedMessage) -> Result<forest_cid::Cid,String> {
    let auth_res = env::var("auth");
    if auth_res.is_err(){
        return Err("no auth key".to_string());
    }

    let auth = auth_res.unwrap();
    let mut rt = tokioRuntime::new().unwrap();
    let http = lotus_api_forest::Http::new_auth("http://127.0.0.1:1234/rpc/v0",auth);
    match rt.block_on(http.mpool_push(&message)){
        Ok(cid) => Ok(cid),
        Err(err) => Err("".to_string()),
    }
}