# Purpl: Docker & API Implementation Roadmap

## 1. Overview
This document outlines the roadmap for achieving full feature parity across the Docker container environment and the REST API. The goal is to ensure `purpl` can function as a robust, remote-controlled security agent.

## 2. Docker Image Completeness
The current `Dockerfile` lacks several tools required by the `exploit`, `wifi`, and `bluetooth` modules.

### Checklist
- [x] **SQLMap:** Add `sqlmap` to `apt-get install`.
- [x] **Wireless Tools:** Add `wifite`, `aircrack-ng`, `wireless-tools`, `pciutils` (for hardware detection inside container).
- [x] **Bluetooth:** Add `bluez`, `bluez-tools`.
- [x] **Wordlists:** Ensure basic wordlists (SecLists or similar) are present or downloaded.

**Effort:** Low (30-60 mins)

## 3. API Expansion
The current API (`src/api.rs`) only supports `Nmap` and `WebEnum`. We need to expose the remaining modules.

### Checklist
- [x] **Web Fuzzing:** Add `POST /scan/fuzz` endpoint (wraps `fuzzer::execute_fuzzer`).
- [x] **Exploitation:**
    - [x] Add `POST /exploit/search` (wraps `search_exploit`).
    - [x] Add `POST /exploit/active` (wraps `exploit::execute_exploitation`).
    - [x] Add `POST /exploit/brute` (wraps `brute::run_brute_force`).
- [x] **Network Ops:**
    - [x] Add `POST /netops/sniff` (wraps `sniffer::run_sniffer` - needs duration/packet limit).
    - [x] Add `POST /netops/poison` (wraps `poison::run_poisoning` - needs duration/cancellation).
- [x] **Wireless:** (Skipped - Incompatible with Dockerized API context)
    - [ ] ~~Add `POST /wifi/audit`~~
    - [ ] ~~Add `POST /bluetooth/scan`~~

**Effort:** Medium (3-4 hours)

## 4. File Access API
The current API returns job status and stdout. However, tools like Nmap and Gobuster generate valuable report files in the `scans/` directory. The API needs to provide access to these artifacts.

### Checklist
- [x] **List Scans:** Add `GET /storage/scans` to list generated directories/files.
- [x] **Download File:** Add `GET /storage/download/*path` to download specific report files (e.g., `scans/nmap/10.10.10.10/scan.xml`).

**Effort:** Medium (1-2 hours)

## 5. Hardware & Hybrid Strategy
We need to clarify the "Purpl in Docker" vs "Purpl on Host" execution models, especially for hardware-dependent tools (Wifi/Bluetooth).

- **Scenario A (Purpl on Host):** `HybridExecutor` works as designed.
- **Scenario B (Purpl in Docker):**
    - `ShellExecutor` runs inside the container.
    - Container needs `--privileged` and `--net=host` to access hardware.
    - `Dockerfile` needs all tools installed (addressed in Section 2).

### Tasks
- [x] Update `src/executor.rs` to detect if running inside a container (check `/.dockerenv`) and adjust `HybridExecutor` logic if necessary (though current logic might be fine if tools are present).
    - *Note:* `ShellExecutor` correctly handles tool execution inside the container. Explicit detection is not required for the API server context.
- [x] Document the `docker run` command required for hardware access (`--privileged`, etc.).

**Effort:** Low (1 hour)

## 6. Implementation Timeline

| Phase | Task | Estimate |
| :--- | :--- | :--- |
| **1** | Update Dockerfile & Build | 1 hr |
| **2** | Implement Missing API Endpoints | 3 hrs |
| **3** | Implement File Access API | 2 hrs |
| **4** | Testing & Validation | 2 hrs |
| **Total** | | **~8 Hours** |

## 7. Next Steps
1.  [x] Review and approve this roadmap.
2.  [x] Begin with **Phase 1 (Docker Update)**.
3.  [x] Proceed to **Phase 2 (API Expansion)**.
4.  [x] Proceed to **Phase 4 (Testing & Validation)**.
5.  Proceed to update storage method (Phase 3 on hold).
