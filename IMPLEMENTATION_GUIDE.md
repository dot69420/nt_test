# PURPL: Implementation Guide & Development Manual

This document provides a comprehensive guide to the architecture, implementation details, and step-by-step build process of the `purpl` Network Testing & Automation Tool.

## 1. Core Philosophy & Architecture

`purpl` is designed as a unified command-line interface (CLI) that acts as an intelligent wrapper around industry-standard security tools (`nmap`, `wifite`, `tcpdump`, `gobuster`, `hydra`, `searchsploit`).

### Key Design Principles:
1.  **Safety & Validation:** Never run a dangerous command without validation. Checks root privileges abstractly via `CommandExecutor`.
2.  **Profile-Based Execution:** Instead of asking users for raw flags, offer "Profiles" (e.g., "Stealth Scan", "WPS Only").
3.  **Unified Reporting:** Consolidate outputs (XML, JSON, Text) into a single, human-readable terminal report.
4.  **Persistence:** Save scan history and results automatically to a structured directory hierarchy.
5.  **Testability:** All components use Dependency Injection (`CommandExecutor`, `IoHandler`) to allow unit testing without executing real commands or blocking on user input.

### Directory Structure
```
purpl/
├── src/
│   ├── main.rs       # Entry point, interactive loop, and dependency wiring
│   ├── executor.rs   # Command execution abstraction (Shell, Docker, Mock)
│   ├── io_handler.rs # Input/Output abstraction (Real, Mock, Capturing)
│   ├── job_manager.rs # Thread management and job tracking
│   ├── nmap.rs       # Nmap execution logic & profiles
│   ├── web.rs        # Web Enumeration (Gobuster)
│   ├── fuzzer.rs     # Web Fuzzing (Ffuf)
│   ├── exploit.rs    # Active Exploitation (SQLMap, Curl)
│   ├── search_exploit.rs # Searchsploit logic
│   ├── brute.rs      # Credential Access (Hydra)
│   ├── poison.rs     # LAN Poisoning (Responder)
│   ├── wifi.rs       # Wifite execution logic & profiles
│   ├── bluetooth.rs  # Bluetooth Discovery & Attacks
│   ├── sniffer.rs    # Tcpdump logic with live stream parsing
│   ├── report.rs     # Report parsing (XML, JSON, TXT) & display
│   ├── history.rs    # History tracking (JSON based)
│   ├── api.rs        # REST API Server
│   └── dashboard.rs  # Terminal UI for job monitoring
├── scans/            # Output directory
│   ├── <target_ip>/  # For Nmap scans
│   ├── web/          # For Gobuster results
│   ├── brute/        # For Hydra results
│   ├── poison/       # For Responder logs
│   ├── bluetooth/    # For Bluetooth scans
│   ├── wifi/         # For Wifi audits
│   └── packets/      # For Packet captures
└── Cargo.toml        # Dependencies
```

---

## 2. Dependencies

The project relies on the following Rust crates:
- `clap`: Command-line argument parsing.
- `colored`: Terminal text coloring.
- `indicatif`: Progress bars and spinners.
- `regex`: Output parsing.
- `serde` & `serde_json`: Serialization for history and report parsing.
- `roxmltree`: Lightweight XML parsing for Nmap results.
- `chrono`: Date and time formatting.
- `libc`: System calls (checking root privileges in RealExecutor).
- `axum` & `tokio`: For the REST API server and async runtime.

---

## 3. Implemented Modules

### Phase 1: Foundation & Network Recon (`nmap.rs`)
*   **Tool:** `nmap`
*   **Profiles:** "Stealth", "Intense", "Mass Scan".
*   **Logic:** Uses `execute_streamed` for real-time output capture.
*   **Output:** `scans/<target>/<date>/`.

### Phase 2: Web Arsenal (`web.rs`, `fuzzer.rs`)
*   **Tools:** `gobuster`, `ffuf`.
*   **Logic:** Validates URL, auto-detects wordlists, profiles.
*   **Output:** `scans/web/<target>/<date>/`.

### Phase 3: Exploitation Hub (`search_exploit.rs`, `exploit.rs`, `brute.rs`)
*   **Tools:** `searchsploit`, `sqlmap`, `curl`, `hydra`.
*   **Logic:** Parses Nmap XML for auto-correlation, interactive request builder for Curl.
*   **Output:** Terminal display and `scans/exploit/`.

### Phase 4: Network Operations (`poison.rs`, `sniffer.rs`)
*   **Tools:** `responder`, `tcpdump`.
*   **Logic:**
    - **Responder:** Requires Root. Moves logs after execution.
    - **Tcpdump:** Live stream parsing (Source, Dest, Proto) or Passive capture (.pcap).
*   **Output:** `scans/poison/` and `scans/packets/`.

### Phase 5: Wireless & RF (`wifi.rs`, `bluetooth.rs`)
*   **Tools:** `wifite`, `bluez-utils`.
*   **Logic:**
    - **Wifite:** Automates `airmon-ng` setup/teardown. Requires hardware access (local execution preferred).
    - **Bluetooth:** Discovery (`hcitool`), Enumeration (`sdptool`), Stress (`l2ping`).
*   **Output:** `scans/wifi/` and `scans/bluetooth/`.

### Phase 6: Core Infrastructure (`job_manager.rs`, `api.rs`)
*   **Job Manager:** Handles threaded execution, cancellation, and output buffering.
*   **API Server:** Exposes functionality via HTTP endpoints.
*   **Docker:** Abstracted execution via `DockerExecutor`.

---

## 4. API Implementation Plan
The REST API in `src/api.rs` is expanded to cover all CLI capabilities.

### 4.1. Web Fuzzing (`POST /scan/fuzz`)
- **Request Body:** `FuzzerConfig` struct (target URL, wordlist path, flags).
- **Validation:** URL format check, valid flags for `ffuf`.
- **Logic:** Spawns a background job wrapping `fuzzer::execute_fuzzer`.

### 4.2. Exploitation Endpoints
- **`POST /exploit/search`**:
    - **Input:** Search query string.
    - **Logic:** Wraps `search_exploit::run_searchsploit`.
- **`POST /exploit/active`**:
    - **Input:** `ExploitConfig` (target, tool selection: sqlmap/curl).
    - **Logic:** Wraps `exploit::execute_exploitation`.
- **`POST /exploit/brute`**:
    - **Input:** `BruteConfig` (target, service, username/password lists).
    - **Logic:** Wraps `brute::run_brute_force`.

### 4.3. Network Operations
- **`POST /netops/sniff`**:
    - **Input:** Interface name, capture duration.
    - **Logic:** Wraps `sniffer::run_sniffer`.
- **`POST /netops/poison`**:
    - **Input:** Interface name.
    - **Logic:** Wraps `poison::run_poisoning`.

### 4.4. File Access
- **`GET /storage/scans`**: Lists directories in `./scans`.
- **`GET /storage/download/*path`**: Streams file content from `./scans`.

---

## 5. Building & Running

### Build
```bash
cargo build --release
```

### Run
```bash
./target/release/purpl
```

### Testing
Run the comprehensive unit test suite:
```bash
cargo test
```
The project maintains >60% code coverage.
