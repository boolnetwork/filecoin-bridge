#![allow(dead_code)]

use chain::{Transaction,TransactionInput};
use serialization::{Stream,Serializable};
use serialization::bytes::Bytes;
use script::{TransactionInputSigner, Script, SignatureVersion, verify_script
             , Opcode, VerificationFlags, ScriptWitness, TransactionSignatureChecker};
use script::builder::{Builder};
use keys::{Public};
use btprimitives::hash::{H256 , H520};
use bitcrypto::{sha256, ripemd160};
use libsecp256k1::PublicKey;
#[macro_use]
extern crate lazy_static;
use std::sync::Mutex;
use std::collections::HashMap;
use node_tss::sign_vec;
use rustc_hex::FromHex;
use rustc_hex::ToHex;
use serialization::{deserialize};

use libsecp256k1::{Signature as secpSignature};
use secp256k1::key::SecretKey;

use std::thread::sleep;
use std::time::Duration;
use curv::FE;
use curv::elliptic::curves::traits::ECScalar;

use log::{debug, info};

lazy_static!{
    pub static ref PUBKEY_STORE: Mutex<HashMap<&'static str,Vec<u8>>> = Mutex::new(HashMap::default());
    pub static ref STORE_LIST: Mutex<Vec<&'static str>> =  Mutex::new(vec!["test"]);
}

pub fn save_pubkey_store(pubkey:Vec<u8>,store: &'static str){
    let mut a = PUBKEY_STORE.lock().unwrap();
    a.insert(store,pubkey);
}

pub fn get_pubkey_store() -> HashMap<&'static str,Vec<u8>> {
    PUBKEY_STORE.lock().unwrap().clone()
}


/// Transaction transform
fn tx_to_hex(st_tx: Transaction) -> Bytes {
    let mut stream = Stream::new();
    st_tx.serialize(&mut stream);
    stream.out()
}

/// Transaction transform
fn tx_from_hex(s:&str) -> Transaction {
    let tx: Transaction = deserialize(&s.from_hex::<Vec<u8>>().unwrap() as &[u8]).unwrap();
    tx
}

/// Transaction transform
fn str_to_u8(s:&str) -> Vec<u8> {
    s.from_hex::<Vec<u8>>().unwrap()
}

/// pubkey and script
pub fn set_pubkey(pubkey:Vec<u8>,store: &'static str) {
    save_pubkey_store(pubkey , store);
}

/// pubkey and script
fn get_pubkey() -> Vec<u8>{
    let map = get_pubkey_store();
    if map.contains_key("boolbtc.store") {
        return vec![0u8];
    }else{
        return map.get("boolbtc.store").unwrap().clone();
    }
}

/// pubkey and script
fn get_public_key() -> Public {
    let mut pub_key_u8: [u8;65] = [0u8;65];
    pub_key_u8.copy_from_slice(&get_pubkey());

    let pub_key = PublicKey::parse(&pub_key_u8).unwrap();
    let mut public = H520::default();
    public.copy_from_slice(&pub_key.serialize());
    Public::Normal(public)
}

/// pubkey and script
#[allow(dead_code)]
fn get_public_key_in_2(pubkey:Vec<u8>) -> Public {
    let mut pub_key_u8: [u8;65] = [0u8;65];
    pub_key_u8.copy_from_slice(&pubkey[..]);

    let pub_key = PublicKey::parse(&pub_key_u8).unwrap();
    let mut public = H520::default();
    public.copy_from_slice(&pub_key.serialize());
    Public::Normal(public)
}

/// pubkey and script
fn get_public_key_in(pubkey:Vec<u8>) -> Public {
    let mut public = H520::default();
    public.copy_from_slice(&pubkey[..]);
    Public::Normal(public)
}

/// pubkey and script
fn pubkey_to_pubkeyhash_script(pubkey: Vec<u8>) -> Script{
    let sha256_pubkey: H256 = sha256(&pubkey[..]).into();
    let u8_sha256_pubkey: [u8;32] = sha256_pubkey.into();
    let pubkeyhash = ripemd160(&u8_sha256_pubkey);
    Builder::build_p2pkh(&pubkeyhash)
}

#[allow(dead_code)]
fn pubkey_to_pubkeyhash_script_2(pubkey: Vec<u8>) -> Script{
    let sha256_pubkey: H256 = sha256(&pubkey[..]).into();
    let u8_sha256_pubkey: [u8;32] = sha256_pubkey.into();
    let pubkeyhash = ripemd160(&u8_sha256_pubkey);
    Builder::build_p2pkh(&pubkeyhash)
}

/// pubkey and script
fn get_pk2sh_script() -> Script {
    pubkey_to_pubkeyhash_script(get_pubkey())
}

/// pubkey and script
fn get_pk2sh_script_in(pubkey: Vec<u8>) -> Script {
    pubkey_to_pubkeyhash_script(pubkey)
}

/*-------------- sign btc transcation --------------*/

