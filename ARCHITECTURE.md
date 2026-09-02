# Arsitektur & Struktur Proyek: `ai-debug` CLI

## 1. Filosofi Desain

`ai-debug` adalah CLI murni bergaya UNIX, standalone. Tidak ada TUI, tidak butuh aplikasi host lain berjalan. Alurnya linier:

1. Terima input teks (log error) dari `stdin` (piping) atau argumen `--file`.
2. Ekstrak referensi file & nomor baris dari stack trace.
3. Baca file sumber lokal untuk ambil konteks kode di sekitar baris error.
4. Kirim log mentah + potongan kode ke satu AI provider (cloud atau lokal) lewat HTTP API langsung.
5. Cetak penjelasan AI (akar masalah & solusi) ke `stdout`, dengan pewarnaan opsional.

> **Catatan penting:** versi sebelumnya dari dokumen ini memakai Model Context Protocol (MCP) sebagai jalur untuk "memanggil AI". Itu salah arah — MCP client hidup *di dalam* aplikasi host AI (Claude Code, Claude Desktop, Cursor) dan tugasnya konek ke MCP *server* yang expose tools/context, bukan jalur untuk memanggil model AI secara langsung. Untuk kebutuhan "kirim prompt, dapat jawaban", yang benar dipakai adalah API model langsung (Anthropic Messages API, OpenAI API, atau endpoint lokal seperti Ollama). MCP dilepas dari desain inti; lihat bagian 6 untuk opsi lanjutan yang justru memakai MCP dengan arah yang benar.

## 2. Arsitektur Lapis (3-Tier)

### A. Lapisan Input (Ingestion)
- **Crate utama:** `clap` (fitur `derive`).
- **Logika:** mendukung `cat error.log | ai-debug` maupun `ai-debug --file error.log`. Tambahan: `--context-lines <N>` (default 10), `--no-color`, `--local-only`, `--provider <anthropic|openai|ollama>`.

### B. Lapisan Ekstraktor Konteks (Context Engine)
- **Crate utama:** `regex`.
- **Logika:** stack trace tiap bahasa punya format berbeda (Rust panic, Python traceback, JS/Node, Go). Jangan pakai satu regex universal — pakai trait kecil per bahasa:

```rust
trait TraceParser {
    fn detect(&self, log: &str) -> bool;
    fn extract_locations(&self, log: &str) -> Vec<CodeLocation>; // file, line
}
```

  Implementasi awal: `RustTraceParser`, `PythonTraceParser`, `JsTraceParser`. Untuk tiap lokasi yang ditemukan, baca file lokal dari `line - N` sampai `line + N` (N = `--context-lines`).
- **Opsional lanjutan (v2):** ganti window baris tetap dengan ekstraksi seluruh badan fungsi yang memuat baris error, pakai `tree-sitter`, supaya konteks yang dikirim ke AI lebih relevan daripada sekadar potongan baris.

### C. Lapisan AI Client & Output
- **Crate utama:** `reqwest`, `serde`/`serde_json`, `tokio`, `colored`.
- **Logika:** trait provider tipis supaya tidak lock-in ke satu vendor:

```rust
#[async_trait::async_trait]
trait AiProvider {
    async fn explain(&self, log: &str, context: &str) -> anyhow::Result<String>;
}

struct AnthropicProvider { api_key: String }
struct OpenAiProvider { api_key: String }
struct OllamaProvider { base_url: String } // lokal, tanpa API key
```

  Pilih implementasi berdasarkan `--provider` (atau env var). `--local-only` memaksa pakai `OllamaProvider` — berguna untuk kode sensitif yang tidak boleh keluar ke cloud.
