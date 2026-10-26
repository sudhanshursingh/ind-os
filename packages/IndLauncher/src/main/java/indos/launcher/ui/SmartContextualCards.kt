package indos.launcher.ui

import androidx.compose.animation.animateColorAsState
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
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import indos.launcher.model.PriorityNotification
import indos.launcher.model.PrivacyTogglesState
import indos.launcher.model.UpcomingAlarm
import indos.launcher.ui.theme.IndAmberAlert
import indos.launcher.ui.theme.IndCrimsonShield
import indos.launcher.ui.theme.IndDarkSurface
import indos.launcher.ui.theme.IndDarkSurfaceVariant
import indos.launcher.ui.theme.IndDeepViolet
import indos.launcher.ui.theme.IndElectricCyan
import indos.launcher.ui.theme.IndNeonGreen
import indos.launcher.ui.theme.IndSurfaceBorder
import indos.launcher.ui.theme.IndTextPrimary
import indos.launcher.ui.theme.IndTextSecondary
import indos.launcher.ui.theme.IndTextTertiary

/**
 * Smart Contextual Cards for Ind OS Home:
 * 1. Privacy & System Quick Toggles (Mic Mute, Camera Killswitch, AI Air-Gap, Suraksha Shield)
 * 2. Next Alarm & Indic Greeting Card
 * 3. Priority Notifications Digest (Zero Spam, Bank/OTP alerts only)
 */
@Composable
fun SmartContextualCards(
    togglesState: PrivacyTogglesState,
    onToggleMic: () -> Unit,
    onToggleCamera: () -> Unit,
    onToggleAirGap: () -> Unit,
    onToggleSuraksha: () -> Unit,
    alarm: UpcomingAlarm,
    onDismissAlarm: () -> Unit,
    notifications: List<PriorityNotification>,
    onNotificationClick: (PriorityNotification) -> Unit,
    promoDigest: indos.launcher.model.PromoDigestCardData = indos.launcher.model.PromoDigestCardData(),
    onDismissPromoDigest: () -> Unit = {},
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        // Quick Privacy & Air-Gap Toggles Card
        PrivacyQuickTogglesRow(
            state = togglesState,
            onToggleMic = onToggleMic,
            onToggleCamera = onToggleCamera,
            onToggleAirGap = onToggleAirGap,
            onToggleSuraksha = onToggleSuraksha
        )

        // Upcoming Alarm / Greeting Card
        UpcomingAlarmCard(
            alarm = alarm,
            onDismissAlarm = onDismissAlarm
        )

        // Silent Evening Notification Digest Card (Grouped Promo Spam)
        if (promoDigest.totalPromosCount > 0) {
            EveningNotificationDigestCard(
                digest = promoDigest,
                onDismiss = onDismissPromoDigest
            )
        }

        // Priority Notifications Digest Card
        if (notifications.isNotEmpty()) {
            PriorityNotificationsDigestCard(
                notifications = notifications,
                onNotificationClick = onNotificationClick
            )
        }
    }
}

@Composable
fun PrivacyQuickTogglesRow(
    state: PrivacyTogglesState,
    onToggleMic: () -> Unit,
    onToggleCamera: () -> Unit,
    onToggleAirGap: () -> Unit,
    onToggleSuraksha: () -> Unit,
    modifier: Modifier = Modifier
) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(20.dp))
            .background(IndDarkSurface)
            .border(1.dp, IndSurfaceBorder, RoundedCornerShape(20.dp))
            .padding(vertical = 12.dp, horizontal = 10.dp),
        horizontalArrangement = Arrangement.SpaceEvenly,
        verticalAlignment = Alignment.CenterVertically
    ) {
        ToggleItem(
            label = "Mic Mute",
            icon = "🎤",
            isActive = state.isMicMuted,
            activeColor = IndCrimsonShield,
            inactiveColor = IndTextSecondary,
            onClick = onToggleMic
        )

        ToggleItem(
            label = "Camera Off",
            icon = "📷",
            isActive = state.isCameraKilled,
            activeColor = IndCrimsonShield,
            inactiveColor = IndTextSecondary,
            onClick = onToggleCamera
        )

        ToggleItem(
            label = "Air-Gap",
            icon = "🛡️",
            isActive = state.isAirGapActive,
            activeColor = IndNeonGreen,
            inactiveColor = IndAmberAlert,
            onClick = onToggleAirGap
        )

        ToggleItem(
            label = "Suraksha",
            icon = "⚡",
            isActive = state.isSurakshaShieldActive,
            activeColor = IndElectricCyan,
            inactiveColor = IndTextTertiary,
            onClick = onToggleSuraksha
        )
    }
}

@Composable
fun ToggleItem(
    label: String,
    icon: String,
    isActive: Boolean,
    activeColor: Color,
    inactiveColor: Color,
    onClick: () -> Unit
) {
    val targetBg = if (isActive) activeColor.copy(alpha = 0.16f) else Color(0x11FFFFFF)
    val targetBorder = if (isActive) activeColor.copy(alpha = 0.5f) else Color(0x2230363D)

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(4.dp),
        modifier = Modifier
            .clip(RoundedCornerShape(12.dp))
            .clickable(onClick = onClick)
            .padding(horizontal = 6.dp, vertical = 4.dp)
    ) {
        Box(
            modifier = Modifier
                .size(42.dp)
                .clip(CircleShape)
                .background(targetBg)
                .border(1.dp, targetBorder, CircleShape),
            contentAlignment = Alignment.Center
        ) {
            Text(text = icon, fontSize = 18.sp)
        }

        Text(
            text = label,
            style = TextStyle(
                color = if (isActive) activeColor else IndTextSecondary,
                fontSize = 11.sp,
                fontWeight = if (isActive) FontWeight.SemiBold else FontWeight.Normal
            )
        )
    }
}

