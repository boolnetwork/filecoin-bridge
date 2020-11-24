use std::{ iter::repeat, thread, time, time::Duration};

use crypto::{
    aead::{AeadDecryptor, AeadEncryptor},
    aes::KeySize::KeySize256,
    aes_gcm::AesGcm,
};
use curv::{
    arithmetic::traits::Converter,
    elliptic::curves::traits::{ECPoint, ECScalar},
    BigInt, FE, GE,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use lazy_static;
use rand::Rng;
use serde_json;
use std::collections::HashMap;
use rustbreak::{FileDatabase, deser::Ron};

pub type Key = String;

pub use anyhow::Result;
use crate::tsserror::TssError;

//use std::ops::DerefMut;
//
//impl DerefMut for Url {
//    fn deref_mut(&mut self) -> &mut String {
//        &mut self.url
//    }
//}

pub struct UrlConfig{
    url: String,
}

impl UrlConfig{
    pub fn new(url:String) -> Self{
        UrlConfig{
            url:url,
        }
    }

    pub fn save(mut self,url:String){
        self.url = url;
    }

    pub fn read(self) -> String{
        self.url
    }
}

use std::sync::Mutex;

lazy_static!{
    pub static ref TEST_URL: Mutex<String> = Mutex::new("http://127.0.0.1:8001".to_string());
    pub static ref STORE_MAP: Mutex<String> = Mutex::new("store_map".to_string());
    pub static ref SAVE_STORE: Mutex<Vec<u64>> =  Mutex::new(vec![]);
}

pub fn push(x:u64){
    println!("===filename==in={:?}",x);
    SAVE_STORE.lock().unwrap().push(x);
}

pub fn get() -> String{
    let num = SAVE_STORE.lock().unwrap()[0];
    let filename = format!("{}-{}{}","test",num,".ron");
    filename
}

const SAVE_FILE: &str = "test02.ron";

pub fn save_url(new_url:String){
    let mut a = TEST_URL.lock().unwrap();
    a.clone_from(&new_url);
}

pub fn get_url() -> String {
    TEST_URL.lock().unwrap().clone()
}

pub fn get_store_file() -> String{
    STORE_MAP.lock().unwrap().clone()
}
//========================================================================
pub fn random_tss_store(origin_store:&str) -> String{
    let mut rng = rand::thread_rng();
    let random = rng.gen::<u32>();
    let mut new_str = String::from(origin_store);
    new_str.push_str("_");
    new_str.push_str(&random.to_string());
    new_str
}

pub fn save(pubkey:Vec<u8>, origin_store:&str) -> String{
    let new_store_name = random_tss_store(origin_store);

    let db =
        FileDatabase::<HashMap<Vec<u8>, String>, Ron>::load_from_path_or(get(), HashMap::new()).unwrap();
    db.load().unwrap();
    db.write(|db| {
        db.insert(pubkey,new_store_name.clone().into());
    }).unwrap();
    db.save().unwrap();
    new_store_name
}

pub fn find_store(pubkey:Vec<u8>) -> String{
    let db =
        FileDatabase::<HashMap<Vec<u8>, String>, Ron>::load_from_path_or(get(), HashMap::new()).unwrap();

    db.load().unwrap();
    let a = db.read(|map| {
        let x = map.get(&pubkey).unwrap();
        x.clone()
    }).unwrap();
    a.clone()
}

pub fn init(){
    let db =
        FileDatabase::<HashMap<Vec<u8>, String>, Ron>::load_from_path_or(get(), HashMap::new()).unwrap();
    if db.load().is_ok(){
        return ;
    }else {
        db.save().unwrap();
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AEAD {
    pub ciphertext: Vec<u8>,
    pub tag: Vec<u8>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PartySignup {
    pub number: u16,
    pub uuid: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Index {
    pub key: Key,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub key: Key,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct Params {
    pub parties: String,
    pub threshold: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Message {
    pub key: String,
}

#[allow(dead_code)]
pub fn aes_encrypt(key: &[u8], plaintext: &[u8]) -> AEAD {
    let nonce: Vec<u8> = repeat(3).take(12).collect();
    let aad: [u8; 0] = [];
    let mut gcm = AesGcm::new(KeySize256, key, &nonce[..], &aad);
    let mut out: Vec<u8> = repeat(0).take(plaintext.len()).collect();
    let mut out_tag: Vec<u8> = repeat(0).take(16).collect();
    gcm.encrypt(&plaintext[..], &mut out[..], &mut out_tag[..]);
    AEAD {
        ciphertext: out.to_vec(),
        tag: out_tag.to_vec(),
    }
}

#[allow(dead_code)]
pub fn aes_decrypt(key: &[u8], aead_pack: AEAD) -> Vec<u8> {
    let mut out: Vec<u8> = repeat(0).take(aead_pack.ciphertext.len()).collect();
    let nonce: Vec<u8> = repeat(3).take(12).collect();
    let aad: [u8; 0] = [];
    let mut gcm = AesGcm::new(KeySize256, key, &nonce[..], &aad);
    gcm.decrypt(&aead_pack.ciphertext[..], &mut out, &aead_pack.tag[..]);
    out
}


pub fn postb<T>(client: &Client, path: &str, body: T) -> Option<String>
where
    T: serde::ser::Serialize,
{
    let url = get_url();
    println!("postb -> {:?}",url);
    let retries = 6;
    let retry_delay = time::Duration::from_millis(500);
    for _i in 1..retries {
        let res = client
            .post(&format!("{}/{}", url, path))
            .json(&body)
            .send();

        if let Ok(mut res) = res {
            let res_text = res.text().unwrap();
            println!("res.text -> {:?}",res_text);
            //res.text -> "{\"Ok\":{\"number\":3,\"uuid\":\"bb7cbd36-f4e0-4111-8c11-26a69c979c21\"}}"
            return Some(res_text);
        }
        thread::sleep(retry_delay);
    }
    None
}

//    let res_body = postb(&client, "set", entry).unwrap();
//    serde_json::from_str(&res_body).unwrap()
pub fn broadcast(
    client: &Client,
    party_num: u16,
    round: &str,
    data: String,
    sender_uuid: String,
) -> Result<()> {
    let key = format!("{}-{}-{}", party_num, round, sender_uuid);
    let entry = Entry {
        key: key.clone(),
        value: data,
    };

    if let Some(res_body) = postb(&client, "set", entry){
          let serde_res: serde_json::error::Result<core::result::Result<(), ()>> = serde_json::from_str(&res_body);
          match serde_res {
              Ok(res) => {
                  match res{
                      Ok(_) => return Ok(()),
                      _ => return Err(TssError::CommonError("broadcast postdata error".into()).into())
                  }
              },
              _ => return Err(TssError::CommonError("broadcast serde error".into()).into()),
          }
    }
    return Err(TssError::CommonError("broadcast postb error".into()).into());
}

pub fn sendp2p(
    client: &Client,
    party_from: u16,
    party_to: u16,
    round: &str,
    data: String,
    sender_uuid: String,
) -> Result<()> {
    let key = format!("{}-{}-{}-{}", party_from, party_to, round, sender_uuid);

    let entry = Entry {
        key: key.clone(),
        value: data,
    };

    if let Some(res_body) = postb(&client, "set", entry){
        let serde_res: serde_json::error::Result<core::result::Result<(), ()>> = serde_json::from_str(&res_body);
        match serde_res {
            Ok(res) => {
                match res{
                    Ok(_) => return Ok(()),
                    _ => return Err(TssError::CommonError("sendp2p postdata error".into()).into())
                }
            },
            _ => return Err(TssError::CommonError("sendp2p serde error".into()).into()),
        }
    }
    return Err(TssError::CommonError("sendp2p postb error".into()).into());
}

//let res_body = postb(client, "get", index.clone()).unwrap();
//let answer: Result<Entry, ()> = serde_json::from_str(&res_body).unwrap();
pub fn poll_for_broadcasts(
    client: &Client,
    party_num: u16,
    n: u16,
    delay: Duration,
    round: &str,
    sender_uuid: String,
) -> Result<Vec<String>> {
    let mut ans_vec = Vec::new();
    for i in 1..=n {
        let mut retry = 0u64;
        let mut error_flag = false;
        if i != party_num {
            let key = format!("{}-{}-{}", i, round, sender_uuid);
            let index = Index { key };
            loop {
                // add delay to allow the server to process request:
                thread::sleep(delay);
                if let Some(res_body)  = postb(client, "get", index.clone()){
                    let answer: core::result::Result<Entry, ()> = serde_json::from_str(&res_body).unwrap();
                    if let Ok(answer) = answer {
                        ans_vec.push(answer.value);
                        println!("[{:?}] party {:?} => party {:?}", round, i, party_num);
                        break;
                    }
                }
                if retry >= 6{
                    error_flag = true;
                    break;
                }
                retry = retry + 1;
            }
        }
        if error_flag == true{
            return Err(TssError::CommonError("poll_for_broadcasts error".into()).into());
        }
    }
    Ok(ans_vec)
}

pub fn poll_for_p2p(
    client: &Client,
    party_num: u16,
    n: u16,
    delay: Duration,
    round: &str,
    sender_uuid: String,
) -> Vec<String> {
    let mut ans_vec = Vec::new();
    for i in 1..=n {
        if i != party_num {
            let key = format!("{}-{}-{}-{}", i, party_num, round, sender_uuid);
            let index = Index { key };
            loop {
                // add delay to allow the server to process request:
                if let Some(res_body)  = postb(client, "get", index.clone()){
                    let answer: core::result::Result<Entry, ()> = serde_json::from_str(&res_body).unwrap();
                    if let Ok(answer) = answer {
                        ans_vec.push(answer.value);
                        println!("[{:?}] party {:?} => party {:?}", round, i, party_num);
                        break;
                    }
                }
            }
        }
    }
    ans_vec
}

#[allow(dead_code)]
pub fn check_sig(r: &FE, s: &FE, msg: &BigInt, pk: &GE) {
    use secp256k1::{verify, Message, PublicKey, PublicKeyFormat, Signature};
    let raw_msg = BigInt::to_vec(&msg);
    println!("check_sig 1 1 raw_msg {:?}",raw_msg.len());
    let mut msg: Vec<u8> = Vec::new(); // padding
    msg.extend(vec![0u8; 32 - raw_msg.len()]);
    msg.extend(raw_msg.iter());
    let msg = Message::parse_slice(msg.as_slice()).unwrap();
    let mut raw_pk = pk.pk_to_key_slice();
    if raw_pk.len() == 64 {
        raw_pk.insert(0, 4u8);
    }
    let pk = PublicKey::parse_slice(&raw_pk, Some(PublicKeyFormat::Full)).unwrap();

    let mut compact: Vec<u8> = Vec::new();
    let bytes_r = &r.get_element()[..];
    compact.extend(vec![0u8; 32 - bytes_r.len()]);
    compact.extend(bytes_r.iter());

    let bytes_s = &s.get_element()[..];
    compact.extend(vec![0u8; 32 - bytes_s.len()]);
    compact.extend(bytes_s.iter());

    let secp_sig = Signature::parse_slice(compact.as_slice()).unwrap();

    let is_correct = verify(&msg, &secp_sig, &pk);
    assert!(is_correct);
}
