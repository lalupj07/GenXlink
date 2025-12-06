use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use tokio::sync::{RwLock, mpsc};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, error, warn, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::media_manager::{FileManager, FileInfo};
use crate::crypto::encryption::EncryptionManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferSession {
    pub id: Uuid,
    pub file_info: FileInfo,
    pub status: TransferStatus,
    pub progress: f64, // 0.0 to 1.0
    pub bytes_transferred: u64,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

pub struct FileTransferManager {
    active_transfers: Arc<RwLock<HashMap<Uuid, FileTransferSession>>>,
    encryption: Arc<EncryptionManager>,
    download_dir: PathBuf,
    max_file_size: u64,
    chunk_size: usize,
}

impl FileTransferManager {
    pub fn new(
        encryption: Arc<EncryptionManager>,
        download_dir: PathBuf,
        max_file_size: u64,
        chunk_size: usize,
    ) -> Result<Self> {
        // Ensure download directory exists
        std::fs::create_dir_all(&download_dir)?;
        
        Ok(Self {
            active_transfers: Arc::new(RwLock::new(HashMap::new())),
            encryption,
            download_dir,
            max_file_size,
            chunk_size,
        })
    }

    pub async fn send_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        recipient_id: String,
        compression: bool,
    ) -> Result<Uuid> {
        let file_path = file_path.as_ref();
        
        // Get file metadata
        let metadata = tokio::fs::metadata(file_path).await?;
        let file_size = metadata.len();
        
        if file_size > self.max_file_size {
            return Err(anyhow!("File size {} exceeds maximum allowed {}", file_size, self.max_file_size));
        }

        // Create file info
        let file_info = FileInfo {
            id: Uuid::new_v4(),
            name: file_path.file_name()
                .ok_or_else(|| anyhow!("Invalid file name"))?
                .to_string_lossy()
                .to_string(),
            size: file_size,
            mime_type: self.get_mime_type(file_path).await?,
            checksum: String::new(), // Will be calculated
            created_at: Utc::now(),
            compression_enabled: compression,
            encryption_enabled: true,
            metadata: serde_json::json!({
                "sender": "current_user",
                "recipient": recipient_id,
                "original_path": file_path.to_string_lossy(),
            }),
        };

        // Create transfer session
        let session = FileTransferSession {
            id: file_info.id,
            file_info: file_info.clone(),
            status: TransferStatus::Pending,
            progress: 0.0,
            bytes_transferred: 0,
            started_at: Utc::now(),
            completed_at: None,
            error_message: None,
        };

        // Store session
        let mut transfers = self.active_transfers.write().await;
        transfers.insert(file_info.id, session.clone());
        drop(transfers);

        info!("Starting file transfer: {} ({})", file_info.name, file_info.id);

        // Start transfer in background
        let transfers_ref = self.active_transfers.clone();
        let encryption = self.encryption.clone();
        let file_path = file_path.to_path_buf();
        let chunk_size = self.chunk_size;
        
        tokio::spawn(async move {
            if let Err(e) = Self::perform_file_upload(
                file_path,
                file_info,
                transfers_ref,
                encryption,
                chunk_size,
            ).await {
                error!("File upload failed: {}", e);
            }
        });

        Ok(file_info.id)
    }

    async fn perform_file_upload(
        file_path: PathBuf,
        file_info: FileInfo,
        transfers: Arc<RwLock<HashMap<Uuid, FileTransferSession>>>,
        encryption: Arc<EncryptionManager>,
        chunk_size: usize,
    ) -> Result<()> {
        // Update status to in progress
        {
            let mut transfers_guard = transfers.write().await;
            if let Some(session) = transfers_guard.get_mut(&file_info.id) {
                session.status = TransferStatus::InProgress;
                session.started_at = Utc::now();
            }
        }

        // Open file
        let mut file = File::open(&file_path).await?;
        let mut buffer = vec![0u8; chunk_size];
        let mut bytes_transferred = 0u64;
        let file_size = file_info.size;

        // Calculate checksum while reading
        let mut hasher = sha2::Sha256::new();

        while bytes_transferred < file_size {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }

            let chunk = &buffer[..bytes_read];
            hasher.update(chunk);

            // Compress if enabled
            let processed_chunk = if file_info.compression_enabled {
                self.compress_chunk(chunk).await?
            } else {
                chunk.to_vec()
            };

            // Encrypt chunk
            let encrypted_chunk = encryption.encrypt_data(&processed_chunk).await?;

            // Send chunk (in real implementation, this would go through WebRTC data channel)
            debug!("Sending chunk {} of {} bytes", bytes_transferred / chunk_size as u64 + 1, encrypted_chunk.len());

            // Update progress
            bytes_transferred += bytes_read as u64;
            let progress = bytes_transferred as f64 / file_size as f64;

            {
                let mut transfers_guard = transfers.write().await;
                if let Some(session) = transfers_guard.get_mut(&file_info.id) {
                    session.progress = progress;
                    session.bytes_transferred = bytes_transferred;
                }
            }

            // Small delay to prevent overwhelming the network
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Update file info with checksum
        let checksum = format!("{:x}", hasher.finalize());
        
        // Mark as completed
        {
            let mut transfers_guard = transfers.write().await;
            if let Some(session) = transfers_guard.get_mut(&file_info.id) {
                session.status = TransferStatus::Completed;
                session.progress = 1.0;
                session.completed_at = Some(Utc::now());
                session.file_info.checksum = checksum;
            }
        }

        info!("File upload completed: {} ({})", file_info.name, file_info.id);
        Ok(())
    }

    pub async fn receive_file(&self, file_info: FileInfo) -> Result<Uuid> {
        // Validate file info
        if file_info.size > self.max_file_size {
            return Err(anyhow!("File size {} exceeds maximum allowed {}", file_info.size, self.max_file_size));
        }

        // Create transfer session
        let session = FileTransferSession {
            id: file_info.id,
            file_info: file_info.clone(),
            status: TransferStatus::Pending,
            progress: 0.0,
            bytes_transferred: 0,
            started_at: Utc::now(),
            completed_at: None,
            error_message: None,
        };

        // Store session
        let mut transfers = self.active_transfers.write().await;
        transfers.insert(file_info.id, session.clone());
        drop(transfers);

        info!("Starting file download: {} ({})", file_info.name, file_info.id);

        // Create destination file
        let dest_path = self.download_dir.join(&file_info.name);
        let dest_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&dest_path)
            .await?;

        // Start download in background
        let transfers_ref = self.active_transfers.clone();
        let encryption = self.encryption.clone();
        let chunk_size = self.chunk_size;
        
        tokio::spawn(async move {
            if let Err(e) = Self::perform_file_download(
                file_info,
                dest_path,
                transfers_ref,
                encryption,
                chunk_size,
            ).await {
                error!("File download failed: {}", e);
            }
        });

        Ok(file_info.id)
    }

    async fn perform_file_download(
        file_info: FileInfo,
        dest_path: PathBuf,
        transfers: Arc<RwLock<HashMap<Uuid, FileTransferSession>>>,
        encryption: Arc<EncryptionManager>,
        chunk_size: usize,
    ) -> Result<()> {
        // Update status to in progress
        {
            let mut transfers_guard = transfers.write().await;
            if let Some(session) = transfers_guard.get_mut(&file_info.id) {
                session.status = TransferStatus::InProgress;
                session.started_at = Utc::now();
            }
        }

        // Open destination file
        let mut dest_file = OpenOptions::new()
            .write(true)
            .open(&dest_path)
            .await?;

        let mut bytes_received = 0u64;
        let file_size = file_info.size;
        let mut hasher = sha2::Sha256::new();

        // Simulate receiving chunks (in real implementation, this would come from WebRTC data channel)
        while bytes_received < file_size {
            // Simulate receiving encrypted chunk
            let chunk_size = std::cmp::min(chunk_size as u64, file_size - bytes_received) as usize;
            let mut encrypted_chunk = vec![0u8; chunk_size];
            
            // In real implementation, you'd receive this from the data channel
            // For now, we'll simulate with dummy data
            for i in 0..chunk_size {
                encrypted_chunk[i] = (i % 256) as u8;
            }

            // Decrypt chunk
            let decrypted_chunk = encryption.decrypt_data(&encrypted_chunk).await?;

            // Decompress if enabled
            let processed_chunk = if file_info.compression_enabled {
                Self::decompress_chunk(&decrypted_chunk).await?
            } else {
                decrypted_chunk
            };

            // Write to file
            dest_file.write_all(&processed_chunk).await?;
            hasher.update(&processed_chunk);

            // Update progress
            bytes_received += chunk_size as u64;
            let progress = bytes_received as f64 / file_size as f64;

            {
                let mut transfers_guard = transfers.write().await;
                if let Some(session) = transfers_guard.get_mut(&file_info.id) {
                    session.progress = progress;
                    session.bytes_transferred = bytes_received;
                }
            }

            // Simulate network delay
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        // Verify checksum
        let received_checksum = format!("{:x}", hasher.finalize());
        if received_checksum != file_info.checksum {
            return Err(anyhow!("File checksum mismatch: expected {}, got {}", file_info.checksum, received_checksum));
        }

        // Mark as completed
        {
            let mut transfers_guard = transfers.write().await;
            if let Some(session) = transfers_guard.get_mut(&file_info.id) {
                session.status = TransferStatus::Completed;
                session.progress = 1.0;
                session.completed_at = Some(Utc::now());
            }
        }

        info!("File download completed: {} ({})", file_info.name, file_info.id);
        Ok(())
    }

    async fn compress_chunk(&self, chunk: &[u8]) -> Result<Vec<u8>> {
        // Simple compression using run-length encoding
        // In production, use better compression algorithms like LZ4 or Zstd
        
        let mut compressed = Vec::new();
        let mut i = 0;
        
        while i < chunk.len() {
            let current_byte = chunk[i];
            let mut count = 1;
            
            // Count consecutive identical bytes
            while i + count < chunk.len() && chunk[i + count] == current_byte && count < 255 {
                count += 1;
            }
            
            if count > 3 || current_byte == 0 {
                // Use run-length encoding
                compressed.push(0); // Escape byte
                compressed.push(count as u8);
                compressed.push(current_byte);
            } else {
                // Use literal bytes
                for _ in 0..count {
                    compressed.push(current_byte);
                }
            }
            
            i += count;
        }
        
        debug!("Compressed chunk from {} to {} bytes", chunk.len(), compressed.len());
        Ok(compressed)
    }

    async fn decompress_chunk(compressed_chunk: &[u8]) -> Result<Vec<u8>> {
        let mut decompressed = Vec::new();
        let mut i = 0;
        
        while i < compressed_chunk.len() {
            if compressed_chunk[i] == 0 && i + 2 < compressed_chunk.len() {
                // Run-length encoded sequence
                let count = compressed_chunk[i + 1] as usize;
                let byte_value = compressed_chunk[i + 2];
                
                for _ in 0..count {
                    decompressed.push(byte_value);
                }
                
                i += 3;
            } else {
                // Literal byte
                decompressed.push(compressed_chunk[i]);
                i += 1;
            }
        }
        
        debug!("Decompressed chunk from {} to {} bytes", compressed_chunk.len(), decompressed.len());
        Ok(decompressed)
    }

    async fn get_mime_type(&self, file_path: &Path) -> Result<String> {
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        let mime_type = match extension.to_lowercase().as_str() {
            "txt" => "text/plain",
            "pdf" => "application/pdf",
            "doc" | "docx" => "application/msword",
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "mp4" => "video/mp4",
            "mp3" => "audio/mpeg",
            "zip" => "application/zip",
            "exe" => "application/octet-stream",
            _ => "application/octet-stream",
        };
        
        Ok(mime_type.to_string())
    }

    pub async fn cancel_transfer(&self, transfer_id: Uuid) -> Result<()> {
        let mut transfers = self.active_transfers.write().await;
        
        if let Some(session) = transfers.get_mut(&transfer_id) {
            session.status = TransferStatus::Cancelled;
            session.completed_at = Some(Utc::now());
            info!("Cancelled transfer: {}", transfer_id);
            Ok(())
        } else {
            Err(anyhow!("Transfer not found: {}", transfer_id))
        }
    }

    pub async fn pause_transfer(&self, transfer_id: Uuid) -> Result<()> {
        let mut transfers = self.active_transfers.write().await;
        
        if let Some(session) = transfers.get_mut(&transfer_id) {
            if matches!(session.status, TransferStatus::InProgress) {
                session.status = TransferStatus::Paused;
                info!("Paused transfer: {}", transfer_id);
                Ok(())
            } else {
                Err(anyhow!("Cannot pause transfer in status: {:?}", session.status))
            }
        } else {
            Err(anyhow!("Transfer not found: {}", transfer_id))
        }
    }

    pub async fn resume_transfer(&self, transfer_id: Uuid) -> Result<()> {
        let mut transfers = self.active_transfers.write().await;
        
        if let Some(session) = transfers.get_mut(&transfer_id) {
            if matches!(session.status, TransferStatus::Paused) {
                session.status = TransferStatus::InProgress;
                info!("Resumed transfer: {}", transfer_id);
                Ok(())
            } else {
                Err(anyhow!("Cannot resume transfer in status: {:?}", session.status))
            }
        } else {
            Err(anyhow!("Transfer not found: {}", transfer_id))
        }
    }

    pub async fn get_transfer_status(&self, transfer_id: Uuid) -> Result<Option<FileTransferSession>> {
        let transfers = self.active_transfers.read().await;
        Ok(transfers.get(&transfer_id).cloned())
    }

    pub async fn get_active_transfers(&self) -> Vec<FileTransferSession> {
        let transfers = self.active_transfers.read().await;
        transfers.values().cloned().collect()
    }

    pub async fn cleanup_completed_transfers(&self, older_than_hours: u64) -> Result<usize> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(older_than_hours as i64);
        let mut transfers = self.active_transfers.write().await;
        
        let initial_count = transfers.len();
        transfers.retain(|_, session| {
            !matches!(session.status, TransferStatus::Completed | TransferStatus::Failed | TransferStatus::Cancelled)
                || session.completed_at.map_or(true, |completed| completed > cutoff_time)
        });
        
        let cleaned_count = initial_count - transfers.len();
        info!("Cleaned up {} completed transfers", cleaned_count);
        Ok(cleaned_count)
    }
}

#[async_trait::async_trait]
impl FileManager for FileTransferManager {
    async fn send_file(&self, file_info: FileInfo, data: Vec<u8>) -> Result<()> {
        // This is called by the media manager when receiving file data
        // Create a new transfer session for the received file
        self.receive_file(file_info).await?;
        Ok(())
    }

    async fn receive_file(&self, file_id: Uuid) -> Result<FileInfo> {
        let transfers = self.active_transfers.read().await;
        
        if let Some(session) = transfers.get(&file_id) {
            Ok(session.file_info.clone())
        } else {
            Err(anyhow!("Transfer not found: {}", file_id))
        }
    }

    async fn cancel_transfer(&self, file_id: Uuid) -> Result<()> {
        self.cancel_transfer(file_id).await
    }
}
