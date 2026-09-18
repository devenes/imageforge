# Security Policy

ImageForge is committed to ensuring the safety, privacy, and integrity of users' data and systems. As a local-first desktop application, ImageForge prioritizes privacy by architecture: your images never leave your computer.

---

## Supported Versions

Security updates and patches are provided for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

We strongly recommend always running the latest release of ImageForge.

---

## Architectural Security Guarantees

### 1. Local-First & Zero Outbound Network Requests
- **No Remote Processing**: All image decoding, compression, optimization, and conversion take place 100% locally on your machine using native binaries.
- **Zero Telemetry or Analytics**: ImageForge does not collect user data, tracking identifiers, usage metrics, or error reports.
- **Air-Gap Compatible**: ImageForge operates with full functionality when disconnected from the internet.

### 2. Non-Destructive File Operations
- **Preservation by Default**: ImageForge never overwrites source images.
- **Collision Avoidance**: If an output filename already exists (e.g., `image-compressed.jpg`), ImageForge automatically appends an incremental counter (e.g., `image-compressed-1.jpg`, `image-compressed-2.jpg`).
- **Atomic Writes**: Output files are written to temporary scratch files (`.tmp_<pid>_<uuid>`) within the destination directory and committed using atomic filesystem renames (`fs::rename`). In the event of a crash, power loss, or user cancellation, partial or corrupted files are never left in place of valid outputs.

### 3. Memory & Input Hardening
- **Dimension Verification**: Output dimensions are strictly validated against original dimensions twice per image (after encoding and upon independent re-decode) to ensure image dimensions are preserved and avoid buffer overflows.
- **Native Memory Safety**: The core application logic and queue coordination are built in safe Rust.
- **Resource Bounds**: Concurrency is automatically clamped (`cpus.clamp(1, 6)`) to prevent CPU starvation and denial-of-service conditions during bulk processing.

---

## Reporting a Vulnerability

If you discover a security vulnerability in ImageForge, please report it responsibly:

### How to Report
1. **GitHub Security Advisory**: Submit a report privately via [GitHub Security Advisories](https://github.com/devenes/imageforge/security/advisories/new).
2. **Email Disclosure**: Alternatively, email the maintainers at `security@imageforge.app` with:
   - A clear description of the vulnerability and affected component.
   - Step-by-step reproduction instructions or a minimal proof-of-concept (PoC).
   - Any potential impact on user data or system integrity.
   - (Optional) Suggested remediation or patch.

> [!IMPORTANT]
> Please **do not** report security vulnerabilities through public GitHub issues, discussions, or social media channels until a fix has been coordinated and released.

---

## Response Timeline

- **Initial Acknowledgment**: Within 48 hours of receiving your report.
- **Assessment & Triage**: Within 5 business days, confirming whether the issue is reproducible and determining severity.
- **Fix & Patch**: A fix will be developed, tested, and released as an advisory and software update as quickly as possible.
- **Public Disclosure**: Coordinated disclosure will occur after a patch is published, crediting the reporter (unless anonymity is requested).

---

## Third-Party Dependencies

ImageForge relies on vetted, industry-standard image processing libraries:
- `libjpeg-turbo` for JPEG compression and decompression
- `oxipng` for lossless PNG optimization
- `libwebp` for WebP encoding and decoding
- `libheif` for HEIC/HEIF decoding and HEVC compression
- `Tauri 2.x` and `WKWebView` for application windowing and native integration

If a vulnerability is identified in an upstream dependency, we will promptly update our dependencies and publish a new release.
