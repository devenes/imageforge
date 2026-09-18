# ImageForge — Architecture

This document describes the internal architecture of ImageForge: how the Tauri shell, Svelte frontend, and Rust backend are structured, how data flows between them, and the design decisions behind each key subsystem.

---

## Table of Contents

1. [High-level overview](#1-high-level-overview)
2. [Process model](#2-process-model)
3. [IPC contract](#3-ipc-contract)
4. [Rust backend](#4-rust-backend)
   - [Module map](#module-map)
   - [Error model](#error-model)
   - [Type model](#type-model)
   - [Format engines](#format-engines)
   - [Compression strategy](#compression-strategy)
   - [Processor](#processor)
   - [Queue manager](#queue-manager)
   - [Output resolution & atomic writes](#output-resolution--atomic-writes)
   - [Dimension invariant](#dimension-invariant)
5. [Svelte frontend](#5-svelte-frontend)
   - [Component tree](#component-tree)
   - [Reactive state](#reactive-state)
   - [Event wiring](#event-wiring)
6. [Data flow walkthrough](#6-data-flow-walkthrough)
7. [Concurrency model](#7-concurrency-model)
8. [NO_SAVINGS semantics](#8-no_savings-semantics)
9. [Alpha compositing](#9-alpha-compositing)
10. [Metadata & colour profile preservation](#10-metadata--colour-profile-preservation)
11. [Key design decisions](#11-key-design-decisions)
12. [Test strategy](#12-test-strategy)

---

## 1. High-level overview

```
┌──────────────────────────────────────────────────────────────┐
│  macOS process: ImageForge.app                               │
│                                                              │
│  ┌─────────────────────┐      IPC (JSON over stdin/async)   │
│  │  WebView (WKWebView)│ ◄──────────────────────────────►  │
│  │  Svelte 5 + TS      │                                    │
│  │                     │      Tauri events (async push)     │
│  │  appState (runes)   │ ◄──────────────────────────────   │
│  └─────────────────────┘                                    │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Rust (main thread + Tokio async runtime)            │   │
│  │                                                      │   │
│  │  commands.rs  ──►  QueueManager  ──►  ImageProcessor │   │
│  │                         │                │           │   │
│  │                   Semaphore pool    FixedQualityStrategy  │
│  │                                         │           │   │
│  │                             formats/{jpeg,png,webp,heic, │
│  │                                         convert}.rs  │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

ImageForge is a **Tauri 2.x** desktop application. The frontend (Svelte 5) runs in a WKWebView sandbox. The backend (Rust) runs in the native process. They communicate through Tauri's IPC bridge:

- **Frontend → Backend**: `invoke()` calls map to `#[tauri::command]` handlers.
- **Backend → Frontend**: `app.emit()` pushes named events that the frontend subscribes to with `listen()`.

---

## 2. Process model

```
main (Rust, single thread)
│
└─ Tauri application loop
     ├─ WebView process (Svelte UI)
     └─ Tokio runtime (multi-threaded)
          ├─ async tasks: command handlers, queue coordination
          └─ spawn_blocking tasks: CPU-bound codec work (one per image)
```

All codec operations (libjpeg-turbo, oxipng, libwebp, libheif) are synchronous C/Rust library calls. They run inside `tokio::task::spawn_blocking` so they don't starve the async runtime.

---

## 3. IPC contract

### Commands (frontend → backend, via `invoke`)

| Command | Arguments | Return |
|---|---|---|
| `inspect_files` | `paths: Vec<String>` | `Vec<ImageInfo>` |
| `start_batch` | `items: Vec<ImageInfo>`, `settings: CompressionSettings` | `()` |
| `cancel_batch` | — | `()` |
| `open_output_folder` | `path: String` | `()` |
| `reveal_file` | `path: String` | `()` |
| `get_default_settings` | — | `CompressionSettings` |

### Events (backend → frontend, via `emit`)

| Event | Payload | When |
|---|---|---|
| `file_progress` | `ProgressUpdate` | When a worker begins processing a file |
| `file_completed` | `CompressionResult` | When a single file finishes (success, no-savings, failed, or skipped) |
| `batch_completed` | `Vec<CompressionResult>` | When all files in a batch are done |
| `batch_cancelled` | `()` | When cancellation completes |

All payloads are serialised as `camelCase` JSON (serde `rename_all = "camelCase"`). Rust enums use `SCREAMING_SNAKE_CASE` for status values (e.g. `"NO_SAVINGS"`, `"COMPLETED"`).

---

## 4. Rust backend

### Module map

```
src-tauri/src/
├── main.rs          Entry point (thin: calls lib::run)
├── lib.rs           Tauri builder, plugin registration, managed state
├── commands.rs      #[tauri::command] handlers — thin dispatch layer
├── models.rs        All shared IPC types (ImageInfo, CompressionResult, …)
├── errors/
│   └── mod.rs       AppError enum with user_friendly_message()
├── metadata.rs      Format detection (magic bytes) + EXIF/ICC introspection
├── validation.rs    verify_dimensions() + calculate_savings()
├── output.rs        resolve_output_path() + atomic_write_file()
├── processor.rs     ImageProcessor: per-file pipeline orchestration
├── queue.rs         QueueManager: async worker pool + CancellationToken
└── formats/
    ├── mod.rs       CompressionStrategy trait + FixedQualityStrategy
    ├── jpeg.rs      libjpeg-turbo encode/decode
    ├── png.rs       oxipng lossless optimization
    ├── webp.rs      libwebp encode/decode
    ├── heic.rs      libheif-rs encode/decode
    └── convert.rs   Unified any-format → JPG conversion
```

---

### Error model

```rust
pub enum AppError {
    UnsupportedFormat(String),
    DimensionMismatch { expected_w, expected_h, actual_w, actual_h },
    DecodeFailed { user_message, technical_detail },
    EncodeFailed  { user_message, technical_detail },
    IoError(String),
    NoSavings,
    Cancelled,
    Validation(String),
}
```

- `DecodeFailed` and `EncodeFailed` carry two strings: a **user-friendly message** shown in the UI, and a **technical detail** logged via `tracing` but never surfaced to the user. Raw C-library error strings never reach the frontend.
- `user_friendly_message()` produces the final UI string. Tauri command handlers convert `AppError` to `String` at the IPC boundary.

---

### Type model

```
ImageInfo                     -- produced by inspect_image(), sent to frontend
  id: Uuid v4 (String)
  path, filename
  format: ImageFormat         -- Jpg | Png | Webp | Heic
  width, height: u32
  bytes: u64
  orientation: Option<u32>    -- EXIF tag 274
  has_alpha: bool
  color_profile: Option<String>
  metadata_available: bool

CompressionSettings           -- sent from frontend to start_batch
  preset: CompressionPreset   -- BestQuality | Balanced | Smallest
  output_mode: OutputMode     -- SameFormat | ConvertToJpg
  preserve_metadata: bool
  preserve_color_profile: bool
  custom_output_dir: Option<String>
  filename_suffix: Option<String>   -- default "-compressed"

CompressionResult             -- emitted per file
  id, input_bytes, output_bytes
  saved_bytes, saved_percent
  input/output width/height
  input/output format
  output_path: Option<String>
  duration_ms: u64
  status: CompressionStatus   -- COMPLETED | NO_SAVINGS | FAILED | SKIPPED
  error: Option<String>
  color_profile_preserved: bool
```

---

### Format engines

Each format module exposes a single public function:

```
jpeg::compress_jpeg(path, info, settings) -> Result<ProcessedOutput, AppError>
png::compress_png(path, info, settings)   -> Result<ProcessedOutput, AppError>
webp::compress_webp(path, info, settings) -> Result<ProcessedOutput, AppError>
heic::compress_heic(path, info, settings) -> Result<ProcessedOutput, AppError>
convert::convert_to_jpg(path, info, settings) -> Result<ProcessedOutput, AppError>
```

`ProcessedOutput` is a plain struct `{ data: Vec<u8>, width: u32, height: u32, format: ImageFormat, color_profile_preserved: bool }`.

All format modules follow the same invariant: **call `verify_dimensions` before returning**, both once after encoding and once after an independent re-decode of the output bytes. A mismatch panics the operation with `AppError::DimensionMismatch`.

| Engine | Library | Quality levers |
|---|---|---|
| JPEG | libjpeg-turbo 3.2 (turbojpeg crate) | quality 92/84/74, Sub2x2 chroma |
| PNG | oxipng 10.2 (pure Rust) | optimisation level 2/4/6 |
| WebP | libwebp 1.6 (webp crate + img-parts) | quality 92/82/70 |
| HEIC | libheif 1.23 (libheif-rs 3.0) | quality 88/78/65 |
| →JPG | libjpeg-turbo (same as JPEG) | quality 92/84/74 |

---

### Compression strategy

`CompressionStrategy` is a `Send + Sync` trait:

```rust
pub trait CompressionStrategy: Send + Sync {
    fn process(&self, path: &Path, info: &ImageInfo, settings: &CompressionSettings)
        -> Result<ProcessedOutput, AppError>;
}
```

The only production implementation is `FixedQualityStrategy`. It dispatches based on `(settings.output_mode, info.format)`:

```
ConvertToJpg         → convert::convert_to_jpg
SameFormat + Jpg     → jpeg::compress_jpeg
SameFormat + Png     → png::compress_png
SameFormat + Webp    → webp::compress_webp
SameFormat + Heic    → heic::compress_heic
```

This trait boundary makes it straightforward to add a future `PerceptualStrategy` (e.g. binary-search over quality until SSIM target is met) without touching the queue or processor.

---

### Processor

`ImageProcessor::process_file` is the **per-file orchestration function**. It is synchronous (runs inside `spawn_blocking`):

```
process_file(info, settings) -> CompressionResult
  1. Check input file exists   → Failed if not
  2. strategy.process()        → ProcessedOutput | AppError::*
  3. calculate_savings()       → (saved_bytes, saved_percent, keep_candidate)
  4a. keep_candidate = false   → read original bytes, atomic_write_file(original)
                                 return CompressionStatus::NoSavings
  4b. keep_candidate = true    → resolve_output_path()
                                 atomic_write_file(compressed_bytes)
                                 return CompressionStatus::Completed
```

Every code path that writes a file uses `atomic_write_file` — there is no path that writes directly to the final filename.

---

### Queue manager

`QueueManager` is held as Tauri managed state (`Arc<QueueManager>`, effectively a singleton for the lifetime of the app).

```rust
pub struct QueueManager {
    current_token: Mutex<Option<CancellationToken>>,
    processor: Arc<ImageProcessor>,
}
```

`start_batch` flow:

1. Cancel any running batch (drop old `CancellationToken`).
2. Create a new `CancellationToken` and store it.
3. `tokio::spawn` a task that:
   - Creates a `Semaphore(cpus.clamp(1, 6))`.
   - Iterates items; for each, acquires a semaphore permit, spawns a `JoinSet` task.
   - Each task calls `spawn_blocking(|| processor.process_file(...))`.
   - Emits `file_progress` before and `file_completed` after each file.
4. After all tasks join, emits `batch_completed` or `batch_cancelled`.

Cancellation is cooperative: each task checks `cancel_token.is_cancelled()` before starting real work and emits `CompressionStatus::Skipped` immediately if set.

---

### Output resolution & atomic writes

`resolve_output_path(input_path, target_format, settings) -> PathBuf`

Logic:

```
output_dir  = settings.custom_output_dir ?? input_path.parent()
file_stem   = input_path.file_stem()         -- e.g. "photo"
suffix      = settings.filename_suffix ?? "-compressed"

SameFormat:   "{stem}{suffix}.{ext}"         -- e.g. "photo-compressed.jpg"
ConvertToJpg
  if input is .jpg/.jpeg: "{stem}{suffix}.jpg"
  else:                   "{stem}.jpg"        -- e.g. "photo.jpg"

Collision avoidance (never overwrites):
  "photo-compressed.jpg" exists?
    → try "photo-compressed-1.jpg"
    → try "photo-compressed-2.jpg"  …
```

`atomic_write_file(target, data)`:

```
1. Compute temp filename: ".tmp_{pid}_{uuid}" in same directory as target
2. File::create(temp) + write_all + flush
3. std::fs::rename(temp, target)   -- atomic on POSIX (same mountpoint)
4. On rename failure: remove temp, return IoError
```

Because temp and target are always on the same filesystem (same parent directory), `rename` is guaranteed to be atomic on macOS (POSIX).

---

### Dimension invariant

```rust
pub fn verify_dimensions(input_w, input_h, output_w, output_h) -> Result<(), AppError>
```

Called **twice** per format operation:
1. Immediately after encode (using the encoder's reported dimensions).
2. After an independent decode of the output bytes (using a fresh decoder).

If either call fails, the error propagates up through `process_file` and the file is marked `Failed`. This makes it structurally impossible for a format engine to silently return resized output.

---

## 5. Svelte frontend

### Component tree

```
App.svelte
├── ErrorBanner.svelte          -- dismissable error stripe
├── DropZone.svelte             -- drag-and-drop overlay + file-picker
├── EmptyState.svelte           -- shown when queue is empty
├── FileQueue.svelte            -- scrollable list of queued files
│   └── FileRow.svelte          -- per-file row (thumbnail, name, status, savings)
│       └── Thumbnail.svelte    -- lazy image preview
├── CompressionControls.svelte  -- preset picker, mode toggle, start/clear buttons
├── ProgressView.svelte         -- live progress bar + current filename
├── ResultsSummary.svelte       -- post-batch aggregate stats
└── PreferencesModal.svelte     -- output dir, suffix, metadata toggles
```

---

### Reactive state

All global state lives in one Svelte 5 runes class, `AppState` (`src/lib/stores/appState.svelte.ts`):

```typescript
class AppState {
  queue:             $state<QueueItem[]>           // files added by user
  settings:          $state<CompressionSettings>   // current settings
  isProcessing:      $state<boolean>
  progress:          $state<ProgressUpdate | null>
  results:           $state<CompressionResult[]>
  hasCompletedBatch: $state<boolean>
  showPreferences:   $state<boolean>
  errorBanner:       $state<string | null>

  // Derived:
  isQueueEmpty           → queue.length === 0
  totalOriginalBytes     → sum(queue[].info.bytes)
  totalOutputBytes       → sum(results by status)
  totalSavedBytes        → sum(results[status=COMPLETED].savedBytes)
  overallReductionPercent
  completedCount
}

export const appState = new AppState();  // singleton, module-scoped
```

Components import `appState` directly and read/write its `$state` fields. There are no prop-drilling chains or event buses — all coordination goes through this one reactive object.

---

### Event wiring

`App.svelte` (on `onMount`) registers four Tauri event listeners:

| Tauri event | Handler |
|---|---|
| `file_progress` | `appState.progress = payload` |
| `file_completed` | `appState.updateItemStatus(id, status, result)` |
| `batch_completed` | `appState.finishBatch(results)` |
| `batch_cancelled` | `appState.cancelBatch()` |

Keyboard shortcuts are also registered in `App.svelte`:

| Key | Action |
|---|---|
| `⌘O` | `dropZoneRef.chooseFiles()` |
| `⌘K` | `appState.clearQueue()` |
| `⌘↵` | `handleStartCompress()` |
| `⌘,` | `appState.showPreferences = true` |
| `Esc` | `invoke('cancel_batch')` |

---

## 6. Data flow walkthrough

**Adding files:**

```
User drops files onto DropZone
  └─ DropZone calls invoke('inspect_files', { paths })
       └─ commands::inspect_files
            └─ metadata::inspect_image (magic bytes, EXIF, ICC, dimensions)
            └─ returns Vec<ImageInfo>
  └─ appState.addFiles(infos)  →  queue grows
```

**Starting compression:**

```
User presses ⌘↵ (or "Compress" button)
  └─ handleStartCompress()
       └─ invoke('start_batch', { items, settings })
            └─ commands::start_batch
                 └─ QueueManager::start_batch  (returns immediately)
                      └─ tokio::spawn (background task)

Background task (per file, concurrent):
  ├─ emit 'file_progress'   →  ProgressView updates
  ├─ spawn_blocking: ImageProcessor::process_file
  │    ├─ FixedQualityStrategy::process  (codec work)
  │    ├─ calculate_savings
  │    ├─ resolve_output_path
  │    └─ atomic_write_file
  └─ emit 'file_completed'  →  FileRow updates (status, savings %)

All done:
  └─ emit 'batch_completed' →  ResultsSummary appears
```

---

## 7. Concurrency model

```
Tokio runtime (multi-threaded)
│
├─ Async task: command handler (inspect_files, start_batch, cancel_batch)
│
└─ Async task: QueueManager worker loop
     │
     ├─ Semaphore: cpus.clamp(1, 6) permits
     │
     ├─ JoinSet task 1 ──► spawn_blocking ──► process_file (thread pool)
     ├─ JoinSet task 2 ──► spawn_blocking ──► process_file (thread pool)
     └─ JoinSet task N ──► spawn_blocking ──► process_file (thread pool)
```

- **Semaphore** ensures at most N files process concurrently (default = logical CPU count, capped at 6).
- **`spawn_blocking`** moves CPU-intensive codec work off the async runtime's thread pool onto Tokio's dedicated blocking thread pool (default size: 512 threads, but practically capped by the semaphore).
- **`CancellationToken`** is checked at two points: before acquiring the semaphore permit (loop-level) and after acquiring it (task-level). Both checks emit `Skipped` results.
- **`Arc<ImageProcessor>`** is cloned into each task — `ImageProcessor` contains only a boxed strategy which is `Send + Sync`, so there is no shared mutable state.

---

## 8. NO_SAVINGS semantics

When the encoded candidate is ≥ the original file size:

- For `SameFormat`: `keep_candidate = false` — the original bytes are used.
- For `ConvertToJpg`: `keep_candidate = true` — a size increase is acceptable because the format changed.

When `keep_candidate = false`:

1. The original file is re-read from disk.
2. The original bytes are atomically written to the resolved output path.
3. `CompressionResult.status = NO_SAVINGS`, `output_bytes = input_bytes`, `saved_bytes = 0`.
4. `output_path` is **always** `Some(...)` — the user always gets a file.

This means the UI can always offer "Open in Finder" for every result, and the output folder always contains one file per input file regardless of outcome.

---

## 9. Alpha compositing

When converting a format with an alpha channel to JPEG (which has no alpha):

```rust
// composite_rgba_onto_white(rgba: &[u8], width, height) -> Vec<u8>
for [r, g, b, a] in rgba.as_chunks::<4>().0 {
    if a == 255 { output RGB directly }
    else if a == 0 { output [255, 255, 255] }
    else {
        // Straight-alpha blend over white:
        out_r = (r * a + 255 * (255 - a) + 127) / 255
        // same for g, b
    }
}
```

The `+127` is integer rounding bias (equivalent to `round()` rather than `floor()`).

This path is triggered for:
- PNG with alpha channel → JPG conversion
- WebP with alpha channel → JPG conversion
- HEIC with alpha channel → JPG conversion

PNG and WebP with no alpha channel skip this step and pass RGB bytes directly to the JPEG encoder.

---

## 10. Metadata & colour profile preservation

### JPEG
- ICC profile and EXIF are read from the source via `img_parts::jpeg::Jpeg`.
- After encoding by libjpeg-turbo, the container is rebuilt with `img_parts` to inject the original ICC/EXIF segments.
- `color_profile_preserved: true` is set in the result when ICC was successfully carried over.

### PNG
- `oxipng` is configured with `StripChunks::Safe` — it preserves `iCCP`, `sRGB`, and `cHRM` chunks but drops non-standard vendor chunks.

### WebP
- ICC and EXIF are extracted from the source via `img_parts::webp::WebP`.
- After encoding by libwebp, segments are injected into the new WebP container.

### HEIC
- EXIF metadata blocks are extracted using `handle.metadata_block_ids(b"Exif")`.
- After encoding, EXIF bytes are attached to the output context with `out_ctx.add_exif_metadata`.
- ICC is not yet preserved for HEIC (libheif-rs API limitation at v3.0).

### JPG conversion
- ICC and EXIF from the source are extracted (per format above) and injected into the output JPEG via `img_parts`.

---

## 11. Key design decisions

**Why not libvips?**
The `vips` Homebrew formula pulls in ~75 transitive dependencies (cairo, pango, poppler, imagemagick, gcc). Using per-format codec crates keeps the dependency surface minimal and the `.app` bundle self-contained.

**Why Tauri 2.x instead of Electron?**
Tauri uses the system WebView (WKWebView on macOS) rather than bundling Chromium, which gives a ~4× smaller binary and native memory/CPU characteristics. Rust for the backend avoids a Node.js runtime.

**Why Svelte 5 runes instead of a store library?**
Svelte 5 runes (`$state`, `$derived`) provide reactive primitives at the language level. A single `AppState` class replaces Redux/Zustand/Pinia entirely — no store adapters, no subscriptions, no selectors.

**Why `as_chunks::<4>()` instead of `chunks_exact(4)`?**
Clippy lint `chunks-exact-to-as-chunks` (Rust 1.98+): `as_chunks` is a const-generic API that the compiler can prove is always aligned/complete, enabling better optimisation. `chunks_exact` requires a runtime remainder check.

**Why atomic rename for output files?**
`std::fs::rename` on POSIX is guaranteed atomic when source and destination are on the same filesystem. By writing to a `.tmp_{pid}_{uuid}` file in the same directory and then renaming, a crash or power loss can never leave a partially-written output file in place.

**Why collision-safe naming instead of overwrite?**
ImageForge follows a strict non-destructive policy. Overwriting an existing file — even a previous ImageForge output — could destroy data. The `-compressed-1`, `-compressed-2` suffix scheme makes every run idempotent and reversible.

---

## 12. Test strategy

```
src-tauri/src/
  validation.rs  (unit, #[cfg(test)])
    ✓ verify_dimensions success
    ✓ verify_dimensions mismatch
    ✓ calculate_savings: savings, no savings, conversion exception

  output.rs  (unit, #[cfg(test)])
    ✓ collision resolution: photo-compressed.png → photo-compressed-1.png
    ✓ atomic_write_file: temp → rename → readable

src-tauri/tests/
  common/mod.rs              -- in-process synthetic fixture generator
    Generates: JPEG 800×600 @q98, PNG RGB 400×300, PNG RGBA 400×300,
               WebP RGBA 500×500 @q98, HEIC 320×240 @q98

  compression_pipeline_test.rs  (integration)
    ✓ JPEG: all 3 presets, dimension invariant, independent decode
    ✓ PNG: lossless round-trip, dimension invariant
    ✓ WebP: lossy round-trip, dimension invariant
    ✓ HEIC: lossy round-trip, dimension invariant
    ✓ PNG RGBA → JPG: transparent pixels composite to #FFFFFF
    ✓ WebP RGBA → JPG: transparent pixels composite to #FFFFFF
    ✓ HEIC → JPG: correct output dimensions
    ✓ collision: 3 sequential runs → -compressed / -compressed-1 / -compressed-2
    ✓ NO_SAVINGS: output_path always Some, output bytes == original bytes
```

All tests run with real codec libraries (no mocks). Fixtures are created in `TempDir` and cleaned up automatically. The test binary links against the same Homebrew dylibs used in production.

Run the full suite:

```sh
export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:$PKG_CONFIG_PATH"
cargo test --manifest-path src-tauri/Cargo.toml
```
