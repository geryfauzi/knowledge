# Laporan Analisis Reverse Engineering Obsidian & Software Requirements Specification (SRS)

Dokumen ini merangkum hasil investigasi rekayasa balik (*reverse engineering*) terhadap paket internal aplikasi **Obsidian** (v1.14.4) menggunakan REA, serta mendefinisikan spesifikasi kebutuhan perangkat lunak (*Software Requirements Specification*) untuk pembuatan aplikasi clone berbasis **Tauri v2 + Svelte 5** dengan dukungan sinkronisasi **Google Drive** di Windows dan Linux.

---

## Bagian 1: Temuan Analisis Reverse Engineering (REA Findings)

### 1.1 Informasi Target
* **Target Paket:** `resources/obsidian.asar` & `resources/app.asar`
* **Lokasi Instalasi Lokal:** `C:\Users\Gery Akbar Fauzi\AppData\Local\Programs\Obsidian`
* **Versi Engine:** Obsidian v1.14.4
* **Arsitektur Host:** Electron (Chromium + Node.js)

### 1.2 Ekosistem Pustaka Inti (Terdeteksi di `index.html` & `lib/`)
Bedah statis terhadap artefak mengungkap komponen pihak ketiga utama yang menyusun fungsionalitas Obsidian:

| Komponen | Pustaka Internal | Fungsi pada Aplikasi |
| :--- | :--- | :--- |
| **Editor Mesin** | `CodeMirror` (`lib/codemirror/`) | Penanganan pengetikan teks, parser markdown, VIM mode, dan overlay syntax. |
| **Graph View** | `PixiJS` (`lib/pixi.min.js`) | Render graf keterhubungan dokumen menggunakan 2D WebGL berkecepatan tinggi. |
| **Math / Rumus** | `MathJax TeX` (`lib/mathjax/`) | Rendering formula matematika LaTeX inline dan blok. |
| **Clipboard / Paste** | `Turndown` (`lib/turndown.js`) | Konversi otomatis HTML kaya (*rich-text*) saat pengguna mem-paste konten web menjadi format Markdown. |
| **Tanggal / Waktu** | `Moment.js` (`lib/moment.min.js`) | Pengelolaan timestamp catatan harian (*daily notes*). |
| **Kriptografi / Sync** | `Scrypt` (`lib/scrypt.js`) | Enkripsi end-to-end (E2EE) untuk fitur sinkronisasi cloud. |
| **Lokalisasi** | `i18next` (`lib/i18next.min.js`) | Manajemen multi-bahasa interface. |

### 1.3 Dua Pilar Arsitektur Internal Obsidian
Dari penelusuran kode bundle `app.js` (3.9 MB), sistem Obsidian bertumpu pada 2 modul inti:

1. **`Vault Engine` (Abstraksi Filesystem Lokal):**
   * Operasi file: `read`, `readBinary`, `cachedRead`, `process`, `copy`, `rename`, `delete`, `trash`.
   * Operasi folder: `createFolder`, `getRoot`, `getFolderByPath`, `getFileByPath`.
   * Event system: `vault.on('modify')`, `vault.on('delete')`, `vault.on('create')`.
   * Konfigurasi internal: Tersimpan dalam folder `.obsidian/` (`app.json`, `workspace.json`, `hotkeys.json`, dll).

2. **`MetadataCache` (Mesin Pengindeks In-Memory):**
   * Mengurai struktur dokumen secara asinkron tanpa memblokir UI (`computeFileMetadataAsync`).
   * Ekstraksi metadata: `getTags()`, `getFrontmatterPropertyValuesForKey()`.
   * Resolusi relasi: `getFirstLinkpathDest()`, `getBacklinksForFile()`, `queueFileForLinkResolution()`.
   * Melacak *unresolved links* (tautan ke catatan yang filenya belum dibuat).

