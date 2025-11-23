use anyhow::Result;
use chrono::{DateTime, Utc};
use postgrest::Postgrest;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<String>,
    pub email: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: Option<String>,
    pub device_id: String,
    pub device_name: String,
    pub device_type: String,
    pub ip_address: Option<String>,
    pub last_seen: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: Option<String>,
    pub from_device_id: String,
    pub to_device_id: String,
    pub connection_type: String,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i32>,
    pub status: String,
    pub error_message: Option<String>,
}

pub struct Database {
    client: Postgrest,
}

impl Database {
    pub fn new() -> Result<Self> {
        let supabase_url = env::var("SUPABASE_URL")
            .unwrap_or_else(|_| "https://xdzwbouvcmhhfnfsnffo.supabase.co".to_string())
            .trim()
            .to_string();
        
        let api_key = env::var("SUPABASE_ANON_KEY")
            .unwrap_or_else(|_| "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Inhkendib3V2Y21oaGZuZnNuZmZvIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjM4ODY5MTAsImV4cCI6MjA3OTQ2MjkxMH0.TZZx9dvos-8gXhuEuOU_I6SXNeFCY_3Rni80VTF6j0U".to_string())
            .trim()
            .to_string();

        // Validate that we have proper values
        if supabase_url.is_empty() || api_key.is_empty() {
            return Err(anyhow::anyhow!("SUPABASE_URL and SUPABASE_ANON_KEY must be set"));
        }

        let client = Postgrest::new(&supabase_url)
            .insert_header("apikey", api_key.as_str())
            .insert_header("Authorization", format!("Bearer {}", api_key).as_str());

        Ok(Self { client })
    }

    /// Register or update a device
    pub async fn upsert_device(&self, device: &Device) -> Result<()> {
        let body = serde_json::to_string(device)?;
        
        self.client
            .from("devices")
            .upsert(body)
            .on_conflict("device_id")
            .execute()
            .await?;

        Ok(())
    }

    /// Get all devices
    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        let response = self.client
            .from("devices")
            .select("*")
            .order("last_seen.desc")
            .execute()
            .await?;

        let body = response.text().await?;
        
        // Handle empty response or parse as array
        if body.is_empty() || body == "{}" {
            return Ok(Vec::new());
        }
        
        let devices: Vec<Device> = serde_json::from_str(&body)
            .unwrap_or_else(|_| Vec::new());
        
        Ok(devices)
    }

    /// Get active devices (last seen within 5 minutes)
    pub async fn get_active_devices(&self) -> Result<Vec<Device>> {
        let response = self.client
            .from("active_devices")
            .select("*")
            .execute()
            .await?;

        let body = response.text().await?;
        
        // Handle empty response or parse as array
        if body.is_empty() || body == "{}" {
            return Ok(Vec::new());
        }
        
        let devices: Vec<Device> = serde_json::from_str(&body)
            .unwrap_or_else(|_| Vec::new());
        
        Ok(devices)
    }

    /// Get a specific device by ID
    pub async fn get_device(&self, device_id: &str) -> Result<Option<Device>> {
        let response = self.client
            .from("devices")
            .select("*")
            .eq("device_id", device_id)
            .single()
            .execute()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;
            let device: Device = serde_json::from_str(&body)?;
            Ok(Some(device))
        } else {
            Ok(None)
        }
    }

    /// Update device last seen timestamp
    pub async fn update_last_seen(&self, device_id: &str) -> Result<()> {
        let now = Utc::now();
        let body = serde_json::json!({
            "last_seen": now
        });

        self.client
            .from("devices")
            .update(body.to_string())
            .eq("device_id", device_id)
            .execute()
            .await?;

        Ok(())
    }

    /// Create a new connection record
    pub async fn create_connection(&self, connection: &Connection) -> Result<String> {
        let body = serde_json::to_string(connection)?;
        
        let response = self.client
            .from("connections")
            .insert(body)
            .execute()
            .await?;

        let body = response.text().await?;
        let result: Vec<Connection> = serde_json::from_str(&body)?;
        
        Ok(result.first()
            .and_then(|c| c.id.clone())
            .unwrap_or_default())
    }

    /// Update connection status
    pub async fn update_connection_status(
        &self,
        connection_id: &str,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<()> {
        let mut body = serde_json::json!({
            "status": status
        });

        if let Some(error) = error_message {
            body["error_message"] = serde_json::json!(error);
        }

        if status == "completed" || status == "failed" {
            body["ended_at"] = serde_json::json!(Utc::now());
        }

        self.client
            .from("connections")
            .update(body.to_string())
            .eq("id", connection_id)
            .execute()
            .await?;

        Ok(())
    }

    /// Get connection history
    pub async fn get_connection_history(&self, limit: i32) -> Result<Vec<Connection>> {
        let response = self.client
            .from("connection_history")
            .select("*")
            .limit(limit as usize)
            .execute()
            .await?;

        let body = response.text().await?;
        let connections: Vec<Connection> = serde_json::from_str(&body)?;
        
        Ok(connections)
    }

    /// Delete a device
    pub async fn delete_device(&self, device_id: &str) -> Result<()> {
        self.client
            .from("devices")
            .delete()
            .eq("device_id", device_id)
            .execute()
            .await?;

        Ok(())
    }

    // ========================================================================
    // User Management Methods
    // ========================================================================

    /// Create a new user
    pub async fn create_user(&self, user: &User) -> Result<String> {
        let body = serde_json::to_string(user)?;
        
        let response = self.client
            .from("app_users")
            .insert(body)
            .execute()
            .await?;

        let body = response.text().await?;
        let result: Vec<User> = serde_json::from_str(&body)?;
        
        Ok(result.first()
            .and_then(|u| u.id.clone())
            .unwrap_or_default())
    }

    /// Get user by email
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let response = self.client
            .from("app_users")
            .select("*")
            .eq("email", email)
            .single()
            .execute()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;
            if body.is_empty() || body == "{}" {
                return Ok(None);
            }
            let user: User = serde_json::from_str(&body)?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    /// Update user last login
    pub async fn update_last_login(&self, user_id: &str) -> Result<()> {
        let now = Utc::now();
        let body = serde_json::json!({
            "last_login": now
        });

        self.client
            .from("app_users")
            .update(body.to_string())
            .eq("id", user_id)
            .execute()
            .await?;

        Ok(())
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new().expect("Failed to create database client")
    }
}
