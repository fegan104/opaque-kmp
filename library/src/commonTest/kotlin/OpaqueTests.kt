package com.frankegan.opaque_kmp

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertFailsWith

class OpaqueTests {

    @Test
    fun registrationAndLoginDeriveMatchingSessionKeys() {
        val password = "correct-horse-battery-staple".encodeToByteArray()
        val credentialId = "alice@example.com".encodeToByteArray()

        // 1. Server setup
        val serverSetup = OpaqueServerSetup.generate()

        // 2. Registration
        val regStart = clientRegistrationStart(password)
        val regResponse = serverSetup.registrationStart(
            registrationRequest = regStart.request,
            credentialIdentifier = credentialId,
        )
        val regFinish = regStart.state.finish(
            password = password,
            serverResponse = regResponse,
            clientIdentity = null,
            serverIdentity = null,
        )
        val passwordFile = serverSetup.registrationFinish(
            registrationUpload = regFinish.upload,
        )

        // 3. Login
        val loginStart = clientLoginStart(password)
        val serverLoginStart = serverSetup.loginStart(
            passwordFile = passwordFile,
            credentialRequest = loginStart.credentialRequest,
            credentialIdentifier = credentialId,
        )
        val clientLoginFinish = loginStart.state.finish(
            password = password,
            credentialResponse = serverLoginStart.credentialResponse,
            clientIdentity = null,
            serverIdentity = null,
        )
        val serverLoginFinish = serverLoginStart.state.finish(
            credentialFinalization = clientLoginFinish.credentialFinalization,
        )

        // Both sides should agree on the session key
        assertContentEquals(clientLoginFinish.sessionKey, serverLoginFinish.sessionKey)

        // Reusing a consumed state object should fail
        assertFailsWith<ExportedOpaqueException.InvalidState> {
            serverLoginStart.state.finish(
                credentialFinalization = clientLoginFinish.credentialFinalization,
            )
        }
    }
}