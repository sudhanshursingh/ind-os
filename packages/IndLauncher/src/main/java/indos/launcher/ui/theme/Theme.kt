package indos.launcher.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable

// ==============================================================================
// Ind OS — Launcher Theme
// ==============================================================================

private val IndDarkColorScheme = darkColorScheme(
    primary = IndElectricCyan,
    onPrimary = IndObsidianBackground,
    secondary = IndDeepViolet,
    onSecondary = IndTextPrimary,
    tertiary = IndNeonGreen,
    background = IndObsidianBackground,
    onBackground = IndTextPrimary,
    surface = IndDarkSurface,
    onSurface = IndTextPrimary,
    surfaceVariant = IndDarkSurfaceVariant,
    onSurfaceVariant = IndTextSecondary,
    outline = IndSurfaceBorder,
    error = IndCrimsonShield,
    onError = IndTextPrimary
)

@Composable
fun IndLauncherTheme(
    content: @Composable () -> Unit
) {
    MaterialTheme(
        colorScheme = IndDarkColorScheme,
        typography = IndTypography,
        content = content
    )
}
