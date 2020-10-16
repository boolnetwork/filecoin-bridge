pub mod transports;
pub mod error;
pub mod api;
pub mod types;
mod helper;
pub use num_traits::cast::ToPrimitive;
pub use transports::Http;

#[cfg(test)]
mod tests {
    use super::*;
    use super::api::ChainApi;
    use tokio::runtime::Runtime;
    use crate::types::tipset::TipSet;
    use cid::Cid;
    use std::convert::TryFrom;
    use num_bigint::BigInt;
    use num_traits::cast::ToPrimitive;

    #[test]
    fn test() {
        let mut rt = Runtime::new().unwrap();
        let http = Http::new("http://47.52.21.141:1234/rpc/v0");
        let ret:TipSet = rt.block_on(http.chain_head()).unwrap();

        println!("height {:?}",ret.height);
        let cids = ret.cids[0].clone();
        let ret = rt.block_on(http.chain_get_block_messages(&cids)).unwrap();

        let m_cid = Cid::try_from("bafy2bzacedxqegiy4be7m5pxif6hiam67evc37p4ci6tn33nkma4kvivkunhw").unwrap();
        let ret = rt.block_on(http.chain_get_message(&m_cid)).unwrap();
        let val:BigInt = ret.value;
        println!("result: {:?}", val);
        let val_u128 = val.to_u128().unwrap();
        println!("result: {:?}", val_u128);
    }
}