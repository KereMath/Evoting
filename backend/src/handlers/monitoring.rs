use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::Serialize;
use bollard::Docker;
use bollard::container::ListContainersOptions;
use std::collections::HashMap;

use crate::AppState;

#[derive(Serialize, Clone)]
pub struct DockerNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub status: String,
    pub connections: Vec<String>,
    pub port: Option<i32>,
    pub election_id: Option<String>,
    pub metadata: Option<String>,
}

#[derive(Serialize)]
pub struct DataFlow {
    pub from_node: String,
    pub to_node: String,
    pub flow_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct MonitoringResponse {
    pub nodes: Vec<DockerNode>,
    pub data_flows: Vec<DataFlow>,
    pub total_containers: usize,
    pub active_containers: usize,
}

pub async fn get_network_topology(
    State(state): State<Arc<AppState>>,
) -> Result<Json<MonitoringResponse>, StatusCode> {
    // Connect to Docker
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| {
            tracing::error!("Failed to connect to Docker: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // List all containers
    let mut filters = HashMap::new();
    filters.insert("status".to_string(), vec!["running".to_string(), "created".to_string(), "exited".to_string()]);

    let options = ListContainersOptions {
        all: true,
        filters,
        ..Default::default()
    };

    let containers = docker.list_containers(Some(options))
        .await
        .map_err(|e| {
            tracing::error!("Failed to list containers: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut nodes = vec![];
    let mut data_flows = vec![];

    // Parse all Docker containers
    for container in containers {
        let container_name = container.names
            .as_ref()
            .and_then(|names| names.first())
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let container_id = container.id.clone().unwrap_or_default();
        let status = container.state.clone().unwrap_or_else(|| "unknown".to_string());
        let image = container.image.clone().unwrap_or_else(|| "unknown".to_string());

        // Extract port
        let port = container.ports
            .as_ref()
            .and_then(|ports| ports.first())
            .and_then(|p| p.public_port)
            .map(|p| p as i32);

        // Determine node type and election_id from container name
        let (node_type, election_id, metadata) = if container_name.starts_with("ttp-") {
            let election_id = container_name.strip_prefix("ttp-").map(|s| s.to_string());
            ("ttp".to_string(), election_id, Some("Trusted Third Party".to_string()))
        } else if container_name.starts_with("trustee-") {
            // trustee-{election_id}-{trustee_id}
            let parts: Vec<&str> = container_name.splitn(3, '-').collect();
            let election_id = if parts.len() >= 2 { Some(parts[1].to_string()) } else { None };
            let trustee_id = if parts.len() >= 3 { Some(parts[2].to_string()) } else { None };
            let metadata = trustee_id.map(|id| format!("Trustee ID: {}", id));
            ("trustee".to_string(), election_id, metadata)
        } else if container_name.starts_with("voter-") {
            // voter-{election_id}-{voter_id}
            let parts: Vec<&str> = container_name.splitn(3, '-').collect();
            let election_id = if parts.len() >= 2 { Some(parts[1].to_string()) } else { None };
            let voter_id = if parts.len() >= 3 { Some(parts[2].to_string()) } else { None };
            let metadata = voter_id.map(|id| format!("Voter ID: {}", id));
            ("voter".to_string(), election_id, metadata)
        } else if container_name.contains("postgres") {
            ("database".to_string(), None, Some("PostgreSQL Database".to_string()))
        } else if container_name.contains("frontend") {
            ("frontend".to_string(), None, Some("React Admin Panel".to_string()))
        } else if container_name.contains("main-server") {
            ("backend".to_string(), None, Some("Rust Backend Server".to_string()))
        } else {
            ("unknown".to_string(), None, Some(image.clone()))
        };

        nodes.push(DockerNode {
            id: container_id.clone(),
            name: container_name,
            node_type,
            status,
            connections: vec![],  // Will be populated based on relationships
            port,
            election_id,
            metadata,
        });
    }

    // Build data flows based on node types and relationships
    // System infrastructure flows
    let frontend_node = nodes.iter().find(|n| n.node_type == "frontend");
    let backend_node = nodes.iter().find(|n| n.node_type == "backend");
    let database_node = nodes.iter().find(|n| n.node_type == "database");

    if let (Some(frontend), Some(backend)) = (frontend_node, backend_node) {
        data_flows.push(DataFlow {
            from_node: frontend.name.clone(),
            to_node: backend.name.clone(),
            flow_type: "HTTP Request".to_string(),
            timestamp: chrono::Utc::now(),
        });
    }

    if let (Some(backend), Some(database)) = (backend_node, database_node) {
        data_flows.push(DataFlow {
            from_node: backend.name.clone(),
            to_node: database.name.clone(),
            flow_type: "SQL Query".to_string(),
            timestamp: chrono::Utc::now(),
        });
    }

    // Election-specific flows
    // Group nodes by election_id
    let mut election_nodes: HashMap<String, Vec<&DockerNode>> = HashMap::new();
    for node in &nodes {
        if let Some(ref election_id) = node.election_id {
            election_nodes.entry(election_id.clone())
                .or_insert_with(Vec::new)
                .push(node);
        }
    }

    // Create flows for each election
    for (election_id, election_nodes) in election_nodes {
        let ttp = election_nodes.iter().find(|n| n.node_type == "ttp");
        let trustees: Vec<&&DockerNode> = election_nodes.iter().filter(|n| n.node_type == "trustee").collect();
        let voters: Vec<&&DockerNode> = election_nodes.iter().filter(|n| n.node_type == "voter").collect();

        // Trustees -> TTP (Key Share)
        if let Some(ttp_node) = ttp {
            for trustee in &trustees {
                data_flows.push(DataFlow {
                    from_node: trustee.name.clone(),
                    to_node: ttp_node.name.clone(),
                    flow_type: "Key Share".to_string(),
                    timestamp: chrono::Utc::now(),
                });
            }

            // Voters -> TTP (Vote Submission)
            for voter in &voters {
                data_flows.push(DataFlow {
                    from_node: voter.name.clone(),
                    to_node: ttp_node.name.clone(),
                    flow_type: "Vote Submission".to_string(),
                    timestamp: chrono::Utc::now(),
                });
            }

            // TTP -> Backend (Coordination)
            if let Some(backend) = backend_node {
                data_flows.push(DataFlow {
                    from_node: ttp_node.name.clone(),
                    to_node: backend.name.clone(),
                    flow_type: "TTP Coordination".to_string(),
                    timestamp: chrono::Utc::now(),
                });
            }
        }
    }

    Ok(Json(MonitoringResponse {
        total_containers: nodes.len(),
        active_containers: nodes.iter().filter(|n| n.status == "running").count(),
        nodes,
        data_flows,
    }))
}
