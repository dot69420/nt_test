use axum::{
    Json, Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use serde::Serialize;
use std::fs;
use std::path::Path as StdPath;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::brute::{self, BruteConfig};
use crate::executor::CommandExecutor;
use crate::exploit::{self, ExploitConfig};
use crate::fuzzer::{self, FuzzerConfig};
use crate::job_manager::JobManager;
use crate::nmap::{self, NmapConfig};
use crate::poison::{self, PoisonConfig};
use crate::search_exploit::{self, SearchExploitConfig};
use crate::sniffer::{self, SnifferConfig};
use crate::validation::{validate_nmap_flags, validate_target, validate_web_flags};
use crate::web::{self, WebConfig};

#[derive(Clone)]
pub struct AppState {
    pub job_manager: Arc<JobManager>,
    pub executor: Arc<dyn CommandExecutor + Send + Sync>,
}

#[derive(Serialize)]
pub struct JobResponse {
    pub id: usize,
    pub name: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct JobDetails {
    pub id: usize,
    pub name: String,
    pub status: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub output: String,
}

pub async fn serve(
    port: u16,
    job_manager: Arc<JobManager>,
    executor: Arc<dyn CommandExecutor + Send + Sync>,
) {
    let state = AppState {
        job_manager,
        executor,
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/jobs", get(list_jobs))
        .route("/jobs/:id", get(get_job))
        .route("/scan/nmap", post(trigger_nmap))
        .route("/scan/web", post(trigger_web))
        .route("/scan/fuzz", post(trigger_fuzz))
        .route("/exploit/search", post(trigger_exploit_search))
        .route("/exploit/active", post(trigger_exploit_active))
        .route("/exploit/brute", post(trigger_exploit_brute))
        .route("/netops/sniff", post(trigger_netops_sniff))
        .route("/netops/poison", post(trigger_netops_poison))
        .route("/storage/scans", get(list_scans_files))
        .route("/storage/download/*path", get(download_file))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("Server listening on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_jobs(State(state): State<AppState>) -> Json<Vec<JobResponse>> {
    let jobs = state.job_manager.list_jobs();
    let responses = jobs
        .iter()
        .map(|j| {
            let status = j.status.lock().unwrap();
            JobResponse {
                id: j.id,
                name: j.name.clone(),
                status: format!("{:?}", *status),
            }
        })
        .collect();
    Json(responses)
}

async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<usize>,
) -> Result<Json<JobDetails>, StatusCode> {
    if let Some(job) = state.job_manager.get_job(id) {
        let status = job.status.lock().unwrap();
        let end_time = job.end_time.lock().unwrap();

        let details = JobDetails {
            id: job.id,
            name: job.name.clone(),
            status: format!("{:?}", *status),
            start_time: job.start_time.clone(),
            end_time: end_time.clone(),
            output: job.io.get_output(),
        };
        Ok(Json(details))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn trigger_nmap(
    State(state): State<AppState>,
    Json(config): Json<NmapConfig>,
) -> Result<Json<JobResponse>, (StatusCode, String)> {
    // Input Validation
    if let Err(e) = validate_target(&config.target) {
        return Err((StatusCode::BAD_REQUEST, format!("Invalid Target: {}", e)));
    }
    if let Err(e) = validate_nmap_flags(&config.profile.flags) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid Profile Flags: {}", e),
        ));
    }
    if let Some(extras) = &config.extra_args {
        // Naive split, but good enough to check individual tokens
        let parts: Vec<String> = extras.split_whitespace().map(|s| s.to_string()).collect();
        if let Err(e) = validate_nmap_flags(&parts) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid Extra Args: {}", e),
            ));
        }
    }

    let name = format!("API Nmap {}", config.target);
    let job_name = name.clone();

    let job = state.job_manager.spawn_job(
        &name,
        move |exec, io, job| {
            nmap::execute_nmap_scan(config, false, &*exec, io, Some(job));
        },
        state.executor.clone(),
        true,
    );

    let status = job.status.lock().unwrap();
    Ok(Json(JobResponse {
        id: job.id,
        name: job_name,
        status: format!("{:?}", *status),
    }))
}

async fn trigger_web(
    State(state): State<AppState>,
    Json(config): Json<WebConfig>,
) -> Result<Json<JobResponse>, (StatusCode, String)> {
    // Input Validation
    if let Err(e) = validate_target(&config.target) {
        return Err((StatusCode::BAD_REQUEST, format!("Invalid Target: {}", e)));
    }
    if let Err(e) = validate_web_flags(&config.profile.flags) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid Profile Flags: {}", e),
        ));
    }
    if let Some(extras) = &config.extra_args {
        let parts: Vec<String> = extras.split_whitespace().map(|s| s.to_string()).collect();
        if let Err(e) = validate_web_flags(&parts) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid Extra Args: {}", e),
            ));
        }
    }

    let name = format!("API WebEnum {}", config.target);
    let job_name = name.clone();

    let job = state.job_manager.spawn_job(
        &name,
        move |exec, io, _job| {
            web::execute_web_enum(config, false, &*exec, io);
        },
        state.executor.clone(),
        true,
    );

    let status = job.status.lock().unwrap();
    Ok(Json(JobResponse {
        id: job.id,
        name: job_name,
        status: format!("{:?}", *status),
    }))
}

