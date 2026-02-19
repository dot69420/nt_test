# PURPL Development Context & Architecture Blueprint

## 1. Project Philosophy
`purpl` is a Rust-based CLI wrapper for offensive security tools. It prioritizes:
- **Safety:** Sudo validation, input sanitization, safe command execution.
- **Usability:** Profile-based execution (no manual flag memorization).
- **Persistence:** Structured output (`scans/<type>/<date>/`).
- **Unified Workflow:** "Always a Job" architecture for consistent task management.
- **Testability:** High code coverage through dependency injection and mocking.

## 2. Architecture Patterns
To ensure reliability and testability, all modules **MUST** adhere to this pattern:

### A. Dependency Injection
All core logic is decoupled from side effects (System commands, IO) via traits defined in `src/executor.rs` and `src/io_handler.rs`.
- **`CommandExecutor`:** Abstracts system command execution (`execute`, `execute_output`, `spawn_stdout`, `is_root`, `execute_cancellable`).
    - **Implementations:**
        - `ShellExecutor`: Local execution.
        - `DockerExecutor`: Execution inside a container.
        - `HybridExecutor`: Routes hardware-specific commands locally and others to Docker.
        - `MockExecutor`: For testing.
- **`IoHandler`:** Abstracts Input/Output operations (`println`, `print`, `read_line`, `read_input`, `flush`).

### B. Job Management (`src/job_manager.rs`)
All long-running tasks are encapsulated as "Jobs".
- **`JobManager`:** Handles spawning threads, tracking status (`Running`, `Completed`, `Failed`), and storing output.
- **`Job` Struct:** Contains thread handle, cancellation token, and output buffer.

### C. Module Structure (`src/<module>.rs`)
1.  **Profiles Enum/Struct:** Define presets (e.g., `Fast`, `Thorough`).
2.  **Configuration Function:** Gather all user inputs *before* execution starts.
    ```rust
    pub fn configure_<module>(...) -> Config
    ```
3.  **Execution Function:**
    ```rust
    pub fn execute_<module>(
        config: Config,
        use_proxy: bool,
        executor: &dyn CommandExecutor,
        io: &dyn IoHandler,
        job: Option<Arc<Job>>
    )
    ```
    - Check for cancellation via `job.is_cancelled()`.
    - Stream output to `io` or `job` buffer.

### D. Integration (`src/main.rs`)
1.  **CLI Arg:** Add `#[arg(long)]` for the new module.
2.  **Menu:** Add entry to `tools` vector in `run_interactive_mode` (or appropriate submenu).
3.  **Dispatch:** Call the module's run function using the real executor and io handler.

### E. Reporting (`src/report.rs`)
1.  **Interactive Viewer:** Use `report::view_results(io)` for the main results menu.
2.  **Detection:** Update `display_scan_report` to check for the new module's output file.
3.  **Parsing:** Implement a specific parser (e.g., `parse_gobuster_output`).

---

## 3. Implemented Modules

### Network Recon (`src/nmap.rs`)
- **Tool:** `nmap`.
- **Features:** Host Discovery, Deep Scan, Class A Optimization (/8 support).

### Web Arsenal (`src/web.rs`, `src/fuzzer.rs`)
- **Tools:** `gobuster`, `ffuf`.
- **Features:** Directory enumeration, Parameter fuzzing.

### Exploitation Hub (`src/exploit.rs`, `src/search_exploit.rs`, `src/brute.rs`)
- **Tools:** `searchsploit`, `hydra`, `sqlmap`, `curl`.
- **Features:** Exploit search, Active exploitation, Credential brute-forcing.

### Network Operations (`src/sniffer.rs`, `src/poison.rs`)
- **Tools:** `tcpdump`, `responder`.
- **Features:** Packet capture (Live/Passive), LAN Poisoning.

### Wireless & RF (`src/wifi.rs`, `src/bluetooth.rs`)
- **Tools:** `wifite`, `hcitool`, `l2ping`.
- **Features:** WiFi auditing, Bluetooth scanning and attacks.

---

## 4. Coding Standards
- **Error Handling:** Use `match` or `if let`. Avoid `unwrap()` on external inputs.
- **Dependencies:** Use `CommandExecutor` trait for ANY system call. Use `IoHandler` for ANY printing/reading.
- **Testing:**
    - Write unit tests for every module in `src/<module>_tests.rs`.
    - Use `MockExecutor` and `MockIoHandler` to simulate environment.
    - Ensure >80% coverage.
- **UI:** Use `colored` via `io.println` for consistent look & feel.