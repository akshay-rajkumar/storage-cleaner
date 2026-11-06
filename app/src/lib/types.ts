export type Risk = "safe" | "review" | "advanced";

export interface ScanItem {
  id: string;
  category: string;
  label: string;
  path: string | null;
  size_bytes: number;
  risk: Risk;
  description: string;
}

export interface ScanResult {
  category: string;
  items: ScanItem[];
  total_bytes: number;
}

export interface DiskUsageInfo {
  mount_point: string;
  total_bytes: number;
  used_bytes: number;
  free_bytes: number;
}

export interface FullScanResult {
  disk_usage: DiskUsageInfo;
  categories: ScanResult[];
}

export interface CleanOutcome {
  path: string;
  success: boolean;
  freed_bytes: number;
  error: string | null;
}

export interface CleanReport {
  freed_bytes: number;
  deleted: number;
  failed: number;
  outcomes: CleanOutcome[];
}

