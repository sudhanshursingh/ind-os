package indos.neural

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

/**
 * Unit tests verifying Mathematical Invariant 1 (PRD §3.2):
 * - Peak resident RAM: 1226 MB
 * - Eviction drops RAM to 18 MB in < 12 ms
 * - Reload returns to 1226 MB
 */
class ModelLifecyclePagerTest {

    private lateinit var pager: ModelLifecyclePager

    @Before
    fun setUp() {
        pager = ModelLifecyclePager.instance
        pager.reloadModelWeights()
    }

    @Test
    fun testInitialResidentRamMatchesBudget() {
        assertEquals("Initial resident RAM must be 1226 MB", 1226, pager.getResidentRamMb())
        assertEquals("Pager state must be RESIDENT", ModelLifecyclePager.PagerState.RESIDENT, pager.getState())
    }

    @Test
    fun testEvictionDropsRamAndSatisfiesLatencyBudget() {
        val durationMs = pager.evictModelWeights("UNIT_TEST_TRIGGER")

        assertEquals("Evicted resident RAM must drop to 18 MB", 18, pager.getResidentRamMb())
        assertEquals("Pager state must be EVICTED", ModelLifecyclePager.PagerState.EVICTED, pager.getState())
        assertTrue(
            "Eviction latency must be <= 12.0 ms (actual: %.2f ms)".format(durationMs),
            durationMs <= 12.0
        )
        assertTrue("Total eviction count must be >= 1", pager.getTotalEvictionCount() >= 1)
    }

    @Test
    fun testReloadRestoresPeakResidentRam() {
        pager.evictModelWeights("BEFORE_RELOAD_TEST")
        assertEquals(18, pager.getResidentRamMb())

        val reloadDurationMs = pager.reloadModelWeights()
        assertEquals("Reloaded resident RAM must return to 1226 MB", 1226, pager.getResidentRamMb())
        assertEquals("Pager state must be RESIDENT", ModelLifecyclePager.PagerState.RESIDENT, pager.getState())
        assertTrue("Reload duration must be non-negative", reloadDurationMs >= 0.0)
    }
}
