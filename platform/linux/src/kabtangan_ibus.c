/*
 * kabtangan_ibus.c
 *
 * Linux IBus platform layer for Kabtangan Keyboard.
 *
 * Registers as an IBus engine that provides Bahasa Sūg and Sulat Sūg input.
 * The Rust kabtangan-core is linked via C FFI.
 *
 * Build: gcc / meson + pkg-config ibus-1.0
 * Minimum: IBus 1.5, GLib 2.70
 */

#include <ibus.h>
#include <glib.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

/* ── C FFI declarations from kabtangan_core ────────────────────────────── */
typedef void* KabtanganHandle;

extern KabtanganHandle kabtangan_init(const char* db_path);
extern void            kabtangan_destroy(KabtanganHandle handle);
extern const char*     kabtangan_transliterate(KabtanganHandle handle, const char* latin);
extern const char**    kabtangan_predict(KabtanganHandle handle, const char* partial, int* count);
extern int             kabtangan_is_known_word(KabtanganHandle handle, const char* word);
extern void            kabtangan_record_usage(KabtanganHandle handle, const char* word);
extern void            kabtangan_free_string(const char* s);
extern void            kabtangan_free_string_array(const char** arr, int count);

/* ── Engine state ──────────────────────────────────────────────────────── */
typedef struct {
    IBusEngine parent;
    KabtanganHandle core;
    gchar* composing_buffer;
    gboolean sulat_mode; /* FALSE = Latin, TRUE = Sulat Sūg */
} KabtanganEngine;

typedef struct {
    IBusEngineClass parent_class;
} KabtanganEngineClass;

/* GType boilerplate */
G_DEFINE_TYPE(KabtanganEngine, kabtangan_engine, IBUS_TYPE_ENGINE)

/* ── Lifecycle ─────────────────────────────────────────────────────────── */

static void kabtangan_engine_init(KabtanganEngine* engine) {
    const char* home = g_get_home_dir();
    gchar* db_path = g_build_filename(
        home, ".local", "share", "kabtangan", "kabtangan.db", NULL
    );

    engine->core = kabtangan_init(db_path);
    engine->composing_buffer = g_strdup("");
    engine->sulat_mode = FALSE;

    if (!engine->core) {
        g_warning("kabtangan: failed to initialize core engine");
    }

    g_free(db_path);
}

static void kabtangan_engine_finalize(GObject* object) {
    KabtanganEngine* engine = (KabtanganEngine*) object;
    if (engine->core) {
        kabtangan_destroy(engine->core);
        engine->core = NULL;
    }
    g_free(engine->composing_buffer);
    G_OBJECT_CLASS(kabtangan_engine_parent_class)->finalize(object);
}

/* ── Key handling ──────────────────────────────────────────────────────── */

static gboolean kabtangan_engine_process_key_event(
    IBusEngine* ibus_engine,
    guint       keyval,
    guint       keycode,
    guint       modifiers)
{
    KabtanganEngine* engine = (KabtanganEngine*) ibus_engine;

    /* Ignore key-release events */
    if (modifiers & IBUS_RELEASE_MASK) return FALSE;

    /* Backspace: remove last character from composing buffer */
    if (keyval == IBUS_BackSpace) {
        gsize len = strlen(engine->composing_buffer);
        if (len > 0) {
            engine->composing_buffer[len - 1] = '\0';
            ibus_engine_update_preedit_text(
                ibus_engine,
                ibus_text_new_from_string(engine->composing_buffer),
                len - 1,
                len > 1
            );
            return TRUE;
        }
        return FALSE;
    }

    /* Return / space: commit */
    if (keyval == IBUS_Return || keyval == IBUS_space) {
        if (strlen(engine->composing_buffer) > 0) {
            kabtangan_record_usage(engine->core, engine->composing_buffer);
            ibus_engine_commit_text(
                ibus_engine,
                ibus_text_new_from_string(engine->composing_buffer)
            );
            g_free(engine->composing_buffer);
            engine->composing_buffer = g_strdup("");
            ibus_engine_hide_preedit_text(ibus_engine);
        }
        if (keyval == IBUS_space) {
            ibus_engine_commit_text(
                ibus_engine,
                ibus_text_new_from_string(" ")
            );
        }
        return TRUE;
    }

    /* Only handle printable ASCII for now */
    if (keyval < 0x20 || keyval > 0x7e) return FALSE;

    /* Append character */
    gchar new_char[2] = { (gchar)keyval, '\0' };
    gchar* new_buf = g_strconcat(engine->composing_buffer, new_char, NULL);
    g_free(engine->composing_buffer);
    engine->composing_buffer = new_buf;

    /* Show preedit */
    ibus_engine_update_preedit_text(
        ibus_engine,
        ibus_text_new_from_string(engine->composing_buffer),
        strlen(engine->composing_buffer),
        TRUE
    );

    /* TODO: Update candidate lookup table from kabtangan_predict() */

    return TRUE;
}

/* ── Class initialization ──────────────────────────────────────────────── */

static void kabtangan_engine_class_init(KabtanganEngineClass* klass) {
    GObjectClass* object_class = G_OBJECT_CLASS(klass);
    object_class->finalize = kabtangan_engine_finalize;

    IBusEngineClass* engine_class = IBUS_ENGINE_CLASS(klass);
    engine_class->process_key_event = kabtangan_engine_process_key_event;
}

/* ── Main ──────────────────────────────────────────────────────────────── */

static void on_disconnected(IBusBus* bus, gpointer data) {
    g_main_loop_quit((GMainLoop*) data);
}

int main(int argc, char* argv[]) {
    ibus_init();

    IBusBus* bus = ibus_bus_new();
    if (!ibus_bus_is_connected(bus)) {
        g_error("kabtangan: cannot connect to IBus daemon");
        return 1;
    }

    GMainLoop* loop = g_main_loop_new(NULL, FALSE);
    g_signal_connect(bus, "disconnected", G_CALLBACK(on_disconnected), loop);

    IBusFactory* factory = ibus_factory_new(ibus_bus_get_connection(bus));
    ibus_factory_add_engine(factory, "kabtangan", G_TYPE_FROM_CLASS(kabtangan_engine_get_type()));

    if (!ibus_bus_request_name(bus, "org.freedesktop.IBus.Kabtangan", 0)) {
        g_error("kabtangan: failed to request bus name");
        return 1;
    }

    ibus_bus_register_component(bus,
        ibus_component_new(
            "org.freedesktop.IBus.Kabtangan",
            "Kabtangan Keyboard",
            "0.1.0",
            "Apache-2.0",
            "Kabtangan Contributors",
            "https://github.com/kabtangan/kabtangan-keyboard",
            PKGDATADIR "/kabtangan-ibus-daemon",
            "kabtangan"
        )
    );

    g_main_loop_run(loop);
    return 0;
}
