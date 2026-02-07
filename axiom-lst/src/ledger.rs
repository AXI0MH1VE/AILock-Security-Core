use crate::entry::LSTEntry;
use std::fs::{File, OpenOptions};
use std::io::{Write, BufReader};
use std::path::Path;

/// Immutable Ledger - provides persistence for LST entries with tamper-detection
pub struct ImmutableLedger {
    ledger_path: String,
}

impl ImmutableLedger {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            ledger_path: path.into(),
        }
    }

    /// Commit an entry to the immutable ledger
    pub fn commit(&self, entry: &LSTEntry) -> Result<(), String> {
        let path = Path::new(&self.ledger_path);
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;

        let serialized = serde_json::to_string(entry).map_err(|e| e.to_string())?;
        writeln!(file, "{}", serialized).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Verify the integrity of the entire ledger
    pub fn verify_ledger(&self) -> Result<bool, String> {
        let path = Path::new(&self.ledger_path);
        if !path.exists() {
            return Ok(true);
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        
        let mut prev_hash = "0".to_string();
        
        use std::io::BufRead;
        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            let entry: LSTEntry = serde_json::from_str(&line).map_err(|e| e.to_string())?;
            
            if entry.previous_hash != prev_hash {
                return Ok(false);
            }
            
            prev_hash = entry.current_hash.clone();
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::LSTEntry;
    use tempfile::tempdir;

    #[test]
    fn test_ledger_persistence_and_verification() {
        let dir = tempdir().unwrap();
        let ledger_path = dir.path().join("test_ledger.jsonl");
        let ledger = ImmutableLedger::new(ledger_path.to_str().unwrap());

        let mut entry1 = LSTEntry::new(1, "query1", "0".to_string());
        entry1.compute_hash();
        ledger.commit(&entry1).unwrap();

        let mut entry2 = LSTEntry::new(2, "query2", entry1.current_hash.clone());
        entry2.compute_hash();
        ledger.commit(&entry2).unwrap();

        assert!(ledger.verify_ledger().unwrap());
    }

    #[test]
    fn test_ledger_tamper_detection() {
        let dir = tempdir().unwrap();
        let ledger_path = dir.path().join("tamper_ledger.jsonl");
        let ledger = ImmutableLedger::new(ledger_path.to_str().unwrap());

        let mut entry1 = LSTEntry::new(1, "query1", "0".to_string());
        entry1.compute_hash();
        ledger.commit(&entry1).unwrap();

        // Manually corrupt the ledger file
        let mut file = OpenOptions::new().append(true).open(&ledger_path).unwrap();
        writeln!(file, "CORRUPT DATA").unwrap();

        assert!(ledger.verify_ledger().is_err());
    }
}
