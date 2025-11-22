use crate::ClientError;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncSeekExt};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// File transfer manager
pub struct FileTransferManager {
    active_transfers: Arc<Mutex<Vec<FileTransfer>>>,
    download_dir: PathBuf,
}

impl FileTransferManager {
    /// Create a new file transfer manager
    pub fn new(download_dir: PathBuf) -> Self {
        Self {
            active_transfers: Arc::new(Mutex::new(Vec::new())),
            download_dir,
        }
    }

    /// Start sending a file
    pub async fn send_file(&self, file_path: &Path) -> Result<FileTransfer, ClientError> {
        let metadata = tokio::fs::metadata(file_path).await
            .map_err(|e| ClientError::IoError(format!("Failed to read file metadata: {}", e)))?;

        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| ClientError::IoError("Invalid file name".to_string()))?
            .to_string();

        let transfer = FileTransfer {
            id: Uuid::new_v4().to_string(),
            file_name,
            file_size: metadata.len(),
            file_path: file_path.to_path_buf(),
            direction: TransferDirection::Sending,
            status: TransferStatus::Pending,
            bytes_transferred: 0,
            chunk_size: 64 * 1024, // 64 KB chunks
            started_at: None,
            completed_at: None,
        };

        let mut transfers = self.active_transfers.lock().await;
        transfers.push(transfer.clone());

        tracing::info!("Started file transfer: {} ({} bytes)", transfer.file_name, transfer.file_size);

        Ok(transfer)
    }

    /// Start receiving a file
    pub async fn receive_file(&self, file_name: String, file_size: u64, transfer_id: String) -> Result<FileTransfer, ClientError> {
        let file_path = self.download_dir.join(&file_name);

        let transfer = FileTransfer {
            id: transfer_id,
            file_name,
            file_size,
            file_path,
            direction: TransferDirection::Receiving,
            status: TransferStatus::Pending,
            bytes_transferred: 0,
            chunk_size: 64 * 1024,
            started_at: None,
            completed_at: None,
        };

        let mut transfers = self.active_transfers.lock().await;
        transfers.push(transfer.clone());

        tracing::info!("Started receiving file: {} ({} bytes)", transfer.file_name, transfer.file_size);

        Ok(transfer)
    }

    /// Read next chunk from file
    pub async fn read_chunk(&self, transfer_id: &str) -> Result<Option<Vec<u8>>, ClientError> {
        let mut transfers = self.active_transfers.lock().await;
        let transfer = transfers.iter_mut()
            .find(|t| t.id == transfer_id)
            .ok_or_else(|| ClientError::IoError("Transfer not found".to_string()))?;

        if transfer.bytes_transferred >= transfer.file_size {
            return Ok(None); // Transfer complete
        }

        let mut file = File::open(&transfer.file_path).await
            .map_err(|e| ClientError::IoError(format!("Failed to open file: {}", e)))?;

        // Seek to current position
        file.seek(std::io::SeekFrom::Start(transfer.bytes_transferred)).await
            .map_err(|e| ClientError::IoError(format!("Failed to seek: {}", e)))?;

        // Read chunk
        let chunk_size = std::cmp::min(transfer.chunk_size as u64, transfer.file_size - transfer.bytes_transferred) as usize;
        let mut buffer = vec![0u8; chunk_size];
        let bytes_read = file.read(&mut buffer).await
            .map_err(|e| ClientError::IoError(format!("Failed to read: {}", e)))?;

        buffer.truncate(bytes_read);
        transfer.bytes_transferred += bytes_read as u64;

        // Update status
        if transfer.status == TransferStatus::Pending {
            transfer.status = TransferStatus::InProgress;
            transfer.started_at = Some(std::time::Instant::now());
        }

        if transfer.bytes_transferred >= transfer.file_size {
            transfer.status = TransferStatus::Completed;
            transfer.completed_at = Some(std::time::Instant::now());
        }

        Ok(Some(buffer))
    }

    /// Write chunk to file
    pub async fn write_chunk(&self, transfer_id: &str, data: &[u8]) -> Result<(), ClientError> {
        let mut transfers = self.active_transfers.lock().await;
        let transfer = transfers.iter_mut()
            .find(|t| t.id == transfer_id)
            .ok_or_else(|| ClientError::IoError("Transfer not found".to_string()))?;

        // Open or create file
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&transfer.file_path)
            .await
            .map_err(|e| ClientError::IoError(format!("Failed to open file: {}", e)))?;

        // Seek to current position
        file.seek(std::io::SeekFrom::Start(transfer.bytes_transferred)).await
            .map_err(|e| ClientError::IoError(format!("Failed to seek: {}", e)))?;

        // Write chunk
        file.write_all(data).await
            .map_err(|e| ClientError::IoError(format!("Failed to write: {}", e)))?;

        transfer.bytes_transferred += data.len() as u64;

        // Update status
        if transfer.status == TransferStatus::Pending {
            transfer.status = TransferStatus::InProgress;
            transfer.started_at = Some(std::time::Instant::now());
        }

        if transfer.bytes_transferred >= transfer.file_size {
            transfer.status = TransferStatus::Completed;
            transfer.completed_at = Some(std::time::Instant::now());
            tracing::info!("File transfer completed: {}", transfer.file_name);
        }

        Ok(())
    }

    /// Get transfer by ID
    pub async fn get_transfer(&self, transfer_id: &str) -> Option<FileTransfer> {
        let transfers = self.active_transfers.lock().await;
        transfers.iter().find(|t| t.id == transfer_id).cloned()
    }

    /// Get all active transfers
    pub async fn get_all_transfers(&self) -> Vec<FileTransfer> {
        let transfers = self.active_transfers.lock().await;
        transfers.clone()
    }

    /// Cancel a transfer
    pub async fn cancel_transfer(&self, transfer_id: &str) -> Result<(), ClientError> {
        let mut transfers = self.active_transfers.lock().await;
        if let Some(transfer) = transfers.iter_mut().find(|t| t.id == transfer_id) {
            transfer.status = TransferStatus::Cancelled;
            tracing::info!("Transfer cancelled: {}", transfer.file_name);
        }
        Ok(())
    }

    /// Remove completed transfers
    pub async fn cleanup_completed(&self) {
        let mut transfers = self.active_transfers.lock().await;
        transfers.retain(|t| !matches!(t.status, TransferStatus::Completed | TransferStatus::Cancelled));
    }
}

