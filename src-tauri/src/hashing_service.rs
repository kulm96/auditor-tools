use anyhow::{Context, Result};
use sha2::{Digest, Sha512};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct HashingService {}

impl HashingService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn hash_file_sha512(&self, file_path: &Path) -> Result<String> {
        let mut file = File::open(file_path)
            .with_context(|| format!("Failed to open file for hashing: {}", file_path.display()))?;

        let mut hasher = Sha512::new();
        let mut buffer = vec![0u8; 8192]; // 8KB buffer

        loop {
            let bytes_read = file.read(&mut buffer)
                .with_context(|| format!("Failed to read file for hashing: {}", file_path.display()))?;
            
            if bytes_read == 0 {
                break;
            }
            
            hasher.update(&buffer[..bytes_read]);
        }

        let hash = hasher.finalize();
        let hash_hex = hex::encode(hash);
        
        Ok(hash_hex)
    }
}

