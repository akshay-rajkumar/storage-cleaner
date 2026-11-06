use crate::types::{Risk, ScanItem, ScanResult, Scanner};
use std::path::PathBuf;

use super::user_cache::dir_size;

/// Scans `~/.local/share/Trash` and reports its total size.
pub struct TrashScanner;

impl TrashScanner {
    fn trash_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".local/share/Trash")
        } else {
            PathBuf::from("~/.local/share/Trash")
        }
    }
}

impl Scanner for TrashScanner {
    fn id(&self) -> &'static str {
        "trash"
    }

    fn scan(&self) -> anyhow::Result<ScanResult> {
        let trash_dir = Self::trash_dir();
        let mut items = Vec::new();

        if trash_dir.exists() && trash_dir.is_dir() {
            let size = dir_size(&trash_dir);
            items.push(ScanItem {
                id: uuid::Uuid::new_v4().to_string(),
                category: "Trash".into(),
                label: "Trash".into(),
                path: Some(trash_dir),
                size_bytes: size,
                risk: Risk::Review,
                description: "Files in the desktop trash bin".into(),
            });
        }

        let total_bytes: u64 = items.iter().map(|i| i.size_bytes).sum();

        Ok(ScanResult {
            category: "Trash".into(),
            items,
            total_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_trash_scanner_with_temp_trash() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path();

        // Create fake trash structure
        let trash = home.join(".local/share/Trash");
        let files_dir = trash.join("files");
        let info_dir = trash.join("info");
        fs::create_dir_all(&files_dir).unwrap();
        fs::create_dir_all(&info_dir).unwrap();

        fs::write(files_dir.join("old_doc.txt"), vec![0u8; 4096]).unwrap();
        fs::write(info_dir.join("old_doc.txt.trashinfo"), b"[Trash Info]\nPath=/home/user/old_doc.txt\n").unwrap();

        std::env::set_var("HOME", home.to_str().unwrap());

        let scanner = TrashScanner;
        let result = scanner.scan().unwrap();

        assert_eq!(result.category, "Trash");
        assert_eq!(result.items.len(), 1);
        assert!(result.total_bytes > 4000); // at least the 4096-byte file
        assert_eq!(result.items[0].risk, Risk::Review);

        std::env::remove_var("HOME");
    }

    #[test]
    fn test_trash_scanner_no_trash_dir() {
        let tmp = TempDir::new().unwrap();
        std::env::set_var("HOME", tmp.path().to_str().unwrap());

        let scanner = TrashScanner;
        let result = scanner.scan().unwrap();

        assert_eq!(result.items.len(), 0);
        assert_eq!(result.total_bytes, 0);

        std::env::remove_var("HOME");
    }
}
