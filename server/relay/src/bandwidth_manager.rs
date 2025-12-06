use anyhow::{Result, anyhow};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, error, warn, debug};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::load_balancer::{RelayNode, HealthStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthAllocation {
    pub session_id: Uuid,
    pub node_id: Uuid,
    pub allocated_bandwidth: u64, // Mbps
    pub guaranteed_bandwidth: u64, // Mbps
    pub peak_bandwidth: u64, // Mbps
    pub priority: QoSPriority,
    pub allocated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub usage_pattern: UsagePattern,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QoSPriority {
    Critical = 4,   // Real-time audio/video
    High = 3,       // Interactive desktop sharing
    Normal = 2,     // File transfers
    Low = 1,        // Background sync
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePattern {
    pub average_bitrate: u64, // Mbps
    pub peak_bitrate: u64,    // Mbps
    pub burst_tolerance: f64, // Percentage over average allowed
    pub variability: f64,     // 0.0 (constant) to 1.0 (highly variable)
    pub latency_sensitivity: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthPool {
    pub node_id: Uuid,
    pub total_bandwidth: u64, // Mbps
    pub allocated_bandwidth: u64, // Mbps
    pub available_bandwidth: u64, // Mbps
    pub reserved_bandwidth: u64, // Mbps for critical services
    pub allocations: HashMap<Uuid, BandwidthAllocation>,
    pub priority_queues: HashMap<QoSPriority, VecDeque<Uuid>>,
    pub utilization_history: VecDeque<BandwidthSnapshot>,
    pub qos_policies: QoSPolicies,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QoSPolicies {
    pub critical_reserve_percentage: f64, // % of bandwidth reserved for critical traffic
    pub high_priority_limit: f64,         // % of bandwidth for high priority
    pub burst_allowance: f64,             // % over allocation allowed for bursts
    pub fair_share_threshold: f64,        // Threshold for fair share enforcement
    pub congestion_control: CongestionControl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CongestionControl {
    pub enabled: bool,
    pub algorithm: CongestionAlgorithm,
    pub target_utilization: f64, // Target utilization percentage
    pub backoff_factor: f64,     // Factor to reduce allocation during congestion
    pub recovery_factor: f64,    // Factor for recovery after congestion
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CongestionAlgorithm {
    TokenBucket,
    LeakyBucket,
    RandomEarlyDetection,
    WeightedFairQueueing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthSnapshot {
    pub timestamp: DateTime<Utc>,
    pub total_allocated: u64,
    pub actual_usage: u64,
    pub utilization_percentage: f64,
    pub active_sessions: usize,
    pub queue_depths: HashMap<QoSPriority, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthRequest {
    pub session_id: Uuid,
    pub requested_bandwidth: u64, // Mbps
    pub guaranteed_bandwidth: u64, // Mbps
    pub priority: QoSPriority,
    pub usage_pattern: UsagePattern,
    pub duration_hint: Option<chrono::Duration>,
    pub adaptive_allocation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthAdjustment {
    pub session_id: Uuid,
    pub old_bandwidth: u64,
    pub new_bandwidth: u64,
    pub reason: AdjustmentReason,
    pub adjusted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdjustmentReason {
    Congestion,
    Underutilization,
    PriorityPreemption,
    QualityOfService,
    AdaptiveOptimization,
    PolicyEnforcement,
}

pub struct BandwidthManager {
    pools: Arc<RwLock<HashMap<Uuid, BandwidthPool>>>,
    allocations: Arc<RwLock<HashMap<Uuid, BandwidthAllocation>>>,
    adjustment_history: Arc<RwLock<VecDeque<BandwidthAdjustment>>>,
    monitoring_task: Option<tokio::task::JoinHandle<()>>,
    config: BandwidthManagerConfig,
}

#[derive(Debug, Clone)]
pub struct BandwidthManagerConfig {
    pub monitoring_interval: std::time::Duration,
    pub history_retention: usize,
    pub adaptive_adjustment: bool,
    pub auto_scaling: bool,
    pub congestion_threshold: f64,
    pub underutilization_threshold: f64,
}

impl Default for BandwidthManagerConfig {
    fn default() -> Self {
        Self {
            monitoring_interval: std::time::Duration::from_secs(5),
            history_retention: 1000,
            adaptive_adjustment: true,
            auto_scaling: true,
            congestion_threshold: 0.85, // 85% utilization triggers congestion control
            underutilization_threshold: 0.3, // 30% utilization triggers scaling down
        }
    }
}

impl BandwidthManager {
    pub fn new(config: BandwidthManagerConfig) -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            allocations: Arc::new(RwLock::new(HashMap::new())),
            adjustment_history: Arc::new(RwLock::new(VecDeque::new())),
            monitoring_task: None,
            config,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        // Start monitoring task
        let pools = self.pools.clone();
        let allocations = self.allocations.clone();
        let adjustment_history = self.adjustment_history.clone();
        let config = self.config.clone();
        
        self.monitoring_task = Some(tokio::spawn(async move {
            Self::monitoring_loop(pools, allocations, adjustment_history, config).await;
        }));
        
        info!("Bandwidth manager initialized with monitoring");
        Ok(())
    }

    pub async fn add_node(&self, node: RelayNode) -> Result<()> {
        let pool = BandwidthPool {
            node_id: node.id,
            total_bandwidth: node.bandwidth_limit,
            allocated_bandwidth: 0,
            available_bandwidth: node.bandwidth_limit,
            reserved_bandwidth: (node.bandwidth_limit as f64 * 0.1) as u64, // Reserve 10% for critical
            allocations: HashMap::new(),
            priority_queues: HashMap::new(),
            utilization_history: VecDeque::new(),
            qos_policies: QoSPolicies::default(),
        };
        
        let mut pools = self.pools.write().await;
        pools.insert(node.id, pool);
        
        info!("Added bandwidth pool for node {} ({} Mbps)", node.id, node.bandwidth_limit);
        Ok(())
    }

    pub async fn remove_node(&self, node_id: Uuid) -> Result<()> {
        // Reallocate all sessions from this node
        self.reallocate_sessions(node_id).await?;
        
        let mut pools = self.pools.write().await;
        pools.remove(&node_id);
        
        info!("Removed bandwidth pool for node {}", node_id);
        Ok(())
    }

    pub async fn request_bandwidth(&self, request: BandwidthRequest) -> Result<BandwidthAllocation> {
        let pools = self.pools.read().await;
        
        // Find suitable pool with available bandwidth
        let suitable_pool = self.find_suitable_pool(&pools, &request).await?;
        
        // Create allocation
        let allocation = BandwidthAllocation {
            session_id: request.session_id,
            node_id: suitable_pool.node_id,
            allocated_bandwidth: request.requested_bandwidth,
            guaranteed_bandwidth: request.guaranteed_bandwidth,
            peak_bandwidth: (request.requested_bandwidth as f64 * (1.0 + request.usage_pattern.burst_tolerance / 100.0)) as u64,
            priority: request.priority,
            allocated_at: Utc::now(),
            expires_at: request.duration_hint.map(|d| Utc::now() + d),
            usage_pattern: request.usage_pattern,
        };
        
        // Update pool
        drop(pools);
        self.update_pool_allocation(allocation.clone()).await?;
        
        // Store allocation
        let mut allocations = self.allocations.write().await;
        allocations.insert(request.session_id, allocation.clone());
        
        info!("Allocated {} Mbps to session {} on node {}", 
              allocation.allocated_bandwidth, request.session_id, suitable_pool.node_id);
        
        Ok(allocation)
    }

    async fn find_suitable_pool(&self, pools: &HashMap<Uuid, BandwidthPool>, request: &BandwidthRequest) -> Result<BandwidthPool> {
        let mut candidates: Vec<&BandwidthPool> = pools
            .values()
            .filter(|pool| {
                // Check if pool has enough available bandwidth
                pool.available_bandwidth >= request.requested_bandwidth &&
                // Check QoS policy constraints
                self.check_qos_constraints(pool, request).unwrap_or(false)
            })
            .collect();
        
        if candidates.is_empty() {
            return Err(anyhow!("No suitable bandwidth pool available"));
        }
        
        // Sort by utilization (prefer less utilized pools)
        candidates.sort_by(|a, b| {
            let util_a = a.allocated_bandwidth as f64 / a.total_bandwidth as f64;
            let util_b = b.allocated_bandwidth as f64 / b.total_bandwidth as f64;
            util_a.partial_cmp(&util_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(candidates[0].clone())
    }

    fn check_qos_constraints(&self, pool: &BandwidthPool, request: &BandwidthRequest) -> Result<bool> {
        let policies = &pool.qos_policies;
        
        // Check critical bandwidth reservation
        let critical_reserve = (pool.total_bandwidth as f64 * policies.critical_reserve_percentage / 100.0) as u64;
        let available_for_request = pool.available_bandwidth - critical_reserve;
        
        if request.priority == QoSPriority::Critical {
            // Critical traffic can use reserved bandwidth
            Ok(pool.available_bandwidth >= request.requested_bandwidth)
        } else {
            // Other traffic cannot use reserved bandwidth
            Ok(available_for_request >= request.requested_bandwidth)
        }
    }

    async fn update_pool_allocation(&self, allocation: BandwidthAllocation) -> Result<()> {
        let mut pools = self.pools.write().await;
        if let Some(pool) = pools.get_mut(&allocation.node_id) {
            pool.allocated_bandwidth += allocation.allocated_bandwidth;
            pool.available_bandwidth = pool.total_bandwidth - pool.allocated_bandwidth - pool.reserved_bandwidth;
            pool.allocations.insert(allocation.session_id, allocation.clone());
            
            // Add to priority queue
            pool.priority_queues
                .entry(allocation.priority)
                .or_insert_with(VecDeque::new)
                .push_back(allocation.session_id);
        }
        Ok(())
    }

    pub async fn release_bandwidth(&self, session_id: Uuid) -> Result<()> {
        let mut allocations = self.allocations.write().await;
        if let Some(allocation) = allocations.remove(&session_id) {
            // Update pool
            let mut pools = self.pools.write().await;
            if let Some(pool) = pools.get_mut(&allocation.node_id) {
                pool.allocated_bandwidth -= allocation.allocated_bandwidth;
                pool.available_bandwidth = pool.total_bandwidth - pool.allocated_bandwidth - pool.reserved_bandwidth;
                pool.allocations.remove(&session_id);
                
                // Remove from priority queue
                if let Some(queue) = pool.priority_queues.get_mut(&allocation.priority) {
                    queue.retain(|&id| id != session_id);
                }
            }
            
            info!("Released {} Mbps from session {}", allocation.allocated_bandwidth, session_id);
        }
        Ok(())
    }

    pub async fn adjust_bandwidth(&self, session_id: Uuid, new_bandwidth: u64, reason: AdjustmentReason) -> Result<()> {
        let mut allocations = self.allocations.write().await;
        if let Some(allocation) = allocations.get_mut(&session_id) {
            let old_bandwidth = allocation.allocated_bandwidth;
            
            // Update allocation
            allocation.allocated_bandwidth = new_bandwidth;
            allocation.peak_bandwidth = (new_bandwidth as f64 * (1.0 + allocation.usage_pattern.burst_tolerance / 100.0)) as u64;
            
            // Update pool
            let mut pools = self.pools.write().await;
            if let Some(pool) = pools.get_mut(&allocation.node_id) {
                pool.allocated_bandwidth = pool.allocated_bandwidth - old_bandwidth + new_bandwidth;
                pool.available_bandwidth = pool.total_bandwidth - pool.allocated_bandwidth - pool.reserved_bandwidth;
            }
            
            // Record adjustment
            let adjustment = BandwidthAdjustment {
                session_id,
                old_bandwidth,
                new_bandwidth,
                reason: reason.clone(),
                adjusted_at: Utc::now(),
            };
            
            drop(allocations);
            let mut history = self.adjustment_history.write().await;
            history.push_back(adjustment);
            
            // Trim history if needed
            if history.len() > 1000 {
                history.pop_front();
            }
            
            info!("Adjusted bandwidth for session {} from {} to {} Mbps ({:?})", 
                  session_id, old_bandwidth, new_bandwidth, reason);
        }
        Ok(())
    }

    async fn reallocating_sessions(&self, from_node_id: Uuid) -> Result<()> {
        let allocations = self.allocations.read().await;
        let affected_sessions: Vec<BandwidthAllocation> = allocations
            .values()
            .filter(|alloc| alloc.node_id == from_node_id)
            .cloned()
            .collect();
        
        drop(allocations);
        
        for allocation in affected_sessions {
            // Create new request for reallocation
            let request = BandwidthRequest {
                session_id: allocation.session_id,
                requested_bandwidth: allocation.allocated_bandwidth,
                guaranteed_bandwidth: allocation.guaranteed_bandwidth,
                priority: allocation.priority,
                usage_pattern: allocation.usage_pattern,
                duration_hint: None,
                adaptive_allocation: false,
            };
            
            // Release from old node
            self.release_bandwidth(allocation.session_id).await?;
            
            // Allocate to new node
            if let Err(e) = self.request_bandwidth(request).await {
                error!("Failed to reallocate session {}: {}", allocation.session_id, e);
            }
        }
        
        Ok(())
    }

    async fn reallocate_sessions(&self, from_node_id: Uuid) -> Result<()> {
        self.reallocating_sessions(from_node_id).await
    }

    async fn monitoring_loop(
        pools: Arc<RwLock<HashMap<Uuid, BandwidthPool>>>,
        allocations: Arc<RwLock<HashMap<Uuid, BandwidthAllocation>>>,
        adjustment_history: Arc<RwLock<VecDeque<BandwidthAdjustment>>>,
        config: BandwidthManagerConfig,
    ) {
        let mut interval = tokio::time::interval(config.monitoring_interval);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = Self::perform_monitoring(&pools, &allocations, &adjustment_history, &config).await {
                error!("Bandwidth monitoring error: {}", e);
            }
        }
    }

    async fn perform_monitoring(
        pools: &Arc<RwLock<HashMap<Uuid, BandwidthPool>>>,
        allocations: &Arc<RwLock<HashMap<Uuid, BandwidthAllocation>>>,
        _adjustment_history: &Arc<RwLock<VecDeque<BandwidthAdjustment>>>,
        config: &BandwidthManagerConfig,
    ) -> Result<()> {
        let mut pools_guard = pools.write().await;
        
        for (node_id, pool) in pools_guard.iter_mut() {
            // Create current snapshot
            let snapshot = BandwidthSnapshot {
                timestamp: Utc::now(),
                total_allocated: pool.allocated_bandwidth,
                actual_usage: pool.allocated_bandwidth, // Would be actual measurement in production
                utilization_percentage: pool.allocated_bandwidth as f64 / pool.total_bandwidth as f64,
                active_sessions: pool.allocations.len(),
                queue_depths: pool.priority_queues.iter().map(|(p, q)| (p.clone(), q.len())).collect(),
            };
            
            // Add to history
            pool.utilization_history.push_back(snapshot.clone());
            
            // Trim history
            while pool.utilization_history.len() > config.history_retention {
                pool.utilization_history.pop_front();
            }
            
            // Check for congestion or underutilization
            if config.adaptive_adjustment {
                Self::handle_adaptive_adjustments(pool, &snapshot, config).await?;
            }
        }
        
        Ok(())
    }

    async fn handle_adaptive_adjustments(
        pool: &mut BandwidthPool,
        snapshot: &BandwidthSnapshot,
        config: &BandwidthManagerConfig,
    ) -> Result<()> {
        let utilization = snapshot.utilization_percentage;
        
        if utilization > config.congestion_threshold {
            // Congestion detected - reduce allocations
            warn!("Congestion detected on node {} ({}% utilization)", pool.node_id, utilization * 100.0);
            Self::apply_congestion_control(pool, config).await?;
        } else if utilization < config.underutilization_threshold {
            // Underutilization - can increase allocations
            debug!("Underutilization detected on node {} ({}% utilization)", pool.node_id, utilization * 100.0);
            Self::apply_scaling_up(pool, config).await?;
        }
        
        Ok(())
    }

    async fn apply_congestion_control(pool: &mut BandwidthPool, config: &BandwidthManagerConfig) -> Result<()> {
        let policies = &pool.qos_policies;
        if !policies.congestion_control.enabled {
            return Ok(());
        }
        
        // Reduce allocations for lower priority sessions first
        let priorities = [QoSPriority::Low, QoSPriority::Normal, QoSPriority::High];
        
        for priority in priorities {
            if let Some(queue) = pool.priority_queues.get(&priority) {
                for &session_id in queue {
                    if let Some(allocation) = pool.allocations.get(&session_id) {
                        let reduction = (allocation.allocated_bandwidth as f64 * policies.congestion_control.backoff_factor) as u64;
                        let new_bandwidth = allocation.allocated_bandwidth - reduction;
                        
                        if new_bandwidth >= allocation.guaranteed_bandwidth {
                            pool.allocated_bandwidth -= reduction;
                            pool.available_bandwidth += reduction;
                            
                            // Update allocation
                            // In real implementation, this would trigger a callback to the session
                            info!("Reduced bandwidth for session {} due to congestion: {} -> {} Mbps", 
                                  session_id, allocation.allocated_bandwidth, new_bandwidth);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn apply_scaling_up(pool: &mut BandwidthPool, _config: &BandwidthManagerConfig) -> Result<()> {
        // Increase allocations for sessions that can benefit from more bandwidth
        for allocation in pool.allocations.values_mut() {
            if allocation.allocated_bandwidth < allocation.peak_bandwidth {
                let increase = std::cmp::min(
                    (allocation.peak_bandwidth - allocation.allocated_bandwidth) / 4, // Increase by 25% of remaining capacity
                    pool.available_bandwidth / 4, // Don't use more than 25% of available bandwidth
                );
                
                if increase > 0 {
                    allocation.allocated_bandwidth += increase;
                    pool.allocated_bandwidth += increase;
                    pool.available_bandwidth -= increase;
                    
                    info!("Increased bandwidth for session {}: +{} Mbps (now {} Mbps)", 
                          allocation.session_id, increase, allocation.allocated_bandwidth);
                }
            }
        }
        
        Ok(())
    }

    pub async fn get_bandwidth_metrics(&self) -> BandwidthMetrics {
        let pools = self.pools.read().await;
        let allocations = self.allocations.read().await;
        
        let total_nodes = pools.len();
        let total_capacity: u64 = pools.values().map(|p| p.total_bandwidth).sum();
        let total_allocated: u64 = pools.values().map(|p| p.allocated_bandwidth).sum();
        let total_available: u64 = pools.values().map(|p| p.available_bandwidth).sum();
        let total_sessions = allocations.len();
        
        let utilization_percentage = if total_capacity > 0 {
            total_allocated as f64 / total_capacity as f64
        } else {
            0.0
        };
        
        BandwidthMetrics {
            total_nodes,
            total_capacity,
            total_allocated,
            total_available,
            total_sessions,
            utilization_percentage,
        }
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(task) = self.monitoring_task.take() {
            task.abort();
        }
        info!("Bandwidth manager shutdown");
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthMetrics {
    pub total_nodes: usize,
    pub total_capacity: u64,
    pub total_allocated: u64,
    pub total_available: u64,
    pub total_sessions: usize,
    pub utilization_percentage: f64,
}

impl Default for QoSPolicies {
    fn default() -> Self {
        Self {
            critical_reserve_percentage: 10.0,
            high_priority_limit: 60.0,
            burst_allowance: 20.0,
            fair_share_threshold: 0.8,
            congestion_control: CongestionControl {
                enabled: true,
                algorithm: CongestionAlgorithm::WeightedFairQueueing,
                target_utilization: 0.75,
                backoff_factor: 0.25,
                recovery_factor: 1.1,
            },
        }
    }
}