//pub fn sign_btc_transcation_hex(tx_to_sign:&mut Transaction,url:&str,pubkey:Vec<u8>) ->  Result<Bytes,&'static str>  {
//    Ok(tx_to_hex(sign_btc_transcation(tx_to_sign,pubkey,url).unwrap_or_else({
//        return Err("abort");
//    })))
//}

/// Sign btc hex transcation and return signed tx in hex
pub fn sign_btc_hex_return_hex(hex_tx: Vec<u8>,url:&str,pubkey:Vec<u8>) -> Result<Bytes,&'static str> {
    let hex_tx_str:String = hex_tx.to_hex();
    let mut tx_to_sign = tx_from_hex(&hex_tx_str);
    debug!(target:"keysign", "tx_to_sign {:?}", tx_to_sign);

    let res = sign_btc_transcation(&mut tx_to_sign,pubkey,url);
    if res.is_ok(){
        let tx = res.unwrap();
        Ok(tx_to_hex(tx))
    }else {
        return Err("abort");
    }
}

pub fn test_hex_to_tx(hex_tx: &'static Vec<u8>,_url:&str) -> Bytes {
    let hex_tx_str = core::str::from_utf8(hex_tx).unwrap();
    let tx_to_sign = tx_from_hex(hex_tx_str);
    tx_to_hex(tx_to_sign)
}

pub fn sign_btc_transcation(tx_to_sign:&mut Transaction, pubkey_tss:Vec<u8>,url:&str)
                             -> Result<Transaction,&'static str>{
    debug!(target:"keysign", "tx_to_sign {:?}", tx_to_sign.clone());
    let signer = TransactionInputSigner::from(tx_to_sign.clone());

    let inputs_index = tx_to_sign.inputs().to_vec().len();
    debug!(target:"keysign", "sign_btc_transcation  inputs_index {:?}", inputs_index.clone());
    // data used to sign a input
    let sighashtype = 1;
    let pk2sh = get_pk2sh_script_in(pubkey_tss.clone());

    let pubkey = get_public_key_in(pubkey_tss.clone());

    debug!(target:"keysign", "sign_btc_transcation  inputs_index {:?}", inputs_index.clone());
    // sign all inputs
    for i in 0..inputs_index {
        let res = signer.sign_tx_input(&pubkey,
                                            i, 0,
                                            &pk2sh,
                                            SignatureVersion::Base,
                                            sighashtype,pubkey_tss.clone(),url);
        if res.is_ok(){
            let tx_input = res.unwrap();
            tx_to_sign.inputs[i] = tx_input.clone();
            if true {
                let script_sig = tx_input.script_sig.into();
                let script_pubkey =
                    Builder::build_p2pkh(&pubkey.address_hash());
                let flags = VerificationFlags::default().verify_p2sh(true);
                let checker = TransactionSignatureChecker {
                    signer: TransactionInputSigner::from(tx_to_sign.clone()),
                    input_index: i,
                    input_amount: 0
                };
                let res = verify_script(&script_sig,&script_pubkey,
                              &ScriptWitness::default(),&flags,&checker,SignatureVersion::Base);
                debug!(target:"keysign", "verify_script  result {:?} {:?}", i,res);
            }
            sleep(Duration::from_millis(800));
        }else {
            return Err("abort");
        }

    }

    // return the signed Transaction
    Ok(tx_to_sign.clone())
}

trait SignTxInput {
    fn sign_tx_input( &self,
                       keypair: &Public,
                       input_index: usize,
                       input_amount: u64,
                       script_pubkey: &Script,
                       sigversion: SignatureVersion,
                       sighash: u32,
                       pubkey_tss:Vec<u8>,
                       url:&str) -> Result<TransactionInput,&'static str>;

    fn sign_by_tss(&self, message:Vec<u8>,url:&str,pubkey_tss:Vec<u8>) -> Result<Vec<u8>,&'static str>;
}

impl SignTxInput for TransactionInputSigner{
    fn  sign_tx_input(
        &self,
        pubkey: &Public,
        input_index: usize,
        input_amount: u64,
        script_pubkey: &Script,
        sigversion: SignatureVersion,
        sighash: u32,
        pubkey_tss:Vec<u8>,
        url:&str
    ) -> Result<TransactionInput,&'static str> {
        let hash = self.signature_hash(input_index, input_amount, script_pubkey, sigversion, sighash);

        let message : [u8;32] = hash.into();

        let mut signature_by_tss = vec![0u8];
        let res = self.sign_by_tss(message.to_vec(),url,pubkey_tss);
        if res.is_ok(){
            signature_by_tss = res.unwrap();
        }else {
            return Err("abort");
        }

        let mut signature:Vec<u8> =  signature_by_tss.to_vec();

        signature.push(sighash as u8);
        let script_sig = Builder::default()
            .push_data(&signature)
            .push_data(&pubkey)
            .into_script();

