pub use anyhow::Result;
use std::any::Any;

#[derive(Debug, thiserror::Error)]
pub enum TssError{
    #[error("Tss Key could not be created.")]
    KeyGenError(u64),
    #[error("Message could not be Signed by Tss.")]
    KeySignError(u64),
    #[error("Conmmon Error")]
    CommonError(String),
    #[error("SignUp Error")]
    SignUp(),
}

impl From<Box<dyn Any + Send>> for TssError {
    fn from(inner: Box<dyn Any + Send>) -> TssError {
        TssError::CommonError(format!("{:?}", dbg!(inner)))
    }
}

pub enum RetryAbort{
    Retry,
    Abort,
}