/**
 * kabtangan_windows.cpp
 *
 * Windows platform layer for the Kabtangan Keyboard.
 *
 * Implements a Text Services Framework (TSF) IME that hosts the Rust core
 * via the kabtangan_core C FFI.
 *
 * Performance targets:
 * - Startup: < 150 ms
 * - Key handling: < 10 ms
 * - Prediction: < 30 ms
 *
 * Build: MSVC v143 / CMake 3.28+
 */

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <msctf.h>
#include <string>
#include <vector>

// ─── C FFI declarations from kabtangan_core ─────────────────────────────────
extern "C" {
    typedef void* KabtanganHandle;

    KabtanganHandle kabtangan_init(const char* db_path);
    void            kabtangan_destroy(KabtanganHandle handle);
    const char*     kabtangan_transliterate(KabtanganHandle handle, const char* latin);
    const char**    kabtangan_predict(KabtanganHandle handle, const char* partial, int* count);
    int             kabtangan_is_known_word(KabtanganHandle handle, const char* word);
    void            kabtangan_record_usage(KabtanganHandle handle, const char* word);
    void            kabtangan_free_string(const char* s);
    void            kabtangan_free_string_array(const char** arr, int count);
}

// ─── Engine wrapper ─────────────────────────────────────────────────────────

class KabtanganEngine {
public:
    explicit KabtanganEngine(const std::string& db_path)
        : handle_(kabtangan_init(db_path.c_str())) {
        if (!handle_) {
            throw std::runtime_error("Failed to initialize Kabtangan engine");
        }
    }

    ~KabtanganEngine() {
        if (handle_) {
            kabtangan_destroy(handle_);
            handle_ = nullptr;
        }
    }

    // Non-copyable; unique ownership.
    KabtanganEngine(const KabtanganEngine&) = delete;
    KabtanganEngine& operator=(const KabtanganEngine&) = delete;

    std::string transliterate(const std::string& latin) const {
        const char* result = kabtangan_transliterate(handle_, latin.c_str());
        std::string output(result);
        kabtangan_free_string(result);
        return output;
    }

    std::vector<std::string> predict(const std::string& partial) const {
        int count = 0;
        const char** arr = kabtangan_predict(handle_, partial.c_str(), &count);
        std::vector<std::string> predictions;
        for (int i = 0; i < count; ++i) {
            predictions.emplace_back(arr[i]);
        }
        kabtangan_free_string_array(arr, count);
        return predictions;
    }

    bool is_known_word(const std::string& word) const {
        return kabtangan_is_known_word(handle_, word.c_str()) != 0;
    }

    void record_usage(const std::string& word) const {
        kabtangan_record_usage(handle_, word.c_str());
    }

private:
    KabtanganHandle handle_;
};

// ─── DLL entry point ─────────────────────────────────────────────────────────

BOOL APIENTRY DllMain(HMODULE hModule, DWORD ul_reason_for_call, LPVOID) {
    switch (ul_reason_for_call) {
        case DLL_PROCESS_ATTACH:
            DisableThreadLibraryCalls(hModule);
            break;
        case DLL_PROCESS_DETACH:
            break;
    }
    return TRUE;
}

// TODO: Implement ITfTextInputProcessor, ITfKeyEventSink, ITfCompositionSink
// for the full TSF IME integration.
