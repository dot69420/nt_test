use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt; // for `collect`
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt; // for `oneshot` and `ready`

use crate::api::{AppState, app};
use crate::executor::MockExecutor;
use crate::job_manager::JobManager;

fn setup() -> (AppState, Arc<MockExecutor>) {
    let executor = Arc::new(MockExecutor::new());
    let job_manager = Arc::new(JobManager::new());
    let state = AppState {
        job_manager,
        executor: executor.clone(),
    };
    (state, executor)
}

#[tokio::test]
async fn test_health_check() {
    let (state, _) = setup();
    let app = app(state);

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"OK");
}

#[tokio::test]
async fn test_trigger_nmap_valid() {
    let (state, _) = setup();
    let app = app(state);

    let payload = json!({
        "target": "127.0.0.1",
        "profile": {
            "name": "Quick",
            "description": "desc",
            "flags": ["-F"],
            "requires_root": false
        },
        "custom_ports": null,
        "skip_discovery": false,
        "extra_args": null,
        "use_sudo": false
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/scan/nmap")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["status"], "Running");
    assert!(body["id"].is_number());
}

#[tokio::test]
async fn test_trigger_nmap_invalid_target() {
    let (state, _) = setup();
    let app = app(state);

    let payload = json!({
        "target": "invalid; target",
        "profile": {
            "name": "Quick",
            "description": "desc",
            "flags": ["-F"],
            "requires_root": false
        },
        "custom_ports": null,
        "skip_discovery": false,
        "extra_args": null,
        "use_sudo": false
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/scan/nmap")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_trigger_fuzz_valid() {
    let (state, _) = setup();
    let app = app(state);

    let payload = json!({
        "target": "http://127.0.0.1/FUZZ",
        "wordlist": "wordlists/common.txt",
        "extra_args": null
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/scan/fuzz")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_trigger_fuzz_missing_keyword() {
    let (state, _) = setup();
    let app = app(state);

    let payload = json!({
        "target": "http://127.0.0.1/no-keyword",
        "wordlist": "wordlists/common.txt",
        "extra_args": null
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/scan/fuzz")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&body).contains("must contain 'FUZZ'"));
}

#[tokio::test]
async fn test_list_jobs() {
    let (state, _) = setup();
    let app = app(state);

    // Initially empty
    let response = app
        .oneshot(Request::builder().uri("/jobs").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let jobs: Value = serde_json::from_slice(&body).unwrap();
    assert!(jobs.as_array().unwrap().is_empty());
}
