use super::JsonApi;
use crate::error::Result;
use crate::helper;
use crate::types::{Address, SignatureType, BigInt, BigIntWrapper, Signature, BytesRef, UnsignedMessage,
                   SignedMessage, KeyInfo};

#[async_trait::async_trait]
pub trait WalletApi: JsonApi {
    async fn wallet_new(&self, sig_type: SignatureType) -> Result<Address> {
        self.request("WalletNew", vec![helper::serialize(&sig_type)])
            .await
    }

    async fn wallet_has(&self, addr: &Address) -> Result<bool> {
        self.request("WalletHas", vec![helper::serialize(addr)])
            .await
    }

    async fn wallet_list(&self) -> Result<Vec<Address>> {
        self.request("WalletList", vec![]).await
    }

    async fn wallet_balance(&self, addr: &Address) -> Result<BigInt> {
        let bigint: BigIntWrapper = self
            .request("WalletBalance", vec![helper::serialize(addr)])
            .await?;
        Ok(bigint.into_inner())
    }

    async fn wallet_sign(&self, addr: &Address, msg: &[u8]) -> Result<Signature> {
        self.request(
            "WalletSign",
            vec![
                helper::serialize(addr),
                helper::serialize(&BytesRef::from(msg)),
            ],
        )
            .await
    }

    async fn wallet_sign_message(
        &self,
        addr: &Address,
        msg: &UnsignedMessage,
    ) -> Result<SignedMessage> {
        self.request(
            "WalletSignMessage",
            vec![helper::serialize(addr), helper::serialize(msg)],
        )
            .await
    }

    async fn wallet_verify(
        &self,
        addr: &Address,
        msg: &[u8],
        signature: &Signature,
    ) -> Result<bool> {
        self.request(
            "WalletVerify",
            vec![
                helper::serialize(addr),
                helper::serialize(&BytesRef::from(msg)),
                helper::serialize(signature),
            ],
        )
            .await
    }

    async fn wallet_default_address(&self) -> Result<Address> {
        self.request("WalletDefaultAddress", vec![]).await
    }

    async fn wallet_set_default(&self, addr: &Address) -> Result<()> {
        self.request("WalletSetDefault", vec![helper::serialize(addr)])
            .await
    }

    async fn wallet_export(&self, addr: &Address) -> Result<KeyInfo> {
        self.request("WalletExport", vec![helper::serialize(addr)])
            .await
    }

    async fn wallet_import(&self, info: &KeyInfo) -> Result<Address> {
        self.request("WalletImport", vec![helper::serialize(info)])
            .await
    }
}