//! # kabtangan-core
//!
//! Core engine for the Kabtangan Keyboard.
//!
//! Responsibilities:
//! - Transliteration (Latin → Sulat Sūg)
//! - Dictionary management (SQLite-backed)
//! - Word prediction
//! - Spell checking
//! - Grammar suggestions
//! - Alphabet-correct sorting for Bahasa Sūg
//!
//! This crate is platform-agnostic and shared across all targets:
//! Android (via JNI), Windows (via C FFI), Linux (IBus/Fcitx), macOS (via Swift FFI).

pub mod alphabet;
pub mod dictionary;
pub mod error;
pub mod grammar;
pub mod prediction;
pub mod spell;
pub mod storage;
pub mod transliteration;

/// Version of the core engine.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
