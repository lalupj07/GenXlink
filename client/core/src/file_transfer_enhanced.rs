use crate::file_transfer::{FileTransfer, TransferDirection, TransferStatus};
use crate::ClientError;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs::{File, metadata};
use tokio::io::AsyncReadExt;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashMap;
use std::fs::Metadata as FsMetadata;

/// Enhanced file transfer manager with drag & drop and large file support
pub struct EnhancedFileTransferManager {
    active_transfers: Arc<Mutex<HashMap<String, EnhancedFileTransfer>>>,
    download_dir: PathBuf,
    config: TransferConfig,
}

/// Configuration for file transfers
#[derive(Debug, Clone)]
pub struct TransferConfig {
    /// Chunk size for file transfers (bytes)
    pub chunk_size: usize,
    /// Maximum file size for direct transfer (bytes)
    pub max_file_size: u64,
    /// Enable compression for large files
    pub enable_compression: bool,
    /// Compression threshold (bytes)
    pub compression_threshold: u64,
    /// Number of parallel chunks for large files
    pub parallel_chunks: usize,
    /// Resume support for interrupted transfers
    pub enable_resume: bool,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1024 * 1024, // 1 MB chunks (larger for better performance)
            max_file_size: 10 * 1024 * 1024 * 1024, // 10 GB max
            enable_compression: true,
            compression_threshold: 100 * 1024 * 1024, // 100 MB
            parallel_chunks: 4,
            enable_resume: true,
        }
    }
}

/// Enhanced file transfer with additional features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedFileTransfer {
    pub base_transfer: FileTransfer,
    pub file_type: FileType,
    pub checksum: Option<String>,
    pub compression_enabled: bool,
    pub parallel_transfer: bool,
    pub resume_supported: bool,
    pub priority: TransferPriority,
    pub created_at: std::time::SystemTime,
}

/// File type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    Document,
    Image,
    Video,
    Audio,
    Archive,
    Text,
    Binary,
    Unknown,
}

/// Transfer priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Drag & drop file information
#[derive(Debug, Clone)]
pub struct DroppedFile {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub file_type: FileType,
    pub last_modified: std::time::SystemTime,
}

/// File transfer chunk for parallel transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferChunk {
    pub transfer_id: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub offset: u64,
    pub size: usize,
    pub data: Vec<u8>,
    pub checksum: Option<String>,
}

impl EnhancedFileTransferManager {
    /// Create a new enhanced file transfer manager
    pub fn new(download_dir: PathBuf, config: TransferConfig) -> Self {
        Self {
            active_transfers: Arc::new(Mutex::new(HashMap::new())),
            download_dir,
            config,
        }
    }

    /// Process dropped files from drag & drop
    pub async fn process_dropped_files(&self, paths: Vec<PathBuf>) -> Result<Vec<EnhancedFileTransfer>, ClientError> {
        let mut transfers = Vec::new();
        
        for path in paths {
            if path.exists() && path.is_file() {
                let dropped_file = self.analyze_dropped_file(&path).await?;
                let transfer = self.create_transfer_from_dropped_file(dropped_file).await?;
                transfers.push(transfer);
            }
        }
        
        Ok(transfers)
    }

    /// Analyze a dropped file
    async fn analyze_dropped_file(&self, path: &Path) -> Result<DroppedFile, ClientError> {
        let metadata = metadata(path).await
            .map_err(|e| ClientError::IoError(format!("Failed to read file metadata: {}", e)))?;

        let name = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| ClientError::IoError("Invalid file name".to_string()))?
            .to_string();

        let file_type = self.classify_file_type(&name, &metadata);
        let last_modified = metadata.modified()
            .unwrap_or(std::time::SystemTime::now());

