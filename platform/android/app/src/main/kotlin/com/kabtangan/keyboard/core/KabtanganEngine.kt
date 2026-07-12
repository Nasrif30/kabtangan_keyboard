package com.kabtangan.keyboard.core

/**
 * JNI bridge to the Rust kabtangan-core library.
 *
 * All heavy lifting (transliteration, dictionary lookup, prediction,
 * spell checking) happens in the Rust layer. This class exposes a
 * clean Kotlin API to the rest of the Android codebase.
 *
 * Thread safety: All native methods must be called on a single thread
 * or synchronized externally. The Rust engine is NOT thread-safe by default.
 */
class KabtanganEngine(private val dbPath: String) : AutoCloseable {

    private var nativeHandle: Long = 0L

    init {
        nativeHandle = nativeInit(dbPath)
        check(nativeHandle != 0L) { "Failed to initialize KabtanganEngine" }
    }

    // ─── Writing Mode ─────────────────────────────────────────────────────

    /** Switch between Bahasa Sūg (Latin) and Sulat Sūg. */
    fun setWritingMode(mode: WritingMode) = nativeSetWritingMode(nativeHandle, mode.ordinal)

    fun getWritingMode(): WritingMode = WritingMode.entries[nativeGetWritingMode(nativeHandle)]

    // ─── Transliteration ──────────────────────────────────────────────────

    /** Convert a Latin string to Sulat Sūg Unicode. */
    fun transliterate(latin: String): String = nativeTransliterate(nativeHandle, latin)

    // ─── Prediction ───────────────────────────────────────────────────────

    /**
     * Get word predictions for a partial input.
     * Returns up to 5 candidates ranked by relevance.
     */
    fun predict(partial: String): Array<String> = nativePredict(nativeHandle, partial)

    // ─── Spell Check ──────────────────────────────────────────────────────

    /** Returns true if the word is in the dictionary. */
    fun isKnownWord(word: String): Boolean = nativeIsKnownWord(nativeHandle, word)

    /** Returns suggested corrections for an unknown word. */
    fun spellSuggest(word: String): Array<String> = nativeSpellSuggest(nativeHandle, word)

    // ─── Personal Dictionary ──────────────────────────────────────────────

    fun addPersonalWord(word: String) = nativeAddPersonalWord(nativeHandle, word)
    fun removePersonalWord(word: String) = nativeRemovePersonalWord(nativeHandle, word)

    // ─── Usage Tracking ───────────────────────────────────────────────────

    /** Call after a word is committed to improve future predictions. */
    fun recordWordUsage(word: String) = nativeRecordWordUsage(nativeHandle, word)

    // ─── Lifecycle ────────────────────────────────────────────────────────

    override fun close() {
        if (nativeHandle != 0L) {
            nativeDestroy(nativeHandle)
            nativeHandle = 0L
        }
    }

    // ─── Native Declarations ──────────────────────────────────────────────

    private external fun nativeInit(dbPath: String): Long
    private external fun nativeDestroy(handle: Long)
    private external fun nativeSetWritingMode(handle: Long, mode: Int)
    private external fun nativeGetWritingMode(handle: Long): Int
    private external fun nativeTransliterate(handle: Long, latin: String): String
    private external fun nativePredict(handle: Long, partial: String): Array<String>
    private external fun nativeIsKnownWord(handle: Long, word: String): Boolean
    private external fun nativeSpellSuggest(handle: Long, word: String): Array<String>
    private external fun nativeAddPersonalWord(handle: Long, word: String)
    private external fun nativeRemovePersonalWord(handle: Long, word: String)
    private external fun nativeRecordWordUsage(handle: Long, word: String)

    companion object {
        init {
            System.loadLibrary("kabtangan_core")
        }
    }
}

/** Available writing modes. */
enum class WritingMode {
    /** Bahasa Sūg using the Latin alphabet. */
    LATIN,
    /** Sulat Sūg script. */
    SULAT_SUG,
}
