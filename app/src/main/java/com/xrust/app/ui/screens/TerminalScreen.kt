package com.xrust.app.ui.screens

import android.graphics.Typeface
import android.view.KeyEvent
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.nativeCanvas
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.xrust.app.terminal.KeyCodec
import com.xrust.app.terminal.TerminalController
import com.xrust.app.ui.theme.Attr
import com.xrust.app.ui.theme.schemeByName

/** Screen 2: the interactive terminal. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TerminalScreen(
    getController: () -> TerminalController,
    onClose: () -> Unit,
    schemeName: String = "Alpine Dark",
    fontSizeSp: Int = 13,
) {
    val controller = remember { getController() }
    val scheme = schemeByName(schemeName)

    DisposableEffect(Unit) {
        controller.start()
        onDispose { }
    }

    val paint = remember(fontSizeSp) {
        android.graphics.Paint().apply {
            typeface = Typeface.MONOSPACE
            textSize = fontSizeSp.sp.value * 1.4f
            isAntiAlias = true
        }
    }
    val density = LocalDensity.current

    Column(Modifier.fillMaxSize()) {
        TopAppBar(
            title = { Text(controller.title.ifBlank { "terminal" }, fontSize = 15.sp) },
            navigationIcon = {
                IconButton(onClick = onClose) {
                    Icon(Icons.AutoMirrored.Filled.ArrowBack, "back")
                }
            },
            actions = {
                IconButton(onClick = { controller.signal(9) }) {
                    Icon(Icons.Filled.Close, "kill")
                }
            },
        )

        Surface(
            Modifier.fillMaxSize().background(scheme.bgColor()),
            color = scheme.bgColor(),
        ) {
            Column {
                // Terminal grid.
                BoxWithConstraints(
                    Modifier.fillMaxWidth().weight(1f),
                ) {
                    val cellW = with(density) { paint.measureText("M").toDp() }
                    val cellH = with(density) { (paint.fontSpacing).toDp() }
                    val cols = (this@BoxWithConstraints.maxWidth / cellW).toInt().coerceAtLeast(2)
                    val rows = (this@BoxWithConstraints.maxHeight / cellH).toInt().coerceAtLeast(1)
                    LaunchedEffect(cols, rows) { controller.resize(cols, rows) }
                    Canvas(
                        Modifier
                            .fillMaxSize()
                            .background(scheme.bgColor())
                            .onKeyEvent { event ->
                                if (event.type == KeyEventType.KeyDown) {
                                    val native = event.nativeKeyEvent as KeyEvent
                                    val bytes = KeyCodec.encode(native, false, false)
                                    if (bytes != null) {
                                        controller.send(bytes)
                                        return@onKeyEvent true
                                    }
                                }
                                false
                            },
                    ) {
                        val snap = controller.snapshot ?: return@Canvas
                        val w = cellW.toPx()
                        val h = cellH.toPx()
                        drawRect(scheme.bgColor())
                        for (r in 0 until snap.rows) {
                            for (c in 0 until snap.cols) {
                                val idx = r * snap.cols + c
                                if (snap.attrs[idx] and Attr.WIDE_CONT != 0) continue
                                val bgRaw = snap.bg[idx]
                                val fgRaw = snap.fg[idx]
                                val attrs = snap.attrs[idx]
                                var bg = if (bgRaw == 0xFFFF_FFFF.toInt()) scheme.bgColor() else argb(bgRaw)
                                var fg = if (fgRaw == 0xFFFF_FFFF.toInt()) scheme.fgColor() else argb(fgRaw)
                                if (attrs and Attr.REVERSE != 0) { val t = bg; bg = fg; fg = t }
                                if (bg != scheme.bgColor()) {
                                    drawRect(bg, topLeft = Offset(c * w, r * h), size = Size(w, h))
                                }
                                val cp = snap.cells[idx]
                                if (cp != 0 && cp != 32 && attrs and Attr.INVISIBLE == 0) {
                                    val ch = String(Character.toChars(cp))
                                    val style = android.graphics.Paint(paint).apply {
                                        color = fg.toArgb()
                                        isFakeBoldText = attrs and Attr.BOLD != 0
                                        textSkewX = if (attrs and Attr.ITALIC != 0) -0.25f else 0f
                                        isStrikeThruText = attrs and Attr.STRIKE != 0
                                        isUnderlineText = attrs and Attr.UNDERLINE != 0
                                        alpha = if (attrs and Attr.DIM != 0) 160 else 255
                                    }
                                    drawContext.canvas.nativeCanvas.drawText(
                                        ch, c * w, r * h - paint.ascent(), style,
                                    )
                                }
                            }
                        }
                        // Cursor
                        if (snap.cursorVisible) {
                            drawRect(
                                scheme.cursorColor().copy(alpha = 0.45f),
                                topLeft = Offset(snap.cursorCol * w, snap.cursorRow * h),
                                size = Size(
                                    if (snap.attrs[snap.cursorRow * snap.cols + snap.cursorCol] and Attr.WIDE != 0) 2 * w else w,
                                    h,
                                ),
                            )
                        }
                    }
                }

                // Extra-keys toolbar (scrollable row of common keys).
                Row(
                    Modifier.fillMaxWidth().horizontalScroll(rememberScrollState())
                        .padding(vertical = 4.dp),
                    horizontalArrangement = Arrangement.spacedBy(4.dp),
                ) {
                    extraKeys.forEach { (label, bytes) ->
                        Surface(
                            color = MaterialTheme.colorScheme.surfaceVariant,
                            onClick = { controller.send(bytes) },
                        ) {
                            Text(
                                label,
                                Modifier.padding(horizontal = 10.dp, vertical = 6.dp),
                                fontSize = 12.sp,
                            )
                        }
                    }
                }
            }
        }
    }
}

private val extraKeys = listOf(
    "ESC" to byteArrayOf(0x1B),
    "TAB" to byteArrayOf(9),
    "↑" to "\u001b[A".toByteArray(),
    "↓" to "\u001b[B".toByteArray(),
    "←" to "\u001b[D".toByteArray(),
    "→" to "\u001b[C".toByteArray(),
    "-" to byteArrayOf('-'.code.toByte()),
    "|" to byteArrayOf('|'.code.toByte()),
    "/" to byteArrayOf('/'.code.toByte()),
    "~" to byteArrayOf('~'.code.toByte()),
    "PGUP" to "\u001b[5~".toByteArray(),
    "PGDN" to "\u001b[6~".toByteArray(),
)

private fun argb(v: Int) = Color(v or 0xFF000000u.toInt())
