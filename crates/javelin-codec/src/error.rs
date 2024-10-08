use thiserror::Error;

use crate::aac::AacError;
use crate::avc::AvcError;
use crate::flv::FlvError;
#[cfg(feature = "mpegts")]
use crate::mpegts::TsError;


#[derive(Error, Debug)]
pub enum CodecError {
    #[error(transparent)]
    AvcError(#[from] AvcError),

    #[error(transparent)]
    AacError(#[from] AacError),

    #[error(transparent)]
    FlvError(#[from] FlvError),

    #[cfg(feature = "mpegts")]
    #[error(transparent)]
    TsError(#[from] TsError),
}
