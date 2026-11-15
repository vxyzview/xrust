package com.xrust.app.ui.theme

import androidx.compose.ui.graphics.Color

/** Terminal color schemes mirrored from xrust-core/src/color.rs. */
data class TerminalScheme(
    val name: String,
    val isDark: Boolean,
    val foreground: Int,
    val background: Int,
    val cursor: Int,
    val selection: Int,
    val palette: IntArray, // ANSI 0..15
) {
    fun fgColor(): Color = Color(foreground or 0xFF000000u.toInt())
    fun bgColor(): Color = Color(background or 0xFF000000u.toInt())
    fun cursorColor(): Color = Color(cursor or 0xFF000000u.toInt())
    fun selectionColor(): Color = Color(selection or 0xFF000000u.toInt())
    fun indexed(i: Int): Color = when {
        i in 0..15 -> Color(palette[i] or 0xFF000000u.toInt())
        i in 16..231 -> {
            val v = i - 16
            val f = fun(x: Int) = if (x == 0) 0 else 55 + x * 40
            val r = f(v / 36); val g = f((v % 36) / 6); val b = f(v % 6)
            Color(0xFF000000u.toInt() or (r shl 16) or (g shl 8) or b)
        }
        else -> {
            val g = 8 + (i - 232) * 10
            Color(0xFF000000u.toInt() or (g shl 16) or (g shl 8) or g)
        }
    }
}

private fun scheme(
    name: String,
    isDark: Boolean,
    fg: Int,
    bg: Int,
    cursor: Int,
    selection: Int,
    palette: IntArray,
) = TerminalScheme(name, isDark, fg, bg, cursor, selection, palette)

val ALPINE_DARK_TERM = scheme(
    "Alpine Dark", true, 0xE8E8E8, 0x0E1218, 0x4FC3F7, 0x27476E,
    intArrayOf(
        0x2E3440, 0xD9555F, 0x88C98B, 0xD9B36C, 0x6E9FD4, 0xB57FC6, 0x63C6C1, 0xC8CDD4,
        0x5C6672, 0xEC7179, 0xA5D9A8, 0xE6C98C, 0x8FBCE8, 0xC9A0D8, 0x86DBD6, 0xECEFF3,
    ),
)

val ALPINE_LIGHT_TERM = scheme(
    "Alpine Light", false, 0x1B2430, 0xF4F6F9, 0x0B57D0, 0xC7DBF7,
    intArrayOf(
        0x3A4252, 0xC23B4E, 0x3E7A46, 0x9A6B16, 0x2E5F94, 0x7C4D8F, 0x2E7E78, 0x6B7280,
        0x525B66, 0xD95A6C, 0x3F8A48, 0xA8760F, 0x3E76C4, 0x955F9E, 0x27918A, 0x37414D,
    ),
)

val AMOLED_TERM = scheme(
    "AMOLED Pure Black", true, 0xE4E4E4, 0x000000, 0xFFFFFF, 0x1E6F50,
    intArrayOf(
        0x121212, 0xFF6E6E, 0x7BE28A, 0xE7C878, 0x82AAFF, 0xC792EA, 0x89DDFF, 0xBDBDBD,
        0x4A4A4A, 0xFF8B8B, 0xA5E8B0, 0xF0DCA0, 0xA6C8FF, 0xDCAFF2, 0xA7E8EE, 0xEAEAEA,
    ),
)

val ROSE_ASH_TERM = scheme(
    "Rose Ash", true, 0xEDE6E4, 0x191417, 0xEB8CAC, 0x4A2C3A,
    intArrayOf(
        0x26232B, 0xE8697D, 0x83C99E, 0xD9A66C, 0x8CA6E0, 0xC69ACF, 0x7FC9C4, 0xD8D3D1,
        0x5D5760, 0xF08596, 0xA4DDB9, 0xE7C18E, 0xA9BFEC, 0xD9B4E5, 0x9CDAD6, 0xEFEAEE,
    ),
)

val DRACULA_TERM = scheme(
    "Dracula", true, 0xF8F8F2, 0x282A36, 0xF8F8F0, 0x44475A,
    intArrayOf(
        0x21222C, 0xFF5555, 0x50FA7B, 0xF1FA8C, 0xBD93F9, 0xFF79C6, 0x8BE9FD, 0xF8F8F2,
        0x6272A4, 0xFF6E6E, 0x69FF94, 0xFFFFA5, 0xD6ACFF, 0xFF92DF, 0xA4FFFF, 0xFFFFFF,
    ),
)

