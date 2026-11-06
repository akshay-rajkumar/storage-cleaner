use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// How risky cleaning this item is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Risk {
    Safe,
    Review,
    Advanced,
}

impl std::fmt::Display for Risk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Risk::Safe => write!(f, "Safe"),
            Risk::Review => write!(f, "Review"),
            Risk::Advanced => write!(f, "Advanced"),
        }
    }
}

/// A single item found by a scanner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanItem {
    pub id: String,
    pub category: String,
    pub label: String,
    pub path: Option<PathBuf>,
    pub size_bytes: u64,
    pub risk: Risk,
    pub description: String,
}

/// The result of running one scanner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub category: String,
    pub items: Vec<ScanItem>,
    pub total_bytes: u64,
}

/// Disk usage statistics for a filesystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsageInfo {
    pub mount_point: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
}

/// Everything returned to the frontend from a full scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullScanResult {
    pub disk_usage: DiskUsageInfo,
    pub categories: Vec<ScanResult>,
}

/// Trait that all scanners implement.
pub trait Scanner {
    fn id(&self) -> &'static str;
    fn scan(&self) -> anyhow::Result<ScanResult>;
}
