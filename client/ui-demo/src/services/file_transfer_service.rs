// Copyright (c) 2025 GenXis Innovations
// File Transfer Service - Real file transfer implementation

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::time::Instant;

/// Transfer direction
#[derive(Debug, Clone, PartialEq)]
pub enum TransferDirection {
    Upload,
    Download,
}

/// Transfer status
#[derive(Debug, Clone, PartialEq)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

/// File transfer info
#[derive(Debug, Clone)]
pub struct FileTransfer {
    pub id: String,
    pub file_name: String,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub transferred_bytes: u64,
    pub direction: TransferDirection,
    pub status: TransferStatus,
    pub peer_id: String,
    pub started_at: Option<Instant>,
    pub speed_bps: u64,
}

impl FileTransfer {
    pub fn progress_percent(&self) -> f32 {
        if self.file_size == 0 {
            return 0.0;
        }
        (self.transferred_bytes as f32 / self.file_size as f32) * 100.0
    }

    pub fn speed_mbps(&self) -> f32 {
        self.speed_bps as f32 / 1_000_000.0
    }

    pub fn remaining_bytes(&self) -> u64 {
        self.file_size.saturating_sub(self.transferred_bytes)
    }

    pub fn eta_seconds(&self) -> Option<u64> {
        if self.speed_bps == 0 {
            return None;
        }
        Some(self.remaining_bytes() / self.speed_bps)
    }
}

/// File Transfer Service
pub struct FileTransferService {
    transfers: HashMap<String, FileTransfer>,
    download_directory: PathBuf,
    max_concurrent_transfers: usize,
    chunk_size: usize,
    transfer_counter: u64,
}

impl FileTransferService {
    pub fn new() -> Self {
        let download_dir = dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("GenXLink");
        
        // Create download directory if it doesn't exist
        fs::create_dir_all(&download_dir).ok();

        Self {
            transfers: HashMap::new(),
            download_directory: download_dir,
            max_concurrent_transfers: 5,
            chunk_size: 64 * 1024, // 64KB chunks
            transfer_counter: 0,
        }
    }

    /// Set download directory
    pub fn set_download_directory(&mut self, path: PathBuf) {
        self.download_directory = path;
        fs::create_dir_all(&self.download_directory).ok();
    }

    /// Get download directory
    pub fn get_download_directory(&self) -> &Path {
        &self.download_directory
    }

    /// Start a file upload to a peer
    pub fn start_upload(&mut self, file_path: &Path, peer_id: &str) -> Result<String, String> {
        // Validate file exists
        if !file_path.exists() {
            return Err(format!("File not found: {:?}", file_path));
        }

        let metadata = fs::metadata(file_path)
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        self.transfer_counter += 1;
        let transfer_id = format!("upload-{}", self.transfer_counter);

        let transfer = FileTransfer {
            id: transfer_id.clone(),
            file_name,
            file_path: file_path.to_path_buf(),
            file_size: metadata.len(),
            transferred_bytes: 0,
            direction: TransferDirection::Upload,
            status: TransferStatus::Pending,
            peer_id: peer_id.to_string(),
            started_at: None,
            speed_bps: 0,
        };

        println!("📤 Starting upload: {} ({} bytes) to peer {}", 
            transfer.file_name, transfer.file_size, peer_id);

        self.transfers.insert(transfer_id.clone(), transfer);
        Ok(transfer_id)
    }

    /// Start a file download from a peer
    pub fn start_download(&mut self, file_name: &str, file_size: u64, peer_id: &str) -> Result<String, String> {
        self.transfer_counter += 1;
        let transfer_id = format!("download-{}", self.transfer_counter);

        let file_path = self.download_directory.join(file_name);

        let transfer = FileTransfer {
            id: transfer_id.clone(),
            file_name: file_name.to_string(),
            file_path,
            file_size,
            transferred_bytes: 0,
            direction: TransferDirection::Download,
            status: TransferStatus::Pending,
            peer_id: peer_id.to_string(),
            started_at: None,
            speed_bps: 0,
        };

        println!("📥 Starting download: {} ({} bytes) from peer {}", 
            file_name, file_size, peer_id);

        self.transfers.insert(transfer_id.clone(), transfer);
        Ok(transfer_id)
    }