- **Token/ukuran budget:** sebelum dikirim, potong log + context ke batas karakter/token yang wajar (`--max-context-chars`, default aman), supaya biaya API dan kualitas prompt terkontrol pada log/file besar.
- **Redaksi dasar:** sebelum request keluar ke provider cloud, scan context dengan regex pola umum secret (API key, token, `.env`-style `KEY=value`) dan mask sebelum dikirim.
- **Output:** hasil dicetak ke `stdout` — merah untuk penyebab, hijau untuk solusi. Hormati env var `NO_COLOR` dan flag `--no-color` (standar CLI Unix).

## 3. Struktur Direktori Proyek

```
ai-debug/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point, inisialisasi tokio runtime, wiring semua lapisan
│   ├── cli.rs             # Definisi argumen CLI (clap)
│   ├── extractor/
│   │   ├── mod.rs         # Dispatch ke parser yang cocok + baca konteks lokal
│   │   ├── rust.rs        # RustTraceParser
│   │   ├── python.rs      # PythonTraceParser
│   │   └── js.rs          # JsTraceParser
│   ├── provider/
│   │   ├── mod.rs         # trait AiProvider + pemilihan provider
│   │   ├── anthropic.rs
│   │   ├── openai.rs
│   │   └── ollama.rs
│   ├── redact.rs          # Regex redaksi secret sebelum request keluar
│   └── formatter.rs       # Pewarnaan & format output, hormati NO_COLOR
└── README.md
```

## 4. Instruksi Awal untuk Agen AI (Prompt Utama)

> "AI, tolong buatkan implementasi kerangka dasar untuk CLI `ai-debug` berdasarkan `ARCHITECTURE.md` ini.
>
> Langkah 1: Buat `Cargo.toml` dengan dependencies: `tokio` (full), `clap` (derive), `regex`, `colored`, `reqwest` (json, rustls-tls), `serde`, `serde_json`, `async-trait`, `anyhow`.
>
> Langkah 2: Di `src/cli.rs`, buat struct untuk menerima input dari `stdin` atau `--file`, plus flag `--context-lines`, `--no-color`, `--local-only`, `--provider`.
>
> Langkah 3: Di `src/extractor/`, buat trait `TraceParser` dan minimal satu implementasi (`RustTraceParser`) yang mendeteksi pola `path/to/file.rs:line`, lalu mengembalikan potongan kode dari file tersebut jika ada di disk. Sertakan unit test dengan sample log Rust dan Python untuk memverifikasi deteksi bahasa & ekstraksi baris.
>
> Langkah 4: Di `src/provider/`, buat trait `AiProvider` dan satu implementasi dummy/mock dulu (belum panggil API sungguhan) supaya alur end-to-end bisa dites tanpa API key.
>
> Langkah 5: Rangkai alurnya di `src/main.rs`: baca input → ekstrak konteks → (untuk saat ini) print gabungan log + konteks + hasil dummy provider ke terminal, sebelum kita sambungkan ke provider AI sungguhan."

## 5. Keamanan & Privasi

- Default: jangan kirim apa pun ke provider cloud tanpa konfirmasi eksplisit (`--yes` atau prompt interaktif) jika file yang dibaca berada di luar direktori kerja saat ini.
- `--local-only` membatasi ke provider lokal (Ollama) — cocok untuk kode proprietary/perusahaan.
- Redaksi secret (lihat `redact.rs`) berjalan sebelum data keluar, bukan opsional.

## 6. Opsi Lanjutan (v2, opsional): Ekspos sebagai MCP Server

Kalau ke depan mau terintegrasi dengan Claude Code/Cursor (selaras dengan rencana bisnis AI agent yang lain), arah yang benar adalah membuat `ai-debug` menjadi **MCP server** yang expose tool seperti `get_error_context(log: string) -> CodeContext`. Host (Claude Code/Cursor) yang sudah py MCP client bawaan tinggal memanggil tool ini; AI-nya sudah disediakan host, sehingga seluruh lapisan `provider/` di atas jadi tidak diperlukan lagi untuk mode ini. Ini best dikerjakan setelah versi standalone (bagian 1-5) stabil, bukan di MVP awal.
