use serde::{Deserialize, Serialize};
use super::utils::bytes_json;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Serialize, Deserialize)]
pub struct Ticket {
    /// VRF proof
    #[serde(rename = "VRFProof")]
    #[serde(with = "bytes_json")]
    pub vrf_proof: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticket_json() {
        let json = r#"{"VRFProof":"k4aywRis+mYWN56o3OQOAxEFxKSp777TR1h8hcTEeWlLwvERi2oXnTE7xzS0uoLICnEhoGs9BL5MGDYpf3dfmvLD+h7iBimSpl6rY7bysDbuKreKXa9GwAPN3fQqJB1O"}"#;
        let ticket = serde_json::from_str::<Ticket>(json);
        println!("{:?}", ticket);
    }
}