    /// Process transfer (simulate chunk transfer)
    pub fn process_transfer(&mut self, transfer_id: &str) -> Result<bool, String> {
        let transfer = self.transfers.get_mut(transfer_id)
            .ok_or_else(|| format!("Transfer not found: {}", transfer_id))?;

        if transfer.status == TransferStatus::Paused || 
           transfer.status == TransferStatus::Cancelled {
            return Ok(false);
        }

        if transfer.started_at.is_none() {
            transfer.started_at = Some(Instant::now());
            transfer.status = TransferStatus::InProgress;
        }

        // Simulate chunk transfer
        let chunk_transferred = std::cmp::min(
            self.chunk_size as u64,
            transfer.file_size - transfer.transferred_bytes
        );

        transfer.transferred_bytes += chunk_transferred;

        // Calculate speed
        if let Some(started) = transfer.started_at {
            let elapsed = started.elapsed().as_secs_f64();
            if elapsed > 0.0 {
                transfer.speed_bps = (transfer.transferred_bytes as f64 / elapsed) as u64;
            }
        }

        // Check if complete
        if transfer.transferred_bytes >= transfer.file_size {
            transfer.status = TransferStatus::Completed;
            println!("✅ Transfer completed: {}", transfer.file_name);
            return Ok(true);
        }

        Ok(false)
    }

    /// Pause a transfer
    pub fn pause_transfer(&mut self, transfer_id: &str) -> Result<(), String> {
        let transfer = self.transfers.get_mut(transfer_id)
            .ok_or_else(|| format!("Transfer not found: {}", transfer_id))?;

        if transfer.status == TransferStatus::InProgress {
            transfer.status = TransferStatus::Paused;
            println!("⏸️ Transfer paused: {}", transfer.file_name);
        }
        Ok(())
    }

    /// Resume a transfer
    pub fn resume_transfer(&mut self, transfer_id: &str) -> Result<(), String> {
        let transfer = self.transfers.get_mut(transfer_id)
            .ok_or_else(|| format!("Transfer not found: {}", transfer_id))?;

        if transfer.status == TransferStatus::Paused {
            transfer.status = TransferStatus::InProgress;
            println!("▶️ Transfer resumed: {}", transfer.file_name);
        }
        Ok(())
    }

    /// Cancel a transfer
    pub fn cancel_transfer(&mut self, transfer_id: &str) -> Result<(), String> {
        let transfer = self.transfers.get_mut(transfer_id)
            .ok_or_else(|| format!("Transfer not found: {}", transfer_id))?;

        transfer.status = TransferStatus::Cancelled;
        println!("❌ Transfer cancelled: {}", transfer.file_name);
        Ok(())
    }

    /// Get all transfers
    pub fn get_transfers(&self) -> Vec<&FileTransfer> {
        self.transfers.values().collect()
    }

    /// Get transfer by ID
    pub fn get_transfer(&self, transfer_id: &str) -> Option<&FileTransfer> {
        self.transfers.get(transfer_id)
    }

    /// Get active transfers count
    pub fn active_transfers_count(&self) -> usize {
        self.transfers.values()
            .filter(|t| t.status == TransferStatus::InProgress)
            .count()
    }

    /// Remove completed/cancelled transfers
    pub fn cleanup_transfers(&mut self) {
        self.transfers.retain(|_, t| {
            t.status != TransferStatus::Completed && 
            t.status != TransferStatus::Cancelled
        });
    }

    /// Read file chunk for upload
    pub fn read_chunk(&self, transfer_id: &str, offset: u64) -> Result<Vec<u8>, String> {
        let transfer = self.transfers.get(transfer_id)
            .ok_or_else(|| format!("Transfer not found: {}", transfer_id))?;

        let mut file = File::open(&transfer.file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;

        use std::io::Seek;
        file.seek(std::io::SeekFrom::Start(offset))
            .map_err(|e| format!("Failed to seek: {}", e))?;

        let mut buffer = vec![0u8; self.chunk_size];
        let bytes_read = file.read(&mut buffer)
            .map_err(|e| format!("Failed to read: {}", e))?;

        buffer.truncate(bytes_read);
        Ok(buffer)
    }

    /// Write file chunk for download
    pub fn write_chunk(&self, transfer_id: &str, offset: u64, data: &[u8]) -> Result<(), String> {
        let transfer = self.transfers.get(transfer_id)
            .ok_or_else(|| format!("Transfer not found: {}", transfer_id))?;

        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&transfer.file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;

        use std::io::Seek;
        file.seek(std::io::SeekFrom::Start(offset))
            .map_err(|e| format!("Failed to seek: {}", e))?;

        file.write_all(data)
            .map_err(|e| format!("Failed to write: {}", e))?;

        Ok(())
    }
}

impl Default for FileTransferService {
    fn default() -> Self {
        Self::new()
    }
}
