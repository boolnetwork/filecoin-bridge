#![allow(non_snake_case)]
/// to run:
/// 1: go to rocket_server -> cargo run
/// 2: cargo run from PARTIES number of terminals
use curv::{
    arithmetic::traits::Converter,
    cryptographic_primitives::{
        proofs::sigma_dlog::DLogProof, secret_sharing::feldman_vss::VerifiableSS,
    },
    elliptic::curves::traits::{ECPoint, ECScalar},
    BigInt, FE, GE,
};
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2018::party_i::{
    KeyGenBroadcastMessage1, KeyGenDecommitMessage1, Keys, Parameters,
};
use paillier::EncryptionKey;
use reqwest::Client;
use std::{fs, time};

use crate::common::{aes_decrypt, aes_encrypt, broadcast, poll_for_broadcasts
                    , poll_for_p2p, postb, sendp2p, PartySignup, AEAD};

use crate::common::{save_url, save, init};

use secp256k1::{ PublicKey, PublicKeyFormat };
pub use anyhow::Result;
use crate::tsserror::TssError;

//pub fn key_gen<'a>(url:&'a str, store:&'a str) -> Result<([u8;65],Vec<Vec<u8>>),&'a str>{
pub fn key_gen<'a>(url:&'a str, store:&'a str) -> Result<([u8;65],Vec<Vec<u8>>)>{

    println!("============ key_gen Event!key_gen Event!key_gen Event! ============");
    println!("============ key_gen Event url = {:?} ============",url);
    save_url(String::from(url));

    let PARTIES: u16 = 3;
    let THRESHOLD: u16 = 1;


    let client = Client::new();

    // delay:
    let delay = time::Duration::from_millis(450);
    let params = Parameters {
        threshold: THRESHOLD,
        share_count: PARTIES,
    };

    //signup:
    let (party_num_int, uuid) = match signup(&client) {
        Ok(PartySignup { number, uuid }) => (number, uuid),
        Err(_) => {
            return Err(TssError::KeyGenError(0).into());
        },
    };

    println!("number: {:?}, uuid: {:?}", party_num_int, uuid);

    let party_keys = Keys::create(party_num_int as usize);
    let (bc_i, decom_i) = party_keys.phase1_broadcast_phase3_proof_of_correct_key();

    // send commitment to ephemeral public keys, get round 1 commitments of other parties
    broadcast(
        &client,
        party_num_int,
        "round1",
        serde_json::to_string(&bc_i).unwrap(),
        uuid.clone()
    )?;

    let round1_ans_vec = poll_for_broadcasts(
        &client,
        party_num_int,
        PARTIES,
        delay,
        "round1",
        uuid.clone(),
    )?;

    let mut bc1_vec = round1_ans_vec
        .iter()
        .map(|m| serde_json::from_str::<KeyGenBroadcastMessage1>(m).unwrap())
        .collect::<Vec<_>>();

    bc1_vec.insert(party_num_int as usize - 1, bc_i);

    // send ephemeral public keys and check commitments correctness
    broadcast(
        &client,
        party_num_int,
        "round2",
        serde_json::to_string(&decom_i).unwrap(),
        uuid.clone()
    )?;


    let round2_ans_vec = poll_for_broadcasts(
        &client,
        party_num_int,
        PARTIES,
        delay,
        "round2",
        uuid.clone(),
    )?;

    let mut j = 0;
    let mut point_vec: Vec<GE> = Vec::new();
    let mut decom_vec: Vec<KeyGenDecommitMessage1> = Vec::new();
    let mut enc_keys: Vec<BigInt> = Vec::new();
    for i in 1..=PARTIES {
        if i == party_num_int {
            point_vec.push(decom_i.y_i);
            decom_vec.push(decom_i.clone());
        } else {
            let decom_j: KeyGenDecommitMessage1 = serde_json::from_str(&round2_ans_vec[j]).unwrap();
            point_vec.push(decom_j.y_i);
            decom_vec.push(decom_j.clone());
            enc_keys.push((decom_j.y_i.clone() * party_keys.u_i).x_coor().unwrap());
            j = j + 1;
        }
    }

    let (head, tail) = point_vec.split_at(1);
    let y_sum = tail.iter().fold(head[0], |acc, x| acc + x);

    let (vss_scheme, secret_shares, _index) = party_keys
        .phase1_verify_com_phase3_verify_correct_key_phase2_distribute(
            &params, &decom_vec, &bc1_vec,
        )
        .expect("invalid key");

    //////////////////////////////////////////////////////////////////////////////

    let mut j = 0;
    for (k, i) in (1..=PARTIES).enumerate() {
        if i != party_num_int {
            // prepare encrypted ss for party i:
            let key_i = BigInt::to_vec(&enc_keys[j]);
            let plaintext = BigInt::to_vec(&secret_shares[k].to_big_int());
            let aead_pack_i = aes_encrypt(&key_i, &plaintext);
            sendp2p(
                &client,
                party_num_int,
                i,
                "round3",
                serde_json::to_string(&aead_pack_i).unwrap(),
                uuid.clone()
            )?;
            j += 1;
        }
    }

    let round3_ans_vec = poll_for_p2p(
        &client,
        party_num_int,
        PARTIES,
        delay,
        "round3",
        uuid.clone(),
    );

    let mut j = 0;
    let mut party_shares: Vec<FE> = Vec::new();
    for i in 1..=PARTIES {
        if i == party_num_int {
            party_shares.push(secret_shares[(i - 1) as usize]);
        } else {
            let aead_pack: AEAD = serde_json::from_str(&round3_ans_vec[j]).unwrap();
            let key_i = BigInt::to_vec(&enc_keys[j]);
            let out = aes_decrypt(&key_i, aead_pack);
            let out_bn = BigInt::from(&out[..]);
            let out_fe = ECScalar::from(&out_bn);
            party_shares.push(out_fe);

            j += 1;
        }
    }

    // round 4: send vss commitments
    broadcast(
        &client,
        party_num_int,
        "round4",
        serde_json::to_string(&vss_scheme).unwrap(),
        uuid.clone()
    )?;

    let round4_ans_vec = poll_for_broadcasts(
        &client,
        party_num_int,
        PARTIES,
        delay,
        "round4",
        uuid.clone(),
    )?;

    let mut j = 0;
    let mut vss_scheme_vec: Vec<VerifiableSS> = Vec::new();
    for i in 1..=PARTIES {
        if i == party_num_int {
            vss_scheme_vec.push(vss_scheme.clone());
        } else {
            let vss_scheme_j: VerifiableSS = serde_json::from_str(&round4_ans_vec[j]).unwrap();
            vss_scheme_vec.push(vss_scheme_j);
            j += 1;
        }
    }

    let (shared_keys, dlog_proof) = party_keys
        .phase2_verify_vss_construct_keypair_phase3_pok_dlog(
            &params,
            &point_vec,
            &party_shares,
            &vss_scheme_vec,
            party_num_int as usize,
        )
        .expect("invalid vss");

    // round 5: send dlog proof
    broadcast(
        &client,
        party_num_int,
        "round5",
        serde_json::to_string(&dlog_proof).unwrap(),
        uuid.clone()
    )?;

    let round5_ans_vec = poll_for_broadcasts(
        &client,
        party_num_int,
        PARTIES,
        delay,
        "round5",
        uuid.clone(),
    )?;

    let mut j = 0;
    let mut dlog_proof_vec: Vec<DLogProof> = Vec::new();
    for i in 1..=PARTIES {
        if i == party_num_int {
            dlog_proof_vec.push(dlog_proof.clone());
        } else {
            let dlog_proof_j: DLogProof = serde_json::from_str(&round5_ans_vec[j]).unwrap();
            dlog_proof_vec.push(dlog_proof_j);
            j += 1;
        }
    }
    Keys::verify_dlog_proofs(&params, &dlog_proof_vec, &point_vec).expect("bad dlog proof");

    //save key to file:
    let paillier_key_vec = (0..PARTIES)
        .map(|i| bc1_vec[i as usize].e.clone())
        .collect::<Vec<EncryptionKey>>();

    let keygen_json = serde_json::to_string(&(
        party_keys,
        shared_keys,
        party_num_int,
        vss_scheme_vec,
        paillier_key_vec,
        y_sum,
    ))
    .unwrap();


    println!("=================start to convert===================");
    let publickey = y_sum.clone();
    let pkslice = publickey.pk_to_key_slice();
