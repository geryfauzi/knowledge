# PROJECT HANDOVER & STATUS SPECIFICATION: "KNOWLEDGE"

Dokumen ini disusun sebagai panduan serah-terima (_handover document_) resmi untuk developer maupun AI agent berikutnya agar dapat langsung memahami konteks arsitektur, preferensi pengguna, pekerjaan yang telah diselesaikan, serta langkah berikutnya yang harus dikerjakan tanpa kehilangan arah.

---

## 1. Ringkasan Proyek & Identitas

- **Nama Aplikasi**: **Knowledge**
- **Tujuan**: Aplikasi pencatatan pengetahuan personal (_Personal Knowledge Management / PKM_) lintas platform (Windows & Linux) yang terinspirasi dari Obsidian, namun menggunakan sistem desain **Zed Code Editor** dan memiliki fitur sinkronisasi dua arah ke **Google Drive** secara native.
- **Stack Teknologi Utama**:
  - **Desktop Backend**: Tauri v2 (Rust)
  - **Frontend**: Svelte 5 (Runes mode: `$state`, `$derived`, `$effect`, `$props`)
  - **Styling**: Tailwind CSS + Custom Zed Dark Palette
  - **Base Components**: shadcn UI port untuk Svelte (`src/lib/components/ui/`)
  - **Editor Engine**:
    - **Pure Markdown Mode**: CodeMirror 6 (`@codemirror/*`)
    - **WYSIWYG Mode**: TipTap (`@tiptap/core`, `@tiptap/starter-kit`, `@tiptap/extension-underline`, `@tiptap/extension-text-align`)
    - **Bidirectional Parser**: `marked` (Markdown $\rightarrow$ HTML sinkron) dan `turndown` (HTML $\rightarrow$ Markdown)
  - **Penyimpanan**: File fisik `.md` asli di hard disk pengguna (Tauri fs plugin & File System Access API). **SAMA SEKALI TIDAK MENGGUNAKAN LOCALSTORAGE.**

---

## 2. Aturan Mutlak & Preferensi Pengguna (Non-Negotiable)

1. **Design System Zed Code Editor (Bukan Obsidian)**:
   - Menggunakan palet matte dark charcoal (`#18181b` dan `#121215`).
   - Border crisp 1px (`#27272a`).
   - Aksen warna cyan/sky halus (`#38bdf8`).
   - Tipografi monospace untuk status, breadcrumbs, dan metadata (`JetBrains Mono`, `Fira Code`, atau monospace fallback).
2. **Komponen Berbasis shadcn**:
   - Komponen UI dasar diletakkan di `src/lib/components/ui/` agar mudah diedit langsung.
3. **Dilarang Menggunakan Browser Alert / Prompt**:
   - **Tidak boleh** memanggil `window.alert()` atau `window.prompt()`.
   - Semua dialog, konfirmasi penghapusan, unsaved changes, dan pembuatan note/folder harus menggunakan modal dialog shadcn (`PromptModal.svelte` / `Dialog.svelte`).
4. **Dual-Mode Editor (WYSIWYG + Pure MD)**:
   - Pengguna menginginkan 2 mode yang dapat di-toggle: mode Pure Markdown (seperti Obsidian) dan mode visual WYSIWYG dengan toolbar lengkap (Bold, Italic, Headings, Lists, Alignment, Wikilinks, Divider, dll).
   - File disimpan sebagai format Markdown `.md` standar.
5. **Penyimpanan File Fisik 100%**:
   - Catatan disimpan langsung sebagai file `.md` fisik di folder komputer. Tidak ada virtual database terisolasi atau localStorage.

---

## 3. Status Penyelesaian Fitur

### ✅ FASE 1: MVP Core (SELESAI 100%)

- [x] **Arsitektur Dasar & Scaffolding**: Tauri v2 + Svelte 5 + Tailwind CSS.
- [x] **Tema & UI Zed**: `TitleBar.svelte`, `FileTree.svelte`, `TabBar.svelte`, `StatusBar.svelte`.
- [x] **Sistem File Fisik (`vault.ts`)**:
  - Desktop (Tauri): `@tauri-apps/plugin-dialog` + `@tauri-apps/plugin-fs` (`readDir`, `readTextFile`, `writeTextFile`, `mkdir`, `remove`).
  - Web Dev: Native HTML5 File System Access API (`showDirectoryPicker`) yang menulis file fisik langsung ke hard drive.
