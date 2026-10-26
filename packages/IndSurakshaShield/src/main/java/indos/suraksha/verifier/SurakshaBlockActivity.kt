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

package indos.suraksha.verifier

import android.app.Activity
import android.graphics.Color
import android.graphics.Typeface
import android.os.Bundle
import android.view.Gravity
import android.view.ViewGroup
import android.widget.Button
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView

/**
 * Full-screen red Ind Suraksha Block Dialog (PRD v1.0.1 §4.3 & Sprint 12 DoD).
 *
 * Triggered when a sideloaded trojan APK is intercepted.
 * Displays bilingual English and Hindi threat warnings detailing why the application was blocked.
 */
class SurakshaBlockActivity : Activity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val pkgName = intent.getStringExtra("extra_package_name") ?: "com.unknown.trojan"
        val appName = intent.getStringExtra("extra_app_name") ?: "Malicious App"
        val titleEn = intent.getStringExtra("extra_title_en") ?: "MALICIOUS APPLICATION BLOCKED"
        val titleHi = intent.getStringExtra("extra_title_hi") ?: "खतरनाक ऐप को ब्लॉक कर दिया गया है"
        val msgEn = intent.getStringExtra("extra_message_en")
            ?: "This application requests both SMS reading and Accessibility Service control. This signature matches Indian banking trojans that steal OTPs and drain bank accounts."
        val msgHi = intent.getStringExtra("extra_message_hi")
            ?: "यह ऐप आपके SMS पढ़ने और फोन का पूरा नियंत्रण (Accessibility) लेने की अनुमति मांग रहा है। यह भारतीय बैंकिंग फ्रॉड ऐप का संकेत है जो OTP चुराकर पैसे निकाल लेते हैं।"
        val dangerousPerms = intent.getStringArrayListExtra("extra_dangerous_perms") ?: arrayListOf()

        // Construct high-contrast urgent red security UI programmatically
        val scrollView = ScrollView(this).apply {
            setBackgroundColor(Color.parseColor("#B71C1C")) // Deep Red Alert
            layoutParams = ViewGroup.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.MATCH_PARENT
            )
        }

        val container = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(48, 80, 48, 80)
            gravity = Gravity.CENTER_HORIZONTAL
        }

        // Shield Icon & Title (English)
        val tvTitleEn = TextView(this).apply {
            text = "🛡️ $titleEn"
            setTextColor(Color.WHITE)
            textSize = 22f
            typeface = Typeface.DEFAULT_BOLD
            gravity = Gravity.CENTER
            setPadding(0, 0, 0, 16)
        }

        // Title (Hindi)
        val tvTitleHi = TextView(this).apply {
            text = titleHi
            setTextColor(Color.parseColor("#FFF59D")) // Light yellow accent
            textSize = 20f
            typeface = Typeface.DEFAULT_BOLD
            gravity = Gravity.CENTER
            setPadding(0, 0, 0, 40)
        }

        // App Details Card
        val card = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setBackgroundColor(Color.parseColor("#7F0000")) // Darker red background
            setPadding(32, 32, 32, 32)
            layoutParams = LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                LinearLayout.LayoutParams.WRAP_CONTENT
            ).apply {
                setMargins(0, 0, 0, 40)
            }
        }

        val tvAppName = TextView(this).apply {
            text = "Target App: $appName ($pkgName)"
            setTextColor(Color.WHITE)
            textSize = 16f
            typeface = Typeface.MONOSPACE
            setPadding(0, 0, 0, 16)
        }

        val tvPerms = TextView(this).apply {
            text = "Dangerous Permissions Intercepted:\n• " + dangerousPerms.joinToString("\n• ")
            setTextColor(Color.parseColor("#FFCDD2"))
            textSize = 14f
            typeface = Typeface.MONOSPACE
        }

        card.addView(tvAppName)
        card.addView(tvPerms)

        // English Warning Message
        val tvMsgEn = TextView(this).apply {
            text = msgEn
            setTextColor(Color.WHITE)
            textSize = 16f
            setPadding(0, 0, 0, 32)
            setLineSpacing(8f, 1.1f)
        }

        // Hindi Warning Message
        val tvMsgHi = TextView(this).apply {
            text = msgHi
            setTextColor(Color.parseColor("#FFFDE7"))
            textSize = 16f
            setPadding(0, 0, 0, 48)
            setLineSpacing(8f, 1.1f)
        }

        // Dismiss / Abort Action Button
        val btnAbort = Button(this).apply {
            text = "DELETE APK & ABORT INSTALLATION"
            setBackgroundColor(Color.WHITE)
            setTextColor(Color.parseColor("#B71C1C"))
            typeface = Typeface.DEFAULT_BOLD
            textSize = 16f
            setPadding(24, 24, 24, 24)
            setOnClickListener {
                finish()
            }
        }

        container.addView(tvTitleEn)
        container.addView(tvTitleHi)
        container.addView(card)
        container.addView(tvMsgEn)
        container.addView(tvMsgHi)
        container.addView(btnAbort)

        scrollView.addView(container)
        setContentView(scrollView)
    }
}