/// File transfer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransfer {
    pub id: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_path: PathBuf,
    pub direction: TransferDirection,
    pub status: TransferStatus,
    pub bytes_transferred: u64,
    pub chunk_size: usize,
    #[serde(skip)]
    pub started_at: Option<std::time::Instant>,
    #[serde(skip)]
    pub completed_at: Option<std::time::Instant>,
}

impl FileTransfer {
    /// Get transfer progress (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        if self.file_size == 0 {
            return 1.0;
        }
        self.bytes_transferred as f64 / self.file_size as f64
    }

    /// Get transfer speed in bytes per second
    pub fn speed(&self) -> Option<f64> {
        if let Some(started) = self.started_at {
            let elapsed = started.elapsed().as_secs_f64();
            if elapsed > 0.0 {
                return Some(self.bytes_transferred as f64 / elapsed);
            }
        }
        None
    }

    /// Get estimated time remaining in seconds
    pub fn eta(&self) -> Option<f64> {
        if let Some(speed) = self.speed() {
            if speed > 0.0 {
                let remaining = self.file_size - self.bytes_transferred;
                return Some(remaining as f64 / speed);
            }
        }
        None
    }

    /// Format file size as human-readable string
    pub fn format_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }

    /// Format speed as human-readable string
    pub fn format_speed(&self) -> String {
        if let Some(speed) = self.speed() {
            format!("{}/s", Self::format_size(speed as u64))
        } else {
            "-- B/s".to_string()
        }
    }

    /// Format ETA as human-readable string
    pub fn format_eta(&self) -> String {
        if let Some(eta) = self.eta() {
            let seconds = eta as u64;
            if seconds < 60 {
                format!("{}s", seconds)
            } else if seconds < 3600 {
                format!("{}m {}s", seconds / 60, seconds % 60)
            } else {
                format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
            }
        } else {
            "--".to_string()
        }
    }
}

/// Transfer direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferDirection {
    Sending,
    Receiving,
}

/// Transfer status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for TransferStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::InProgress => write!(f, "In Progress"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_size_formatting() {
        assert_eq!(FileTransfer::format_size(500), "500.00 B");
        assert_eq!(FileTransfer::format_size(1024), "1.00 KB");
        assert_eq!(FileTransfer::format_size(1024 * 1024), "1.00 MB");
        assert_eq!(FileTransfer::format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_transfer_progress() {
        let transfer = FileTransfer {
            id: "test".to_string(),
            file_name: "test.txt".to_string(),
            file_size: 1000,
            file_path: PathBuf::from("test.txt"),
            direction: TransferDirection::Sending,
            status: TransferStatus::InProgress,
            bytes_transferred: 500,
            chunk_size: 64 * 1024,
            started_at: None,
            completed_at: None,
        };

        assert_eq!(transfer.progress(), 0.5);
    }

    #[tokio::test]
    async fn test_file_transfer_manager() {
        let temp_dir = std::env::temp_dir();
        let manager = FileTransferManager::new(temp_dir);

        let transfers = manager.get_all_transfers().await;
        assert_eq!(transfers.len(), 0);
    }
}
