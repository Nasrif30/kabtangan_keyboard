# Kabtangan Keyboard

> The premier keyboard for the Tausug language. Kabtangan is a fast, offline, privacy-first input method featuring smart Latin to Sulat Sūg transliteration, dedicated Bahasa Sūg alphabet layouts, and real-time word prediction. Designed to seamlessly preserve cultural heritage through a premium typing experience.

**Bahasa Sūg · Sulat Sūg · Offline · Open Source**

---

## The Journey Begins: Early UI Prototype

Welcome to the Kabtangan Keyboard project! This repository currently houses our **early UI prototype**. 

We are at the very beginning of our journey. This initial build focuses purely on testing the premium design, smooth animations, and the dedicated Tausug dual-layout system. 

*Note: The number pad and the full Rust-based offline typing engine are not yet wired into this specific UI preview, but the foundation is being laid.*

---

## Previews

Here is a look at the Kabtangan Keyboard interface and our cultural themes:

<br>

*(Insert Screenshot 1 Here)*

<br>

*(Insert Screenshot 2 Here)*

<br>

*(Insert Screenshot 3 Here)*

<br>

*(Insert Screenshot 4 Here)*

<br>

---

## Design Philosophy & Themes

Kabtangan is built to be unapologetically Tausug while retaining the premium, fluid feel of modern mobile operating systems. 

**Dual Layouts:**
- **Pure Tausug:** Restricted to the exact 22 characters of the Bahasa Sūg alphabet, optimized for fast indigenous typing with dedicated `ng`, `ny`, and `'` keys.
- **Normal (QWERTY):** A standard layout for typing borrowed words, names, and emails without feeling limited.

**Cultural Themes:**
- **Light & Dark:** Clean, minimalist defaults.
- **Forest:** Inspired by deep forest greens with gold accents.
- **Traditional:** A rich gradient of crimson and maroon, honoring the royal colors of the *Pis Syabit*.

---

## Project Architecture

While the UI is currently in prototyping, the final system architecture is modular and cross-platform:

```text
kabtangan-keyboard/
├── desktop-prototype/           # React/Vite UI Prototype (Current Focus)
├── core/                        # Rust core engine (Transliteration, Prediction, Dict)
├── platform/                    # Native wrappers (Android JNI, Windows TSF, macOS)
└── data/                        # Static bundled dictionaries
```

### Privacy Guarantee
Kabtangan operates strictly offline.
-  No telemetry
-  No analytics
-  No keystroke logging
-  All data is local and owned by the user

---

## Running the UI Prototype

To test the current desktop UI prototype:

```bash
cd desktop-prototype
npm install
npm run dev
```

---

## License & Acknowledgements

**License:** Apache License 2.0

Built for the Tausug people and the preservation of Bahasa Sūg and Sulat Sūg.