async fn trigger_fuzz(
    State(state): State<AppState>,
    Json(config): Json<FuzzerConfig>,
) -> Result<Json<JobResponse>, (StatusCode, String)> {
    if let Some(extras) = &config.extra_args {
        let parts: Vec<String> = extras.split_whitespace().map(|s| s.to_string()).collect();
        if let Err(e) = validate_web_flags(&parts) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid Extra Args: {}", e),
            ));
        }
    }

    if !config.target.contains("FUZZ") {
        return Err((
            StatusCode::BAD_REQUEST,
            "Target URL must contain 'FUZZ'".to_string(),
        ));
    }

    let name = format!("API Fuzz {}", config.target);
    let job_name = name.clone();

    let job = state.job_manager.spawn_job(
        &name,
        move |exec, io, _job| {
            fuzzer::execute_fuzzer(config, false, &*exec, io);
        },
        state.executor.clone(),
        true,
    );

    let status = job.status.lock().unwrap();
    Ok(Json(JobResponse {
        id: job.id,
        name: job_name,
        status: format!("{:?}", *status),
    }))
}

async fn trigger_exploit_search(
    State(state): State<AppState>,
    Json(config): Json<SearchExploitConfig>,
) -> Result<Json<JobResponse>, (StatusCode, String)> {
    let name = format!("API SearchSploit {}", config.query);
    let job_name = name.clone();

    let job = state.job_manager.spawn_job(
        &name,
        move |exec, io, _job| {
            search_exploit::execute_searchsploit(config, false, &*exec, io);
        },
        state.executor.clone(),
        true,
    );

    let status = job.status.lock().unwrap();
    Ok(Json(JobResponse {
        id: job.id,
        name: job_name,
        status: format!("{:?}", *status),
    }))
}

async fn trigger_exploit_active(
    State(state): State<AppState>,
    Json(config): Json<ExploitConfig>,
) -> Result<Json<JobResponse>, (StatusCode, String)> {
    let name = "API Active Exploit".to_string();
    let job_name = name.clone();

    let job = state.job_manager.spawn_job(
        &name,
        move |exec, io, _job| {
            exploit::execute_exploitation(config, false, &*exec, io);
        },
        state.executor.clone(),
        true,
    );

    let status = job.status.lock().unwrap();
    Ok(Json(JobResponse {
        id: job.id,
        name: job_name,
        status: format!("{:?}", *status),
    }))
}

async fn trigger_exploit_brute(
    State(state): State<AppState>,
    Json(config): Json<BruteConfig>,
) -> Result<Json<JobResponse>, (StatusCode, String)> {
    if let Err(e) = validate_target(&config.target) {
        return Err((StatusCode::BAD_REQUEST, format!("Invalid Target: {}", e)));
    }

    let name = format!("API Hydra {}", config.target);
    let job_name = name.clone();

    let job = state.job_manager.spawn_job(
        &name,
        move |exec, io, _job| {
            brute::execute_brute_force(config, false, &*exec, io);
        },
        state.executor.clone(),
        true,
    );

    let status = job.status.lock().unwrap();
    Ok(Json(JobResponse {
        id: job.id,
        name: job_name,
        status: format!("{:?}", *status),
    }))
}

async fn trigger_netops_sniff(
    State(state): State<AppState>,
    Json(config): Json<SnifferConfig>,
) -> Result<Json<JobResponse>, (StatusCode, String)> {
    let name = format!("API Sniffer {}", config.interface);
    let job_name = name.clone();

    let job = state.job_manager.spawn_job(
        &name,
        move |exec, io, _job| {
            sniffer::execute_sniffer(config, false, &*exec, io);
        },
        state.executor.clone(),
        true,
    );

    let status = job.status.lock().unwrap();
    Ok(Json(JobResponse {
        id: job.id,
        name: job_name,
        status: format!("{:?}", *status),
    }))
}

async fn trigger_netops_poison(
    State(state): State<AppState>,
    Json(config): Json<PoisonConfig>,
) -> Result<Json<JobResponse>, (StatusCode, String)> {
    let name = format!("API Poison {}", config.interface);
    let job_name = name.clone();

    let job = state.job_manager.spawn_job(
        &name,
        move |exec, io, _job| {
            poison::execute_poisoning(config, false, &*exec, io);
        },
        state.executor.clone(),
        true,
    );

    let status = job.status.lock().unwrap();
    Ok(Json(JobResponse {
        id: job.id,
        name: job_name,
        status: format!("{:?}", *status),
    }))
}

async fn list_scans_files() -> Json<Vec<String>> {
    let root = StdPath::new("scans");
    let mut files = Vec::new();

    if root.exists() {
        visit_dirs(root, &mut files, root);
    }

    Json(files)
}

fn visit_dirs(dir: &StdPath, files: &mut Vec<String>, root: &StdPath) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, files, root);
            } else if let Ok(relative) = path.strip_prefix(root) {
                if let Some(s) = relative.to_str() {
                    files.push(s.to_string());
                }
            }
        }
    }
}

async fn download_file(Path(path): Path<String>) -> impl IntoResponse {
    // Basic sanitization
    if path.contains("..") || path.starts_with('/') || path.contains('\\') {
        return (StatusCode::BAD_REQUEST, "Invalid path").into_response();
    }

    let file_path = StdPath::new("scans").join(&path);

    if !file_path.exists() || !file_path.is_file() {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    match tokio::fs::read(&file_path).await {
        Ok(content) => {
            let mime = "application/octet-stream";
            let headers = [(header::CONTENT_TYPE, mime)];
            (StatusCode::OK, headers, content).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response(),
    }
}
