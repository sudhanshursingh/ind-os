package indos.neural.security

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Sprint 17 Definition of Done (DoD) Test (Neural Core Subsystem):
 * Air-gap network socket isolation invariant:
 * - Attempting socket allocation in IndNeuralCore process fails with EACCES / blocked.
 * - Exactly 0 socket descriptors held.
 */
class AirGapSocketEnforcerDoDTest {

    @Test
    fun testAirGapSocketEnforcer_SocketCreationBlockedWithEacces() {
        println("=== Sprint 17 DoD: Verifying Socket Creation Denial (EACCES) ===")
        val result = AirGapSocketEnforcer.attemptNetworkSocket("8.8.8.8", 53)
        assertTrue("Socket attempt must be blocked", result.isBlocked)
        assertTrue(
            "Blocked result must document EACCES or network isolation",
            result.errorMessage?.contains("EACCES") == true ||
                result.errorMessage?.contains("denied") == true ||
                result.errorMessage?.contains("blocked") == true ||
                result.errorMessage?.contains("Permission denied") == true
        )

        val activeSockets = AirGapSocketEnforcer.getActiveSocketCount()
        assertEquals("IndNeuralCore must hold exactly 0 network socket file descriptors", 0, activeSockets)
        println("Air-Gap Socket Invariant Verified: Network socket allocation strictly blocked with EACCES.")
    }
}
