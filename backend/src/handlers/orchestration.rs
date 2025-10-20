use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use serde::Serialize;
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, RemoveContainerOptions, StopContainerOptions};
use bollard::network::CreateNetworkOptions;
use bollard::models::{HostConfig, PortBinding};

use crate::AppState;

#[derive(Serialize)]
pub struct SetupResponse {
    pub success: bool,
    pub message: String,
    pub ttp_port: i32,
    pub ttp_container_id: String,
    pub trustee_containers: Vec<ContainerInfo>,
    pub voter_containers: Vec<ContainerInfo>,
    pub network_name: String,
}

#[derive(Serialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub port: i32,
    pub container_id: String,
    pub ip_address: Option<String>,
}

pub async fn setup_election(
    State(state): State<Arc<AppState>>,
    Path(election_id): Path<Uuid>,
) -> Result<Json<SetupResponse>, StatusCode> {
    tracing::info!("Starting election setup for ID: {}", election_id);

    // Get election details
    let election: crate::models::Election = sqlx::query_as(
        "SELECT * FROM elections WHERE id = $1"
    )
    .bind(&election_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch election: {}", e);
        StatusCode::NOT_FOUND
    })?;

    // Check if election is in phase 3
    if election.phase != 3 {
        tracing::warn!("Election {} is not in phase 3, current phase: {}", election_id, election.phase);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Connect to Docker daemon
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| {
            tracing::error!("Failed to connect to Docker: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Connected to Docker daemon");

    // Create Docker network for this election
    let network_name = format!("election_{}", election_id);

    let create_network_options = CreateNetworkOptions {
        name: network_name.clone(),
        check_duplicate: true,
        driver: "bridge".to_string(),
        ..Default::default()
    };

    match docker.create_network(create_network_options).await {
        Ok(_) => {
            tracing::info!("Created Docker network: {}", network_name);
        }
        Err(e) => {
            tracing::warn!("Network might already exist: {}", e);
        }
    }

    // Log network creation
    let _ = sqlx::query(
        "INSERT INTO system_events (event_type, entity_type, entity_id, data) VALUES ($1, $2, $3, $4)"
    )
    .bind("docker_network_created")
    .bind("network")
    .bind(&election_id)
    .bind(serde_json::json!({
        "network_name": &network_name
    }))
    .execute(&state.db)
    .await;

    // Get all trustees
    let trustees: Vec<crate::models::Trustee> = sqlx::query_as(
        "SELECT * FROM trustees WHERE election_id = $1"
    )
    .bind(&election_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get all voters
    let voters: Vec<crate::models::Voter> = sqlx::query_as(
        "SELECT * FROM voters WHERE election_id = $1"
    )
    .bind(&election_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!("Found {} trustees and {} voters", trustees.len(), voters.len());

    // Update election with network info (no TTP needed for DKG)
    sqlx::query(
        "UPDATE elections SET docker_network = $1, phase = 4 WHERE id = $2"
    )
    .bind(&network_name)
    .bind(&election_id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut trustee_containers = Vec::new();
    let mut port_counter = 10000;

    // Create trustee containers
    for trustee in trustees.iter() {
        if trustee.docker_type == "auto" {
            let container_port = port_counter;
            port_counter += 2;  // Reserve 2 ports: API + UI

            let container_name = format!("trustee-{}-{}", election_id, trustee.id);

            tracing::info!("Creating trustee container: {}", container_name);

            let container_id = create_trustee_container(
                &docker,
                &container_name,
                &network_name,
                container_port,
                &trustee.id.to_string(),
                &trustee.name,
                &election_id.to_string(),
            ).await?;

            tracing::info!("Trustee container created: {}", container_id);

            // Get container IP address from network
            let container_ip = docker.inspect_container(&container_id, None)
                .await
                .ok()
                .and_then(|info| info.network_settings)
                .and_then(|ns| ns.networks)
                .and_then(|networks| networks.get(&network_name).cloned())
                .and_then(|network| network.ip_address)
                .unwrap_or_else(|| "unknown".to_string());

            tracing::info!("Trustee container IP: {}", container_ip);

            // Update trustee with Docker info including IP address
            sqlx::query(
                "UPDATE trustees SET docker_port = $1, container_id = $2, ip_address = $3 WHERE id = $4"
            )
            .bind(container_port)
            .bind(&container_id)
            .bind(&container_ip)
            .bind(&trustee.id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            trustee_containers.push(ContainerInfo {
                id: trustee.id.to_string(),
                name: trustee.name.clone(),
                port: container_port,
                container_id: container_id.clone(),
                ip_address: None,
            });

            // Log container creation
            let _ = sqlx::query(
                "INSERT INTO system_events (event_type, entity_type, entity_id, data) VALUES ($1, $2, $3, $4)"
            )
            .bind("trustee_container_created")
            .bind("container")
            .bind(&trustee.id)
            .bind(serde_json::json!({
                "container_id": &container_id,
                "container_name": &container_name,
                "port": container_port,
                "network": &network_name,
                "trustee_name": &trustee.name
            }))
            .execute(&state.db)
            .await;
        }
    }

    let mut voter_containers = Vec::new();

    // Force voter ports to start at 10020 to match Dockerfile.voter EXPOSE
    port_counter = 10020;

    // Create voter containers
    for voter in voters.iter() {
        let container_port = port_counter;
        port_counter += 2;  // Reserve 2 ports: API + UI

        let container_name = format!("voter-{}-{}", election_id, voter.id);

        tracing::info!("Creating voter container: {}", container_name);

        let container_id = create_voter_container(
            &docker,
            &container_name,
            &network_name,
            container_port,
            &voter.id.to_string(),
            &voter.voter_id,
            &election_id.to_string(),
        ).await?;

        tracing::info!("Voter container created: {}", container_id);

        // Update voter with Docker info
        sqlx::query(
            "UPDATE voters SET docker_port = $1, container_id = $2 WHERE id = $3"
        )
        .bind(container_port)
        .bind(&container_id)
        .bind(&voter.id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        voter_containers.push(ContainerInfo {
            id: voter.id.to_string(),
            name: format!("Voter {}", voter.voter_id),
            port: container_port,
            container_id: container_id.clone(),
            ip_address: None,
        });

        // Log container creation
        let _ = sqlx::query(
            "INSERT INTO system_events (event_type, entity_type, entity_id, data) VALUES ($1, $2, $3, $4)"
        )
        .bind("voter_container_created")
        .bind("container")
        .bind(&voter.id)
        .bind(serde_json::json!({
            "container_id": &container_id,
            "container_name": &container_name,
            "port": container_port,
            "tc_id": &voter.voter_id,
            "network": &network_name
        }))
        .execute(&state.db)
        .await;
    }

    tracing::info!("Election setup complete! Created {} trustee and {} voter containers",
        trustee_containers.len(), voter_containers.len());

    Ok(Json(SetupResponse {
        success: true,
        message: format!(
            "Election setup complete! Created {} trustee containers and {} voter containers. Ready for DKG.",
            trustee_containers.len(),
            voter_containers.len()
        ),
        ttp_port: 0,  // Not used with DKG
        ttp_container_id: String::new(),  // Not used with DKG
        trustee_containers,
        voter_containers,
        network_name,
    }))
}

async fn create_trustee_container(
    docker: &Docker,
    container_name: &str,
    network_name: &str,
    port: i32,
    trustee_id: &str,
    trustee_name: &str,
    election_id: &str,
) -> Result<String, StatusCode> {
    let trustee_id_env = format!("TRUSTEE_ID={}", trustee_id);
    let trustee_name_env = format!("TRUSTEE_NAME={}", trustee_name);
    let election_id_env = format!("ELECTION_ID={}", election_id);
    let api_port_env = format!("API_PORT={}", port);
    let ui_port_env = format!("UI_PORT={}", port + 1);
    let container_type_env = "CONTAINER_TYPE=Trustee".to_string();

    let env: Vec<&str> = vec![
        trustee_id_env.as_str(),
        trustee_name_env.as_str(),
        election_id_env.as_str(),
        api_port_env.as_str(),
        ui_port_env.as_str(),
        container_type_env.as_str(),
    ];

    let mut port_bindings = HashMap::new();
    // API port
    port_bindings.insert(
        format!("{}/tcp", port),
        Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some(port.to_string()),
        }]),
    );
    // UI port
    port_bindings.insert(
        format!("{}/tcp", port + 1),
        Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some((port + 1).to_string()),
        }]),
    );

    let host_config = HostConfig {
        port_bindings: Some(port_bindings),
        network_mode: Some(network_name.to_string()),  // Connect to election network
        ..Default::default()
    };

    let config = Config {
        image: Some("evoting-trustee:latest"),
        env: Some(env),
        host_config: Some(host_config),
        ..Default::default()
    };

    let options = CreateContainerOptions {
        name: container_name,
        ..Default::default()
    };

    let container = docker.create_container(Some(options), config)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create trustee container: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    docker.start_container(&container.id, None::<StartContainerOptions<String>>)
        .await
        .map_err(|e| {
            tracing::error!("Failed to start trustee container: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Trustee container {} started successfully", container.id);

    Ok(container.id)
}

async fn create_voter_container(
    docker: &Docker,
    container_name: &str,
    network_name: &str,
    port: i32,
    voter_id: &str,
    tc_id: &str,
    election_id: &str,
) -> Result<String, StatusCode> {
    let voter_id_env = format!("VOTER_ID={}", voter_id);
    let tc_id_env = format!("TC_ID={}", tc_id);
    let election_id_env = format!("ELECTION_ID={}", election_id);
    let api_port_env = format!("API_PORT={}", port);
    let ui_port_env = format!("UI_PORT={}", port + 1);
    let container_type_env = "CONTAINER_TYPE=Voter".to_string();

    let env: Vec<&str> = vec![
        voter_id_env.as_str(),
        tc_id_env.as_str(),
        election_id_env.as_str(),
        api_port_env.as_str(),
        ui_port_env.as_str(),
        container_type_env.as_str(),
    ];

    let mut port_bindings = HashMap::new();
    // API port
    port_bindings.insert(
        format!("{}/tcp", port),
        Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some(port.to_string()),
        }]),
    );
    // UI port
    port_bindings.insert(
        format!("{}/tcp", port + 1),
        Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some((port + 1).to_string()),
        }]),
    );

    let host_config = HostConfig {
        port_bindings: Some(port_bindings),
        network_mode: Some(network_name.to_string()),  // Connect to election network
        ..Default::default()
    };

    let config = Config {
        image: Some("evoting-voter:latest"),
        env: Some(env),
        host_config: Some(host_config),
        ..Default::default()
    };

    let options = CreateContainerOptions {
        name: container_name,
        ..Default::default()
    };

    let container = docker.create_container(Some(options), config)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create voter container: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    docker.start_container(&container.id, None::<StartContainerOptions<String>>)
        .await
        .map_err(|e| {
            tracing::error!("Failed to start voter container: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Voter container {} started successfully", container.id);

    Ok(container.id)
}

/// Cleanup all containers and network for an election
pub async fn cleanup_election(
    State(state): State<Arc<AppState>>,
    Path(election_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::info!("🧹 Starting cleanup for election: {}", election_id);

    // Connect to Docker daemon
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| {
            tracing::error!("Failed to connect to Docker: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut cleanup_results = Vec::new();

    // Get all trustees with container IDs
    let trustees: Vec<crate::models::Trustee> = sqlx::query_as(
        "SELECT * FROM trustees WHERE election_id = $1 AND container_id IS NOT NULL"
    )
    .bind(&election_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Get all voters with container IDs
    let voters: Vec<crate::models::Voter> = sqlx::query_as(
        "SELECT * FROM voters WHERE election_id = $1 AND container_id IS NOT NULL"
    )
    .bind(&election_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // No TTP container to remove (using DKG instead)

    // Stop and remove trustee containers
    for trustee in trustees.iter() {
        let container_name = format!("trustee-{}-{}", election_id, trustee.id);
        match stop_and_remove_container(&docker, &container_name).await {
            Ok(_) => {
                tracing::info!("✅ Trustee container stopped and removed: {}", container_name);
                cleanup_results.push(format!("Trustee: {}", container_name));
            }
            Err(e) => {
                tracing::warn!("⚠️  Failed to remove trustee container {}: {}", container_name, e);
            }
        }
    }

    // Stop and remove voter containers
    for voter in voters.iter() {
        let container_name = format!("voter-{}-{}", election_id, voter.id);
        match stop_and_remove_container(&docker, &container_name).await {
            Ok(_) => {
                tracing::info!("✅ Voter container stopped and removed: {}", container_name);
                cleanup_results.push(format!("Voter: {}", container_name));
            }
            Err(e) => {
                tracing::warn!("⚠️  Failed to remove voter container {}: {}", container_name, e);
            }
        }
    }

    // Delete all trustees from database
    let trustees_deleted = sqlx::query("DELETE FROM trustees WHERE election_id = $1")
        .bind(&election_id)
        .execute(&state.db)
        .await
        .map(|r| r.rows_affected())
        .unwrap_or(0);

    tracing::info!("🗑️  Deleted {} trustees from database", trustees_deleted);

    // Delete all voters from database
    let voters_deleted = sqlx::query("DELETE FROM voters WHERE election_id = $1")
        .bind(&election_id)
        .execute(&state.db)
        .await
        .map(|r| r.rows_affected())
        .unwrap_or(0);

    tracing::info!("🗑️  Deleted {} voters from database", voters_deleted);

    // Remove Docker network
    let network_name = format!("election_{}", election_id);
    match docker.remove_network(&network_name).await {
        Ok(_) => {
            tracing::info!("✅ Docker network removed: {}", network_name);
            cleanup_results.push(format!("Network: {}", network_name));
        }
        Err(e) => {
            tracing::warn!("⚠️  Failed to remove network: {}", e);
        }
    }

    // Reset election docker info in database
    let _ = sqlx::query(
        "UPDATE elections SET docker_network = NULL, ttp_port = NULL WHERE id = $1"
    )
    .bind(&election_id)
    .execute(&state.db)
    .await;

    // Log cleanup event
    let _ = sqlx::query(
        "INSERT INTO system_events (event_type, entity_type, entity_id, data) VALUES ($1, $2, $3, $4)"
    )
    .bind("election_cleanup")
    .bind("election")
    .bind(&election_id)
    .bind(serde_json::json!({
        "cleaned_resources": cleanup_results
    }))
    .execute(&state.db)
    .await;

    tracing::info!("🎉 Cleanup complete for election: {}", election_id);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Cleanup complete for election {}", election_id),
        "cleaned_resources": cleanup_results
    })))
}

/// Helper function to stop and remove a container
async fn stop_and_remove_container(docker: &Docker, container_name: &str) -> Result<(), String> {
    // Stop container with 1 second timeout for instant shutdown
    let stop_options = Some(StopContainerOptions {
        t: 1,
    });

    match docker.stop_container(container_name, stop_options).await {
        Ok(_) => tracing::debug!("Container stopped: {}", container_name),
        Err(e) => tracing::debug!("Container might not be running: {} - {}", container_name, e),
    }

    // Remove container
    let remove_options = Some(RemoveContainerOptions {
        force: true,
        v: true, // Remove volumes
        ..Default::default()
    });

    docker.remove_container(container_name, remove_options)
        .await
        .map_err(|e| format!("Failed to remove container: {}", e))?;

    Ok(())
}