- [x] **Dual-Mode Editor**:
  - `MarkdownEditor.svelte` (CodeMirror 6 dengan `EditorView.lineWrapping` aktif sehingga teks otomatis wrap tanpa horizontal scroll).
  - `WysiwygEditor.svelte` (TipTap dengan styling Zed prose-invert).
  - `EditorToolbar.svelte` (Toolbar lengkap untuk format teks dan toggle mode).
- [x] **Manajemen Tab**:
  - Multi-tab file dengan indikator perubahan (_dirty indicator_ `*` / dot).
  - Modal konfirmasi simpan jika tab yang belum disimpan ditutup.
- [x] **Shortcuts**:
  - `Ctrl + S`: Simpan catatan aktif.
  - `Ctrl + N`: Dialog buat catatan baru.
  - `Ctrl + \` atau `Alt + B`: Toggle sidebar panel proyek (bebas konflik dengan `Ctrl + B` Bold).
- [x] **Bugfixes Kritis yang Sudah Diselesaikan**:
  - _Bug 1_: Mode Pure MD teks memanjang ke kanan $\rightarrow$ Diperbaiki dengan `EditorView.lineWrapping`.
  - _Bug 2_: File hilang saat di-refresh $\rightarrow$ Dihapus semua mock localStorage, diganti dengan File System Access API & Tauri fs plugin.
  - _Bug 3_: Kursor WYSIWYG loncat ke baris paling bawah saat ketik/hapus 1 huruf $\rightarrow$ Diperbaiki dengan guard `isSelfUpdating`, `untrack` Svelte 5, dan fungsi `markdownToHtml` sinkron di `WysiwygEditor.svelte`.
  - _Bug 4_: Error missing `icons/icon.ico` saat kompilasi Windows resource `tauri-build` $\rightarrow$ Di-generate seluruh icon bundle resmi (`icon.ico`, `icon.icns`, PNGs) dari `app-icon.svg` via `npx tauri icon`.
  - _Build Windows Native_: Berhasil dikompilasi ke executable biner Windows native (`src-tauri/target/debug/knowledge.exe`).

### ✅ FASE 2: PKM / Wikilinks, Backlinks & Indexer (SELESAI 100%)

- [x] **Mesin Pengindeks In-Memory (`indexer.ts`)**:
  - Regex ekstraksi `[[wikilinks]]` dan `#tags`.
  - Index relasi bolak-balik: Catatan A menautkan Catatan B $\rightarrow$ Catatan B memiliki _Backlink_ dari Catatan A beserta potongan konteks barisnya.
  - Mesin pencarian teks cepat (_full-text search_).
- [x] **Autokomplet Wikilink (`wikilinkExtension.ts`)**:
  - Mengetik `[[` di editor CodeMirror secara otomatis membuka popup autocomplete nama catatan yang ada di vault.
- [x] **Panel Sidebar Kanan (`RightSidebar.svelte`)**:
  - Tab _Backlinks_: Menampilkan daftar catatan yang me-referensikan catatan yang sedang aktif beserta preview konteks kalimatnya, dan dapat diklik untuk langsung membuka catatan tersebut.
  - Tab _Tags_: Menampilkan daftar tag `#...` yang tersebar di seluruh vault.
- [x] **Command Palette / Quick Search (`QuickSearchModal.svelte`)**:
  - Menekan `Ctrl + P` membuka modal pencarian cepat shadcn bergaya Zed, mencari judul dan isi dokumen secara real-time.

---

### ⏳ FASE 3: Google Login & Google Drive Two-Way Sync (LANGKAH BERIKUTNYA)

Ini adalah tugas utama yang harus dilanjutkan oleh AI agent berikutnya:

1. **Google OAuth2 PKCE Desktop Flow (Rust Backend)**:
   - Client ID & Secret untuk Google OAuth (Desktop Application).
   - Backend Tauri menjalankan listener loopback lokal sementara (`http://127.0.0.1:<port>/callback`) untuk menerima token otorisasi dari browser secara otomatis.
   - Penyimpanan token aman menggunakan crate `keyring` (mengakses OS Keyring di Linux Secret Service / Windows Credential Manager).
2. **Google Drive Sync Engine (Rust Backend)**:
   - Folder target di Google Drive: Folder khusus (misal `Knowledge-Vault`).
   - SQLite Metadata Ledger lokal (`rusqlite`) untuk mencatat: `file_path`, `local_hash`, `drive_file_id`, `remote_modified_time`, `sync_status`.
   - Push on save: Mengunggah file `.md` yang baru dibuat/diubah ke Google Drive via REST API v3 (`multipart upload`).
   - Pull delta changes: Menggunakan Google Drive API endpoint `changes.list` untuk menarik perubahan baru secara periodik atau saat aplikasi dibuka.
   - Penanganan Konflik: Jika terjadi modifikasi bersamaan offline/cloud, simpan versi cloud sebagai `NamaCatatan (Conflict [timestamp]).md` agar tidak ada data yang tertimpa (_zero data loss_).