        Ok(DroppedFile {
            path: path.to_path_buf(),
            name,
            size: metadata.len(),
            file_type,
            last_modified,
        })
    }

    /// Classify file type based on extension and metadata
    fn classify_file_type(&self, name: &str, _metadata: &FsMetadata) -> FileType {
        let extension = Path::new(name)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        match extension.as_deref() {
            Some("txt") | Some("md") | Some("rtf") => FileType::Text,
            Some("doc") | Some("docx") | Some("pdf") | Some("odt") => FileType::Document,
            Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") | Some("svg") => FileType::Image,
            Some("mp4") | Some("avi") | Some("mkv") | Some("mov") | Some("wmv") => FileType::Video,
            Some("mp3") | Some("wav") | Some("flac") | Some("ogg") | Some("m4a") => FileType::Audio,
            Some("zip") | Some("rar") | Some("7z") | Some("tar") | Some("gz") => FileType::Archive,
            Some("exe") | Some("dll") | Some("so") | Some("bin") => FileType::Binary,
            _ => FileType::Unknown,
        }
    }

    /// Create transfer from dropped file
    async fn create_transfer_from_dropped_file(&self, dropped_file: DroppedFile) -> Result<EnhancedFileTransfer, ClientError> {
        // Check file size limits
        if dropped_file.size > self.config.max_file_size {
            return Err(ClientError::IoError(format!(
                "File too large: {} (max: {})", 
                dropped_file.size, 
                self.config.max_file_size
            )));
        }

        // Determine transfer settings based on file characteristics
        let compression_enabled = self.should_compress(&dropped_file);
        let parallel_transfer = self.should_use_parallel(&dropped_file);
        let priority = self.determine_priority(&dropped_file);

        // Create base transfer
        let base_transfer = FileTransfer {
            id: Uuid::new_v4().to_string(),
            file_name: dropped_file.name.clone(),
            file_size: dropped_file.size,
            file_path: dropped_file.path.clone(),
            direction: TransferDirection::Sending,
            status: TransferStatus::Pending,
            bytes_transferred: 0,
            chunk_size: self.config.chunk_size,
            started_at: None,
            completed_at: None,
        };

        // Calculate checksum for integrity verification
        let checksum = if self.config.enable_resume {
            Some(self.calculate_file_checksum(&dropped_file.path).await?)
        } else {
            None
        };

        let enhanced_transfer = EnhancedFileTransfer {
            base_transfer,
            file_type: dropped_file.file_type,
            checksum,
            compression_enabled,
            parallel_transfer,
            resume_supported: self.config.enable_resume,
            priority,
            created_at: std::time::SystemTime::now(),
        };

        // Store the transfer
        let mut transfers = self.active_transfers.lock().await;
        transfers.insert(enhanced_transfer.base_transfer.id.clone(), enhanced_transfer.clone());

        tracing::info!("Created enhanced file transfer: {} ({} bytes, type: {:?})", 
            enhanced_transfer.base_transfer.file_name, 
            enhanced_transfer.base_transfer.file_size,
            enhanced_transfer.file_type);

        Ok(enhanced_transfer)
    }

    /// Determine if file should be compressed
    fn should_compress(&self, file: &DroppedFile) -> bool {
        if !self.config.enable_compression {
            return false;
        }

        // Don't compress already compressed files
        matches!(file.file_type, FileType::Archive | FileType::Video | FileType::Audio) ||
        file.size >= self.config.compression_threshold
    }

    /// Determine if file should use parallel transfer
    fn should_use_parallel(&self, file: &DroppedFile) -> bool {
        file.size >= (self.config.chunk_size as u64 * self.config.parallel_chunks as u64 * 2)
    }

    /// Determine transfer priority based on file type and size
    fn determine_priority(&self, file: &DroppedFile) -> TransferPriority {
        match file.file_type {
            FileType::Document | FileType::Text => TransferPriority::Normal,
            FileType::Image => TransferPriority::High,
            FileType::Video | FileType::Audio => TransferPriority::Low,
            FileType::Archive => TransferPriority::Normal,
            _ => TransferPriority::Normal,
        }
    }

    /// Calculate file checksum for integrity verification
    async fn calculate_file_checksum(&self, path: &Path) -> Result<String, ClientError> {
        use sha2::{Sha256, Digest};
        
        let mut file = File::open(path).await
            .map_err(|e| ClientError::IoError(format!("Failed to open file for checksum: {}", e)))?;

        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buffer).await
                .map_err(|e| ClientError::IoError(format!("Failed to read file for checksum: {}", e)))?;
            
            if bytes_read == 0 {
                break;
            }
            
            hasher.update(&buffer[..bytes_read]);
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// Get enhanced transfer by ID
    pub async fn get_transfer(&self, transfer_id: &str) -> Option<EnhancedFileTransfer> {
        let transfers = self.active_transfers.lock().await;
        transfers.get(transfer_id).cloned()
    }

    /// Get all active transfers
    pub async fn get_all_transfers(&self) -> Vec<EnhancedFileTransfer> {
        let transfers = self.active_transfers.lock().await;
        transfers.values().cloned().collect()
    }

    /// Cancel a transfer
    pub async fn cancel_transfer(&self, transfer_id: &str) -> Result<(), ClientError> {
        let mut transfers = self.active_transfers.lock().await;
        if let Some(transfer) = transfers.get_mut(transfer_id) {
            transfer.base_transfer.status = TransferStatus::Cancelled;
            tracing::info!("Transfer cancelled: {}", transfer.base_transfer.file_name);
        }
        Ok(())
    }

    /// Remove completed transfers
    pub async fn cleanup_completed(&self) {
        let mut transfers = self.active_transfers.lock().await;
        transfers.retain(|_, t| !matches!(t.base_transfer.status, TransferStatus::Completed | TransferStatus::Cancelled));
    }

    /// Get transfer statistics
    pub async fn get_statistics(&self) -> TransferStatistics {
        let transfers = self.active_transfers.lock().await;
        let mut stats = TransferStatistics::default();

        for transfer in transfers.values() {
            match transfer.base_transfer.status {
                TransferStatus::Pending => stats.pending += 1,
                TransferStatus::InProgress => stats.in_progress += 1,
                TransferStatus::Completed => stats.completed += 1,
                TransferStatus::Failed => stats.failed += 1,
                TransferStatus::Cancelled => stats.cancelled += 1,
            }

            stats.total_size += transfer.base_transfer.file_size;
            stats.transferred += transfer.base_transfer.bytes_transferred;
        }

        stats
    }
}