        let unsigned_input = &self.inputs[input_index];
        Ok(TransactionInput {
            previous_output: unsigned_input.previous_output.clone(),
            sequence: unsigned_input.sequence,
            script_sig: script_sig.to_bytes(),
            script_witness: Vec::new(),
        })
    }

    fn sign_by_tss(&self, message:Vec<u8>, url:&str, pubkey_tss:Vec<u8>) -> Result<Vec<u8>,&'static str>{
        let res = sign_vec(url, &message,pubkey_tss);
        if res.is_ok(){
            let (r,s,fe_r,fe_s,recid):(SecretKey,SecretKey,FE,FE,u8) = res.unwrap();
            return Ok(r_s_to_vec(r,s,&fe_r,&fe_s, recid,SignatureType::BTC));
        }else {
            return Err("abort");
        }
    }

}

pub enum SignatureType{
    //FC,
    BTC,
    SECP512V1,
}
pub fn sign_by_tss(message: Vec<u8>, url: &str, pubkey_tss:Vec<u8>) -> Result<Vec<u8>,&'static str>{
    let res = sign_vec(url, &message, pubkey_tss);
    if res.is_ok(){
        let (r,s,fe_r,fe_s,recid):(SecretKey,SecretKey,FE,FE,u8) = res.unwrap();

        return Ok(r_s_to_vec(r,s,&fe_r,&fe_s, recid, SignatureType::SECP512V1));
    }else {
        return Err("abort");
    }
}

pub fn r_s_to_vec(_r:SecretKey,_s:SecretKey, a:&FE, b:&FE, recid:u8, sigtype:SignatureType) -> Vec<u8> {
    convert_signtature(a,b, recid, sigtype)
}

fn convert_signtature(r:&FE, s:&FE, recid:u8, sigtype:SignatureType) -> Vec<u8>{
    let mut compact: Vec<u8> = Vec::new();
    let bytes_r = &r.get_element()[..];
    compact.extend(vec![0u8; 32 - bytes_r.len()]);
    compact.extend(bytes_r.iter());

    let bytes_s = &s.get_element()[..];
    compact.extend(vec![0u8; 32 - bytes_s.len()]);
    compact.extend(bytes_s.iter());

    let secp_sig = secpSignature::parse_slice(compact.as_slice()).unwrap();
    match sigtype {
        SignatureType::BTC => {
            let mut sig = secp_sig.serialize_der().as_ref().to_vec();
            sig
        },
        SignatureType::SECP512V1 => {
            let mut sig = secp_sig.serialize().as_ref().to_vec();
            sig.push(recid);
            return sig
        },
    }

}

use std::mem;
fn conver_scalar(s:[u8;32]) -> [u32;8] {
    unsafe {
        let t = mem::transmute::<[u8; 32], [u32; 8]>(s);
        t
    }
}

pub fn convert(pack_data: &[u8]){
    let ptr :*const u8 = pack_data.as_ptr();
    let ptr :*const u32 = ptr as *const u32;
    let s = unsafe{ *ptr};
    debug!(target:"keysign", "verify_script  result {:?}", s);
    println!("{:?}", s);
}

#[test]
#[ignore]
fn test() {
    let t: Transaction = "0100000001a6b97044d03da79c005b20ea9c0e1a6d9dc12d9f7b91a5911c9030a439eed8f5000000004948304502206e21798a42fae0e854281abd38bacd1aeed3ee3738d9e1446618c4571d1090db022100e2ac980643b0b82c0e88ffdfec6b64e3e6ba35e7ba5fdd7d5d6cc8d25c6b241501ffffffff0100f2052a010000001976a914404371705fa9bd789a2fcd52d2c580b65d35549d88ac00000000".into();
    assert_eq!(t.version, 1);

    let vec: Vec<u8> = tx_to_hex(t).into();
    assert_eq!(vec, b"0100000001a6b97044d03da79c005b20ea9c0e1a6d9dc12d9f7b91a5911c9030a439eed8f5000000004948304502206e21798a42fae0e854281abd38bacd1aeed3ee3738d9e1446618c4571d1090db022100e2ac980643b0b82c0e88ffdfec6b64e3e6ba35e7ba5fdd7d5d6cc8d25c6b241501ffffffff0100f2052a010000001976a914404371705fa9bd789a2fcd52d2c580b65d35549d88ac00000000".to_vec());
}


#[test]
#[ignore]
fn test_for_transfrom() {
    //cargo test --color=always --no-run --package btc-signer --lib test_for_transfrom -- --exact --nocapture
    let t_str: &str = "0100000001a6b97044d03da79c005b20ea9c0e1a6d9dc12d9f7b91a5911c9030a439eed8f5000000004948304502206e21798a42fae0e854281abd38bacd1aeed3ee3738d9e1446618c4571d1090db022100e2ac980643b0b82c0e88ffdfec6b64e3e6ba35e7ba5fdd7d5d6cc8d25c6b241501ffffffff0100f2052a010000001976a914404371705fa9bd789a2fcd52d2c580b65d35549d88ac00000000";
    let t: Transaction = t_str.into();
    let t_trs: Transaction = tx_from_hex(t_str);
    assert_eq!(t.version, 1);
}
