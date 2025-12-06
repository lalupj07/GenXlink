use anyhow::{Result, anyhow};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, error, warn, debug};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, TimeZone, Timelike};

use crate::load_balancer::{RelayNode, GeographicLocation, HealthStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicRegion {
    pub id: String,
    pub name: String,
    pub country: String,
    pub continent: String,
    pub coordinates: GeographicCoordinate,
    pub preferred_nodes: Vec<Uuid>,
    pub backup_nodes: Vec<Uuid>,
    pub bandwidth_quota: u64, // Mbps
    pub current_usage: u64,
    pub latency_targets: HashMap<String, u32>, // Target latency to other regions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicCoordinate {
    pub latitude: f64,
    pub longitude: f64,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub id: Uuid,
    pub name: String,
    pub source_regions: Vec<String>,
    pub target_regions: Vec<String>,
    pub priority: u8,
    pub conditions: RoutingConditions,
    pub actions: RoutingActions,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConditions {
    pub max_latency: Option<u32>,
    pub min_bandwidth: Option<u64>,
    pub max_load_percentage: Option<f64>,
    pub required_protocols: Vec<String>,
    pub excluded_countries: Vec<String>,
    pub time_based_routing: Option<TimeBasedRouting>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBasedRouting {
    pub timezone: String,
    pub peak_hours: Vec<PeakHour>,
    pub off_peak_preferred_regions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakHour {
    pub start_hour: u8, // 0-23
    pub end_hour: u8,   // 0-23
    pub days: Vec<String>, // ["monday", "tuesday", etc.]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingActions {
    pub preferred_nodes: Vec<Uuid>,
    pub backup_nodes: Vec<Uuid>,
    pub bandwidth_limit: Option<u64>,
    pub quality_of_service: Option<QoSProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QoSProfile {
    pub priority: u8,
    pub guaranteed_bandwidth: u64,
    pub max_latency: u32,
    pub packet_loss_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientLocation {
    pub ip_address: String,
    pub country: String,
    pub region: String,
    pub city: String,
    pub coordinates: GeographicCoordinate,
    pub isp: String,
    pub connection_type: String, // "broadband", "mobile", "satellite"
    pub estimated_bandwidth: u64,
    pub confidence: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub session_id: Uuid,
    pub client_location: ClientLocation,
    pub selected_region: String,
    pub selected_node: Uuid,
    pub backup_nodes: Vec<Uuid>,
    pub routing_rules_applied: Vec<Uuid>,
    pub estimated_latency: u32,
    pub estimated_bandwidth: u64,
    pub confidence_score: f64,
    pub decided_at: DateTime<Utc>,
}

pub struct GeographicRouter {
    regions: Arc<RwLock<HashMap<String, GeographicRegion>>>,
    nodes: Arc<RwLock<HashMap<Uuid, RelayNode>>>,
    routing_rules: Arc<RwLock<HashMap<Uuid, RoutingRule>>>,
    location_cache: Arc<RwLock<HashMap<String, CachedLocation>>>,
    routing_metrics: Arc<RwLock<RoutingMetrics>>,
    ip_geolocation_service: Arc<dyn IpGeolocationService + Send + Sync>,
}

#[derive(Debug, Clone)]
struct CachedLocation {
    location: ClientLocation,
    cached_at: DateTime<Utc>,
    ttl: chrono::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetrics {
    pub total_routings: u64,
    pub successful_routings: u64,
    pub failed_routings: u64,
    pub average_latency: f64,
    pub region_distribution: HashMap<String, u64>,
    pub rule_hits: HashMap<Uuid, u64>,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

#[async_trait::async_trait]
pub trait IpGeolocationService {
    async fn get_location(&self, ip_address: &str) -> Result<ClientLocation>;
    async fn bulk_lookup(&self, ip_addresses: &[String]) -> Result<Vec<ClientLocation>>;
}

impl GeographicRouter {
    pub fn new(geolocation_service: Arc<dyn IpGeolocationService + Send + Sync>) -> Self {
        Self {
            regions: Arc::new(RwLock::new(HashMap::new())),
            nodes: Arc::new(RwLock::new(HashMap::new())),
            routing_rules: Arc::new(RwLock::new(HashMap::new())),
            location_cache: Arc::new(RwLock::new(HashMap::new())),
            routing_metrics: Arc::new(RwLock::new(RoutingMetrics::default())),
            ip_geolocation_service: geolocation_service,
        }
    }

    pub async fn add_region(&self, region: GeographicRegion) -> Result<()> {
        let mut regions = self.regions.write().await;
        regions.insert(region.id.clone(), region.clone());
        info!("Added geographic region: {}", region.name);
        Ok(())
    }

    pub async fn add_routing_rule(&self, rule: RoutingRule) -> Result<()> {
        let mut rules = self.routing_rules.write().await;
        rules.insert(rule.id, rule.clone());
        info!("Added routing rule: {}", rule.name);
        Ok(())
    }

    pub async fn route_session(&self, session_id: Uuid, client_ip: &str, estimated_bandwidth: u64) -> Result<RoutingDecision> {
        // Get client location
        let client_location = self.get_client_location(client_ip).await?;
        
        // Determine client region
        let client_region = self.determine_client_region(&client_location).await?;
        
        // Apply routing rules
        let applicable_rules = self.get_applicable_rules(&client_region, &client_location).await?;
        
        // Select optimal region and node
        let (selected_region, selected_node, backup_nodes) = self.select_optimal_route(
            &client_region,
            &client_location,
            estimated_bandwidth,
            &applicable_rules,
        ).await?;
        
        // Calculate metrics
        let estimated_latency = self.estimate_latency(&client_location, &selected_region).await?;
        let confidence_score = self.calculate_confidence_score(&client_location, &selected_region, &applicable_rules).await?;
        
        let decision = RoutingDecision {
            session_id,
            client_location,
            selected_region: selected_region.clone(),
            selected_node,
            backup_nodes,
            routing_rules_applied: applicable_rules.iter().map(|r| r.id).collect(),
            estimated_latency,
            estimated_bandwidth,
            confidence_score,
            decided_at: Utc::now(),
        };
        
        // Record metrics
        self.record_routing_metrics(&decision).await;
        
        info!("Routed session {} to region {} (node: {})", 
              session_id, selected_region, decision.selected_node);
        
        Ok(decision)
    }

    pub async fn get_client_location(&self, ip_address: &str) -> Result<ClientLocation> {
        // Check cache first
        {
            let cache = self.location_cache.read().await;
            if let Some(cached) = cache.get(ip_address) {
                if Utc::now() < cached.cached_at + cached.ttl {
                    self.record_cache_hit().await;
                    return Ok(cached.location.clone());
                }
            }
        }
        
        self.record_cache_miss().await;
        
        // Fetch from geolocation service
        let location = self.ip_geolocation_service.get_location(ip_address).await?;
        
        // Cache the result
        {
            let mut cache = self.location_cache.write().await;
            cache.insert(ip_address.to_string(), CachedLocation {
                location: location.clone(),
                cached_at: Utc::now(),
                ttl: chrono::Duration::hours(24), // Cache for 24 hours
            });
        }
        
        Ok(location)
    }

    async fn determine_client_region(&self, location: &ClientLocation) -> Result<String> {
        let regions = self.regions.read().await;
        
        // Find region that contains the client coordinates
        for (region_id, region) in regions.iter() {
            if self.is_point_in_bbox(
                location.coordinates.latitude,
                location.coordinates.longitude,
                &region.coordinates.bounding_box,
            ) {
                return Ok(region_id.clone());
            }
        }
        
        // If no exact match, find nearest region
        let nearest_region = self.find_nearest_region(location, &regions).await?;
        Ok(nearest_region)
    }

    async fn get_applicable_rules(&self, client_region: &str, location: &ClientLocation) -> Result<Vec<RoutingRule>> {
        let rules = self.routing_rules.read().await;
        let current_time = Utc::now();
        
        let applicable: Vec<RoutingRule> = rules
            .values()
            .filter(|rule| {
                if !rule.enabled {
                    return false;
                }
                
                // Check source region
                if !rule.source_regions.is_empty() && !rule.source_regions.contains(&client_region.to_string()) {
                    return false;
                }
                
                // Check conditions
                self.check_routing_conditions(&rule.conditions, location, current_time).unwrap_or(false)
            })
            .cloned()
            .collect();
        
        Ok(applicable)
    }

    fn check_routing_conditions(&self, conditions: &RoutingConditions, location: &ClientLocation, current_time: DateTime<Utc>) -> Result<bool> {
        // Check bandwidth condition
        if let Some(min_bandwidth) = conditions.min_bandwidth {
            if location.estimated_bandwidth < min_bandwidth {
                return Ok(false);
            }
        }
        
        // Check excluded countries
        if conditions.excluded_countries.contains(&location.country) {
            return Ok(false);
        }
        
        // Check time-based routing
        if let Some(time_based) = &conditions.time_based_routing {
            if !self.is_in_peak_hours(time_based, current_time) {
                // During off-peak hours, prefer off-peak regions
                // This would be handled in the selection logic
            }
        }
        
        Ok(true)
    }

    fn is_in_peak_hours(&self, time_based: &TimeBasedRouting, current_time: DateTime<Utc>) -> bool {
        // Convert current time to the timezone specified in the rule
        // This is simplified - in production, you'd use proper timezone handling
        let current_hour = current_time.hour() as u8;
        
        for peak_hour in &time_based.peak_hours {
            if current_hour >= peak_hour.start_hour && current_hour <= peak_hour.end_hour {
                return true;
            }
        }
        
        false
    }

    async fn select_optimal_route(
        &self,
        client_region: &str,
        location: &ClientLocation,
        estimated_bandwidth: u64,
        applicable_rules: &[RoutingRule],
    ) -> Result<(String, Uuid, Vec<Uuid>)> {
        let regions = self.regions.read().await;
        let nodes = self.nodes.read().await;
        
        // Get preferred regions from rules
        let preferred_regions: Vec<String> = applicable_rules
            .iter()
            .flat_map(|rule| rule.actions.preferred_nodes.clone())
            .map(|uuid| uuid.to_string())
            .collect();
        
        // If no specific rules, use geographic proximity
        let candidate_regions = if preferred_regions.is_empty() {
            self.get_regions_by_proximity(client_region, location, &regions).await?
        } else {
            preferred_regions
        };
        
        // Select best region with available capacity
        for region_id in candidate_regions {
            if let Some(region) = regions.get(&region_id) {
                // Check bandwidth quota
                if region.current_usage + estimated_bandwidth <= region.bandwidth_quota {
                    // Select best node in this region
                    if let Some(node_id) = self.select_best_node_in_region(region, &nodes).await? {
                        let backup_nodes = self.get_backup_nodes(region, &nodes, Some(node_id)).await?;
                        return Ok((region_id, node_id, backup_nodes));
                    }
                }
            }
        }
        
        Err(anyhow!("No available regions with sufficient capacity"))
    }

    async fn get_regions_by_proximity(&self, client_region: &str, location: &ClientLocation, regions: &HashMap<String, GeographicRegion>) -> Result<Vec<String>> {
        let mut region_distances: Vec<(String, f64)> = regions
            .iter()
            .map(|(id, region)| {
                let distance = self.calculate_distance(
                    location.coordinates.latitude,
                    location.coordinates.longitude,
                    region.coordinates.latitude,
                    region.coordinates.longitude,
                );
                (id.clone(), distance)
            })
            .collect();
        
        // Sort by distance
        region_distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(region_distances.into_iter().map(|(id, _)| id).collect())
    }

    async fn select_best_node_in_region(&self, region: &GeographicRegion, nodes: &HashMap<Uuid, RelayNode>) -> Result<Option<Uuid>> {
        // Filter nodes in this region that are healthy and have capacity
        let available_nodes: Vec<&RelayNode> = region
            .preferred_nodes
            .iter()
            .chain(region.backup_nodes.iter())
            .filter_map(|node_id| nodes.get(node_id))
            .filter(|node| {
                node.health_status == HealthStatus::Healthy &&
                node.current_load < node.capacity
            })
            .collect();
        
        if available_nodes.is_empty() {
            return Ok(None);
        }
        
        // Select node with lowest load and latency
        let best_node = available_nodes
            .iter()
            .min_by(|a, b| {
                let score_a = a.current_load as f64 + a.latency_ms as f64 * 0.1;
                let score_b = b.current_load as f64 + b.latency_ms as f64 * 0.1;
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            });
        
        Ok(best_node.map(|node| node.id))
    }

    async fn get_backup_nodes(&self, region: &GeographicRegion, nodes: &HashMap<Uuid, RelayNode>, exclude_node: Option<Uuid>) -> Result<Vec<Uuid>> {
        let backup_nodes: Vec<Uuid> = region
            .backup_nodes
            .iter()
            .filter_map(|node_id| nodes.get(node_id))
            .filter(|node| {
                node.health_status == HealthStatus::Healthy &&
                node.current_load < node.capacity &&
                exclude_node.map_or(true, |exclude| node.id != exclude)
            })
            .take(2) // Limit to 2 backup nodes
            .map(|node| node.id)
            .collect();
        
        Ok(backup_nodes)
    }

    async fn find_nearest_region(&self, location: &ClientLocation, regions: &HashMap<String, GeographicRegion>) -> Result<String> {
        let mut nearest_region = None;
        let mut min_distance = f64::MAX;
        
        for (region_id, region) in regions.iter() {
            let distance = self.calculate_distance(
                location.coordinates.latitude,
                location.coordinates.longitude,
                region.coordinates.latitude,
                region.coordinates.longitude,
            );
            
            if distance < min_distance {
                min_distance = distance;
                nearest_region = Some(region_id.clone());
            }
        }
        
        nearest_region.ok_or_else(|| anyhow!("No regions available"))
    }

    fn is_point_in_bbox(&self, lat: f64, lon: f64, bbox: &BoundingBox) -> bool {
        lat >= bbox.min_lat && lat <= bbox.max_lat && lon >= bbox.min_lon && lon <= bbox.max_lon
    }

    fn calculate_distance(&self, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        // Haversine formula
        let earth_radius = 6371.0;
        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let delta_lat = (lat2 - lat1).to_radians();
        let delta_lon = (lon2 - lon1).to_radians();
        
        let a = (delta_lat / 2.0).sin().powi(2) +
                lat1_rad.cos() * lat2_rad.cos() *
                (delta_lon / 2.0).sin().powi(2);
        
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        earth_radius * c
    }

    async fn estimate_latency(&self, location: &ClientLocation, target_region: &str) -> Result<u32> {
        let regions = self.regions.read().await;
        
        if let Some(region) = regions.get(target_region) {
            // Use base latency + distance factor
            let base_latency = 50; // 50ms base latency
            let distance_factor = self.calculate_distance(
                location.coordinates.latitude,
                location.coordinates.longitude,
                region.coordinates.latitude,
                region.coordinates.longitude,
            ) * 0.01; // 1ms per 100km
            
            Ok((base_latency + distance_factor as u32) as u32)
        } else {
            Ok(100) // Default estimate
        }
    }

    async fn calculate_confidence_score(&self, location: &ClientLocation, region: &str, rules: &[RoutingRule]) -> Result<f64> {
        let mut score = location.confidence;
        
        // Boost score if specific routing rules apply
        if !rules.is_empty() {
            score += 0.1;
        }
        
        // Reduce score based on distance
        let regions = self.regions.read().await;
        if let Some(target_region) = regions.get(region) {
            let distance = self.calculate_distance(
                location.coordinates.latitude,
                location.coordinates.longitude,
                target_region.coordinates.latitude,
                target_region.coordinates.longitude,
            );
            
            if distance > 5000.0 { // More than 5000km
                score -= 0.2;
            } else if distance > 2000.0 { // More than 2000km
                score -= 0.1;
            }
        }
        
        Ok(score.max(0.0).min(1.0))
    }

    async fn record_routing_metrics(&self, decision: &RoutingDecision) {
        let mut metrics = self.routing_metrics.write().await;
        metrics.total_routings += 1;
        metrics.successful_routings += 1;
        
        // Update region distribution
        *metrics.region_distribution.entry(decision.selected_region.clone()).or_insert(0) += 1;
        
        // Update rule hits
        for rule_id in &decision.routing_rules_applied {
            *metrics.rule_hits.entry(*rule_id).or_insert(0) += 1;
        }
        
        // Update average latency
        let total_latency = metrics.average_latency * (metrics.successful_routings - 1) as f64 + decision.estimated_latency as f64;
        metrics.average_latency = total_latency / metrics.successful_routings as f64;
    }

    async fn record_cache_hit(&self) {
        let mut metrics = self.routing_metrics.write().await;
        metrics.cache_hits += 1;
    }

    async fn record_cache_miss(&self) {
        let mut metrics = self.routing_metrics.write().await;
        metrics.cache_misses += 1;
    }

    pub async fn get_routing_metrics(&self) -> RoutingMetrics {
        self.routing_metrics.read().await.clone()
    }

    pub async fn clear_location_cache(&self) {
        let mut cache = self.location_cache.write().await;
        cache.clear();
        info!("Cleared location cache");
    }
}

impl Default for RoutingMetrics {
    fn default() -> Self {
        Self {
            total_routings: 0,
            successful_routings: 0,
            failed_routings: 0,
            average_latency: 0.0,
            region_distribution: HashMap::new(),
            rule_hits: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}
