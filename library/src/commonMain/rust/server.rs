use std::sync::{Arc, Mutex};

use opaque_ke::{
    CredentialFinalization, CredentialRequest, RegistrationRequest, RegistrationUpload,
    ServerLogin, ServerLoginParameters, ServerRegistration, ServerSetup,
};
use rand::rngs::OsRng;

use crate::error::ExportedOpaqueError;
use crate::suite::OpaqueSuite;

#[derive(uniffi::Record)]
pub struct ServerLoginStartOutput {
    pub credential_response: Vec<u8>,
    pub state: Arc<ServerLoginState>,
}

#[derive(uniffi::Record)]
pub struct ServerLoginFinishOutput {
    pub session_key: Vec<u8>,
}

#[derive(uniffi::Object)]
pub struct OpaqueServerSetup {
    inner: ServerSetup<OpaqueSuite>,
}

#[uniffi::export]
impl OpaqueServerSetup {
    #[uniffi::constructor]
    pub fn generate() -> Self {
        let mut rng = OsRng;
        Self {
            inner: ServerSetup::<OpaqueSuite>::new(&mut rng),
        }
    }

    #[uniffi::constructor]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, ExportedOpaqueError> {
        let inner = ServerSetup::<OpaqueSuite>::deserialize(&bytes)
            .map_err(|_| ExportedOpaqueError::Serialization)?;
        Ok(Self { inner })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.serialize().to_vec()
    }

    pub fn registration_start(
        &self,
        registration_request: Vec<u8>,
        credential_identifier: Vec<u8>,
    ) -> Result<Vec<u8>, ExportedOpaqueError> {
        let request = RegistrationRequest::<OpaqueSuite>::deserialize(&registration_request)
            .map_err(|_| ExportedOpaqueError::Serialization)?;

        let started = ServerRegistration::<OpaqueSuite>::start(
            &self.inner,
            request,
            &credential_identifier,
        )
        .map_err(ExportedOpaqueError::from)?;

        Ok(started.message.serialize().to_vec())
    }

    pub fn registration_finish(
        &self,
        registration_upload: Vec<u8>,
    ) -> Result<Vec<u8>, ExportedOpaqueError> {
        let upload = RegistrationUpload::<OpaqueSuite>::deserialize(&registration_upload)
            .map_err(|_| ExportedOpaqueError::Serialization)?;

        let finalized = ServerRegistration::<OpaqueSuite>::finish(upload);
        Ok(finalized.serialize().to_vec())
    }

    pub fn login_start(
        &self,
        password_file: Option<Vec<u8>>,
        credential_request: Vec<u8>,
        credential_identifier: Vec<u8>,
    ) -> Result<ServerLoginStartOutput, ExportedOpaqueError> {
        let request = CredentialRequest::<OpaqueSuite>::deserialize(&credential_request)
            .map_err(|_| ExportedOpaqueError::Serialization)?;

        let parsed_file = match password_file {
            Some(bytes) => Some(
                ServerRegistration::<OpaqueSuite>::deserialize(&bytes)
                    .map_err(|_| ExportedOpaqueError::Serialization)?,
            ),
            None => None,
        };

        let mut rng = OsRng;
        let started = ServerLogin::<OpaqueSuite>::start(
            &mut rng,
            &self.inner,
            parsed_file,
            request,
            &credential_identifier,
            ServerLoginParameters::default(),
        )
        .map_err(ExportedOpaqueError::from)?;

        Ok(ServerLoginStartOutput {
            credential_response: started.message.serialize().to_vec(),
            state: Arc::new(ServerLoginState {
                inner: Mutex::new(Some(started.state)),
            }),
        })
    }
}

#[derive(uniffi::Object)]
pub struct ServerLoginState {
    inner: Mutex<Option<ServerLogin<OpaqueSuite>>>,
}

#[uniffi::export]
impl ServerLoginState {
    pub fn finish(
        &self,
        credential_finalization: Vec<u8>,
    ) -> Result<ServerLoginFinishOutput, ExportedOpaqueError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| ExportedOpaqueError::LockPoisoned)?;
        let state = guard
            .take()
            .ok_or(ExportedOpaqueError::InvalidState)?;

        let finalization =
            CredentialFinalization::<OpaqueSuite>::deserialize(&credential_finalization)
                .map_err(|_| ExportedOpaqueError::Serialization)?;

        let result = state
            .finish(finalization, ServerLoginParameters::default())
            .map_err(ExportedOpaqueError::from)?;

        Ok(ServerLoginFinishOutput {
            session_key: result.session_key.to_vec(),
        })
    }
}