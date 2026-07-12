package com.kabtangan.keyboard.ui

import android.content.Context
import android.util.AttributeSet
import android.view.LayoutInflater
import android.widget.LinearLayout
import com.kabtangan.keyboard.core.KabtanganEngine

/**
 * The keyboard view component.
 *
 * Renders:
 * - Prediction bar (top)
 * - Key grid (middle)
 * - Action row: language switch / space / enter / settings (bottom)
 *
 * All key events are handled in this class and dispatched to [KabtanganEngine].
 * Typing must never lag: key event → input connection commit in < 10 ms.
 *
 * Theming is driven by the active [KeyboardTheme]; switching themes triggers
 * a fade transition (no layout re-inflation required).
 */
class KeyboardView @JvmOverloads constructor(
    context: Context,
    private val engine: KabtanganEngine,
    attrs: AttributeSet? = null,
) : LinearLayout(context, attrs) {

    private val composingBuffer = StringBuilder()

    init {
        orientation = VERTICAL
        // TODO: Inflate keyboard_view.xml layout here
        // inflateLayout()
    }

    /** Reset state for a new input field. */
    fun reset() {
        composingBuffer.clear()
        // TODO: Clear prediction bar
    }

    /** Clear the composing text when input ends. */
    fun clearComposing() {
        composingBuffer.clear()
    }

    /**
     * Handle a key press.
     * This must complete in < 10 ms.
     */
    fun onKeyPressed(key: String) {
        when (key) {
            BACKSPACE -> handleBackspace()
            ENTER -> handleEnter()
            SPACE -> handleSpace()
            LANGUAGE_SWITCH -> handleLanguageSwitch()
            else -> handleCharacter(key)
        }
    }

    private fun handleCharacter(char: String) {
        composingBuffer.append(char)
        // TODO: Update prediction bar via engine.predict(composingBuffer.toString())
    }

    private fun handleBackspace() {
        if (composingBuffer.isNotEmpty()) {
            composingBuffer.deleteCharAt(composingBuffer.length - 1)
        }
        // TODO: Send backspace to InputConnection
    }

    private fun handleEnter() {
        commitComposing()
        // TODO: Send enter to InputConnection
    }

    private fun handleSpace() {
        commitComposing()
        // TODO: Send space to InputConnection
    }

    private fun handleLanguageSwitch() {
        val current = engine.getWritingMode()
        val next = when (current) {
            com.kabtangan.keyboard.core.WritingMode.LATIN -> com.kabtangan.keyboard.core.WritingMode.SULAT_SUG
            com.kabtangan.keyboard.core.WritingMode.SULAT_SUG -> com.kabtangan.keyboard.core.WritingMode.LATIN
        }
        engine.setWritingMode(next)
        // TODO: Animate key labels to reflect new mode
    }

    private fun commitComposing() {
        if (composingBuffer.isNotEmpty()) {
            engine.recordWordUsage(composingBuffer.toString())
            composingBuffer.clear()
        }
    }

    companion object {
        private const val BACKSPACE = "⌫"
        private const val ENTER = "⏎"
        private const val SPACE = " "
        private const val LANGUAGE_SWITCH = "🌐"
    }
}
