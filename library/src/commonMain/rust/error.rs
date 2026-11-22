use opaque_ke::errors::ProtocolError;
use std::fmt;

#[derive(Debug, uniffi::Error)]
#[uniffi::export(Display)]
pub enum ExportedOpaqueError {
    Serialization,
    Verification,
    InvalidState,
    LockPoisoned,
}

impl fmt::Display for ExportedOpaqueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportedOpaqueError::Serialization => write!(f, "Invalid or malformed protocol message"),
            ExportedOpaqueError::Verification => write!(f, "Cryptographic verification failed"),
            ExportedOpaqueError::InvalidState => {
                write!(f, "Handshake state was already consumed or invalid")
            }
            ExportedOpaqueError::LockPoisoned => write!(f, "Internal synchronization error"),
        }
    }
}

impl From<ProtocolError> for ExportedOpaqueError {
    fn from(err: ProtocolError) -> Self {
        match err {
            ProtocolError::InvalidLoginError | ProtocolError::ReflectedValueError => {
                ExportedOpaqueError::Verification
            }
            _ => ExportedOpaqueError::Serialization,
        }
    }
}