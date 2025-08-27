package com.finova.example.ui.theme

import android.app.Activity
import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat

// Finova Brand Colors
val FinovaPrimary = Color(0xFF6366F1) // Indigo 500
val FinovaPrimaryVariant = Color(0xFF4F46E5) // Indigo 600
val FinovaSecondary = Color(0xFF10B981) // Emerald 500
val FinovaSecondaryVariant = Color(0xFF059669) // Emerald 600
val FinovaTertiary = Color(0xFFF59E0B) // Amber 500
val FinovaTertiaryVariant = Color(0xFFD97706) // Amber 600
val FinovaError = Color(0xFFEF4444) // Red 500
val FinovaWarning = Color(0xFFF97316) // Orange 500
val FinovaSuccess = Color(0xFF22C55E) // Green 500

// Mining & XP Colors
val MiningGold = Color(0xFFFFD700)
val MiningBronze = Color(0xFFCD7F32)
val MiningPlatinum = Color(0xFFE5E7EB)
val XPBlue = Color(0xFF3B82F6)
val RPPurple = Color(0xFF8B5CF6)

// NFT Rarity Colors
val CommonGray = Color(0xFF6B7280)
val UncommonGreen = Color(0xFF10B981)
val RareBlue = Color(0xFF3B82F6)
val EpicPurple = Color(0xFF8B5CF6)
val LegendaryGold = Color(0xFFFFD700)
val MythicRed = Color(0xFFEF4444)

// Surface Colors
val SurfaceLight = Color(0xFFFAFAFA)
val SurfaceDark = Color(0xFF121212)
val SurfaceVariantLight = Color(0xFFF5F5F5)
val SurfaceVariantDark = Color(0xFF1E1E1E)

private val DarkColorScheme = darkColorScheme(
    primary = FinovaPrimary,
    onPrimary = Color.White,
    primaryContainer = FinovaPrimaryVariant,
    onPrimaryContainer = Color.White,
    secondary = FinovaSecondary,
    onSecondary = Color.Black,
    secondaryContainer = FinovaSecondaryVariant,
    onSecondaryContainer = Color.White,
    tertiary = FinovaTertiary,
    onTertiary = Color.Black,
    tertiaryContainer = FinovaTertiaryVariant,
    onTertiaryContainer = Color.White,
    error = FinovaError,
    errorContainer = Color(0xFF93000A),
    onError = Color.White,
    onErrorContainer = Color(0xFFFFDAD6),
    background = Color(0xFF0F172A),
    onBackground = Color(0xFFF8FAFC),
    surface = SurfaceDark,
    onSurface = Color(0xFFF8FAFC),
    surfaceVariant = SurfaceVariantDark,
    onSurfaceVariant = Color(0xFFE2E8F0),
    outline = Color(0xFF475569),
    inverseOnSurface = Color(0xFF0F172A),
    inverseSurface = Color(0xFFF8FAFC),
    inversePrimary = FinovaPrimaryVariant,
)

private val LightColorScheme = lightColorScheme(
    primary = FinovaPrimary,
    onPrimary = Color.White,
    primaryContainer = Color(0xFFEEF2FF),
    onPrimaryContainer = FinovaPrimaryVariant,
    secondary = FinovaSecondary,
    onSecondary = Color.White,
    secondaryContainer = Color(0xFFD1FAE5),
    onSecondaryContainer = FinovaSecondaryVariant,
    tertiary = FinovaTertiary,
    onTertiary = Color.White,
    tertiaryContainer = Color(0xFFFEF3C7),
    onTertiaryContainer = FinovaTertiaryVariant,
    error = FinovaError,
    errorContainer = Color(0xFFFFDAD6),
    onError = Color.White,
    onErrorContainer = Color(0xFF93000A),
    background = Color(0xFFFAFAFA),
    onBackground = Color(0xFF1E293B),
    surface = SurfaceLight,
    onSurface = Color(0xFF1E293B),
    surfaceVariant = SurfaceVariantLight,
    onSurfaceVariant = Color(0xFF475569),
    outline = Color(0xFF94A3B8),
    inverseOnSurface = Color(0xFFF8FAFC),
    inverseSurface = Color(0xFF1E293B),
    inversePrimary = FinovaPrimary,
)

@Composable
fun FinovaTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    dynamicColor: Boolean = false,
    content: @Composable () -> Unit
) {
    val colorScheme = when {
        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
            val context = LocalContext.current
            if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
        }
        darkTheme -> DarkColorScheme
        else -> LightColorScheme
    }
    val view = LocalView.current
    if (!view.isInEditMode) {
        SideEffect {
            val window = (view.context as Activity).window
            window.statusBarColor = colorScheme.primary.toArgb()
            WindowCompat.getInsetsController(window, view).isAppearanceLightStatusBars = darkTheme
        }
    }

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content
    )
}