val NORD_TERM = scheme(
    "Nord", true, 0xD8DEE9, 0x2E3440, 0x88C0D0, 0x434C5E,
    intArrayOf(
        0x3B4252, 0xBF616A, 0xA3BE8C, 0xEBCB8B, 0x81A1C1, 0xB48EAD, 0x88C0D0, 0xE5E9F0,
        0x4C566A, 0xD08770, 0xB48EAD, 0xEBCB8B, 0x81A1C1, 0xB48EAD, 0x8FBCBB, 0xECEFF4,
    ),
)

val GRUVBOX_DARK_TERM = scheme(
    "Gruvbox Dark", true, 0xEBDBB2, 0x282828, 0xFABD2F, 0x504945,
    intArrayOf(
        0x282828, 0xCC241D, 0x98971A, 0xD79921, 0x458588, 0xB16286, 0x689D6A, 0xA89984,
        0x928374, 0xFB4934, 0xB8BB26, 0xFABD2F, 0x83A598, 0xD3869B, 0x8EC07C, 0xEBDBB2,
    ),
)

val GRUVBOX_LIGHT_TERM = scheme(
    "Gruvbox Light", false, 0x3C3836, 0xFBF1C7, 0xB57614, 0xDDD6C0,
    intArrayOf(
        0xFBF1C7, 0x9D0006, 0x79740E, 0xB57614, 0x076678, 0x8F3F71, 0x427B58, 0x7C6F64,
        0x928374, 0xAF3A03, 0x879100, 0xD65D0E, 0x458588, 0xB0548D, 0x4899A0, 0x3C3836,
    ),
)

val SOLARIZED_DARK_TERM = scheme(
    "Solarized Dark", true, 0x93A1A1, 0x002B36, 0x268BD2, 0x073642,
    intArrayOf(
        0x073642, 0xDC322F, 0x859900, 0xB58900, 0x268BD2, 0xD33682, 0x2AA198, 0xEEE8D5,
        0x586E75, 0xCB4B16, 0x859900, 0xB58900, 0x268BD2, 0xD33682, 0x2AA198, 0xFDF6E3,
    ),
)

val SOLARIZED_LIGHT_TERM = scheme(
    "Solarized Light", false, 0x52676F, 0xFDF6E3, 0x268BD2, 0xEEE8D5,
    intArrayOf(
        0xFDF6E3, 0xDC322F, 0x859900, 0xB58900, 0x268BD2, 0xD33682, 0x2AA198, 0x93A1A1,
        0x839496, 0xCB4B16, 0x859900, 0xB58900, 0x268BD2, 0xD33682, 0x2AA198, 0x002B36,
    ),
)

val ONE_DARK_TERM = scheme(
    "One Dark", true, 0xDCDFE4, 0x282C34, 0x61AFEF, 0x3E4451,
    intArrayOf(
        0x282C34, 0xE06C75, 0x98C379, 0xE5C07B, 0x61AFEF, 0xC678DD, 0x56B6C2, 0xDCDFE4,
        0x5C6370, 0xE06C75, 0x98C379, 0xE5C07B, 0x61AFEF, 0xC678DD, 0x56B6C2, 0xFFFFFF,
    ),
)

val TOKYO_NIGHT_TERM = scheme(
    "Tokyo Night", true, 0xC0CAF5, 0x1A1B26, 0x7AA2F7, 0x33467C,
    intArrayOf(
        0x1D202F, 0xF7768E, 0x9ECE6A, 0xE0AF68, 0x7AA2F7, 0xBB9AF7, 0x7DCFFF, 0xA9B1D6,
        0x414868, 0xFF7A93, 0xB9F27C, 0xFF9E64, 0x7DA6FF, 0xBB9AF7, 0x0DB9D7, 0xC0CAF5,
    ),
)

val ALL_TERMINAL_SCHEMES = listOf(
    ALPINE_DARK_TERM, ALPINE_LIGHT_TERM, AMOLED_TERM, ROSE_ASH_TERM,
    DRACULA_TERM, NORD_TERM, GRUVBOX_DARK_TERM, GRUVBOX_LIGHT_TERM,
    SOLARIZED_DARK_TERM, SOLARIZED_LIGHT_TERM, ONE_DARK_TERM, TOKYO_NIGHT_TERM,
)

fun schemeByName(name: String): TerminalScheme =
    ALL_TERMINAL_SCHEMES.firstOrNull { it.name == name } ?: ALPINE_DARK_TERM

/** Attribute bit flags — must match xrust-core/src/grid.rs. */
object Attr {
    const val BOLD = 1 shl 0
    const val ITALIC = 1 shl 1
    const val UNDERLINE = 1 shl 2
    const val STRIKE = 1 shl 3
    const val REVERSE = 1 shl 4
    const val INVISIBLE = 1 shl 5
    const val DIM = 1 shl 6
    const val WIDE = 1 shl 7
    const val WIDE_CONT = 1 shl 8
    const val BLINK = 1 shl 9
}
