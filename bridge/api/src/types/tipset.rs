use serde::{ser, de, Deserialize, Serialize};
use super::utils::vec_cid_json;
use cid::Cid;
use super::header::{BlockHeader, ChainEpoch};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TipSet {
    #[serde(with = "vec_cid_json")]
    pub cids: Vec<Cid>,
    pub blocks: Vec<BlockHeader>,
    pub height: ChainEpoch,
}

#[derive(Clone, Debug)]
pub struct TipSetKey {
    cids: Vec<Cid>,
}

// Implement JSON serialization for TipsetKey.
impl ser::Serialize for TipSetKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
    {
        vec_cid_json::serialize(&self.cids, serializer)
    }
}

// Implement JSON deserialization for TipsetKey.
impl<'de> de::Deserialize<'de> for TipSetKey {
    fn deserialize<D>(deserializer: D) -> Result<TipSetKey, D::Error>
        where
            D: de::Deserializer<'de>,
    {
        let cids = vec_cid_json::deserialize(deserializer)?;
        Ok(TipSetKey { cids })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct VC {
        #[serde(with = "vec_cid_json")]
        cids: Vec<Cid>
    }

    #[test]
    fn cids_json() {
        let json = r#"{"Cids":[{"/":"bafy2bzacec43cadndvrgpiq3lia65pbyj2t32jltlqp2oszlvknwn5wx3vyii"}]}"#;
        let vc = serde_json::from_str::<VC>(json);
        println!("{:?}", vc);
    }

    #[test]
    fn tip_set_json() {
        let json = r#"{
        "Cids":[{"/":"bafy2bzacec43cadndvrgpiq3lia65pbyj2t32jltlqp2oszlvknwn5wx3vyii"}],
        "Blocks":[{"Miner":"t01000","Ticket":{"VRFProof":"k4aywRis+mYWN56o3OQOAxEFxKSp777TR1h8hcTEeWlLwvERi2oXnTE7xzS0uoLICnEhoGs9BL5MGDYpf3dfmvLD+h7iBimSpl6rY7bysDbuKreKXa9GwAPN3fQqJB1O"},"ElectionProof":{"VRFProof":"g7Ki1qDQtj0Q1o2bRpHZqD++UqFfjPaOJ5WYT2wJjCgGxg/+2L4cozSU/F7IzGIfE1E79C0brMGROGCLMui4qiSZr1D9sJmn+EBwrLjbqpiJEVXqoXoFEkw7/xpFjIat"},"BeaconEntries":null,"WinPoStProof":[{"RegisteredProof":9,"ProofBytes":"scKG734ZjZjlLv1I9z/7R4qmL3M0kpkTKtBa00pGVxA8cd3myhwhocX8BL4pHl8QmMbkPqp5iXh0sbCdJjbJ6/OmAvpATiAYf3R7pTMOdkLvxFofq4NDEtv8t/I4fnOJAcTvG0ozeNA3MM0KjR2X+kfz4Fo4kVflCdhcT9cKlYBO7IiVKYm/RN0zyvJi6pzhmBtryhGzYyNYv3jWVde8qUtIQnD0169SzYVrbZlfF4ydpgGj5PriYRXrCTi9DXmz"}],"Parents":[{"/":"bafy2bzacebwut2il7udv5d3yzscpwbomvj5ocq6lkxh4kcusiy5juesvpun4c"}],"ParentWeight":"629642112","Height":149063,"ParentStateRoot":{"/":"bafy2bzaceae7pqh2wupmp3fqnlbsxx2czjku5rbisl3qdtaa5mehs2hkjak3a"},"ParentMessageReceipts":{"/":"bafy2bzaceaa43et73tgxsoh2xizd4mxhbrcfig4kqp25zfa5scdgkzppllyuu"},"Messages":{"/":"bafy2bzacecgw6dqj4bctnbnyqfujltkwu7xc7ttaaato4i5miroxr4bayhfea"},"BLSAggregate":{"Type":2,"Data":"wAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"},"Timestamp":1592693392,"BlockSig":{"Type":2,"Data":"l+3ZTa9Q1mj8UcVMAetZSuZphQQJUDfaSbXbZf6rNTBhrqE7feLMcTCCMcUOClNnFH+P8HQmOZ8YwH47vU2vw6maLU33bS5Bc6+MvF7gjFx2pRHgq5GM8SPunDA3fKFe"},"ForkSignaling":0}],
        "Height":149063}"#;

        let tip_set = serde_json::from_str::<TipSet>(json);
        println!("{:?}", tip_set);
    }
}