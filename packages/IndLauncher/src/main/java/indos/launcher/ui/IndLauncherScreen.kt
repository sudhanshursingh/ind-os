package indos.launcher.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import indos.launcher.model.AppInfo
import indos.launcher.model.PriorityNotification
import indos.launcher.model.PrivacyTogglesState
import indos.launcher.model.UpcomingAlarm
import indos.launcher.ui.theme.IndObsidianBackground

/**
 * Root Composable for Ind OS Zero-Bloat Intent Launcher.
 * Features:
 * 1. Ind Command Bar (Ambient Intent & App Entry)
 * 2. Smart Contextual Cards (Privacy Toggles, Alarm, Priority Notifications)
 * 3. 120 Hz App Drawer Grid
 */
@Composable
fun IndLauncherScreen(
    apps: List<AppInfo>,
    onLaunchApp: (AppInfo) -> Unit,
    onExecuteIntent: (String) -> Unit,
    onTriggerVoiceOrb: () -> Unit,
    initialToggles: PrivacyTogglesState = PrivacyTogglesState(),
    initialAlarm: UpcomingAlarm = UpcomingAlarm(),
    initialNotifications: List<PriorityNotification> = emptyList(),
    modifier: Modifier = Modifier
) {
    var queryText by remember { mutableStateOf("") }
    var togglesState by remember { mutableStateOf(initialToggles) }
    var currentAlarm by remember { mutableStateOf(initialAlarm) }
    var notificationsList by remember { mutableStateOf(initialNotifications) }

    // Instant live search & fuzzy filtering
    val filteredApps = remember(queryText, apps) {
        if (queryText.isBlank()) {
            apps
        } else {
            val q = queryText.trim().lowercase()
            apps.filter { app ->
                app.label.lowercase().contains(q) ||
                    app.packageName.lowercase().contains(q)
            }
        }
    }

    Box(
        modifier = modifier
            .fillMaxSize()
            .background(IndObsidianBackground)
    ) {
        LazyColumn(
            modifier = Modifier.fillMaxSize()
        ) {
            // Top Ambient Ind Command Bar
            item {
                Spacer(modifier = Modifier.height(28.dp))
                IndCommandBar(
                    queryText = queryText,
                    onQueryChange = { queryText = it },
                    onSubmitIntent = { intentQuery ->
                        if (filteredApps.isNotEmpty() && !intentQuery.startsWith("sys:", ignoreCase = true)) {
                            // If direct app match exists, launch top match
                            onLaunchApp(filteredApps.first())
                        } else {
                            onExecuteIntent(intentQuery)
                        }
                    },
                    onTriggerVoiceOrb = onTriggerVoiceOrb,
                    isAirGapActive = togglesState.isAirGapActive
                )
                Spacer(modifier = Modifier.height(12.dp))
            }

            // Only show contextual cards when not searching
            if (queryText.isBlank()) {
                item {
                    SmartContextualCards(
                        togglesState = togglesState,
                        onToggleMic = {
                            togglesState = togglesState.copy(isMicMuted = !togglesState.isMicMuted)
                        },
                        onToggleCamera = {
                            togglesState = togglesState.copy(isCameraKilled = !togglesState.isCameraKilled)
                        },
                        onToggleAirGap = {
                            togglesState = togglesState.copy(isAirGapActive = !togglesState.isAirGapActive)
                        },
                        onToggleSuraksha = {
                            togglesState = togglesState.copy(isSurakshaShieldActive = !togglesState.isSurakshaShieldActive)
                        },
                        alarm = currentAlarm,
                        onDismissAlarm = {
                            currentAlarm = currentAlarm.copy(isEnabled = false, label = "Alarm Dismissed")
                        },
                        notifications = notificationsList,
                        onNotificationClick = { notif ->
                            // Remove or handle clicked notification
                            notificationsList = notificationsList.filter { it.id != notif.id }
                        }
                    )
                    Spacer(modifier = Modifier.height(20.dp))
                }
            }

            // App Drawer Grid Section
            item {
                AppDrawerGrid(
                    apps = filteredApps,
                    onAppClick = onLaunchApp
                )
                Spacer(modifier = Modifier.height(32.dp))
            }
        }
    }
}
