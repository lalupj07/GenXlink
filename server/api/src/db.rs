use anyhow::Result;
use sqlx::{PgPool, Row};
use std::env;
use tracing::{info, error, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::models::*;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/genxlink".to_string());
        
        info!("Connecting to database...");
        let pool = PgPool::connect(&database_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;
        
        info!("Database connected and migrations completed");
        Ok(Database { pool })
    }

    // User operations
    pub async fn create_user(&self, user: &User) -> Result<User> {
        let row = sqlx::query!(
            r#"
            INSERT INTO users (id, email, username, display_name, avatar_url, is_active, is_verified, subscription_type, preferences)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            user.id,
            user.email,
            user.username,
            user.display_name,
            user.avatar_url,
            user.is_active,
            user.is_verified,
            user.subscription_type,
            user.preferences
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            id: row.id,
            email: row.email,
            username: row.username,
            display_name: row.display_name,
            avatar_url: row.avatar_url,
            is_active: row.is_active,
            is_verified: row.is_verified,
            subscription_type: row.subscription_type,
            created_at: row.created_at.unwrap_or_else(Utc::now),
            updated_at: row.updated_at.unwrap_or_else(Utc::now),
            last_login: row.last_login,
            preferences: row.preferences,
        })
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let row = sqlx::query!(
            "SELECT * FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| User {
            id: row.id,
            email: row.email,
            username: row.username,
            display_name: row.display_name,
            avatar_url: row.avatar_url,
            is_active: row.is_active,
            is_verified: row.is_verified,
            subscription_type: row.subscription_type,
            created_at: row.created_at.unwrap_or_else(Utc::now),
            updated_at: row.updated_at.unwrap_or_else(Utc::now),
            last_login: row.last_login,
            preferences: row.preferences,
        }))
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query!(
            "SELECT * FROM users WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| User {
            id: row.id,
            email: row.email,
            username: row.username,
            display_name: row.display_name,
            avatar_url: row.avatar_url,
            is_active: row.is_active,
            is_verified: row.is_verified,
            subscription_type: row.subscription_type,
            created_at: row.created_at.unwrap_or_else(Utc::now),
            updated_at: row.updated_at.unwrap_or_else(Utc::now),
            last_login: row.last_login,
            preferences: row.preferences,
        }))
    }

    pub async fn update_user_last_login(&self, user_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET last_login = NOW() WHERE id = $1",
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Device operations
    pub async fn register_device(&self, device: &Device) -> Result<Device> {
        let row = sqlx::query!(
            r#"
            INSERT INTO devices (user_id, device_id, device_name, device_type, os_version, ip_address, mac_address, capabilities, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (device_id) DO UPDATE SET
                device_name = EXCLUDED.device_name,
                device_type = EXCLUDED.device_type,
                os_version = EXCLUDED.os_version,
                ip_address = EXCLUDED.ip_address,
                mac_address = EXCLUDED.mac_address,
                last_seen = NOW(),
                capabilities = EXCLUDED.capabilities,
                metadata = EXCLUDED.metadata,
                updated_at = NOW()
            RETURNING *
            "#,
            device.user_id,
            device.device_id,
            device.device_name,
            device.device_type,
            device.os_version,
            device.ip_address,
            device.mac_address,
            device.capabilities,
            device.metadata
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Device {
            id: row.id,
            user_id: row.user_id,
            device_id: row.device_id,
            device_name: row.device_name,
            device_type: row.device_type,
            os_version: row.os_version,
            ip_address: row.ip_address,
            mac_address: row.mac_address,
            last_seen: row.last_seen.unwrap_or_else(Utc::now),
            is_online: row.is_online,
            capabilities: row.capabilities,
            metadata: row.metadata,
            created_at: row.created_at.unwrap_or_else(Utc::now),
            updated_at: row.updated_at.unwrap_or_else(Utc::now),
        })
    }

    pub async fn get_user_devices(&self, user_id: Uuid) -> Result<Vec<Device>> {
        let rows = sqlx::query!(
            "SELECT * FROM devices WHERE user_id = $1 ORDER BY last_seen DESC",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| Device {
            id: row.id,
            user_id: row.user_id,
            device_id: row.device_id,
            device_name: row.device_name,
            device_type: row.device_type,
            os_version: row.os_version,
            ip_address: row.ip_address,
            mac_address: row.mac_address,
            last_seen: row.last_seen.unwrap_or_else(Utc::now),
            is_online: row.is_online,
            capabilities: row.capabilities,
            metadata: row.metadata,
            created_at: row.created_at.unwrap_or_else(Utc::now),
            updated_at: row.updated_at.unwrap_or_else(Utc::now),
        }).collect())
    }

    pub async fn update_device_online_status(&self, device_id: &str, is_online: bool) -> Result<()> {
        sqlx::query!(
            "UPDATE devices SET is_online = $1, last_seen = NOW() WHERE device_id = $2",
            is_online,
            device_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Session operations
    pub async fn create_session(&self, session: &Session) -> Result<Session> {
        let row = sqlx::query!(
            r#"
            INSERT INTO sessions (user_id, device_id, remote_device_id, session_type, status, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            session.user_id,
            session.device_id,
            session.remote_device_id,
            session.session_type,
            session.status,
            session.metadata
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Session {
            id: row.id,
            user_id: row.user_id,
            device_id: row.device_id,
            remote_device_id: row.remote_device_id,
            session_type: row.session_type,
            started_at: row.started_at.unwrap_or_else(Utc::now),
            ended_at: row.ended_at,
            duration_seconds: row.duration_seconds,
            status: row.status,
            connection_quality: row.connection_quality,
            metadata: row.metadata,
        })
    }

    pub async fn get_user_sessions(&self, user_id: Uuid, limit: i64) -> Result<Vec<Session>> {
        let rows = sqlx::query!(
            "SELECT * FROM sessions WHERE user_id = $1 ORDER BY started_at DESC LIMIT $2",
            user_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| Session {
            id: row.id,
            user_id: row.user_id,
            device_id: row.device_id,
            remote_device_id: row.remote_device_id,
            session_type: row.session_type,
            started_at: row.started_at.unwrap_or_else(Utc::now),
            ended_at: row.ended_at,
            duration_seconds: row.duration_seconds,
            status: row.status,
            connection_quality: row.connection_quality,
            metadata: row.metadata,
        }).collect())
    }

    pub async fn end_session(&self, session_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE sessions 
            SET ended_at = NOW(), 
                duration_seconds = EXTRACT(EPOCH FROM (NOW() - started_at))::INTEGER,
                status = 'ended'
            WHERE id = $1
            "#,
            session_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // License operations
    pub async fn create_license(&self, license: &License) -> Result<License> {
        let row = sqlx::query!(
            r#"
            INSERT INTO licenses (user_id, license_key, license_type, expires_at, is_active, max_devices, max_concurrent_sessions, features)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            license.user_id,
            license.license_key,
            license.license_type,
            license.expires_at,
            license.is_active,
            license.max_devices,
            license.max_concurrent_sessions,
            license.features
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(License {
            id: row.id,
            user_id: row.user_id,
            license_key: row.license_key,
            license_type: row.license_type,
            expires_at: row.expires_at,
            is_active: row.is_active,
            max_devices: row.max_devices,
            max_concurrent_sessions: row.max_concurrent_sessions,
            features: row.features,
            created_at: row.created_at.unwrap_or_else(Utc::now),
            updated_at: row.updated_at.unwrap_or_else(Utc::now),
        })
    }

    pub async fn get_license_by_key(&self, license_key: &str) -> Result<Option<License>> {
        let row = sqlx::query!(
            "SELECT * FROM licenses WHERE license_key = $1 AND is_active = true",
            license_key
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| License {
            id: row.id,
            user_id: row.user_id,
            license_key: row.license_key,
            license_type: row.license_type,
            expires_at: row.expires_at,
            is_active: row.is_active,
            max_devices: row.max_devices,
            max_concurrent_sessions: row.max_concurrent_sessions,
            features: row.features,
            created_at: row.created_at.unwrap_or_else(Utc::now),
            updated_at: row.updated_at.unwrap_or_else(Utc::now),
        }))
    }

    pub async fn get_user_licenses(&self, user_id: Uuid) -> Result<Vec<License>> {
        let rows = sqlx::query!(
            "SELECT * FROM licenses WHERE user_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| License {
            id: row.id,
            user_id: row.user_id,
            license_key: row.license_key,
            license_type: row.license_type,
            expires_at: row.expires_at,
            is_active: row.is_active,
            max_devices: row.max_devices,
            max_concurrent_sessions: row.max_concurrent_sessions,
            features: row.features,
            created_at: row.created_at.unwrap_or_else(Utc::now),
            updated_at: row.updated_at.unwrap_or_else(Utc::now),
        }).collect())
    }
}
