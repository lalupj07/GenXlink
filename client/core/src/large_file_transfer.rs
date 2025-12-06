use crate::ClientError;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs::{File, OpenOptions, metadata};
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncSeekExt};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Large file transfer manager with optimized chunking and parallel processing
pub struct LargeFileTransferManager {
    active_transfers: Arc<Mutex<HashMap<String, LargeFileTransfer>>>,
    config: LargeFileConfig,
}

/// Configuration for large file transfers
#[derive(Debug, Clone)]
pub struct LargeFileConfig {
    /// Chunk size for large files (bytes)
    pub chunk_size: usize,
    /// Number of parallel upload streams
    pub parallel_streams: usize,
    /// Enable compression for large files
    pub enable_compression: bool,
    /// Compression level (1-9)
    pub compression_level: u32,
    /// Enable resume support
    pub enable_resume: bool,
    /// Maximum memory usage per transfer (bytes)
    pub max_memory_usage: usize,
    /// Progress reporting interval (bytes)
    pub progress_interval: u64,
}

impl Default for LargeFileConfig {
    fn default() -> Self {
        Self {
            chunk_size: 4 * 1024 * 1024, // 4 MB chunks
            parallel_streams: 8,
            enable_compression: true,
            compression_level: 6,
            enable_resume: true,
            max_memory_usage: 100 * 1024 * 1024, // 100 MB
            progress_interval: 1024 * 1024, // 1 MB
        }
    }
}

/// Large file transfer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeFileTransfer {
    pub id: String,
    pub file_path: PathBuf,
    pub file_name: String,
    pub file_size: u64,
    pub total_chunks: usize,
    pub completed_chunks: Vec<usize>,
    pub failed_chunks: Vec<usize>,
    pub chunk_checksums: HashMap<usize, String>,
    pub status: LargeFileTransferStatus,
    pub progress: f64,
    pub speed: f64,
    pub eta: Option<f64>,
    pub created_at: std::time::SystemTime,
    #[serde(skip)]
    pub started_at: Option<std::time::Instant>,
    #[serde(skip)]
    pub completed_at: Option<std::time::Instant>,
}

/// Large file transfer status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LargeFileTransferStatus {
    Preparing,
    Ready,
    Uploading,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// File chunk for parallel transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunk {
    pub transfer_id: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub offset: u64,
    pub size: usize,
    pub data: Vec<u8>,
    pub checksum: String,
    pub compressed: bool,
}

/// Transfer progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    pub transfer_id: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub chunks_completed: usize,
    pub total_chunks: usize,
    pub speed: f64,
    pub eta: Option<f64>,
}

