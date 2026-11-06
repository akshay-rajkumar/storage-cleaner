<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { FullScanResult } from "./types";

  let scanResult: FullScanResult | null = $state(null);
  let scanning = $state(false);
  let error: string | null = $state(null);

  function formatSize(bytes: number): string {
    if (bytes >= 1024 * 1024 * 1024) {
      return (bytes / (1024 * 1024 * 1024)).toFixed(2) + " GiB";
    } else if (bytes >= 1024 * 1024) {
      return (bytes / (1024 * 1024)).toFixed(2) + " MiB";
    } else if (bytes >= 1024) {
      return (bytes / 1024).toFixed(2) + " KiB";
    }
    return bytes + " B";
  }

  function usedPercent(du: FullScanResult["disk_usage"]): number {
    if (du.total_bytes === 0) return 0;
    return Math.round((du.used_bytes / du.total_bytes) * 100);
  }

  function riskColor(risk: string): string {
    switch (risk) {
      case "safe":
        return "var(--safe-color)";
      case "review":
        return "var(--review-color)";
      case "advanced":
        return "var(--advanced-color)";
      default:
        return "var(--text-muted)";
    }
  }

  function riskLabel(risk: string): string {
    return risk.charAt(0).toUpperCase() + risk.slice(1);
  }

  async function runScan() {
    scanning = true;
    error = null;
    try {
      scanResult = await invoke<FullScanResult>("run_scan");
    } catch (e) {
      error = String(e);
    } finally {
      scanning = false;
    }
  }
</script>

