/*
 * Copyright (C) 2026 Ind OS Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package indos.zerotouch.audio

import android.content.Context
import android.content.ContextWrapper
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.io.File

/**
 * Unit test suite verifying encrypted persistence of 192-dim voice biometric profile.
 */
class VoiceEnrollmentManagerTest {

    @get:Rule
    val tempFolder = TemporaryFolder()

    private lateinit var fakeContext: Context
    private lateinit var enrollmentManager: VoiceEnrollmentManager

    private class TestContext(private val baseDir: File) : ContextWrapper(null) {
        override fun getFilesDir(): File = baseDir
    }

    @Before
    fun setUp() {
        fakeContext = TestContext(tempFolder.newFolder("app_files"))
        enrollmentManager = VoiceEnrollmentManager(fakeContext)
    }

    @Test
    fun testEnrollmentSaveAndLoadRoundTrip() {
        assertFalse("Profile should initially be absent", enrollmentManager.isEnrolled())

        // Create 192-dim normalized embedding
        val testEmbedding = FloatArray(192) { (it * 0.005f) }
        val saved = enrollmentManager.saveEnrollment(testEmbedding)
        assertTrue("Save enrollment should succeed", saved)

        assertTrue("Profile should report enrolled after save", enrollmentManager.isEnrolled())

        val loaded = enrollmentManager.loadEnrollment()
        assertNotNull("Loaded enrollment should not be null", loaded)
        assertEquals(192, loaded!!.size)

        for (i in 0 until 192) {
            assertEquals("Embedding at index $i must match with float precision", testEmbedding[i], loaded[i], 0.0001f)
        }
    }

    @Test
    fun testClearEnrollment() {
        val testEmbedding = FloatArray(192) { 0.5f }
        enrollmentManager.saveEnrollment(testEmbedding)
        assertTrue(enrollmentManager.isEnrolled())

        val cleared = enrollmentManager.clearEnrollment()
        assertTrue(cleared)
        assertFalse(enrollmentManager.isEnrolled())
    }
}
