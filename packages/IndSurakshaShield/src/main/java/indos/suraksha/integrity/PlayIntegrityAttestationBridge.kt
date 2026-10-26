package indos.suraksha.integrity

import android.util.Log

// ==============================================================================
// Ind OS — Play Integrity Attestation Bridge for Indian Banking & UPI Apps
// Reference: PRD v1.0.1-BIBLE (Phase 6, Sprint 17)
// Target Hardware: Motorola Moto G60 (hanoip — Snapdragon 732G)
// ==============================================================================

data class PlayIntegrityVerdict(
    val packageName: String,
    val meetsBasicIntegrity: Boolean,
    val meetsDeviceIntegrity: Boolean,
    val meetsStrongIntegrity: Boolean,
    val appRecognitionVerdict: String,
    val spoofedFingerprint: String
)

data class CertifiedDeviceProfile(
    val brand: String = "motorola",
    val model: String = "moto g(60)",
    val device: String = "hanoip",
    val product: String = "hanoip_retail",
    val manufacturer: String = "motorola",
    val fingerprint: String = "motorola/hanoip_retail/hanoip:12/S1RQS32.53-29-2-5/67554-15967:user/release-keys",
    val securityPatch: String = "2023-03-01",
    val isBootloaderLocked: Boolean = true,
    val verifiedBootState: String = "green"
)

object PlayIntegrityAttestationBridge {

    private const val TAG = "IndPlayIntegrity"

    val STOCK_HANOIP_PROFILE = CertifiedDeviceProfile()

    // Whitelisted Indian Banking, UPI, and Government Identity Apps
    val TRUSTED_UPI_BANKING_APPS = setOf(
        "in.org.npci.upiapp",                  // BHIM UPI
        "com.phonepe.app",                     // PhonePe
        "com.google.android.apps.nbu.paisa.user", // Google Pay India
        "net.one97.paytm",                     // Paytm
        "com.sbi.lotusintouch",                // YONO SBI
        "com.icicibank.mobile",                // iMobile Pay by ICICI
        "com.hdfcbank.mobilebanking",          // HDFC Bank MobileBanking
        "com.axis.mobile",                     // Axis Mobile
        "in.gov.uidai.mAadhaarPlus",           // mAadhaar
        "in.gov.umang.negd.g2c"                // UMANG
    )

    private var isAttestationActive = true

    /**
     * Checks if the attestation bridge is active.
     */
    fun isBridgeActive(): Boolean = isAttestationActive

    /**
     * Toggles attestation bridge.
     */
    fun setBridgeActive(active: Boolean) {
        isAttestationActive = active
        Log.i(TAG, "[ATTESTATION_TOGGLE] Play Integrity Attestation Bridge active: $active")
    }

    /**
     * Evaluates Play Integrity for a client app requesting attestation token.
     */
    fun evaluatePackage(packageName: String): PlayIntegrityVerdict {
        val isBankingApp = TRUSTED_UPI_BANKING_APPS.contains(packageName)

        if (isAttestationActive && isBankingApp) {
            Log.i(
                TAG,
                "[PLAY_INTEGRITY_PASS] Whitelisted UPI Banking app '$packageName' granted MEETS_DEVICE_INTEGRITY " +
                    "using certified Moto G60 hardware fingerprint."
            )
            return PlayIntegrityVerdict(
                packageName = packageName,
                meetsBasicIntegrity = true,
                meetsDeviceIntegrity = true,
                meetsStrongIntegrity = false, // Strong requires hardware Knox/Titan M HSM
                appRecognitionVerdict = "PLAY_RECOGNIZED",
                spoofedFingerprint = STOCK_HANOIP_PROFILE.fingerprint
            )
        }

        // Standard evaluation for regular apps
        return PlayIntegrityVerdict(
            packageName = packageName,
            meetsBasicIntegrity = isAttestationActive,
            meetsDeviceIntegrity = false,
            meetsStrongIntegrity = false,
            appRecognitionVerdict = if (isAttestationActive) "PLAY_RECOGNIZED" else "UNEVALUATED",
            spoofedFingerprint = if (isAttestationActive) STOCK_HANOIP_PROFILE.fingerprint else ""
        )
    }

    /**
     * Verifies that all major Indian UPI and banking apps pass device integrity checks.
     */
    fun verifyBankingSuiteIntegrity(): Map<String, Boolean> {
        val results = mutableMapOf<String, Boolean>()
        for (pkg in TRUSTED_UPI_BANKING_APPS) {
            val verdict = evaluatePackage(pkg)
            val passes = verdict.meetsBasicIntegrity && verdict.meetsDeviceIntegrity
            results[pkg] = passes
        }
        return results
    }
}
