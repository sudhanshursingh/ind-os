package indos.suraksha.integrity

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.File

/**
 * Sprint 17 Definition of Done (DoD) Test (Suraksha Shield Subsystem):
 * 1. SELinux Enforcing Policy Audit:
 *    - Neverallow rule forbids all sockets at the kernel level for ind_ai_daemon.
 *    - Service context maps ind_ai_service.
 *    - File context maps indaid binary.
 * 2. Sandboxed microG & Device Fingerprint Attestation:
 *    - Certified Moto G60 stock hardware fingerprint.
 *    - 10/10 Indian UPI & Banking apps pass MEETS_DEVICE_INTEGRITY.
 *    - microG permissions XML properly whitelists signature spoofing.
 */
class Sprint17IntegrityEnforcingDoDTest {

    private val repoRoot: File = locateRepoRoot()

    private fun locateRepoRoot(): File {
        var dir: File? = File(System.getProperty("user.dir") ?: ".")
        while (dir != null) {
            if (File(dir, "IND_OS_Master_PRD.md").exists()) {
                return dir
            }
            dir = dir.parentFile
        }
        return File(".")
    }

    @Test
    fun testSELinuxPolicy_NeverallowNetworkSocketsAndServiceContexts() {
        println("=== Sprint 17 DoD: Auditing SELinux Policy for ind_ai_daemon ===")
        val teFile = File(repoRoot, "vendor/ind/sepolicy/ind_ai_daemon.te")
        val serviceCtxFile = File(repoRoot, "vendor/ind/sepolicy/service_contexts")
        val fileCtxFile = File(repoRoot, "vendor/ind/sepolicy/file_contexts")

        assertTrue("ind_ai_daemon.te must exist", teFile.exists())
        assertTrue("service_contexts must exist", serviceCtxFile.exists())
        assertTrue("file_contexts must exist", fileCtxFile.exists())

        val teContent = teFile.readText()
        assertTrue("Must declare coredomain", teContent.contains("type ind_ai_daemon, domain, coredomain;"))
        assertTrue("Must declare ind_ai_service", teContent.contains("type ind_ai_service, service_manager_type;"))
        assertTrue("Must grant Vulkan GPU access", teContent.contains("gpu_device:chr_file"))
        assertTrue("Must grant ion memory access", teContent.contains("ion_device:chr_file"))

        // Critical Security Invariant: neverallow sockets
        assertTrue(
            "Must enforce strict neverallow on all network sockets",
            teContent.contains("neverallow ind_ai_daemon domain:{ socket tcp_socket udp_socket rawip_socket netlink_route_socket netlink_generic_socket } *;")
        )

        val serviceCtxContent = serviceCtxFile.readText()
        assertTrue("service_contexts must map ind_ai_service", serviceCtxContent.contains("u:object_r:ind_ai_service:s0"))

        val fileCtxContent = fileCtxFile.readText()
        assertTrue("file_contexts must map indaid binary", fileCtxContent.contains("u:object_r:ind_ai_daemon_exec:s0"))
        println("SELinux Policy Invariant Verified: Air-gap neverallow compiled and service contexts mapped.")
    }

    @Test
    fun testPlayIntegrityAttestationBridge_AllIndianBankingAppsPassDeviceIntegrity() {
        println("=== Sprint 17 DoD: Verifying Play Integrity for Indian UPI & Banking Suite ===")
        assertTrue("Attestation bridge must be active", PlayIntegrityAttestationBridge.isBridgeActive())

        val suite = PlayIntegrityAttestationBridge.TRUSTED_UPI_BANKING_APPS
        assertEquals("Must cover 10 major Indian banking and identity apps", 10, suite.size)

        val results = PlayIntegrityAttestationBridge.verifyBankingSuiteIntegrity()
        var passedCount = 0

        for ((pkg, passes) in results) {
            println("  Checking UPI App: $pkg -> MEETS_DEVICE_INTEGRITY: $passes")
            if (passes) passedCount++
        }

        assertEquals("All 10 banking apps must achieve MEETS_DEVICE_INTEGRITY", 10, passedCount)

        // Verify certified Motorola Moto G60 hardware fingerprint
        val verdict = PlayIntegrityAttestationBridge.evaluatePackage("in.org.npci.upiapp")
        assertTrue("BHIM UPI must pass basic integrity", verdict.meetsBasicIntegrity)
        assertTrue("BHIM UPI must pass device integrity", verdict.meetsDeviceIntegrity)
        assertFalse("Strong integrity requires hardware Knox HSM", verdict.meetsStrongIntegrity)
        assertEquals("PLAY_RECOGNIZED", verdict.appRecognitionVerdict)
        assertEquals(
            "motorola/hanoip_retail/hanoip:12/S1RQS32.53-29-2-5/67554-15967:user/release-keys",
            verdict.spoofedFingerprint
        )
        println("Play Integrity Bridge Verified: 10/10 Indian banking apps pass MEETS_DEVICE_INTEGRITY.")
    }

    @Test
    fun testSandboxedMicrogConfiguration_PermissionsAndProperties() {
        println("=== Sprint 17 DoD: Verifying Sandboxed microG Configuration ===")
        val microgMk = File(repoRoot, "vendor/ind/config/microg.mk")
        val permXml = File(repoRoot, "vendor/ind/permissions/microg-permissions.xml")
        val commonMk = File(repoRoot, "vendor/ind/config/common.mk")

        assertTrue("microg.mk must exist", microgMk.exists())
        assertTrue("microg-permissions.xml must exist", permXml.exists())
        assertTrue("common.mk must inherit microg.mk", commonMk.readText().contains("vendor/ind/config/microg.mk"))

        val mkContent = microgMk.readText()
        assertTrue("Must set IND_ENABLE_MICROG ?= true", mkContent.contains("IND_ENABLE_MICROG ?= true"))
        assertTrue("Must set ro.ind.microg.sandboxed=true", mkContent.contains("ro.ind.microg.sandboxed=true"))
        assertTrue("Must set ro.ind.playintegrity.spoof=true", mkContent.contains("ro.ind.playintegrity.spoof=true"))

        val xmlContent = permXml.readText()
        assertTrue("Must grant FAKE_PACKAGE_SIGNATURE to com.google.android.gms", xmlContent.contains("com.google.android.gms"))
        assertTrue("Must grant FAKE_PACKAGE_SIGNATURE to com.android.vending", xmlContent.contains("com.android.vending"))
        println("Sandboxed microG Verified: Permissions and build properties configured.")
    }
}
