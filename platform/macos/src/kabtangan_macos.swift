/*
 * kabtangan_macos.swift
 *
 * macOS platform layer for Kabtangan Keyboard.
 *
 * Uses the Input Method Kit (IMK) framework to register as a system IME.
 * The Rust kabtangan-core is linked via a C bridging header.
 *
 * Performance targets:
 * - Startup: < 150 ms
 * - Key handling: < 10 ms
 * - Prediction: < 30 ms
 *
 * Minimum deployment target: macOS 13 (Ventura)
 */

import Foundation
import InputMethodKit

// ─── Engine bridge (C FFI via bridging header) ────────────────────────────

/// Wraps the Rust kabtangan-core handle with RAII lifecycle management.
final class KabtanganEngine {

    private var handle: OpaquePointer?

    init(dbPath: String) throws {
        handle = kabtangan_init(dbPath)
        guard handle != nil else {
            throw KabtanganError.initFailed
        }
    }

    deinit {
        if let h = handle {
            kabtangan_destroy(h)
        }
    }

    func transliterate(_ latin: String) -> String {
        guard let h = handle else { return latin }
        let result = kabtangan_transliterate(h, latin)
        defer { kabtangan_free_string(result) }
        return String(cString: result!)
    }

    func predict(_ partial: String) -> [String] {
        guard let h = handle else { return [] }
        var count: Int32 = 0
        let arr = kabtangan_predict(h, partial, &count)
        defer { kabtangan_free_string_array(arr, count) }
        return (0..<Int(count)).map { String(cString: arr![$0]!) }
    }

    func isKnownWord(_ word: String) -> Bool {
        guard let h = handle else { return false }
        return kabtangan_is_known_word(h, word) != 0
    }

    func recordUsage(_ word: String) {
        guard let h = handle else { return }
        kabtangan_record_usage(h, word)
    }
}

enum KabtanganError: Error {
    case initFailed
}

// ─── IMK Controller ───────────────────────────────────────────────────────

/// The IMK input controller — handles key events and composes text.
@objc(KabtanganController)
class KabtanganController: IMKInputController {

    private var engine: KabtanganEngine?
    private var composingBuffer = ""

    override init!(server: IMKServer!, delegate: Any!, client sender: Any!) {
        super.init(server: server, delegate: delegate, client: sender)
        let dbPath = (NSHomeDirectory() as NSString)
            .appendingPathComponent("Library/Application Support/Kabtangan/kabtangan.db")
        engine = try? KabtanganEngine(dbPath: dbPath)
    }

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard event.type == .keyDown else { return false }
        // TODO: Implement full key handling, composing, and candidate window
        return false
    }

    override func commitComposition(_ sender: Any!) {
        if !composingBuffer.isEmpty {
            engine?.recordUsage(composingBuffer)
            composingBuffer = ""
        }
        super.commitComposition(sender)
    }
}
