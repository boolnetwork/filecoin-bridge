use sp_std::vec::Vec;
use super::AccountId;

sp_api::decl_runtime_apis! {
    pub trait VendorApi{
	    fn account_nonce(account: &AccountId) -> u64 ;
	    fn is_tss_party(id: &AccountId) -> bool;
	    fn tss_pub_key() -> Vec<u8>;
	    fn tss_pub_key_bool() -> Vec<u8>;
	    fn tss_pub_key_fc() -> Vec<u8>;
	    fn tss_url() -> Vec<u8>;
    }
}