<div class="max-w-4xl mx-auto">
  <!-- Header -->
  <div class="flex items-center justify-between mb-8">
    <div class="flex items-center gap-3">
      <div class="text-3xl">🧹</div>
      <div>
        <h1 class="text-2xl font-bold" style="color: var(--text-primary)">Linux Cleaner</h1>
        <p class="text-sm" style="color: var(--text-secondary)">Storage scanner — read-only mode</p>
      </div>
    </div>
    <button
      onclick={runScan}
      disabled={scanning}
      class="px-5 py-2.5 rounded-lg font-medium text-white transition-all duration-200 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
      style="background-color: var(--accent);"
      onmouseenter={(e) => { if (!scanning) (e.currentTarget as HTMLElement).style.backgroundColor = 'var(--accent-hover)'; }}
      onmouseleave={(e) => (e.currentTarget as HTMLElement).style.backgroundColor = 'var(--accent)'}
    >
      {#if scanning}
        <span class="inline-flex items-center gap-2">
          <svg class="animate-spin h-4 w-4" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"/>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
          </svg>
          Scanning…
        </span>
      {:else}
        🔍 Scan System
      {/if}
    </button>
  </div>

  {#if error}
    <div class="rounded-lg p-4 mb-6 border" style="background-color: color-mix(in srgb, var(--advanced-color) 10%, transparent); border-color: var(--advanced-color); color: var(--advanced-color);">
      <strong>Error:</strong> {error}
    </div>
  {/if}

  {#if !scanResult && !scanning}
    <div class="rounded-xl p-12 text-center border" style="background-color: var(--bg-card); border-color: var(--border-color);">
      <div class="text-5xl mb-4">💾</div>
      <h2 class="text-xl font-semibold mb-2" style="color: var(--text-primary)">Ready to Scan</h2>
      <p style="color: var(--text-secondary)">Click "Scan System" to analyze your disk usage and find cleanable items.</p>
    </div>
  {/if}

  {#if scanResult}
    <!-- Disk Usage Card -->
    <div class="rounded-xl p-6 mb-6 border" style="background-color: var(--bg-card); border-color: var(--border-color);">
      <div class="flex items-center gap-2 mb-4">
        <span class="text-xl">💿</span>
        <h2 class="text-lg font-semibold" style="color: var(--text-primary)">Disk Usage</h2>
        <span class="text-sm ml-auto" style="color: var(--text-muted)">{scanResult.disk_usage.mount_point}</span>
      </div>

      <!-- Progress bar -->
      <div class="w-full h-6 rounded-full overflow-hidden mb-3" style="background-color: var(--bar-bg);">
        <div
          class="h-full rounded-full transition-all duration-500"
          style="width: {usedPercent(scanResult.disk_usage)}%; background-color: {usedPercent(scanResult.disk_usage) > 90 ? 'var(--advanced-color)' : usedPercent(scanResult.disk_usage) > 75 ? 'var(--review-color)' : 'var(--bar-used)'};"
        ></div>
      </div>

      <div class="grid grid-cols-3 gap-4 text-center">
        <div>
          <div class="text-sm font-medium" style="color: var(--text-muted)">Total</div>
          <div class="text-lg font-bold" style="color: var(--text-primary)">{formatSize(scanResult.disk_usage.total_bytes)}</div>
        </div>
        <div>
          <div class="text-sm font-medium" style="color: var(--text-muted)">Used ({usedPercent(scanResult.disk_usage)}%)</div>
          <div class="text-lg font-bold" style="color: var(--bar-used)">{formatSize(scanResult.disk_usage.used_bytes)}</div>
        </div>
        <div>
          <div class="text-sm font-medium" style="color: var(--text-muted)">Free</div>
          <div class="text-lg font-bold" style="color: var(--bar-free)">{formatSize(scanResult.disk_usage.free_bytes)}</div>
        </div>
      </div>
    </div>

    <!-- Category Cards -->
    {#each scanResult.categories as category}
      <div class="rounded-xl p-6 mb-6 border" style="background-color: var(--bg-card); border-color: var(--border-color);">
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-2">
            <span class="text-xl">{category.category === "User Cache" ? "📁" : "🗑️"}</span>
            <h2 class="text-lg font-semibold" style="color: var(--text-primary)">{category.category}</h2>
          </div>
          <div class="px-3 py-1 rounded-full text-sm font-medium" style="background-color: color-mix(in srgb, var(--accent) 15%, transparent); color: var(--accent);">
            {formatSize(category.total_bytes)}
          </div>
        </div>

        {#if category.items.length === 0}
          <p class="text-sm italic" style="color: var(--text-muted)">No items found.</p>
        {:else}
          <div class="overflow-x-auto">
            <table class="w-full text-sm">
              <thead>
                <tr style="border-bottom: 1px solid var(--border-color);">
                  <th class="text-left py-2 pr-4 font-medium" style="color: var(--text-muted)">Name</th>
                  <th class="text-right py-2 px-4 font-medium" style="color: var(--text-muted)">Size</th>
                  <th class="text-right py-2 pl-4 font-medium" style="color: var(--text-muted)">Risk</th>
                </tr>
              </thead>
              <tbody>
                {#each category.items.slice(0, 20) as item}
                  <tr style="border-bottom: 1px solid var(--border-color);">
                    <td class="py-2 pr-4" style="color: var(--text-primary)">
                      <div class="font-medium">{item.label}</div>
                      {#if item.path}
                        <div class="text-xs truncate max-w-md" style="color: var(--text-muted)">{item.path}</div>
                      {/if}
                    </td>
                    <td class="py-2 px-4 text-right font-mono whitespace-nowrap" style="color: var(--text-secondary)">{formatSize(item.size_bytes)}</td>
                    <td class="py-2 pl-4 text-right">
                      <span
                        class="inline-block px-2 py-0.5 rounded text-xs font-medium"
                        style="background-color: color-mix(in srgb, {riskColor(item.risk)} 15%, transparent); color: {riskColor(item.risk)};"
                      >
                        {riskLabel(item.risk)}
                      </span>
                    </td>
                  </tr>
                {/each}
                {#if category.items.length > 20}
                  <tr>
                    <td colspan="3" class="py-2 text-center text-sm italic" style="color: var(--text-muted)">
                      … and {category.items.length - 20} more items
                    </td>
                  </tr>
                {/if}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    {/each}
  {/if}
</div>
