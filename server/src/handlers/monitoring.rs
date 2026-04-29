//! # Health Monitoring Dashboard
//!
//! Provides comprehensive system health monitoring including:
//! - Database connectivity
//! - Redis connectivity
//! - Disk space availability
//! - Memory availability

use axum::{extract::State, response::IntoResponse, response::Response, http::StatusCode};
use serde::Serialize;
use sqlx::PgPool;
use sysinfo::{Disks, System};

use crate::cache::RedisCache;
use crate::utils::error::AppError;
use crate::utils::response::success;

#[derive(Serialize)]
pub struct MonitoringDashboard {
    pub status: String,
    pub checks: HealthChecks,
}

#[derive(Serialize)]
pub struct HealthChecks {
    pub database: ComponentHealth,
    pub redis: ComponentHealth,
    pub disk: DiskHealth,
    pub memory: MemoryHealth,
}

#[derive(Serialize)]
pub struct ComponentHealth {
    pub status: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct DiskHealth {
    pub status: String,
    pub available_gb: f64,
    pub total_gb: f64,
    pub usage_percent: f64,
}

#[derive(Serialize)]
pub struct MemoryHealth {
    pub status: String,
    pub available_gb: f64,
    pub total_gb: f64,
    pub usage_percent: f64,
}

/// Application state for monitoring
#[derive(Clone)]
pub struct MonitoringState {
    pub pool: PgPool,
    pub redis: RedisCache,
}

/// GET /api/v1/monitoring/dashboard
///
/// Returns comprehensive system health status.
/// Returns 200 OK if all systems are healthy.
/// Returns 503 Service Unavailable if any critical component is down.
pub async fn monitoring_dashboard(State(state): State<MonitoringState>) -> Response {
    let mut redis = state.redis.clone();
    
    // Check database
    let db_health = match sqlx::query("SELECT 1").fetch_one(&state.pool).await {
        Ok(_) => ComponentHealth {
            status: "healthy".to_string(),
            message: "Database connection successful".to_string(),
        },
        Err(e) => ComponentHealth {
            status: "unhealthy".to_string(),
            message: format!("Database connection failed: {}", e),
        },
    };

    // Check Redis
    let redis_health = match redis.ping().await {
        Ok(_) => ComponentHealth {
            status: "healthy".to_string(),
            message: "Redis connection successful".to_string(),
        },
        Err(e) => ComponentHealth {
            status: "unhealthy".to_string(),
            message: format!("Redis connection failed: {}", e),
        },
    };

    // Check disk space
    let disks = Disks::new_with_refreshed_list();
    let disk_health = if let Some(disk) = disks.iter().next() {
        let total_bytes = disk.total_space();
        let available_bytes = disk.available_space();
        let total_gb = total_bytes as f64 / 1_073_741_824.0;
        let available_gb = available_bytes as f64 / 1_073_741_824.0;
        let usage_percent = ((total_bytes - available_bytes) as f64 / total_bytes as f64) * 100.0;

        DiskHealth {
            status: if available_gb > 5.0 { "healthy" } else { "warning" }.to_string(),
            available_gb,
            total_gb,
            usage_percent,
        }
    } else {
        DiskHealth {
            status: "unknown".to_string(),
            available_gb: 0.0,
            total_gb: 0.0,
            usage_percent: 0.0,
        }
    };

    // Check memory
    let mut sys = System::new_all();
    sys.refresh_memory();
    let total_memory = sys.total_memory();
    let available_memory = sys.available_memory();
    let total_gb = total_memory as f64 / 1_073_741_824.0;
    let available_gb = available_memory as f64 / 1_073_741_824.0;
    let usage_percent = ((total_memory - available_memory) as f64 / total_memory as f64) * 100.0;

    let memory_health = MemoryHealth {
        status: if available_gb > 1.0 { "healthy" } else { "warning" }.to_string(),
        available_gb,
        total_gb,
        usage_percent,
    };

    // Determine overall status
    let all_healthy = db_health.status == "healthy"
        && redis_health.status == "healthy"
        && disk_health.status != "unhealthy"
        && memory_health.status != "unhealthy";

    let dashboard = MonitoringDashboard {
        status: if all_healthy { "healthy" } else { "degraded" }.to_string(),
        checks: HealthChecks {
            database: db_health,
            redis: redis_health,
            disk: disk_health,
            memory: memory_health,
        },
    };

    if all_healthy {
        success(dashboard, "All systems healthy").into_response()
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({
                "success": false,
                "message": "One or more critical components are down",
                "data": dashboard
            }))
        ).into_response()
    }
}