### 1.4 Katalog Perintah Utama (*Command Registry*)
Teridentifikasi sejumlah kategori perintah yang menjadi standar interaksi pengguna:
* **`editor` (71 perintah):** Bold, italic, strikethrough, highlight, callout, table, footnote, inline math, math block, fold/unfold headings, insert wikilink `[[...]]`, embed `![[...]]`, toggle checklist, cycle list.
* **`workspace` (24 perintah):** Split horizontal, split vertical, tab management (new tab, close tab, pin tab, undo close pane), stacked tabs, export PDF.
* **`app` (21 perintah):** Open vault, switch vault, toggle left/right sidebar, toggle ribbon, theme switch.
* **`file-explorer` (5 perintah):** New file, new file in split pane, move file, duplicate file.

---

## Bagian 2: Software Requirements Specification (SRS) Clone App

Aplikasi clone ditargetkan berjalan secara native di **Windows** dan **Linux** dengan arsitektur **Tauri v2 (Rust Backend)** dan **Svelte 5 (Frontend)**.

### 2.1 Ringkasan Kebutuhan Fungsional (Use Cases)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          ARSITEKTUR FITUR CLONE                             │
├───────────────────────────────┬─────────────────────────────────────────────┤
│ 1. Vault & File Management    │ Operasi file .md lokal & watcher            │
│ 2. Markdown Editor            │ CodeMirror 6, Live Formatting, Math, Embeds │
│ 3. Link & Metadata Indexer    │ Resolusi [[Wikilinks]], Backlinks, Tags     │
│ 4. Workspace & Layout         │ Tabs, Splits (Horizontal/Vertical), Sidebar │
│ 5. Google Drive Sync Engine   │ OAuth2 PKCE, Two-Way Sync, Conflict Handler │
│ 6. Graph View (Fase Lanjut)   │ Visualisasi jejaring relasi catatan WebGL   │
└───────────────────────────────┴─────────────────────────────────────────────┘
```

#### UC-01: Manajemen Vault & Filesystem
* **UC-01.1:** Pengguna dapat membuka folder mana saja di sistem operasi sebagai ruang kerja (*Vault*).
* **UC-01.2:** Pengguna dapat membuat file baru, membuat folder, menghapus (ke sistem Trash), mengganti nama, dan memindahkan file/folder via *drag-and-drop*.
* **UC-01.3:** Sistem Rust memantau perubahan eksternal via crate `notify` dan langsung memicu pembaruan antarmuka jika ada file yang diubah dari luar aplikasi.
* **UC-01.4:** Pengaturan preferensi lokal disimpan di `.myvault/config.json`.

#### UC-02: Editor Markdown & Wikilinks (CodeMirror 6)
* **UC-02.1:** Sintaks standar Markdown terformat otomatis (Heading 1–6, Bold `**`, Italic `*`, List `-`, Checklist `- [ ]`, Code blocks ````).
* **UC-02.2 (Wikilinks):** Mengetik `[[` memicu dropdown autocomplete nama file yang ada di dalam vault.
* **UC-02.3 (Embeds):** Mengetik `![[gambar.png]]` menampilkan preview gambar lokal; `![[catatan.md]]` menampilkan transklusi konten.
* **UC-02.4 (LaTeX Math):** Mendukung formula inline `$E=mc^2$` dan blok `$$...$$` menggunakan KaTeX/MathJax.
* **UC-02.5 (Frontmatter):** Parsing blok YAML di baris awal dokumen untuk properti metadata (`tags`, `date`, `aliases`).

#### UC-03: Mesin Pengindeks Metadata & Tautan (Metadata Engine)
* **UC-03.1 (Link Resolver):** Mengaitkan string tautan `[[Nama Catatan]]` ke path file sebenarnya. Jika file belum ada, tandai sebagai *unresolved link* (dapat diklik untuk langsung membuat file baru).
* **UC-03.2 (Backlinks Panel):** Menyajikan daftar catatan lain yang mencantumkan tautan ke catatan yang sedang dibuka.
* **UC-03.3 (Tag Aggregator):** Mengindeks seluruh `#tag` untuk navigasi cepat via tag pane.
* **UC-03.4 (Pencarian Teks Penuh):** Fitur pencarian kilat in-memory di seluruh file markdown.

#### UC-04: Tata Letak Ruang Kerja (Workspace Layout)
* **UC-04.1:** Sidebar kiri: Tab File Explorer, Tab Search, dan Tab Tags.
* **UC-04.2:** Area tengah: Sistem multi-tab dengan dukungan pembagian jendela (*Split Horizontal* dan *Split Vertical*).
* **UC-04.3:** Sidebar kanan: Panel Backlinks dan Daftar Heading (*Outline*).
* **UC-04.4:** Dukungan tema bawaan: Dark Mode dan Light Mode.

#### UC-05: Google Login & Google Drive Sync (Fitur Khusus)
* **UC-05.1 (Otentikasi OAuth2 PKCE Desktop):**
  * Aplikasi memicu browser default pengguna membuka halaman login otorisasi Google.
  * Backend Rust menjalankan HTTP loopback server sementara di `http://127.0.0.1:<port>/callback` untuk menangkap authorization code.
  * Access Token dan Refresh Token disimpan secara aman menggunakan OS Credential Manager (Windows Credential Manager / Linux Secret Service via crate `keyring`).
* **UC-05.2 (Struktur Folder Drive):**
  * Aplikasi membuat folder khusus di Google Drive (misal: `MyNotes-Vault/`).
* **UC-05.3 (Sinkronisasi Dua Arah):**
  * **Lokal $\rightarrow$ Cloud:** Saat file disimpan atau dimodifikasi, engine Rust mengunggah file yang bersangkutan ke Google Drive.
  * **Cloud $\rightarrow$ Lokal:** Menggunakan Google Drive API endpoint `changes.list` untuk menarik perubahan delta terbaru tanpa mengunduh ulang seluruh vault.
* **UC-05.4 (Resolusi Konflik):**
  * Jika sebuah file diubah di perangkat lokal dan cloud secara bersamaan dalam kondisi offline, sistem membuat duplikat berlabel `NamaFile (Conflict [timestamp]).md` agar tidak ada data yang terhapus.

---

## Bagian 3: Arsitektur Teknologi yang Dipilih

```
┌────────────────────────────────────────────────────────┐
│               FRONTEND (Webview - Svelte 5)            │
│  - Framework: Svelte 5 (Runes) + Vite                  │
│  - Styling: Tailwind CSS                               │
│  - Editor: CodeMirror 6 (Modular extensions)           │
│  - Visualizer: PixiJS (Graph view) / KaTeX (Math)      │
└───────────────────────────┬────────────────────────────┘
                            │ Tauri IPC (Commands & Events)
┌───────────────────────────┴────────────────────────────┐
│               BACKEND (Rust - Tauri v2)                │
│  - Local FS & Watcher: std::fs + crate notify          │
│  - Key Storage: crate keyring                          │
│  - Sync & HTTP: crate reqwest + crate tokio            │
│  - Sync Ledger Database: crate rusqlite (SQLite lokal) │
└────────────────────────────────────────────────────────┘
```

---

## Bagian 4: Roadmap Rencana Implementasi

1. **Fase 1: Minimum Viable Product (MVP Core)**
   * Inisialisasi template proyek Tauri v2 + Svelte 5 + Tailwind CSS.
   * Modul Rust untuk memilih folder vault dan membaca struktur file pohon (*tree view*).
   * Integrasi CodeMirror 6 dasar di Svelte 5 dengan dukungan edit & simpan file `.md`.
2. **Fase 2: Wikilinks & Indexer**
   * Ekstensi CodeMirror untuk autocomplete `[[...]]`.
   * In-memory indexer di Svelte untuk mengumpulkan relasi tautan dan menampilkan panel *Backlinks*.
3. **Fase 3: Google Drive Sync**
   * Implementasi Google OAuth 2.0 PKCE loopback di Rust.
   * SQLite metadata ledger lokal (mencatat hash file & Google Drive File ID).
   * Engine sinkronisasi dua arah (Push on save / Pull on changes).
4. **Fase 4: Polesan Workspace & Fitur Lanjutan**
   * Workspace split pane (vertical/horizontal) dan tab bar.
   * Graph View menggunakan PixiJS.
