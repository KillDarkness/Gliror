
use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use crate::cluster::{ClusterCoordinator, WorkerState, WorkerStatus, WorkerProgress, AttackCommand};

async fn root() -> &'static str {
    "Master Server is running"
}

async fn register_worker(
    axum::extract::State(coordinator): axum::extract::State<Arc<ClusterCoordinator>>,
    Json(payload): Json<WorkerState>,
) -> impl IntoResponse {
    let worker_id = payload.id.clone();
    println!("{} Worker {} connected.", colored::Colorize::blue("INFO"), worker_id);
    let mut workers = coordinator.workers.write().await;
    if workers.contains_key(&worker_id) {
        println!("{} Worker {} reconnected.", colored::Colorize::blue("INFO"), worker_id);
    } else {
        println!("{} Worker {} connected.", colored::Colorize::blue("INFO"), worker_id);
    }
    let mut worker_state = payload;
    worker_state.last_seen = Some(Instant::now());
    workers.insert(worker_id.clone(), worker_state);
    println!("{} Current workers in coordinator: {}", colored::Colorize::blue("INFO"), workers.len());
    (StatusCode::CREATED, Json(worker_id))
}

async fn update_worker_status(
    axum::extract::State(coordinator): axum::extract::State<Arc<ClusterCoordinator>>,
    axum::extract::Path(worker_id): axum::extract::Path<String>,
    Json(status): Json<WorkerStatus>,
) -> impl IntoResponse {
    let mut workers = coordinator.workers.write().await;
    if let Some(worker) = workers.get_mut(&worker_id) {
        worker.status = status;
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn update_worker_progress(
    axum::extract::State(coordinator): axum::extract::State<Arc<ClusterCoordinator>>,
    axum::extract::Path(worker_id): axum::extract::Path<String>,
    Json(progress): Json<WorkerProgress>,
) -> impl IntoResponse {
    let mut workers = coordinator.workers.write().await;
    if let Some(worker) = workers.get_mut(&worker_id) {
        worker.progress = progress;
        worker.last_seen = Some(Instant::now());
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn get_cluster_status(
    axum::extract::State(coordinator): axum::extract::State<Arc<ClusterCoordinator>>,
) -> impl IntoResponse {
    let workers = coordinator.workers.read().await;
    Json(workers.values().cloned().collect::<Vec<_>>()).into_response()
}


    async fn start_attack(
    axum::extract::State(coordinator): axum::extract::State<Arc<ClusterCoordinator>>,
    Json(payload): Json<AttackCommand>,
) -> impl IntoResponse {
    let mut attack_command = coordinator.attack_command.write().await;
    *attack_command = Some(payload);
    println!("{} Attack command received and stored.", colored::Colorize::blue("INFO"));
    (StatusCode::OK, "Attack initiated")
}

async fn get_task(
    axum::extract::Path(worker_id): axum::extract::Path<String>,
    axum::extract::State(coordinator): axum::extract::State<Arc<ClusterCoordinator>>,
) -> impl IntoResponse {
    // Update worker's last_seen timestamp
    {
        let mut workers = coordinator.workers.write().await;
        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.last_seen = Some(Instant::now());
        }
    }

    let attack_command_guard = coordinator.attack_command.read().await;
    let attack_command_option = attack_command_guard.clone();

    if let Some(command) = attack_command_option.clone() {
        let workers = coordinator.workers.read().await;
        let total_workers = workers.len() as u32;
        let mut distributed_command = command.clone();
        if let Some(concurrent) = distributed_command.concurrent {
            distributed_command.concurrent = Some((concurrent / total_workers).max(1));
        }
        (StatusCode::OK, Json(Some(distributed_command)))
    } else {
        (StatusCode::OK, Json(None::<AttackCommand>))
    }
}

pub async fn run_master_server(coordinator: Arc<ClusterCoordinator>, port: u16) {
    let app = Router::new()
        .route("/", get(root))
        .route("/workers", post(register_worker))
        .route("/workers/:id/status", post(update_worker_status))
        .route("/workers/:id/progress", post(update_worker_progress))
        .route("/status", get(get_cluster_status))
        .route("/attack", post(start_attack))
        .route("/workers/:id/task", get(get_task))
        .with_state(coordinator);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("{} Failed to bind to port {}: {}", colored::Colorize::red("ERROR"), port, e);
            std::process::exit(1);
        }
    };
    
    println!("{} Master server successfully started on http://127.0.0.1:{}", colored::Colorize::blue("INFO"), port);
    
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("{} Server error: {}", colored::Colorize::red("ERROR"), e);
    }
}