3. **Integrasi UI**:
   - Tombol Login/Logout Google di TitleBar atau Settings Modal.
   - Status sinkronisasi real-time di `StatusBar.svelte` (Idle, Syncing, Synced, Error).

---

### ⏳ FASE 4: Workspace Split Pane & Graph View (FASE LANJUT)

- [ ] Split horizontal & split vertical untuk membuka dua catatan berdampingan.
- [ ] Visualisasi Graph View 2D keterhubungan antar catatan menggunakan PixiJS / WebGL.

---

## 4. Struktur Direktori Proyek

```
knowledge/
├── src/                               # Frontend (Svelte 5)
│   ├── lib/
│   │   ├── components/
│   │   │   ├── ui/                    # Komponen dasar shadcn
│   │   │   │   ├── button/Button.svelte
│   │   │   │   ├── input/Input.svelte
│   │   │   │   └── dialog/
│   │   │   │       ├── Dialog.svelte
│   │   │   │       └── PromptModal.svelte
│   │   │   ├── editor/                # Komponen Editor
│   │   │   │   ├── EditorToolbar.svelte
│   │   │   │   ├── MarkdownEditor.svelte (CodeMirror 6)
│   │   │   │   └── WysiwygEditor.svelte (TipTap)
│   │   │   ├── TitleBar.svelte        # Header gaya Zed
│   │   │   ├── FileTree.svelte        # Panel file explorer kiri
│   │   │   ├── TabBar.svelte          # Tab file di atas editor
│   │   │   ├── StatusBar.svelte       # Status bar bawah gaya Zed
│   │   │   ├── RightSidebar.svelte    # Panel Backlinks & Tags
│   │   │   ├── QuickSearchModal.svelte# Command Palette Ctrl+P
│   │   │   └── Editor.svelte          # Wrapper Dual-mode
│   │   ├── editor/
│   │   │   ├── zedTheme.ts            # Tema CodeMirror gaya Zed
│   │   │   └── wikilinkExtension.ts   # Autocomplete [[...]] CodeMirror
│   │   ├── services/
│   │   │   ├── vault.ts               # File I/O fisik (Tauri + Web FS API)
│   │   │   ├── indexer.ts             # PKM indexer (wikilinks, backlinks, search)
│   │   │   └── markdownConverter.ts   # Marked & Turndown bidirectional
│   │   ├── types.ts                   # Definisi tipe TypeScript
│   │   └── utils.ts                   # Helper shadcn (cn / clsx / twMerge)
│   ├── App.svelte                     # Root application
│   └── main.ts                        # Entry point Vite
├── src-tauri/                         # Backend Desktop Tauri v2 (Rust)
│   ├── src/
│   │   ├── lib.rs                     # Inisialisasi plugin fs & dialog
│   │   └── main.rs                    # Entrypoint Tauri binary
│   ├── capabilities/
│   │   └── default.json               # Izin fs dan dialog Tauri v2
│   ├── Cargo.toml                     # Dependensi Rust (tauri, plugins)
│   └── tauri.conf.json                # Konfigurasi jendela & metadata Tauri
├── OBSIDIAN_ANALYSIS_SRS.md           # Laporan REA Obsidian & spesifikasi awal
├── PROJECT_STATUS_HANDOVER.md         # Dokumen ini (Handover & status kerja)
└── package.json                       # Dependensi frontend npm
```

---

## 5. Panduan Menjalankan Proyek di Linux

Karena user melanjutkan pengembangan di Linux dengan toolchain Rust yang sudah terpasang:

### 1. Instal Dependensi WebKitGTK (Linux)

- **Ubuntu / Debian / Linux Mint**:
  ```bash
  sudo apt update
  sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
  ```
- **Fedora / RHEL**:
  ```bash
  sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file libappindicator-gtk3-devel librsvg2-devel
  ```
- **Arch Linux / Manjaro**:
  ```bash
  sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget openssl appmenu-gtk-module libappindicator-gtk3 librsvg
  ```

### 2. Jalankan Mode Pengembangan Desktop

```bash
npm install
npm run tauri dev
```

### 3. Verifikasi Frontend

```bash
npx svelte-check   # Harus 0 errors, 0 warnings
npm run build      # Vite production build harus sukses
```
