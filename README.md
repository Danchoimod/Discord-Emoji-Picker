# 🚀 Discord Emoji Picker

A lightweight, blazing-fast desktop widget written in **Rust** powered by the **egui / eframe** GUI framework. It runs as an unobtrusive, always-on-top overlay that lets you select custom emojis/GIFs and **instantly auto-pastes them into Discord** (or any active chat window) with a single click—no Discord Nitro required!
<img width="1109" height="989" alt="image" src="https://github.com/user-attachments/assets/fde9d78a-802c-45e6-ab25-6bba170eb6dc" />

---

## ✨ Features

* ⚡ **Zero-Friction Auto-Paste:**
  * **Event-driven focus tracking:** Uses Win32 `SetWinEventHook` to detect and track the last focused window (e.g., Discord) with **0% background CPU polling**.
  * **Seamless paste simulation:** Upon clicking an emoji, the app copies the image link, forces focus back to your previous application (`AttachThreadInput` + `SetForegroundWindow`), and dispatches a synthetic `Ctrl + V` via `SendInput`.
* 🎨 **Discord-Themed Dark UI:**
  * Frameless, modern borderless window with smooth rounded corners.
  * **Always on Top** overlay support.
  * Custom title bar with drag-to-move window handling and minimal window controls.
  * Multi-category tab navigation (pre-configured with `General` and `Omori 2`).
* 🎬 **Smart GIF Rendering & Resource Saving:**
  * Emojis are displayed as static thumbnail images by default to eliminate rendering overhead.
  * **GIF animations activate only when hovered**, keeping GPU and CPU usage exceptionally low.
* 🧹 **Aggressive Memory Optimization (RAM Trimming):**
  * When minimized, the application calls `ctx.forget_all_images()` to dump all GPU/CPU image textures and invokes `SetProcessWorkingSetSize` to purge the working set RAM footprint.

---

## 🏗️ Tech Stack

* **Language:** [Rust](https://www.rust-lang.org/)
* **GUI Framework:** [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) / [egui](https://github.com/emilk/egui) with `egui_extras` image loaders
* **Clipboard:** [arboard](https://crates.io/crates/arboard)
* **Windows API:** `windows-sys` (`Win32::UI::WindowsAndMessaging`, `Win32::UI::Input::KeyboardAndMouse`, `Win32::UI::Accessibility`)

---

## 📂 Project Structure

```text
Discord-Emoji-Picker/
├── Cargo.toml          # Rust package configuration & dependencies
├── Cargo.lock          # Dependency lockfile
├── src/
│   ├── main.rs         # GUI lifecycle, Win32 hooks, focus restoration & paste logic
│   └── emojis.rs       # Static & animated emoji URL catalogues
└── README.md
```

---

## 🚀 Getting Started

### Prerequisites

* Operating System: **Windows 10 / 11** (relies on Win32 API calls).
* [Rust & Cargo](https://rustup.rs/) (latest stable toolchain recommended).

### 1. Clone the repository

```bash
git clone https://github.com/Danchoimod/Discord-Emoji-Picker.git
cd Discord-Emoji-Picker
```

### 2. Run in development mode

```bash
cargo run
```

### 3. Build optimized release binary

Compile an optimized standalone executable without the terminal console attached:

```bash
cargo build --release
```

The resulting binary will be placed at:
```text
target/release/discord_emoji_picker.exe
```

---

## 🎯 How to Use

1. Click into your Discord chat input field.
2. Bring up the **Discord Emoji Picker** window.
3. Switch tabs to find your desired category.
4. Hover over an emoji to preview its animation.
5. **Left-click on any emoji:** The picker copies the link, focuses back to Discord, and automatically sends `Ctrl + V` into the message box!

---

## 🛠️ Customizing Emojis

You can add or update emoji packs by modifying the string slices in `src/emojis.rs`:

```rust
pub const DEFAULT_EMOJIS: &[&str] = &[
    "https://cdn.discordapp.com/emojis/123456789012345678.webp?size=96&animated=true",
    "https://cdn.discordapp.com/emojis/987654321098765432.png?size=96",
    // Add your custom Discord emoji/GIF URLs here...
];
```

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
