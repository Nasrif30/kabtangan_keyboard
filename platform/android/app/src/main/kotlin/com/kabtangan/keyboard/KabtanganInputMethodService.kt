package com.kabtangan.keyboard

import android.inputmethodservice.InputMethodService
import android.view.View
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import com.kabtangan.keyboard.core.KabtanganEngine
import com.kabtangan.keyboard.ui.KeyboardView

/**
 * Kabtangan Keyboard IME Service.
 *
 * Entry point for the Android Input Method. Delegates all language-processing
 * to [KabtanganEngine] (JNI bridge to the Rust core) and rendering to [KeyboardView].
 *
 * Performance targets:
 * - Startup: < 150 ms
 * - Key event handling: < 10 ms
 * - Prediction update: < 30 ms
 */
class KabtanganInputMethodService : InputMethodService() {

    private lateinit var engine: KabtanganEngine
    private lateinit var keyboardView: KeyboardView

    override fun onCreate() {
        super.onCreate()
        // Initialize the Rust core engine via JNI.
        // The database is stored in the app's private files directory.
        val dbPath = "${filesDir.absolutePath}/kabtangan.db"
        engine = KabtanganEngine(dbPath)
    }

    override fun onCreateInputView(): View {
        keyboardView = KeyboardView(this, engine)
        return keyboardView
    }

    override fun onStartInput(attribute: EditorInfo?, restarting: Boolean) {
        super.onStartInput(attribute, restarting)
        keyboardView.reset()
    }

    override fun onFinishInput() {
        super.onFinishInput()
        keyboardView.clearComposing()
    }

    override fun onDestroy() {
        super.onDestroy()
        engine.close()
    }
}
