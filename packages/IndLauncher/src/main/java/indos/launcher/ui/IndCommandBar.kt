package indos.launcher.ui

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import indos.launcher.ui.theme.IndAirGapBadgeBg
import indos.launcher.ui.theme.IndAirGapBadgeBorder
import indos.launcher.ui.theme.IndAirGapBadgeText
import indos.launcher.ui.theme.IndDarkSurface
import indos.launcher.ui.theme.IndElectricCyan
import indos.launcher.ui.theme.IndNeonGreen
import indos.launcher.ui.theme.IndObsidianBackground
import indos.launcher.ui.theme.IndSurfaceBorder
import indos.launcher.ui.theme.IndTextPrimary
import indos.launcher.ui.theme.IndTextSecondary
import indos.launcher.ui.theme.IndTextTertiary

/**
 * Ind Command Bar: Zero-Bloat Ambient Intent Entry Bar.
 * Supports typing natural language intents ("turn on torch", "call Papa", "open UPI")
 * or fuzzy app queries with instant execution, voice orb trigger, and air-gap telemetry.
 */
@Composable
fun IndCommandBar(
    queryText: String,
    onQueryChange: (String) -> Unit,
    onSubmitIntent: (String) -> Unit,
    onTriggerVoiceOrb: () -> Unit,
    isAirGapActive: Boolean = true,
    modifier: Modifier = Modifier
) {
    val barBorderBrush = remember {
        Brush.horizontalGradient(
            colors = listOf(
                IndElectricCyan.copy(alpha = 0.6f),
                IndSurfaceBorder,
                IndNeonGreen.copy(alpha = 0.5f)
            )
        )
    }

    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 8.dp)
    ) {
        // Status & Air-Gap Telemetry Row
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 4.dp, vertical = 4.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = "IND OS",
                style = TextStyle(
                    color = IndElectricCyan,
                    fontSize = 13.sp,
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 2.sp
                )
            )

            // Live Air-Gap Badge
            AirGapPill(isAirGapActive = isAirGapActive)
        }

        Spacer(modifier = Modifier.height(4.dp))

        // Main Ambient Command Pill
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(56.dp)
                .shadow(elevation = 12.dp, shape = RoundedCornerShape(28.dp), ambientColor = IndElectricCyan)
                .clip(RoundedCornerShape(28.dp))
                .background(IndDarkSurface)
                .border(width = 1.dp, brush = barBorderBrush, shape = RoundedCornerShape(28.dp))
                .padding(horizontal = 16.dp),
            contentAlignment = Alignment.CenterStart
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                // Intent / App Search Input Field
                Box(
                    modifier = Modifier.weight(1f),
                    contentAlignment = Alignment.CenterStart
                ) {
                    if (queryText.isEmpty()) {
                        Text(
                            text = "Search apps or type intent ('open WhatsApp', 'torch')...",
                            style = TextStyle(
                                color = IndTextTertiary,
                                fontSize = 13.5.sp,
                                fontWeight = FontWeight.Normal
                            ),
                            maxLines = 1
                        )
                    }

                    BasicTextField(
                        value = queryText,
                        onValueChange = onQueryChange,
                        singleLine = true,
                        cursorBrush = SolidColor(IndElectricCyan),
                        textStyle = TextStyle(
                            color = IndTextPrimary,
                            fontSize = 14.5.sp,
                            fontWeight = FontWeight.Medium
                        ),
                        keyboardOptions = KeyboardOptions(
                            imeAction = ImeAction.Search
                        ),
                        keyboardActions = KeyboardActions(
                            onSearch = {
                                if (queryText.isNotBlank()) {
                                    onSubmitIntent(queryText)
                                }
                            }
                        ),
                        modifier = Modifier.fillMaxWidth()
                    )
                }

                if (queryText.isNotEmpty()) {
                    // Clear query button
                    Box(
                        modifier = Modifier
                            .size(28.dp)
                            .clip(CircleShape)
                            .background(Color(0x3364748B))
                            .clickable { onQueryChange("") },
                        contentAlignment = Alignment.Center
                    ) {
                        Text(
                            text = "✕",
                            style = TextStyle(color = IndTextSecondary, fontSize = 12.sp, fontWeight = FontWeight.Bold)
                        )
                    }
                    Spacer(modifier = Modifier.width(8.dp))
                }

                // Voice Orb Trigger Pill
                VoiceOrbTriggerButton(onClick = onTriggerVoiceOrb)
            }
        }
    }
}

@Composable
fun AirGapPill(
    isAirGapActive: Boolean,
    modifier: Modifier = Modifier
) {
    val badgeBg = if (isAirGapActive) IndAirGapBadgeBg else Color(0x22FFB300)
    val badgeBorder = if (isAirGapActive) IndAirGapBadgeBorder else Color(0x66FFB300)
    val badgeText = if (isAirGapActive) IndAirGapBadgeText else Color(0xFFFFB300)
    val statusLabel = if (isAirGapActive) "AIR-GAP SHIELD ON" else "AIR-GAP DISABLED"

    Box(
        modifier = modifier
            .clip(RoundedCornerShape(12.dp))
            .background(badgeBg)
            .border(width = 0.8.dp, color = badgeBorder, shape = RoundedCornerShape(12.dp))
            .padding(horizontal = 8.dp, vertical = 2.dp)
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            Box(
                modifier = Modifier
                    .size(6.dp)
                    .clip(CircleShape)
                    .background(badgeText)
            )
            Text(
                text = statusLabel,
                style = TextStyle(
                    color = badgeText,
                    fontSize = 9.sp,
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 0.5.sp
                )
            )
        }
    }
}

@Composable
fun VoiceOrbTriggerButton(
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    Box(
        modifier = modifier
            .size(38.dp)
            .clip(CircleShape)
            .background(
                Brush.radialGradient(
                    colors = listOf(
                        IndElectricCyan,
                        IndDarkSurface
                    )
                )
            )
            .border(1.dp, IndElectricCyan.copy(alpha = 0.8f), CircleShape)
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        // Glowing microphone / Voice Orb symbol
        Text(
            text = "🎙",
            fontSize = 16.sp
        )
    }
}
