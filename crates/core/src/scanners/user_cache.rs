use crate::types::{Risk, ScanItem, ScanResult, Scanner};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Scans each direct subfolder of `$XDG_CACHE_HOME` (or `~/.cache`).
/// Produces one `ScanItem` per subfolder with its total size.
pub struct UserCacheScanner;

impl UserCacheScanner {
    fn cache_dir() -> PathBuf {
        if let Ok(dir) = std::env::var("XDG_CACHE_HOME") {
            PathBuf::from(dir)
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".cache")
        } else {
            PathBuf::from("~/.cache")
        }
    }
}

impl Scanner for UserCacheScanner {
    fn id(&self) -> &'static str {
        "user_cache"
    }

    fn scan(&self) -> anyhow::Result<ScanResult> {
        let cache_dir = Self::cache_dir();
        let mut items = Vec::new();

        if !cache_dir.exists() {
            return Ok(ScanResult {
                category: "User Cache".into(),
                items,
                total_bytes: 0,
            });
        }

        // Read the direct children of the cache dir
        let entries = match std::fs::read_dir(&cache_dir) {
            Ok(e) => e,
            Err(e) => {
                anyhow::bail!("Cannot read cache directory {}: {}", cache_dir.display(), e);
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue, // permission error — skip
            };

            let path = entry.path();

            // Skip symlinks — do not follow them
            if path.symlink_metadata().map_or(true, |m| m.file_type().is_symlink()) {
                continue;
            }

            // Only interested in directories
            if !path.is_dir() {
                continue;
            }

            let size = dir_size(&path);
            let label = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "unknown".into());

            items.push(ScanItem {
                id: uuid::Uuid::new_v4().to_string(),
                category: "User Cache".into(),
                label: label.clone(),
                path: Some(path),
                size_bytes: size,
                risk: Risk::Safe,
                description: format!("Cache folder: {}", label),
            });
        }

        // Sort by size descending so biggest items show first
        items.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

        let total_bytes: u64 = items.iter().map(|i| i.size_bytes).sum();

        Ok(ScanResult {
            category: "User Cache".into(),
            items,
            total_bytes,
        })
    }
}

/// Calculate the total size of a directory tree.
/// Does NOT follow symlinks. Skips entries that can't be read (permissions).
pub fn dir_size(path: &std::path::Path) -> u64 {
    WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok()) // skip permission errors
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::symlink;
    use tempfile::TempDir;

    #[test]
    fn test_user_cache_scans_subfolders() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path();

        // Create two subfolders with files
        let dir_a = cache.join("app_a");
        let dir_b = cache.join("app_b");
        fs::create_dir(&dir_a).unwrap();
        fs::create_dir(&dir_b).unwrap();

        fs::write(dir_a.join("file1.bin"), vec![0u8; 1024]).unwrap();
        fs::write(dir_b.join("file2.bin"), vec![0u8; 2048]).unwrap();
        fs::write(dir_b.join("file3.bin"), vec![0u8; 512]).unwrap();

        // Point XDG_CACHE_HOME at our temp dir
        std::env::set_var("XDG_CACHE_HOME", cache.to_str().unwrap());

        let scanner = UserCacheScanner;
        let result = scanner.scan().unwrap();

        assert_eq!(result.category, "User Cache");
        assert_eq!(result.items.len(), 2);

        // Items are sorted descending by size; dir_b should be first
        assert_eq!(result.items[0].label, "app_b");
        assert_eq!(result.items[0].size_bytes, 2048 + 512);
        assert_eq!(result.items[1].label, "app_a");
        assert_eq!(result.items[1].size_bytes, 1024);

        assert_eq!(result.total_bytes, 1024 + 2048 + 512);

        // Cleanup env
        std::env::remove_var("XDG_CACHE_HOME");
    }

    #[test]
    fn test_symlinks_are_skipped() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path();

        // Real directory
        let real_dir = cache.join("real");
        fs::create_dir(&real_dir).unwrap();
        fs::write(real_dir.join("data.bin"), vec![0u8; 100]).unwrap();

        // Symlink pointing to real
        let link = cache.join("linked");
        symlink(&real_dir, &link).unwrap();

        std::env::set_var("XDG_CACHE_HOME", cache.to_str().unwrap());

        let scanner = UserCacheScanner;
        let result = scanner.scan().unwrap();

        // Only the real dir should appear, symlink is skipped
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].label, "real");

        std::env::remove_var("XDG_CACHE_HOME");
    }

    #[test]
    fn test_unreadable_folder_is_skipped() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = TempDir::new().unwrap();
        let cache = tmp.path();

        // Readable dir
        let readable = cache.join("readable");
        fs::create_dir(&readable).unwrap();
        fs::write(readable.join("data.bin"), vec![0u8; 50]).unwrap();

        // Unreadable dir
        let unreadable = cache.join("unreadable");
        fs::create_dir(&unreadable).unwrap();
        fs::write(unreadable.join("secret.bin"), vec![0u8; 99]).unwrap();
        fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000)).unwrap();

        std::env::set_var("XDG_CACHE_HOME", cache.to_str().unwrap());

        let scanner = UserCacheScanner;
        let result = scanner.scan().unwrap();

        // The unreadable dir still shows up as a subfolder (we can stat it),
        // but its internal size will be 0 because walkdir can't read inside
        let unreadable_item = result.items.iter().find(|i| i.label == "unreadable");
        assert!(unreadable_item.is_some());
        assert_eq!(unreadable_item.unwrap().size_bytes, 0);

        // Restore permissions so tempdir cleanup succeeds
        fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o755)).unwrap();

        std::env::remove_var("XDG_CACHE_HOME");
    }

    #[test]
    fn test_dir_size_function() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("stuff");
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("a.txt"), vec![0u8; 100]).unwrap();

        let nested = dir.join("sub");
        fs::create_dir(&nested).unwrap();
        fs::write(nested.join("b.txt"), vec![0u8; 200]).unwrap();

        assert_eq!(dir_size(&dir), 300);
    }
}
