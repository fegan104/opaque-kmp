use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};

use opaque_ke::{
    ClientLogin, ClientLoginFinishParameters, ClientRegistration,
    ClientRegistrationFinishParameters, CredentialResponse, Identifiers, RegistrationResponse,
};
use rand::rngs::OsRng;

use crate::error::ExportedOpaqueError;
use crate::suite::OpaqueSuite;

#[derive(uniffi::Record)]
pub struct ClientRegistrationStartOutput {
    pub request: Vec<u8>,
    pub state: Arc<ClientRegistrationState>,
}

#[derive(uniffi::Record)]
pub struct ClientRegistrationFinishOutput {
    pub upload: Vec<u8>,
    pub export_key: Vec<u8>,
}

#[derive(uniffi::Record)]
pub struct ClientLoginStartOutput {
    pub credential_request: Vec<u8>,
    pub state: Arc<ClientLoginState>,
}

#[derive(uniffi::Record)]
pub struct ClientLoginFinishOutput {
    pub credential_finalization: Vec<u8>,
    pub session_key: Vec<u8>,
    pub export_key: Vec<u8>,
}

struct InnerClientRegState(ManuallyDrop<ClientRegistration<OpaqueSuite>>);

impl Drop for InnerClientRegState {
    fn drop(&mut self) {
        // SAFETY: `self.0` is only extracted once via `ManuallyDrop::take` in `finish()`.
        unsafe { ManuallyDrop::drop(&mut self.0) };
    }
}

#[derive(uniffi::Object)]
pub struct ClientRegistrationState {
    inner: Mutex<Option<InnerClientRegState>>,
}

#[uniffi::export]
impl ClientRegistrationState {
    pub fn finish(
        &self,
        password: Vec<u8>,
        server_response: Vec<u8>,
        client_identity: Option<Vec<u8>>,
        server_identity: Option<Vec<u8>>,
    ) -> Result<ClientRegistrationFinishOutput, ExportedOpaqueError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| ExportedOpaqueError::LockPoisoned)?;
        let mut state = guard.take().ok_or(ExportedOpaqueError::InvalidState)?;

        let response = RegistrationResponse::<OpaqueSuite>::deserialize(&server_response)
            .map_err(|_| ExportedOpaqueError::Serialization)?;

        let params = ClientRegistrationFinishParameters::new(
            Identifiers {
                client: client_identity.as_deref(),
                server: server_identity.as_deref(),
            },
            None,
        );

        let mut rng = OsRng;
        // SAFETY: `state` was removed from the Mutex<Option<...>> above and forgotten immediately
        // after extracting `inner_reg`, preventing any subsequent use or double-drop.
        let inner_reg = unsafe { ManuallyDrop::take(&mut state.0) };
        std::mem::forget(state);

        let result = inner_reg
            .finish(&mut rng, &password, response, params)
            .map_err(ExportedOpaqueError::from)?;

        Ok(ClientRegistrationFinishOutput {
            upload: result.message.serialize().to_vec(),
            export_key: result.export_key.to_vec(),
        })
    }
}

struct InnerClientLoginState(ManuallyDrop<ClientLogin<OpaqueSuite>>);

impl Drop for InnerClientLoginState {
    fn drop(&mut self) {
        // SAFETY: `self.0` is only extracted once via `ManuallyDrop::take` in `finish()`.
        unsafe { ManuallyDrop::drop(&mut self.0) };
    }
}

#[derive(uniffi::Object)]
pub struct ClientLoginState {
    inner: Mutex<Option<InnerClientLoginState>>,
}

#[uniffi::export]
impl ClientLoginState {
    pub fn finish(
        &self,
        password: Vec<u8>,
        credential_response: Vec<u8>,
        client_identity: Option<Vec<u8>>,
        server_identity: Option<Vec<u8>>,
    ) -> Result<ClientLoginFinishOutput, ExportedOpaqueError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| ExportedOpaqueError::LockPoisoned)?;
        let mut state = guard.take().ok_or(ExportedOpaqueError::InvalidState)?;

        let response = CredentialResponse::<OpaqueSuite>::deserialize(&credential_response)
            .map_err(|_| ExportedOpaqueError::Serialization)?;

        let params = ClientLoginFinishParameters::new(
            None,
            Identifiers {
                client: client_identity.as_deref(),
                server: server_identity.as_deref(),
            },
            None,
        );

        let mut rng = OsRng;
        // SAFETY: `state` was removed from the Mutex<Option<...>> above and forgotten immediately
        // after extracting `inner_login`, preventing any subsequent use or double-drop.
        let inner_login = unsafe { ManuallyDrop::take(&mut state.0) };
        std::mem::forget(state);

        let result = inner_login
            .finish(&mut rng, &password, response, params)
            .map_err(ExportedOpaqueError::from)?;

        Ok(ClientLoginFinishOutput {
            credential_finalization: result.message.serialize().to_vec(),
            session_key: result.session_key.to_vec(),
            export_key: result.export_key.to_vec(),
        })
    }
}

#[uniffi::export]
pub fn client_registration_start(
    password: Vec<u8>,
) -> Result<ClientRegistrationStartOutput, ExportedOpaqueError> {
    let mut rng = OsRng;
    let started = ClientRegistration::<OpaqueSuite>::start(&mut rng, &password)
        .map_err(ExportedOpaqueError::from)?;

    Ok(ClientRegistrationStartOutput {
        request: started.message.serialize().to_vec(),
        state: Arc::new(ClientRegistrationState {
            inner: Mutex::new(Some(InnerClientRegState(ManuallyDrop::new(started.state)))),
        }),
    })
}

#[uniffi::export]
pub fn client_login_start(
    password: Vec<u8>,
) -> Result<ClientLoginStartOutput, ExportedOpaqueError> {
    let mut rng = OsRng;
    let started = ClientLogin::<OpaqueSuite>::start(&mut rng, &password)
        .map_err(ExportedOpaqueError::from)?;

    Ok(ClientLoginStartOutput {
        credential_request: started.message.serialize().to_vec(),
        state: Arc::new(ClientLoginState {
            inner: Mutex::new(Some(InnerClientLoginState(ManuallyDrop::new(started.state)))),
        }),
    })
}