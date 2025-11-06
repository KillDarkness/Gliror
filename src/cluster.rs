use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClusterConfig {
    pub leader_id: String,
    pub workers: Vec<String>,
    pub total_workers: usize,
    pub worker_id: Option<String>,
    pub coordinator_addr: String,
}

impl ClusterConfig {
    pub fn new(total_workers: usize, coordinator_addr: String) -> Self {
        ClusterConfig {
            leader_id: Uuid::new_v4().to_string(),
            workers: vec![],
            total_workers,
            worker_id: None,
            coordinator_addr,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkerState {
    pub id: String,
    pub status: WorkerStatus,
    pub progress: WorkerProgress,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WorkerStatus {
    Initializing,
    Ready,
    Working,
    Paused,
    Completed,
    Error(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkerProgress {
    pub requests_sent: u64,
    pub requests_success: u64,
    pub requests_error: u64,
    pub avg_response_time: f64,
    pub current_rps: f64,
}

pub struct ClusterCoordinator {
    pub config: ClusterConfig,
    pub workers: Arc<RwLock<HashMap<String, WorkerState>>>,
    pub master_progress: Arc<RwLock<WorkerProgress>>,
}

impl ClusterCoordinator {
    pub fn new(config: ClusterConfig) -> Self {
        ClusterCoordinator {
            config,
            workers: Arc::new(RwLock::new(HashMap::new())),
            master_progress: Arc::new(RwLock::new(WorkerProgress {
                requests_sent: 0,
                requests_success: 0,
                requests_error: 0,
                avg_response_time: 0.0,
                current_rps: 0.0,
            })),
        }
    }

    pub async fn register_worker(&self, worker_id: String) -> bool {
        let mut workers = self.workers.write().await;
        workers.insert(
            worker_id.clone(),
            WorkerState {
                id: worker_id,
                status: WorkerStatus::Initializing,
                progress: WorkerProgress {
                    requests_sent: 0,
                    requests_success: 0,
                    requests_error: 0,
                    avg_response_time: 0.0,
                    current_rps: 0.0,
                },
            },
        );
        true
    }

    pub async fn update_worker_state(&self, worker_id: String, status: WorkerStatus) -> bool {
        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.status = status;
            true
        } else {
            false
        }
    }

    pub async fn update_worker_progress(&self, worker_id: String, progress: WorkerProgress) -> bool {
        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.progress = progress;
            true
        } else {
            false
        }
    }

    pub async fn get_total_progress(&self) -> WorkerProgress {
        let workers = self.workers.read().await;
        let mut total = WorkerProgress {
            requests_sent: 0,
            requests_success: 0,
            requests_error: 0,
            avg_response_time: 0.0,
            current_rps: 0.0,
        };

        let mut response_times_sum = 0.0;
        let mut active_workers = 0;

        for (_, worker) in workers.iter() {
            total.requests_sent += worker.progress.requests_sent;
            total.requests_success += worker.progress.requests_success;
            total.requests_error += worker.progress.requests_error;
            if worker.progress.avg_response_time > 0.0 {
                response_times_sum += worker.progress.avg_response_time;
                active_workers += 1;
            }
            total.current_rps += worker.progress.current_rps;
        }

        if active_workers > 0 {
            total.avg_response_time = response_times_sum / active_workers as f64;
        }

        total
    }

    pub async fn is_all_workers_completed(&self) -> bool {
        let workers = self.workers.read().await;
        for (_, worker) in workers.iter() {
            if !matches!(worker.status, WorkerStatus::Completed) {
                return false;
            }
        }
        workers.len() == self.config.total_workers
    }
}