impl LargeFileTransferManager {
    /// Create a new large file transfer manager
    pub fn new(config: LargeFileConfig) -> Self {
        Self {
            active_transfers: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Initialize a large file transfer
    pub async fn initialize_transfer(&self, file_path: &Path) -> Result<LargeFileTransfer, ClientError> {
        let metadata = metadata(file_path).await
            .map_err(|e| ClientError::IoError(format!("Failed to read file metadata: {}", e)))?;

        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| ClientError::IoError("Invalid file name".to_string()))?
            .to_string();

        let file_size = metadata.len();
        let total_chunks = (file_size + self.config.chunk_size as u64 - 1) / self.config.chunk_size as u64;

        let transfer = LargeFileTransfer {
            id: Uuid::new_v4().to_string(),
            file_path: file_path.to_path_buf(),
            file_name,
            file_size,
            total_chunks: total_chunks as usize,
            completed_chunks: Vec::new(),
            failed_chunks: Vec::new(),
            chunk_checksums: HashMap::new(),
            status: LargeFileTransferStatus::Preparing,
            progress: 0.0,
            speed: 0.0,
            eta: None,
            created_at: std::time::SystemTime::now(),
            started_at: None,
            completed_at: None,
        };

        // Store the transfer
        let mut transfers = self.active_transfers.lock().await;
        transfers.insert(transfer.id.clone(), transfer.clone());

        tracing::info!("Initialized large file transfer: {} ({} bytes, {} chunks)", 
            transfer.file_name, transfer.file_size, transfer.total_chunks);

        Ok(transfer)
    }

    /// Prepare chunks for transfer
    pub async fn prepare_chunks(&self, transfer_id: &str) -> Result<Vec<FileChunk>, ClientError> {
        let mut transfers = self.active_transfers.lock().await;
        let transfer = transfers.get_mut(transfer_id)
            .ok_or_else(|| ClientError::IoError("Transfer not found".to_string()))?;

        if transfer.status != LargeFileTransferStatus::Preparing {
            return Err(ClientError::IoError("Transfer not in preparing state".to_string()));
        }

        let mut chunks = Vec::new();
        let mut file = File::open(&transfer.file_path).await
            .map_err(|e| ClientError::IoError(format!("Failed to open file: {}", e)))?;

        for chunk_index in 0..transfer.total_chunks {
            let offset = chunk_index as u64 * self.config.chunk_size as u64;
            let remaining_size = transfer.file_size - offset;
            let chunk_size = std::cmp::min(self.config.chunk_size, remaining_size as usize);

            // Seek to chunk position
            file.seek(std::io::SeekFrom::Start(offset)).await
                .map_err(|e| ClientError::IoError(format!("Failed to seek to chunk: {}", e)))?;

            // Read chunk data
            let mut buffer = vec![0u8; chunk_size];
            let bytes_read = file.read(&mut buffer).await
                .map_err(|e| ClientError::IoError(format!("Failed to read chunk: {}", e)))?;

            buffer.truncate(bytes_read);

            // Compress if enabled
            let compressed = self.config.enable_compression;
            let data = if compressed {
                self.compress_data(&buffer)?
            } else {
                buffer
            };

            // Calculate checksum
            let checksum = self.calculate_checksum(&data);

            let chunk = FileChunk {
                transfer_id: transfer_id.to_string(),
                chunk_index,
                total_chunks: transfer.total_chunks,
                offset,
                size: data.len(),
                data,
                checksum: checksum.clone(),
                compressed,
            };

            chunks.push(chunk);
            transfer.chunk_checksums.insert(chunk_index, checksum);
        }

        transfer.status = LargeFileTransferStatus::Ready;
        tracing::info!("Prepared {} chunks for transfer: {}", chunks.len(), transfer.file_name);

        Ok(chunks)
    }

    /// Start parallel upload
    pub async fn start_parallel_upload(&self, transfer_id: &str) -> Result<(), ClientError> {
        let mut transfers = self.active_transfers.lock().await;
        let transfer = transfers.get_mut(transfer_id)
            .ok_or_else(|| ClientError::IoError("Transfer not found".to_string()))?;

        if transfer.status != LargeFileTransferStatus::Ready {
            return Err(ClientError::IoError("Transfer not ready".to_string()));
        }

        transfer.status = LargeFileTransferStatus::Uploading;
        transfer.started_at = Some(std::time::Instant::now());

        // Start parallel upload tasks
        let transfer_id = transfer_id.to_string();
        let config = self.config.clone();
        let transfers = self.active_transfers.clone();
        
        for stream_index in 0..config.parallel_streams {
            let transfer_id_clone = transfer_id.clone();
            let config_clone = config.clone();
            let transfers_clone = transfers.clone();
            
            tokio::spawn(async move {
                // Simulate parallel upload
                LargeFileTransferManager::upload_chunk_stream(transfer_id_clone, stream_index, config_clone, transfers_clone).await;
            });
        }

        tracing::info!("Started parallel upload for transfer: {}", transfer.file_name);
        Ok(())
    }

    /// Upload chunk stream (simulated)
    async fn upload_chunk_stream(transfer_id: String, _stream_index: usize, _config: LargeFileConfig, transfers: Arc<Mutex<HashMap<String, LargeFileTransfer>>>) {
        // This would be implemented with actual network upload logic
        // For now, simulate the upload process
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let mut transfers_guard = transfers.lock().await;
        if let Some(transfer) = transfers_guard.get_mut(&transfer_id) {
            // Update progress
            let progress = transfer.completed_chunks.len() as f64 / transfer.total_chunks as f64;
            transfer.progress = progress;
            
            // Calculate speed and ETA
            if let Some(started) = transfer.started_at {
                let elapsed = started.elapsed().as_secs_f64();
                if elapsed > 0.0 {
                    transfer.speed = (transfer.file_size as f64 * progress) / elapsed;
                    if transfer.speed > 0.0 {
                        transfer.eta = Some((transfer.file_size as f64 * (1.0 - progress)) / transfer.speed);
                    }
                }
            }
            
            // Check if completed
            if transfer.completed_chunks.len() == transfer.total_chunks {
                transfer.status = LargeFileTransferStatus::Completed;
                transfer.completed_at = Some(std::time::Instant::now());
                tracing::info!("Completed large file transfer: {}", transfer.file_name);
            }
        }
    }

    /// Process received chunk
    pub async fn process_received_chunk(&self, chunk: FileChunk) -> Result<(), ClientError> {
        let mut transfers = self.active_transfers.lock().await;
        let transfer = transfers.get_mut(&chunk.transfer_id)
            .ok_or_else(|| ClientError::IoError("Transfer not found".to_string()))?;

        // Verify checksum
        let received_checksum = self.calculate_checksum(&chunk.data);
        if received_checksum != chunk.checksum {
            return Err(ClientError::IoError("Chunk checksum verification failed".to_string()));
        }

        // Decompress if needed
        let data = if chunk.compressed {
            self.decompress_data(&chunk.data)?
        } else {
            chunk.data.to_vec()
        };

        // Write chunk to file
        self.write_chunk_to_file(&transfer.file_path, chunk.offset, &data).await?;

        // Mark chunk as completed
        transfer.completed_chunks.push(chunk.chunk_index);
        transfer.failed_chunks.retain(|&i| i != chunk.chunk_index);

        // Update progress
        transfer.progress = transfer.completed_chunks.len() as f64 / transfer.total_chunks as f64;

        Ok(())
    }

    /// Write chunk to file
    async fn write_chunk_to_file(&self, file_path: &Path, offset: u64, data: &[u8]) -> Result<(), ClientError> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(file_path)
            .await
            .map_err(|e| ClientError::IoError(format!("Failed to open file for writing: {}", e)))?;

        file.seek(std::io::SeekFrom::Start(offset)).await
            .map_err(|e| ClientError::IoError(format!("Failed to seek in file: {}", e)))?;

        file.write_all(data).await
            .map_err(|e| ClientError::IoError(format!("Failed to write chunk: {}", e)))?;

        Ok(())
    }

