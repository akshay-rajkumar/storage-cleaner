mod user_cache;
mod trash;
mod disk_usage;

pub use user_cache::UserCacheScanner;
pub use trash::TrashScanner;
pub use disk_usage::DiskUsageScanner;

use crate::types::{FullScanResult, Scanner};

/// Run all scanners and return a combined result.
pub fn run_full_scan() -> anyhow::Result<FullScanResult> {
    let scanners: Vec<Box<dyn Scanner>> = vec![
        Box::new(UserCacheScanner),
        Box::new(TrashScanner),
    ];

    let mut categories = Vec::new();
    for scanner in &scanners {
        match scanner.scan() {
            Ok(result) => categories.push(result),
            Err(e) => eprintln!("Warning: scanner '{}' failed: {}", scanner.id(), e),
        }
    }

    let disk_usage = DiskUsageScanner::get_usage()?;

    Ok(FullScanResult {
        disk_usage,
        categories,
    })
}
