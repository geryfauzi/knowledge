# Knowledge

A lightweight, distraction-free markdown knowledge base with Google Drive sync, designed with the **Zed Code Editor** aesthetic.

## Tech Stack
- **Frontend**: Svelte 5 (Runes) + Tailwind CSS + CodeMirror 6 + TipTap
- **Desktop Core**: Tauri v2 (Rust)
- **Styling**: Zed Dark inspired matte palette (`#18181b` & `#121215`), custom typography and line gutter.

> [!NOTE]
> Panduan status proyek dan serah-terima untuk pengembang / AI agent dapat dibaca lengkap di [PROJECT_STATUS_HANDOVER.md](file:///d:/Meja%20Kerja/Project/knowledge/PROJECT_STATUS_HANDOVER.md).


## Quick Start (Development)

### Web Dev Mode (Instant Live Reload)
```bash
npm run dev
```
Buka browser di `http://localhost:1420` untuk menguji editor, tree view folder, tab manager, dan shortcut keyboard.

### Desktop App Mode (Tauri v2)
Pastikan toolchain Rust terpasang di sistem (`rustup`), lalu jalankan:
```bash
npm run tauri dev
```

### Build Production Binary
```bash
npm run tauri build
```
Hasil binary `.exe` (Windows) atau binary Linux akan otomatis dibuat di folder `src-tauri/target/release/bundle/`.

## Key Shortcuts
- `Ctrl + N`: Buat catatan baru
- `Ctrl + B`: Buka/Tutup panel proyek (Sidebar)
- `Ctrl + S`: Simpan perubahan
- `Enter` saat membuat file/folder: Konfirmasi pembuatan
- `Escape`: Batalkan pembuatan
