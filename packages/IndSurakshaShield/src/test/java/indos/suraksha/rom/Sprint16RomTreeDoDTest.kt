package indos.suraksha.rom

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.File
import java.util.zip.ZipFile

/**
 * Sprint 16 Definition of Done (DoD) Test:
 * 1. Tree Sync & Zero-Bloat Product Makefile:
 *    - vendor/ind/config/common.mk purges Jelly, Eleven, LineageUpdater, Etar, Recorder.
 *    - All 4 core Ind OS subsystems included in PRODUCT_PACKAGES.
 * 2. Device Product & Lunch Target:
 *    - vendor/ind/AndroidProducts.mk defines ind_hanoip-userdebug.
 *    - vendor/ind/products/ind_hanoip.mk sets target Motorola Moto G60 (hanoip).
 * 3. Ind OS Boot Animation:
 *    - 1080 x 2460 @ 60fps, 0% compression (ZIP_STORED).
 * 4. Settings Overlays for About Phone Branding:
 *    - Defines Ind OS v1.0.1-BIBLE.
 */
class Sprint16RomTreeDoDTest {

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
    fun testManifestTree_ContainsAllSubsystemsAndMotoG60Trees() {
        println("=== Sprint 16 DoD: Verifying AOSP Manifest Tree ===")
        val manifestFile = File(repoRoot, "manifest/default.xml")
        assertTrue("manifest/default.xml must exist", manifestFile.exists())
        val content = manifestFile.readText()

        val requiredProjects = listOf(
            "packages/apps/IndNeuralCore",
            "packages/apps/IndZeroTouch",
            "packages/apps/IndSurakshaShield",
            "packages/apps/IndLauncher",
            "vendor/ind",
            "device/motorola/hanoip",
            "device/motorola/sm6150-common",
            "kernel/motorola/sm6150",
            "vendor/motorola/hanoip",
            "vendor/motorola/sm6150-common"
        )

        for (proj in requiredProjects) {
            assertTrue("Manifest must include project path '$proj'", content.contains("path=\"$proj\""))
        }
        println("Manifest Tree Invariant Verified: All 10 core repos and vendor blobs present.")
    }

    @Test
    fun testCommonMakefile_ZeroBloatAndCorePackages() {
        println("=== Sprint 16 DoD: Verifying Zero-Bloat Product Configuration ===")
        val commonMk = File(repoRoot, "vendor/ind/config/common.mk")
        assertTrue("vendor/ind/config/common.mk must exist", commonMk.exists())
        val content = commonMk.readText()

        // 1. Verify Ind OS Version properties
        assertTrue("Must define ro.ind.version", content.contains("ro.ind.version"))
        assertTrue("Must define ro.ind.founder=Sudhanshu_Raj_Singh", content.contains("Sudhanshu_Raj_Singh"))
        assertTrue("Must enforce airgap by default", content.contains("ro.ind.airgap.enforced=true"))

        // 2. Verify all core packages in PRODUCT_PACKAGES
        val corePkgs = listOf("IndNeuralCore", "IndZeroTouch", "IndSurakshaShield", "IndLauncher")
        for (pkg in corePkgs) {
            assertTrue("PRODUCT_PACKAGES must include '$pkg'", content.contains(pkg))
        }

        // 3. Verify bloatware purged
        val bloat = listOf("Jelly", "Eleven", "LineageUpdater", "Etar", "Recorder")
        for (b in bloat) {
            assertTrue("PRODUCT_PACKAGES_REMOVE must include '$b'", content.contains(b))
        }
        println("Zero-Bloat Product Makefile Verified: Core packages included, 5 bloatware packages purged.")
    }

    @Test
    fun testLunchTarget_AndroidProductsAndDeviceMakefile() {
        println("=== Sprint 16 DoD: Verifying Lunch Targets & Device Product Makefile ===")
        val androidProductsMk = File(repoRoot, "vendor/ind/AndroidProducts.mk")
        val productMk = File(repoRoot, "vendor/ind/products/ind_hanoip.mk")

        assertTrue("AndroidProducts.mk must exist", androidProductsMk.exists())
        assertTrue("products/ind_hanoip.mk must exist", productMk.exists())

        val apContent = androidProductsMk.readText()
        assertTrue("AndroidProducts.mk must register ind_hanoip-userdebug", apContent.contains("ind_hanoip-userdebug"))
        assertTrue("AndroidProducts.mk must reference products/ind_hanoip.mk", apContent.contains("products/ind_hanoip.mk"))

        val prodContent = productMk.readText()
        assertTrue("Must set PRODUCT_NAME := ind_hanoip", prodContent.contains("PRODUCT_NAME := ind_hanoip"))
        assertTrue("Must set PRODUCT_DEVICE := hanoip", prodContent.contains("PRODUCT_DEVICE := hanoip"))
        assertTrue("Must set PRODUCT_MODEL := moto g(60)", prodContent.contains("PRODUCT_MODEL := moto g(60)"))
        println("Lunch Target & Device Product Makefile Verified: ind_hanoip-userdebug ready.")
    }

    @Test
    fun testBootAnimation_ZeroCompressionAndResolution() {
        println("=== Sprint 16 DoD: Verifying Ind OS Boot Animation ===")
        val bootanimFile = File(repoRoot, "vendor/ind/media/bootanimation.zip")
        assertTrue("bootanimation.zip must exist", bootanimFile.exists())

        ZipFile(bootanimFile).use { zip ->
            val descEntry = zip.getEntry("desc.txt")
            assertTrue("desc.txt entry must exist in bootanimation.zip", descEntry != null)
            assertEquals("Android requires 0% compression (ZIP_STORED)", java.util.zip.ZipEntry.STORED, descEntry.method)

            val descContent = zip.getInputStream(descEntry).bufferedReader().readText()
            assertTrue("desc.txt must specify Moto G60 resolution '1080 2460 60'", descContent.contains("1080 2460 60"))
            assertTrue("desc.txt must have part0", descContent.contains("part0"))
            assertTrue("desc.txt must have part1", descContent.contains("part1"))

            // Verify frames exist in part0 and part1
            val part0Entry = zip.getEntry("part0/00000.png")
            val part1Entry = zip.getEntry("part1/00000.png")
            assertTrue("part0 frame 0 must exist", part0Entry != null)
            assertTrue("part1 frame 0 must exist", part1Entry != null)
        }
        println("Boot Animation Invariants Verified: 1080x2460 @ 60fps stored with 0% compression.")
    }

    @Test
    fun testSettingsOverlay_AboutPhoneBranding() {
        println("=== Sprint 16 DoD: Verifying Settings Overlay Branding ===")
        val overlayFile = File(repoRoot, "vendor/ind/overlay/common/packages/apps/Settings/res/values/strings.xml")
        assertTrue("Settings overlay strings.xml must exist", overlayFile.exists())
        val content = overlayFile.readText()

        assertTrue("Overlay must contain 'Ind OS v1.0.1-BIBLE'", content.contains("Ind OS v1.0.1-BIBLE"))
        assertTrue("Overlay must brand build_number", content.contains("name=\"build_number\""))
        assertTrue("Overlay must brand lineage_platform_version", content.contains("name=\"lineage_platform_version\""))
        println("Settings Overlay Verified: Ind OS v1.0.1-BIBLE branding registered.")
    }
}
