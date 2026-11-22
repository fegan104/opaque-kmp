use argon2::Argon2;
use opaque_ke::ciphersuite::CipherSuite;
use opaque_ke::key_exchange::tripledh::TripleDh;
use opaque_ke::Ristretto255;
use sha2::Sha512;

/// Locked RFC 9807 cipher suite shared across JVM, Android, and iOS.
pub struct OpaqueSuite;

impl CipherSuite for OpaqueSuite {
    type OprfCs = Ristretto255;
    type KeyExchange = TripleDh<Ristretto255, Sha512>;
    type Ksf = Argon2<'static>;
}