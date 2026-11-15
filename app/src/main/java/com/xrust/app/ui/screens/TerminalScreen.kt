package com.xrust.app.ui.screens

import android.graphics.Typeface
import android.view.KeyEvent
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
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
import androidx.compose.ui.Alignment
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
import com.xrust.app.ui.responsive.rememberIsLandscape
import com.xrust.app.ui.theme.Attr
import com.xrust.app.ui.theme.schemeByName

/** Screen 2: the interactive terminal — layout adapts to window shape. */
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
    val landscape = rememberIsLandscape()

    Column(
        Modifier
            .fillMaxSize()
            // Keep the grid and toolbar visible above the soft keyboard
            // (manifest already runs with adjustResize).
            .imePadding(),
    ) {
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
            if (landscape) {
                // Landscape / wide windows: keys stand in a rail on the right
                // so the grid keeps every pixel of vertical space.
                Row(Modifier.fillMaxSize()) {
                    TerminalGrid(controller, scheme, paint, density, Modifier.weight(1f))
                    ExtraKeys(controller, Modifier.widthIn(min = 72.dp), vertical = true)
                }
            } else {
                Column {
                    TerminalGrid(controller, scheme, paint, density, Modifier.weight(1f))
                    ExtraKeys(controller, Modifier.fillMaxWidth(), vertical = false)
                }
            }
        }
    }
}

/** The terminal canvas; sizes the grid to the available window. */
@Composable
private fun TerminalGrid(
    controller: TerminalController,
    scheme: com.xrust.app.ui.theme.TerminalScheme,
    paint: android.graphics.Paint,
    density: androidx.compose.ui.unit.Density,
    modifier: Modifier = Modifier,
) {
    BoxWithConstraints(modifier.fillMaxWidth()) {
        val cellW = with(density) { paint.measureText("M").toDp() }
        val cellH = with(density) { (paint.fontSpacing).toDp() }
        val cols = (this@BoxWithConstraints.maxWidth / cellW).toInt().coerceAtLeast(2)
        val rows = (this@BoxWithConstraints.maxHeight / cellH).toInt().coerceAtLeast(1)
        LaunchedEffect(cols, rows) { controller.resize(cols, rows) }
        // Reused across every drawn cell: the old loop allocated a fresh
        // Paint per visible cell per frame (~1000 objects/frame → GC jank on
        // scroll floods). Same-style character runs share one drawText call,
        // cutting JNI text calls by an order of magnitude on normal output.
        val cellPaint = remember(paint) { android.graphics.Paint(paint) }
        val runText = remember { StringBuilder(256) }
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
            val ascent = paint.ascent()
            drawRect(scheme.bgColor())
            for (r in 0 until snap.rows) {
                val rowTop = r * h
                val baseline = r * h - ascent
                var c = 0
                while (c < snap.cols) {
                    val idx = r * snap.cols + c
                    if (snap.attrs[idx] and Attr.WIDE_CONT != 0) { c++; continue }
                    val bgRaw = snap.bg[idx]
                    val fgRaw = snap.fg[idx]
                    val attrs = snap.attrs[idx]
                    var bg = if (bgRaw == 0xFFFF_FFFF.toInt()) scheme.bgColor() else argb(bgRaw)
                    var fg = if (fgRaw == 0xFFFF_FFFF.toInt()) scheme.fgColor() else argb(fgRaw)
                    if (attrs and Attr.REVERSE != 0) { val t = bg; bg = fg; fg = t }
                    if (bg != scheme.bgColor()) {
                        drawRect(bg, topLeft = Offset(c * w, rowTop), size = Size(w, h))
                    }
                    val cp = snap.cells[idx]
                    if (cp != 0 && cp != 32 && attrs and Attr.INVISIBLE == 0) {
                        // Batch the run of adjacent cells sharing this exact
                        // style into one drawText (allocation-free per cell).
                        runText.setLength(0)
                        runText.appendCodePoint(cp)
                        var j = c + 1
                        while (j < snap.cols) {
                            val jidx = r * snap.cols + j
                            if (snap.attrs[jidx] and Attr.WIDE_CONT != 0) break
                            val jcp = snap.cells[jidx]
                            if (jcp == 0 || jcp == 32) break
                            val jattrs = snap.attrs[jidx]
                            if (jattrs != attrs) break
                            var jfg = if (snap.fg[jidx] == 0xFFFF_FFFF.toInt()) {
                                scheme.fgColor()
                            } else {
                                argb(snap.fg[jidx])
                            }
                            if (attrs and Attr.REVERSE != 0) {
                                val jbgRaw = snap.bg[jidx]
                                jfg = if (jbgRaw == 0xFFFF_FFFF.toInt()) {
                                    scheme.bgColor()
                                } else {
                                    argb(jbgRaw)
                                }
                            }
                            if (jfg != fg) break
                            runText.appendCodePoint(jcp)
                            j++
                        }
                        cellPaint.color = fg.toArgb()
                        cellPaint.isFakeBoldText = attrs and Attr.BOLD != 0
                        cellPaint.textSkewX = if (attrs and Attr.ITALIC != 0) -0.25f else 0f
                        cellPaint.isStrikeThruText = attrs and Attr.STRIKE != 0
                        cellPaint.isUnderlineText = attrs and Attr.UNDERLINE != 0
                        cellPaint.alpha = if (attrs and Attr.DIM != 0) 160 else 255
                        drawContext.canvas.nativeCanvas.drawText(
                            runText, 0, runText.length, c * w, baseline, cellPaint,
                        )
                        c = j
                    } else {
                        c++
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
}

/**
 * Extra keys in either orientation. Chips are at least 44dp on their short
 * side so the row is usable with a thumb (Material minimum touch target).
 */
@Composable
private fun ExtraKeys(
    controller: TerminalController,
    modifier: Modifier = Modifier,
    vertical: Boolean,
) {
    if (vertical) {
        Column(
            modifier
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 6.dp, vertical = 4.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            extraKeys.forEach { (label, bytes) -> ExtraKeyChip(controller, label, bytes) }
        }
    } else {
        Row(
            modifier
                .fillMaxWidth()
                .horizontalScroll(rememberScrollState())
                .padding(vertical = 6.dp, horizontal = 4.dp),
            horizontalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            extraKeys.forEach { (label, bytes) -> ExtraKeyChip(controller, label, bytes) }
        }
    }
}

@Composable
private fun ExtraKeyChip(controller: TerminalController, label: String, bytes: ByteArray) {
    Surface(
        color = MaterialTheme.colorScheme.surfaceVariant,
        onClick = { controller.send(bytes) },
    ) {
        Box(
            Modifier
                .heightIn(min = 44.dp)
                .widthIn(min = 44.dp)
                .padding(horizontal = 12.dp, vertical = 8.dp),
            contentAlignment = Alignment.Center,
        ) {
            Text(label, fontSize = 13.sp)
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
