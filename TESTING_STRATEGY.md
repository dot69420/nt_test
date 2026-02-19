# Testing Strategy & Implementation Plan

## 1. Overview
This document outlines the strategy for testing the new API and Docker integration features of `purpl`. The goal is to ensure stability and correctness through a combination of unit tests, functional tests, and integration tests.

## 2. Test Types & Scope

### 2.1. Unit Tests (Rust)
**Scope:** `src/api.rs`, `src/executor.rs`, and configuration parsing.
**Goal:** Verify that API handlers correctly parse JSON inputs, validate data, and trigger the appropriate internal functions.
**Tools:** `axum::test`, `tokio::test`, `mockall` (or existing `MockExecutor`).

*   **API Handlers:**
    *   Test `POST /scan/nmap` with valid/invalid payloads.
    *   Test `POST /scan/fuzz` ensuring `FUZZ` keyword check works.
    *   Test `POST /exploit/*` endpoints for correct routing.
    *   Test `GET /storage/*` for directory traversal prevention.
*   **Executor Logic:**
    *   Verify `DockerExecutor` builds correct `docker run` command strings (using a test harness that captures the command without running it).

### 2.2. Functional Tests (Docker Jobs)
**Scope:** `src/executor.rs` -> `DockerExecutor`.
**Goal:** Ensure that the application *can* invoke the Docker CLI correctly.
**Strategy:**
    *   Since running actual Docker containers in CI/Unit tests is complex/slow, we will rely on **Command Construction Testing**.
    *   We will verify that `DockerExecutor` produces the expected argument list (e.g., `--net=host`, volume mounts) for a given tool command.

### 2.3. Integration Tests (End-to-End)
**Scope:** Running `purpl serve` and `purpl --container`.
**Goal:** Verify the full stack works in a real environment.
**Strategy:**
    *   **Manual/Scripted Verification:** Start the API server and use `curl` to trigger jobs.
    *   **Container Check:** Verify `docker run` is actually called (if testing `DockerExecutor` on a host with Docker).

## 3. Implementation Plan

### Phase 1: API Unit Tests (Estimated Time: 3-4 Hours)
Create a new test module `src/api_tests.rs` (or add to `src/api.rs`).

*   [ ] **Setup:** Create a `mock_state` helper function that returns `AppState` with a `MockExecutor` and `JobManager`.
*   [ ] **Test `trigger_nmap`:**
    *   Send valid JSON -> Expect 200 OK and Job ID.
    *   Send invalid target -> Expect 400 Bad Request.
*   [ ] **Test `trigger_fuzz`:**
    *   Send target without "FUZZ" -> Expect 400.
*   [ ] **Test File Access:**
    *   Try path `../etc/passwd` -> Expect 400/404.
    *   Mock file system access? (Might need to refactor `api.rs` to allow mocking FS, or just test the sanitization logic separately).

### Phase 2: Docker Functional Tests (Estimated Time: 1-2 Hours)
Add tests to `src/executor.rs`.

*   [ ] **Test `DockerExecutor::build_args`:**
    *   Make `build_args` private but testable (or `pub(crate)`).
    *   Assert that `nmap -sS target` becomes `docker run ... nmap -sS target`.
    *   Verify volume mounts are absolute paths.

### Phase 3: Integration/System Check (Estimated Time: 1 Hour)
*   [ ] Create a shell script `test_api.sh` that:
    1.  Starts `purpl serve`.
    2.  Sends `curl` requests to localhost.
    3.  Checks response codes.

## 4. Total Effort Estimate
*   **Total Time:** ~5-7 Hours
*   **Complexity:** Medium (Async testing with Axum and mocking requires some boilerplate).

## 5. Notes on Docker Flag
*   The flag to enable Docker container mode for CLI tools is **`--container`** (e.g., `purpl --nmap 1.1.1.1 --container`).
*   The API Server (`purpl serve`) uses the executor defined in `main.rs`. Currently, `purpl serve` does **not** accept a `--container` flag to switch its *internal* executor to `DockerExecutor`. It uses `ShellExecutor` by default (assuming it is already running inside a container). If we want the API server to spawn *sibling* containers, we might need to expose the `--container` flag to the `serve` command or config. **Decision:** For now, we assume the API server runs *inside* the container and uses `ShellExecutor`.

## 6. Next Steps
1.  Approve this plan.
2.  Begin Phase 1 (API Unit Tests).
