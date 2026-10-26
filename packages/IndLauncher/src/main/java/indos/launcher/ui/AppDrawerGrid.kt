package indos.launcher.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import indos.launcher.model.AppInfo
import indos.launcher.ui.theme.IndDarkSurface
import indos.launcher.ui.theme.IndDeepViolet
import indos.launcher.ui.theme.IndElectricCyan
import indos.launcher.ui.theme.IndSurfaceBorder
import indos.launcher.ui.theme.IndTextPrimary
import indos.launcher.ui.theme.IndTextSecondary
import indos.launcher.ui.theme.IndTextTertiary

/**
 * 120 Hz Frame-Paced App Drawer Grid.
 * Built for ultra-low garbage collection overhead and buttery-smooth scrolling on Moto G60.
 */
@Composable
fun AppDrawerGrid(
    apps: List<AppInfo>,
    onAppClick: (AppInfo) -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp)
    ) {
        // Section Header
        Text(
            text = "APPLICATIONS (${apps.size})",
            style = TextStyle(
                color = IndTextTertiary,
                fontSize = 11.sp,
                fontWeight = FontWeight.Bold,
                letterSpacing = 1.sp
            ),
            modifier = Modifier.padding(start = 4.dp, bottom = 8.dp)
        )

        LazyVerticalGrid(
            columns = GridCells.Fixed(4),
            contentPadding = PaddingValues(vertical = 8.dp),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
            modifier = Modifier.fillMaxWidth()
        ) {
            items(
                items = apps,
                key = { it.packageName + "/" + it.activityName }
            ) { app ->
                AppGridItem(
                    app = app,
                    onClick = { onAppClick(app) }
                )
            }
        }
    }
}

@Composable
fun AppGridItem(
    app: AppInfo,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    val initial = remember(app.label) {
        app.label.take(1).uppercase()
    }

    val iconGradient = remember(app.packageName) {
        val hash = app.packageName.hashCode()
        val c1 = when (kotlin.math.abs(hash) % 4) {
            0 -> IndElectricCyan
            1 -> IndDeepViolet
            2 -> Color(0xFF00E676)
            else -> Color(0xFFFFB300)
        }
        Brush.radialGradient(
            colors = listOf(
                c1.copy(alpha = 0.85f),
                IndDarkSurface
            )
        )
    }

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = modifier
            .clip(RoundedCornerShape(16.dp))
            .clickable(onClick = onClick)
            .padding(vertical = 8.dp, horizontal = 4.dp)
    ) {
        // App Icon Container
        Box(
            modifier = Modifier
                .size(54.dp)
                .clip(CircleShape)
                .background(iconGradient)
                .border(1.dp, IndSurfaceBorder, CircleShape),
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = initial,
                style = TextStyle(
                    color = IndTextPrimary,
                    fontSize = 20.sp,
                    fontWeight = FontWeight.Bold
                )
            )
        }

        Spacer(modifier = Modifier.height(6.dp))

        // App Label
        Text(
            text = app.label,
            style = TextStyle(
                color = IndTextPrimary,
                fontSize = 12.sp,
                fontWeight = FontWeight.Medium
            ),
            textAlign = TextAlign.Center,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.fillMaxWidth()
        )
    }
}