@Composable
fun UpcomingAlarmCard(
    alarm: UpcomingAlarm,
    onDismissAlarm: () -> Unit,
    modifier: Modifier = Modifier
) {
    Box(
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(20.dp))
            .background(
                Brush.horizontalGradient(
                    colors = listOf(
                        IndDarkSurface,
                        IndDarkSurfaceVariant
                    )
                )
            )
            .border(1.dp, IndSurfaceBorder, RoundedCornerShape(20.dp))
            .padding(16.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = "⏰ NEXT ALARM",
                        style = TextStyle(
                            color = IndElectricCyan,
                            fontSize = 10.5.sp,
                            fontWeight = FontWeight.Bold,
                            letterSpacing = 1.sp
                        )
                    )
                    Spacer(modifier = Modifier.width(6.dp))
                    Text(
                        text = "• " + alarm.daysSummary,
                        style = TextStyle(
                            color = IndTextTertiary,
                            fontSize = 10.sp
                        )
                    )
                }

                Spacer(modifier = Modifier.height(4.dp))

                Text(
                    text = alarm.timeString,
                    style = TextStyle(
                        color = IndTextPrimary,
                        fontSize = 24.sp,
                        fontWeight = FontWeight.Bold,
                        letterSpacing = (-0.5).sp
                    )
                )

                Text(
                    text = alarm.label,
                    style = TextStyle(
                        color = IndTextSecondary,
                        fontSize = 12.sp
                    )
                )
            }

            // Quick Dismiss / Snooze action
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(16.dp))
                    .background(Color(0x2200E5FF))
                    .border(0.8.dp, IndElectricCyan.copy(alpha = 0.5f), RoundedCornerShape(16.dp))
                    .clickable(onClick = onDismissAlarm)
                    .padding(horizontal = 12.dp, vertical = 6.dp),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = "Dismiss",
                    style = TextStyle(
                        color = IndElectricCyan,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.SemiBold
                    )
                )
            }
        }
    }
}

@Composable
fun PriorityNotificationsDigestCard(
    notifications: List<PriorityNotification>,
    onNotificationClick: (PriorityNotification) -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(20.dp))
            .background(IndDarkSurface)
            .border(1.dp, IndSurfaceBorder, RoundedCornerShape(20.dp))
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = "🛡️ SURAKSHA PRIORITY DIGEST",
                    style = TextStyle(
                        color = IndNeonGreen,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold,
                        letterSpacing = 1.sp
                    )
                )
            }

            Text(
                text = "Zero Spam Allowed",
                style = TextStyle(
                    color = IndTextTertiary,
                    fontSize = 10.sp,
                    fontWeight = FontWeight.Medium
                )
            )
        }

        notifications.take(3).forEach { notif ->
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(12.dp))
                    .background(IndDarkSurfaceVariant)
                    .clickable { onNotificationClick(notif) }
                    .padding(10.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Box(
                    modifier = Modifier
                        .size(32.dp)
                        .clip(CircleShape)
                        .background(
                            if (notif.isOtpOrFinancial) Color(0x3300E676) else Color(0x3300E5FF)
                        ),
                    contentAlignment = Alignment.Center
                ) {
                    Text(
                        text = if (notif.isOtpOrFinancial) "₹" else "💬",
                        fontSize = 14.sp
                    )
                }

                Spacer(modifier = Modifier.width(10.dp))

                Column(modifier = Modifier.weight(1f)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Text(
                            text = notif.title,
                            style = TextStyle(
                                color = IndTextPrimary,
                                fontSize = 12.5.sp,
                                fontWeight = FontWeight.SemiBold
                            )
                        )
                        Text(
                            text = notif.timestamp,
                            style = TextStyle(
                                color = IndTextTertiary,
                                fontSize = 10.sp
                            )
                        )
                    }

                    Text(
                        text = notif.content,
                        style = TextStyle(
                            color = IndTextSecondary,
                            fontSize = 11.sp
                        ),
                        maxLines = 1
                    )
                }
            }
        }
    }
}

@Composable
fun EveningNotificationDigestCard(
    digest: indos.launcher.model.PromoDigestCardData,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(20.dp))
            .background(
                Brush.horizontalGradient(
                    colors = listOf(
                        IndDarkSurface,
                        IndDeepViolet.copy(alpha = 0.25f)
                    )
                )
            )
            .border(1.dp, IndDeepViolet.copy(alpha = 0.5f), RoundedCornerShape(20.dp))
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = "🌙 EVENING PROMO DIGEST (${digest.totalPromosCount})",
                    style = TextStyle(
                        color = IndDeepViolet,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold,
                        letterSpacing = 1.sp
                    )
                )
            }

            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(12.dp))
                    .background(Color(0x337C4DFF))
                    .clickable(onClick = onDismiss)
                    .padding(horizontal = 8.dp, vertical = 2.dp)
            ) {
                Text(
                    text = "Dismiss",
                    style = TextStyle(color = IndTextPrimary, fontSize = 10.sp, fontWeight = FontWeight.SemiBold)
                )
            }
        }

        Text(
            text = digest.headlineSummary,
            style = TextStyle(
                color = IndTextPrimary,
                fontSize = 12.5.sp,
                fontWeight = FontWeight.Medium
            )
        )

        if (digest.promoSnippets.isNotEmpty()) {
            Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                digest.promoSnippets.take(3).forEach { snippet ->
                    Text(
                        text = "• $snippet",
                        style = TextStyle(
                            color = IndTextSecondary,
                            fontSize = 11.sp
                        ),
                        maxLines = 1
                    )
                }
            }
        }
    }
}