//    let PK = publickey.get_element();
//    let PK_u8 = unsafe{ *PK.as_ptr().clone()};
//    let PK_u8_slice : &[u8]  = PK_u8.index(RangeFull);
    let _really_pubkey: PublicKey =
        PublicKey::parse_slice(&pkslice,Some(PublicKeyFormat::Full)).unwrap();
    let mut pk_return:[u8;65] =  [0;65];
    pk_return.copy_from_slice(&pkslice);
    println!("==== KEYGEN ======== success =============");
    //convert point_vec
    let mut point_u8_vec:Vec<Vec<u8>> = Vec::new();
    point_vec.iter().for_each(|&x|{
        let each_pk = x.pk_to_key_slice();
        point_u8_vec.push(each_pk);
    });

    init();
    let new_store = save(pk_return.to_vec(),store);

    fs::write(new_store, keygen_json).expect("Unable to save !");
    Ok((pk_return,point_u8_vec))
}

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
struct Temp {
    pub Ok:PartySignup,
}

pub fn signup(client: &Client) -> Result<PartySignup, &'static str> {
    let key = "signup-keygen".to_string();

    let ret = postb(&client, "signupkeygen", key);
    if ret.is_some() {
        let x = ret.unwrap();
        println!("signup.x -> {:?}",x);

        let result : Temp = serde_json::from_str(&x).unwrap();
        println!("signup.result -> {:?}",result);
        return Ok(result.Ok);
    }else {
        return Err("error")
    }

//    match postb(&client, "signupkeygen", key) {
//        Some(x) => {
//            println!("signup.x -> {:?}",x);
//            let result: PartySignup = serde_json::from_str(&x).unwrap();
//            Ok(result.clone())},
//        None => return Err("error"),
//    }

//    let res_body = postb(&client, "signupkeygen", key).unwrap_or(String::from("error"));
//
//    match res_body {
//        "error".to_String() => return Err("error"),
//        _ => return serde_json::from_str(&res_body).unwrap(),
//    }
//    if res_body.clone() == "error" { return  Err("error");}
//    serde_json::from_str(&res_body).unwrap()
}

#[test]
fn test(){
    //cargo test --color=always --no-run --package node-tss --lib gg18_keygen_client::test -- --exact --nocapture
    println!("==========Test========");
    save_url(String::from("http://sfadfas"));
    let a = get_url();
    println!("a {:?}",a);
}