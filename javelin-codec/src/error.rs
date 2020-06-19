use {
    thiserror::Error,
    crate::{
        avc::AvcError,
        aac::AacError,
        flv::FlvError,
    }
};


#[derive(Error, Debug)]
pub enum CodecError {
    #[error(transparent)]
    AvcError(#[from] AvcError),

    #[error(transparent)]
    AacError(#[from] AacError),

    #[error(transparent)]
    FlvError(#[from] FlvError),
}