    /// Compress data using simple compression
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, ClientError> {
        // For demonstration, use a simple compression approach
        // In production, use proper compression libraries like zstd or lz4
        Ok(data.to_vec())
    }

    /// Decompress data
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, ClientError> {
        // For demonstration, return the data as-is
        // In production, implement proper decompression
        Ok(data.to_vec())
    }

    /// Calculate checksum for data
    fn calculate_checksum(&self, data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// Get transfer progress
    pub async fn get_transfer_progress(&self, transfer_id: &str) -> Result<TransferProgress, ClientError> {
        let transfers = self.active_transfers.lock().await;
        let transfer = transfers.get(transfer_id)
            .ok_or_else(|| ClientError::IoError("Transfer not found".to_string()))?;

        let bytes_transferred = (transfer.completed_chunks.len() as u64 * self.config.chunk_size as u64)
            .min(transfer.file_size);

        Ok(TransferProgress {
            transfer_id: transfer_id.to_string(),
            bytes_transferred,
            total_bytes: transfer.file_size,
            chunks_completed: transfer.completed_chunks.len(),
            total_chunks: transfer.total_chunks,
            speed: transfer.speed,
            eta: transfer.eta,
        })
    }

    /// Cancel transfer
    pub async fn cancel_transfer(&self, transfer_id: &str) -> Result<(), ClientError> {
        let mut transfers = self.active_transfers.lock().await;
        if let Some(transfer) = transfers.get_mut(transfer_id) {
            transfer.status = LargeFileTransferStatus::Cancelled;
            tracing::info!("Cancelled large file transfer: {}", transfer.file_name);
        }
        Ok(())
    }

    /// Pause transfer
    pub async fn pause_transfer(&self, transfer_id: &str) -> Result<(), ClientError> {
        let mut transfers = self.active_transfers.lock().await;
        if let Some(transfer) = transfers.get_mut(transfer_id) {
            transfer.status = LargeFileTransferStatus::Paused;
            tracing::info!("Paused large file transfer: {}", transfer.file_name);
        }
        Ok(())
    }

    /// Resume transfer
    pub async fn resume_transfer(&self, transfer_id: &str) -> Result<(), ClientError> {
        let mut transfers = self.active_transfers.lock().await;
        if let Some(transfer) = transfers.get_mut(transfer_id) {
            if transfer.status == LargeFileTransferStatus::Paused {
                transfer.status = LargeFileTransferStatus::Uploading;
                tracing::info!("Resumed large file transfer: {}", transfer.file_name);
            }
        }
        Ok(())
    }

    /// Get all active transfers
    pub async fn get_all_transfers(&self) -> Vec<LargeFileTransfer> {
        let transfers = self.active_transfers.lock().await;
        transfers.values().cloned().collect()
    }

    /// Cleanup completed transfers
    pub async fn cleanup_completed(&self) {
        let mut transfers = self.active_transfers.lock().await;
        transfers.retain(|_, t| !matches!(t.status, LargeFileTransferStatus::Completed | LargeFileTransferStatus::Cancelled | LargeFileTransferStatus::Failed));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_large_file_config_default() {
        let config = LargeFileConfig::default();
        assert_eq!(config.chunk_size, 4 * 1024 * 1024);
        assert_eq!(config.parallel_streams, 8);
        assert!(config.enable_compression);
    }

    #[tokio::test]
    async fn test_large_file_transfer_creation() {
        let config = LargeFileConfig::default();
        let manager = LargeFileTransferManager::new(config);

        let test_file = std::env::temp_dir().join("test_large_file.txt");
        tokio::fs::write(&test_file, "This is a test file for large file transfer").await.unwrap();

        let transfer = manager.initialize_transfer(&test_file).await.unwrap();
        assert!(!transfer.id.is_empty());
        assert_eq!(transfer.file_name, "test_large_file.txt");
        assert_eq!(transfer.status, LargeFileTransferStatus::Preparing);
        
        // Cleanup
        tokio::fs::remove_file(test_file).await.unwrap();
    }

    #[test]
    fn test_checksum_calculation() {
        let config = LargeFileConfig::default();
        let manager = LargeFileTransferManager::new(config);
        
        let data = b"Hello, world!";
        let checksum1 = manager.calculate_checksum(data);
        let checksum2 = manager.calculate_checksum(data);
        
        assert_eq!(checksum1, checksum2);
        assert!(!checksum1.is_empty());
    }
}
