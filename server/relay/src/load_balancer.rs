use anyhow::{Result, anyhow};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, error, warn, debug};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayNode {
    pub id: Uuid,
    pub endpoint: String,
    pub location: GeographicLocation,
    pub capacity: u32,
    pub current_load: u32,
    pub bandwidth_limit: u64, // Mbps
    pub current_bandwidth: u64,
    pub health_status: HealthStatus,
    pub last_health_check: DateTime<Utc>,
    pub latency_ms: u32,
    pub supported_protocols: Vec<String>,
    pub priority: u8, // 1-10, higher is more preferred
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicLocation {
    pub country: String,
    pub region: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingStrategy {
    pub algorithm: BalancingAlgorithm,
    pub geographic_weight: f64,
    pub performance_weight: f64,
    pub capacity_weight: f64,
    pub latency_threshold: u32,
    pub bandwidth_threshold: f64, // Percentage of capacity
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BalancingAlgorithm {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    WeightedLeastConnections,
    Geographic,
    PerformanceBased,
    Adaptive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAssignment {
    pub session_id: Uuid,
    pub node_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub strategy_used: BalancingAlgorithm,
    pub client_location: Option<GeographicLocation>,
    pub estimated_bandwidth: u64,
}

pub struct LoadBalancer {
    nodes: Arc<RwLock<HashMap<Uuid, RelayNode>>>,
    assignments: Arc<RwLock<HashMap<Uuid, SessionAssignment>>>,
    strategy: Arc<RwLock<LoadBalancingStrategy>>,
    health_checker: Arc<HealthChecker>,
    metrics_collector: Arc<MetricsCollector>,
    round_robin_index: Arc<RwLock<usize>>,
}

impl LoadBalancer {
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        let health_checker = Arc::new(HealthChecker::new());
        let metrics_collector = Arc::new(MetricsCollector::new());
        
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            assignments: Arc::new(RwLock::new(HashMap::new())),
            strategy: Arc::new(RwLock::new(strategy)),
            health_checker,
            metrics_collector,
            round_robin_index: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn add_node(&self, node: RelayNode) -> Result<()> {
        let mut nodes = self.nodes.write().await;
        nodes.insert(node.id, node.clone());
        
        info!("Added relay node: {} ({})", node.id, node.endpoint);
        
        // Start health monitoring for this node
        self.health_checker.start_monitoring(node.id, node.endpoint.clone()).await?;
        
        Ok(())
    }

    pub async fn remove_node(&self, node_id: Uuid) -> Result<()> {
        let mut nodes = self.nodes.write().await;
        nodes.remove(&node_id);
        
        // Stop health monitoring
        self.health_checker.stop_monitoring(node_id).await?;
        
        // Reassign sessions from this node
        self.reassign_sessions(node_id).await?;
        
        info!("Removed relay node: {}", node_id);
        Ok(())
    }

    pub async fn assign_session(&self, session_id: Uuid, client_location: Option<GeographicLocation>, estimated_bandwidth: u64) -> Result<Uuid> {
        let strategy = self.strategy.read().await.clone();
        let nodes = self.nodes.read().await;
        
        // Filter healthy and available nodes
        let available_nodes: Vec<&RelayNode> = nodes
            .values()
            .filter(|node| {
                node.health_status == HealthStatus::Healthy &&
                node.current_load < node.capacity &&
                (node.current_bandwidth as f64) < (node.bandwidth_limit as f64 * strategy.bandwidth_threshold / 100.0) &&
                node.latency_ms <= strategy.latency_threshold
            })
            .collect();

        if available_nodes.is_empty() {
            return Err(anyhow!("No healthy relay nodes available"));
        }

        // Select node based on strategy
        let selected_node = match strategy.algorithm {
            BalancingAlgorithm::RoundRobin => self.select_round_robin(&available_nodes).await?,
            BalancingAlgorithm::WeightedRoundRobin => self.select_weighted_round_robin(&available_nodes).await?,
            BalancingAlgorithm::LeastConnections => self.select_least_connections(&available_nodes).await?,
            BalancingAlgorithm::WeightedLeastConnections => self.select_weighted_least_connections(&available_nodes).await?,
            BalancingAlgorithm::Geographic => self.select_geographic(&available_nodes, client_location.as_ref()).await?,
            BalancingAlgorithm::PerformanceBased => self.select_performance_based(&available_nodes).await?,
            BalancingAlgorithm::Adaptive => self.select_adaptive(&available_nodes, client_location.as_ref(), estimated_bandwidth).await?,
        };

        // Create assignment
        let assignment = SessionAssignment {
            session_id,
            node_id: selected_node.id,
            assigned_at: Utc::now(),
            strategy_used: strategy.algorithm.clone(),
            client_location,
            estimated_bandwidth,
        };

        // Store assignment
        let mut assignments = self.assignments.write().await;
        assignments.insert(session_id, assignment.clone());

        // Extract node ID before dropping nodes
        let selected_node_id = selected_node.id;

        // Update node load
        self.increment_node_load(selected_node_id).await;

        info!("Assigned session {} to node {} ({})", session_id, selected_node.id, selected_node.endpoint);
        
        // Record metrics
        self.metrics_collector.record_assignment(&assignment).await;
        
        // Now we can drop nodes
        drop(nodes);
        
        Ok(selected_node_id)
    }

    async fn select_round_robin<'a>(&self, nodes: &[&'a RelayNode]) -> Result<&'a RelayNode> {
        let mut index = self.round_robin_index.write().await;
        let selected = nodes[*index % nodes.len()];
        *index = (*index + 1) % nodes.len();
        Ok(selected)
    }

    async fn select_weighted_round_robin<'a>(&self, nodes: &[&'a RelayNode]) -> Result<&'a RelayNode> {
        // Calculate weights based on capacity and priority
        let mut weighted_nodes: Vec<(usize, &RelayNode)> = nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let weight = (node.capacity - node.current_load) as usize * node.priority as usize;
                (i, *node)
            })
            .collect();

        // Sort by weight (descending)
        weighted_nodes.sort_by(|a, b| b.0.cmp(&a.0));

        Ok(weighted_nodes[0].1)
    }

    async fn select_least_connections<'a>(&self, nodes: &[&'a RelayNode]) -> Result<&'a RelayNode> {
        let selected = nodes
            .iter()
            .min_by_key(|node| node.current_load)
            .ok_or_else(|| anyhow!("No nodes available"))?;
        Ok(selected)
    }

    async fn select_weighted_least_connections<'a>(&self, nodes: &[&'a RelayNode]) -> Result<&'a RelayNode> {
        let strategy = self.strategy.read().await.clone();
        
        let selected = nodes
            .iter()
            .min_by(|a, b| {
                let score_a = self.calculate_node_score(a, &strategy);
                let score_b = self.calculate_node_score(b, &strategy);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| anyhow!("No nodes available"))?;
        Ok(selected)
    }

    async fn select_geographic<'a>(&self, nodes: &[&'a RelayNode], client_location: Option<&GeographicLocation>) -> Result<&'a RelayNode> {
        let client_loc = client_location.ok_or_else(|| anyhow!("Client location required for geographic routing"))?;
        
        let selected = nodes
            .iter()
            .min_by(|a, b| {
                let dist_a = self.calculate_distance(&a.location, client_loc);
                let dist_b = self.calculate_distance(&b.location, client_loc);
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| anyhow!("No nodes available"))?;
        Ok(selected)
    }

    async fn select_performance_based<'a>(&self, nodes: &[&'a RelayNode]) -> Result<&'a RelayNode> {
        let selected = nodes
            .iter()
            .min_by(|a, b| {
                // Consider latency, current load, and bandwidth usage
                let score_a = a.latency_ms as f64 + (a.current_load as f64 / a.capacity as f64) * 100.0;
                let score_b = b.latency_ms as f64 + (b.current_load as f64 / b.capacity as f64) * 100.0;
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| anyhow!("No nodes available"))?;
        Ok(selected)
    }

    async fn select_adaptive<'a>(&self, nodes: &[&'a RelayNode], client_location: Option<&GeographicLocation>, estimated_bandwidth: u64) -> Result<&'a RelayNode> {
        let strategy = self.strategy.read().await.clone();
        
        let selected = nodes
            .iter()
            .min_by(|a, b| {
                let score_a = self.calculate_adaptive_score(a, client_location, estimated_bandwidth, &strategy);
                let score_b = self.calculate_adaptive_score(b, client_location, estimated_bandwidth, &strategy);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| anyhow!("No nodes available"))?;
        Ok(selected)
    }

    fn calculate_node_score(&self, node: &RelayNode, strategy: &LoadBalancingStrategy) -> f64 {
        let load_ratio = node.current_load as f64 / node.capacity as f64;
        let bandwidth_ratio = node.current_bandwidth as f64 / node.bandwidth_limit as f64;
        let latency_score = node.latency_ms as f64;
        
        strategy.performance_weight * latency_score +
        strategy.capacity_weight * load_ratio * 100.0 +
        (1.0 - strategy.performance_weight - strategy.capacity_weight) * bandwidth_ratio * 100.0
    }

    fn calculate_adaptive_score(&self, node: &RelayNode, client_location: Option<&GeographicLocation>, estimated_bandwidth: u64, strategy: &LoadBalancingStrategy) -> f64 {
        let mut score = 0.0;
        
        // Performance factor
        score += strategy.performance_weight * node.latency_ms as f64;
        
        // Capacity factor
        let load_ratio = node.current_load as f64 / node.capacity as f64;
        score += strategy.capacity_weight * load_ratio * 100.0;
        
        // Geographic factor
        if let Some(client_loc) = client_location {
            let distance = self.calculate_distance(&node.location, client_loc);
            score += strategy.geographic_weight * distance;
        }
        
        // Bandwidth availability factor
        let bandwidth_available = node.bandwidth_limit - node.current_bandwidth;
        if bandwidth_available < estimated_bandwidth {
            score += 1000.0; // Heavy penalty for insufficient bandwidth
        }
        
        // Priority factor
        score -= (11 - node.priority) as f64 * 10.0;
        
        score
    }

    fn calculate_distance(&self, loc1: &GeographicLocation, loc2: &GeographicLocation) -> f64 {
        // Haversine formula for calculating distance between two geographic points
        let earth_radius = 6371.0; // Earth's radius in kilometers
        
        let lat1_rad = loc1.latitude.to_radians();
        let lat2_rad = loc2.latitude.to_radians();
        let delta_lat = (loc2.latitude - loc1.latitude).to_radians();
        let delta_lon = (loc2.longitude - loc1.longitude).to_radians();
        
        let a = (delta_lat / 2.0).sin().powi(2) +
                lat1_rad.cos() * lat2_rad.cos() *
                (delta_lon / 2.0).sin().powi(2);
        
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        
        earth_radius * c
    }

    async fn increment_node_load(&self, node_id: Uuid) {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(&node_id) {
            node.current_load += 1;
        }
    }

    async fn decrement_node_load(&self, node_id: Uuid) {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(&node_id) {
            node.current_load = node.current_load.saturating_sub(1);
        }
    }

    async fn reassign_sessions(&self, from_node_id: Uuid) -> Result<()> {
        let mut assignments = self.assignments.write().await;
        let affected_sessions: Vec<Uuid> = assignments
            .values()
            .filter(|assignment| assignment.node_id == from_node_id)
            .map(|assignment| assignment.session_id)
            .collect();

        for session_id in affected_sessions {
            if let Some(assignment) = assignments.remove(&session_id) {
                // Reassign to a different node
                if let Ok(new_node_id) = self.assign_session(session_id, assignment.client_location, assignment.estimated_bandwidth).await {
                    info!("Reassigned session {} from node {} to node {}", session_id, from_node_id, new_node_id);
                } else {
                    warn!("Failed to reassign session {} from node {}", session_id, from_node_id);
                }
            }
        }

        Ok(())
    }

    pub async fn release_session(&self, session_id: Uuid) -> Result<()> {
        let mut assignments = self.assignments.write().await;
        if let Some(assignment) = assignments.remove(&session_id) {
            self.decrement_node_load(assignment.node_id).await;
            info!("Released session {} from node {}", session_id, assignment.node_id);
        }
        Ok(())
    }

    pub async fn update_strategy(&self, new_strategy: LoadBalancingStrategy) -> Result<()> {
        let mut strategy = self.strategy.write().await;
        *strategy = new_strategy;
        info!("Updated load balancing strategy");
        Ok(())
    }

    pub async fn get_node_status(&self, node_id: Uuid) -> Result<Option<RelayNode>> {
        let nodes = self.nodes.read().await;
        Ok(nodes.get(&node_id).cloned())
    }

    pub async fn get_all_nodes(&self) -> Vec<RelayNode> {
        let nodes = self.nodes.read().await;
        nodes.values().cloned().collect()
    }

    pub async fn get_active_sessions(&self) -> Vec<SessionAssignment> {
        let assignments = self.assignments.read().await;
        assignments.values().cloned().collect()
    }

    pub async fn get_metrics(&self) -> LoadBalancerMetrics {
        let nodes = self.nodes.read().await;
        let assignments = self.assignments.read().await;
        
        let total_nodes = nodes.len();
        let healthy_nodes = nodes.values().filter(|n| n.health_status == HealthStatus::Healthy).count();
        let total_sessions = assignments.len();
        let total_capacity: u32 = nodes.values().map(|n| n.capacity).sum();
        let total_load: u32 = nodes.values().map(|n| n.current_load).sum();
        
        LoadBalancerMetrics {
            total_nodes,
            healthy_nodes,
            total_sessions,
            total_capacity,
            current_load: total_load,
            load_percentage: if total_capacity > 0 { (total_load as f64 / total_capacity as f64) * 100.0 } else { 0.0 },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerMetrics {
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub total_sessions: usize,
    pub total_capacity: u32,
    pub current_load: u32,
    pub load_percentage: f64,
}

pub struct HealthChecker {
    monitored_nodes: Arc<RwLock<HashMap<Uuid, mpsc::Sender<()>>>>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            monitored_nodes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_monitoring(&self, node_id: Uuid, endpoint: String) -> Result<()> {
        let (stop_tx, mut stop_rx) = mpsc::channel(1);
        
        {
            let mut monitored = self.monitored_nodes.write().await;
            monitored.insert(node_id, stop_tx);
        }
        
        // Start health check loop
        let node_id_clone = node_id;
        let endpoint_clone = endpoint.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::check_node_health(node_id_clone, &endpoint_clone).await {
                            error!("Health check failed for node {}: {}", node_id_clone, e);
                        }
                    }
                    _ = stop_rx.recv() => {
                        info!("Stopping health monitoring for node {}", node_id_clone);
                        break;
                    }
                }
            }
        });
        
        info!("Started health monitoring for node {} ({})", node_id, endpoint);
        Ok(())
    }

    pub async fn stop_monitoring(&self, node_id: Uuid) -> Result<()> {
        let mut monitored = self.monitored_nodes.write().await;
        if let Some(stop_tx) = monitored.remove(&node_id) {
            let _ = stop_tx.send(()).await;
        }
        info!("Stopped health monitoring for node {}", node_id);
        Ok(())
    }

    async fn check_node_health(node_id: Uuid, endpoint: &str) -> Result<()> {
        // Implement actual health check logic
        // This would make HTTP requests or WebSocket connections to verify node health
        debug!("Checking health for node {} at {}", node_id, endpoint);
        Ok(())
    }
}

pub struct MetricsCollector {
    // Metrics collection implementation
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn record_assignment(&self, assignment: &SessionAssignment) {
        debug!("Recording assignment: session {} -> node {}", assignment.session_id, assignment.node_id);
        // Implement metrics recording
    }
}
