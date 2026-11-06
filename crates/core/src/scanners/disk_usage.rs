use crate::types::DiskUsageInfo;

/// Provides disk usage info (total/used/free) for the filesystem containing $HOME.
pub struct DiskUsageScanner;

impl DiskUsageScanner {
    pub fn get_usage() -> anyhow::Result<DiskUsageInfo> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
        Self::get_usage_for_path(&home)
    }

    pub fn get_usage_for_path(path: &str) -> anyhow::Result<DiskUsageInfo> {
        let stat = nix::sys::statvfs::statvfs(path)?;

        let block_size = stat.fragment_size() as u64;
        let total_bytes = stat.blocks() as u64 * block_size;
        let free_bytes = stat.blocks_available() as u64 * block_size;
        let used_bytes = total_bytes.saturating_sub(free_bytes);

        Ok(DiskUsageInfo {
            mount_point: path.into(),
            total_bytes,
            used_bytes,
            free_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_usage_returns_sane_values() {
        let usage = DiskUsageScanner::get_usage_for_path("/").unwrap();

        // Total should be > 0
        assert!(usage.total_bytes > 0, "Total bytes should be non-zero");

        // used + free should ≈ total (not exact due to reserved blocks)
        assert!(usage.used_bytes <= usage.total_bytes);
        assert!(usage.free_bytes <= usage.total_bytes);
    }
}
