use crate::entry::LSTEntry;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Configuration for LST logging
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogConfig {
    pub max_entries: usize,
    pub enable_persistence: bool,
    pub log_path: Option<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            enable_persistence: false,
            log_path: None,
        }
    }
}

/// Log-Structured Tensor - immutable, append-only log with Merkle chaining
pub struct LSTLog {
    entries: Arc<Mutex<Vec<LSTEntry>>>,
    config: LogConfig,
}

impl LSTLog {
    pub fn new(config: LogConfig) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }

    /// Append an entry to the log
    pub fn append(&self, mut entry: LSTEntry) -> Result<String, String> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        if entries.len() >= self.config.max_entries {
            return Err(format!("Log at maximum size: {}", self.config.max_entries));
        }

        // Compute hash based on previous entry
        entry.compute_hash();

        let hash = entry.current_hash.clone();

        if self.config.enable_persistence {
            // TODO: Persist to log_path
        }

        entries.push(entry);
        Ok(hash)
    }

    /// Get entry by sequence number
    pub fn get_entry(&self, seq: u64) -> Result<Option<LSTEntry>, String> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        Ok(entries.iter().find(|e| e.sequence_number == seq).cloned())
    }

    /// Get all entries
    pub fn all_entries(&self) -> Result<Vec<LSTEntry>, String> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        Ok(entries.clone())
    }

    /// Get the root hash of the entire log
    pub fn root_hash(&self) -> Result<String, String> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        match entries.last() {
            Some(entry) => Ok(entry.current_hash.clone()),
            None => Ok("0".to_string()),
        }
    }

    /// Verify entire log integrity
    pub fn verify_integrity(&self) -> Result<bool, String> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        if entries.is_empty() {
            return Ok(true);
        }

        for window in entries.windows(2) {
            let prev = &window[0];
            let curr = &window[1];

            if !curr.verify(&prev.current_hash) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Count entries in the log
    pub fn entry_count(&self) -> Result<usize, String> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        Ok(entries.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lst_log_creation() {
        let config = LogConfig::default();
        let log = LSTLog::new(config);
        assert_eq!(log.entry_count().unwrap(), 0);
    }

    #[test]
    fn test_append_entry() {
        let config = LogConfig::default();
        let log = LSTLog::new(config);

        let entry = LSTEntry::new(1, "test", "0".to_string());
        let hash = log.append(entry).unwrap();

        assert!(!hash.is_empty());
        assert_eq!(log.entry_count().unwrap(), 1);
    }

    #[test]
    fn test_verify_integrity() {
        let config = LogConfig::default();
        let log = LSTLog::new(config);

        let entry1 = LSTEntry::new(1, "test1", "0".to_string());
        let hash1 = log.append(entry1).unwrap();

        let entry2 = LSTEntry::new(2, "test2", hash1);
        log.append(entry2).unwrap();

        assert!(log.verify_integrity().unwrap());
    }
}
