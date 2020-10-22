#![feature(proc_macro_hygiene, decl_macro)]
#[warn(unused_assignments)]
use std::sync::RwLock;

use rocket::{post, routes, State, Config, config::Environment};
use rocket_contrib::json::Json;
use uuid::Uuid;

mod common;
use common::{Entry, Index, Key, PartySignup, Message};

#[macro_use]
extern crate lazy_static;

mod gg18_sign_client;
pub use gg18_sign_client::{sign, sign_vec};
mod gg18_keygen_client;
pub use gg18_keygen_client::key_gen;

use lru::*;

#[post("/get", format = "json", data = "<request>")]
fn get(
    db_mtx: State<RwLock<LruCache<Key,String>>>,
    request: Json<Index>,
) -> Json<Result<Entry, ()>> {
    let index: Index = request.0;
    let mut hm = db_mtx.write().unwrap();
    match hm.get(&index.key) {
        Some(v) => {
            let entry = Entry {
                key: index.key,
                value: v.clone().to_string(),
            };
            Json(Ok(entry))
        }
        None => Json(Err(())),
    }
}

#[post("/set", format = "json", data = "<request>")]
fn set(db_mtx: State<RwLock<LruCache<Key,String>>>, request: Json<Entry>) -> Json<Result<(), ()>> {
    let entry: Entry = request.0;
    let mut hm = db_mtx.write().unwrap();
    println!("entry.key: {:?}", entry.key.clone());
    hm.put(entry.key.clone(), entry.value.clone());
    Json(Ok(()))
}

#[post("/signupkeygen", format = "json")]
fn signup_keygen(db_mtx: State<RwLock<LruCache<Key,String>>>) -> Json<Result<PartySignup, ()>> {
    let parties = 3;

    let key = "signup-keygen".to_string();

    let party_signup = {
        let mut hm = db_mtx.write().unwrap();
        let value = hm.get(&key).unwrap();
        let client_signup: PartySignup = serde_json::from_str(&value).unwrap();
        if client_signup.number < parties {
            PartySignup {
                number: client_signup.number + 1,
                uuid: client_signup.uuid,
            }
        } else {
            PartySignup {
                number: 1,
                uuid: Uuid::new_v4().to_string(),
            }
        }
    };

    let mut hm = db_mtx.write().unwrap();
    println!("signup - > key {:?}" ,key);
    hm.put(key, serde_json::to_string(&party_signup).unwrap());
    Json(Ok(party_signup))
}

#[post("/message", format = "json", data = "<request>")]
fn message(db_mtx: State<RwLock<LruCache<String, u64>>>, request: Json<Message>) -> Json<Result<(), ()>> {
    let entry: Message = request.0;
    let mut value = 0;
    let threshold = 1;

    let mut h = db_mtx.write().unwrap();

    value = match h.get(&entry.key.clone()) {
        Some(v) => *v,
        None => 0u64,
    };

    if value < threshold + 1{
        value = value + 1;
    } else {
        return Json(Err(()));
    }

    println!("entry.key messages is : {:?}", entry.key.clone());
    h.put(entry.key.clone(), value.clone());
    Json(Ok(()))
}

#[post("/signupsign", format = "json", data = "<request>")]
fn signup_sign(db_mtx: State<RwLock<LruCache<Key,String>>>, request: Json<Message>) -> Json<Result<PartySignup, ()>> {
    let threshold = 1 + 1;

    let mut key = "signup-sign".to_string();
    let entry: String = request.0.key;
    key.push_str(&entry);

    let party_signup = {
        let mut hm = db_mtx.write().unwrap();

        let value = match hm.get(&key){
            Some(x) => x.clone(),
            None => {
                let party_signup_sign = PartySignup {
                    number: 0,
                    uuid: Uuid::new_v4().to_string(),
                };
                serde_json::to_string(&party_signup_sign).unwrap()
            },
        };
        let client_signup: PartySignup = serde_json::from_str(&value).unwrap();
        if client_signup.number < threshold  {
            PartySignup {
                number: client_signup.number + 1,
                uuid: client_signup.uuid,
            }
        } else {
            return Json(Err(()));
        }
    };

    let mut hm = db_mtx.write().unwrap();
    hm.put(key, serde_json::to_string(&party_signup).unwrap());
    Json(Ok(party_signup))
}

pub fn start_sm_manager() {
    let db:LruCache<Key,u64> = LruCache::new(2500);
    let db_mtx = RwLock::new(db);

    let db2:LruCache<Key,String> = LruCache::new(4500);
    let db2_mtx = RwLock::new(db2);

    {
        let mut hm = db2_mtx.write().unwrap();
        hm.put(
            "signup-keygen".to_string(),
            serde_json::to_string(& PartySignup {
                number: 0,
                uuid: Uuid::new_v4().to_string(),
            }).unwrap(),
        );
    }

    let mut config = Config::new(Environment::Production);
    config.set_port(8001);

    rocket::custom(config)
        .mount("/", routes![get, set, signup_keygen, signup_sign, message])
        .manage(db_mtx)
        .manage(db2_mtx)
        .launch();
}