/// Transfer statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransferStatistics {
    pub pending: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub total_size: u64,
    pub transferred: u64,
}

impl TransferStatistics {
    /// Get overall progress percentage
    pub fn overall_progress(&self) -> f64 {
        if self.total_size == 0 {
            0.0
        } else {
            self.transferred as f64 / self.total_size as f64
        }
    }

    /// Get active transfers count
    pub fn active_transfers(&self) -> usize {
        self.pending + self.in_progress
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_file_type_classification() {
        let config = TransferConfig::default();
        let manager = EnhancedFileTransferManager::new(std::env::temp_dir(), config);

        // Test document classification
        let doc_type = manager.classify_file_type("document.pdf", &std::fs::metadata(".").unwrap());
        assert_eq!(doc_type, FileType::Document);

        // Test image classification
        let img_type = manager.classify_file_type("image.jpg", &std::fs::metadata(".").unwrap());
        assert_eq!(img_type, FileType::Image);
    }

    #[test]
    fn test_transfer_config_default() {
        let config = TransferConfig::default();
        assert_eq!(config.chunk_size, 1024 * 1024);
        assert_eq!(config.max_file_size, 10 * 1024 * 1024 * 1024);
        assert!(config.enable_compression);
        assert_eq!(config.parallel_chunks, 4);
    }

    #[test]
    fn test_transfer_statistics() {
        let mut stats = TransferStatistics::default();
        stats.total_size = 1000;
        stats.transferred = 500;
        
        assert_eq!(stats.overall_progress(), 0.5);
        assert_eq!(stats.active_transfers(), 0);
        
        stats.in_progress = 2;
        assert_eq!(stats.active_transfers(), 2);
    }
}
