# purpl (Purple Team Helper Tool)

A powerful, interactive, and automated CLI tool for Red Teaming and Network Security auditing, written in Rust. `purpl` acts as a central control proxy, orchestrating industry-standard tools like Nmap, Gobuster, and Wifite into a streamlined, safe, and efficient workflow.

## Key Features

*   **Multi-Tool Orchestration:**
    *   **Nmap:** Automated scanning profiles (Stealth, Quick, Intense, Paranoid) with intelligent optimization for large networks.
    *   **Wifite:** Automated WiFi auditing wrapper with monitor mode handling.
    *   **Tcpdump:** Integrated packet sniffing with real-time analysis.
    *   **Web Arsenal:** Gobuster for enumeration and Ffuf for fuzzing.
    *   **Exploitation Hub:** Searchsploit integration and active exploitation tools (SQLMap, Hydra).
    *   **Network Ops:** LAN Poisoning (Responder) and Bluetooth attacks.
*   **Job Management:**
    *   **"Always a Job":** Every task is a managed job that can run in the foreground or background.
    *   **Dashboard:** Monitor active jobs and view historical results in a unified interface.
*   **Container Support:**
    *   **Docker Integration:** Run tools inside a Docker container for isolation and portability (`--container`).
    *   **Hybrid Execution:** Automatically switch between local (hardware-dependent tools like Wifite) and containerized execution.
*   **API Server:**
    *   **REST API:** Expose `purpl` functionality via a REST API for remote control and integration (`purpl serve`).
*   **Evasion and Anonymity:**
    *   **Proxychains Integration:** Global proxy support for all compatible tools.
    *   **Smart Privilege Handling:** Automatically detects root requirements and handles `sudo` elevation securely.
*   **Structured Reporting:**
    *   Parses raw tool output (XML, JSON) into human-readable CLI reports.
    *   Organized file structure: `scans/<tool>/<target>/<timestamp>/`.
*   **History and Persistence:**
    *   Automatically logs every scan execution.
    *   Smart input memory (remembers last target across tools).

## Dependencies

Ensure the following tools are installed on your system (or available in the Docker image):

*   **Nmap:** Core network scanner.
*   **Gobuster / Ffuf:** Web enumeration and fuzzing.
*   **Searchsploit:** Exploit database search.
*   **Hydra / SQLMap:** Active exploitation.
*   **Wifite / Airmon-ng:** WiFi auditing (requires hardware access).
*   **Tcpdump / Responder:** Network sniffing and poisoning.
*   **BlueZ Utils:** Bluetooth tools (`hcitool`, `l2ping`, etc.).
*   **Docker:** (Optional) For containerized execution.

## Usage

**Interactive Mode:**
```bash
./purpl
```

**CLI One-Liners:**
```bash
# Stealth scan a target
./purpl --nmap 192.168.1.10

# Scan through Proxychains
./purpl --nmap 10.0.0.0/8 --proxy

# Run WiFi Audit
./purpl --wifite wlan0

# Run in Docker Container
./purpl --nmap 192.168.1.10 --container

# Start API Server
./purpl serve --port 3000
```

## Build

```bash
cargo build --release
```
The binary will be located at `target/release/purpl`.

## Testing

Run the unit tests:
```bash
cargo test
```

## Roadmap

*   [x] Core Network Recon (Nmap)
*   [x] Web Enumeration (Gobuster, Ffuf)
*   [x] Exploitation Hub (Searchsploit, Hydra, SQLMap)
*   [x] Network Ops (Sniffer, Poisoning)
*   [x] Wireless & RF (WiFi, Bluetooth)
*   [x] Job Manager & Dashboard
*   [x] Docker Container Support
*   [x] REST API Server
*   [ ] Reporting Export (PDF/HTML)
*   [ ] Plugin System for custom tools