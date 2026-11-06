<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type {
    CleanOutcome,
    CleanReport,
    FullScanResult,
    ScanItem,
    ScanResult,
  } from "./types";

  let scanResult = $state<FullScanResult | null>(null);
  let scanning = $state(false);
  let cleaning = $state(false);
  let error: string | null = $state(null);
  let cleanReport = $state<CleanReport | null>(null);
  let confirmOpen = $state(false);
  let selected = $state<Set<string>>(new Set());

  let allItems: ScanItem[] = $derived.by(() =>
    scanResult ? scanResult.categories.flatMap((category) => category.items) : [],
  );
  let selectedItems: ScanItem[] = $derived(allItems.filter((item) => selected.has(item.id)));
  let selectedBytes: number = $derived(
    selectedItems.reduce((sum, item) => sum + item.size_bytes, 0),
  );
  let totalCleanable: number = $derived(
    allItems.reduce((sum, item) => sum + item.size_bytes, 0),
  );
  let allSelected: boolean = $derived(
    allItems.length > 0 && selectedItems.length === allItems.length,
  );
  let failedOutcomes: CleanOutcome[] = $derived.by(() =>
    cleanReport ? cleanReport.outcomes.filter((outcome) => !outcome.success) : [],
  );

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

  function isSelected(id: string): boolean {
    return selected.has(id);
  }

  function toggleItem(id: string) {
    const next = new Set(selected);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    selected = next;
  }

  function toggleAll() {
    selected = allSelected ? new Set() : new Set(allItems.map((item) => item.id));
  }

  function categorySelectedCount(category: ScanResult): number {
    return category.items.filter((item) => selected.has(item.id)).length;
  }

  function categoryAllSelected(category: ScanResult): boolean {
    return category.items.length > 0 && categorySelectedCount(category) === category.items.length;
  }

  function categorySomeSelected(category: ScanResult): boolean {
    const count = categorySelectedCount(category);
    return count > 0 && count < category.items.length;
  }

  function toggleCategory(category: ScanResult) {
    const all = categoryAllSelected(category);
    const next = new Set(selected);
    for (const item of category.items) {
      if (all) {
        next.delete(item.id);
      } else {
        next.add(item.id);
      }
    }
    selected = next;
  }

  // Lets us show the browser's "mixed" checkbox state for partial selection.
  function indeterminate(node: HTMLInputElement, value: boolean) {
    node.indeterminate = value;
    return {
      update(next: boolean) {
        node.indeterminate = next;
      },
    };
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && confirmOpen && !cleaning) {
      confirmOpen = false;
    }
  }

  async function runScan() {
    scanning = true;
    error = null;
    try {
      scanResult = await invoke<FullScanResult>("run_scan");
      selected = new Set();
    } catch (e) {
      error = String(e);
    } finally {
      scanning = false;
    }
  }

  function handleScanClick() {
    cleanReport = null;
    confirmOpen = false;
    runScan();
  }

  function requestClean() {
    if (selectedItems.length === 0 || cleaning) return;
    confirmOpen = true;
  }

  async function runClean() {
    if (selectedItems.length === 0 || cleaning) return;

    const paths = selectedItems
      .map((item: ScanItem) => item.path)
      .filter((path): path is string => path !== null);

    cleaning = true;
    confirmOpen = false;
    error = null;
    cleanReport = null;

    try {
      cleanReport = await invoke<CleanReport>("clean_items", { paths });
      selected = new Set();
      // Refresh sizes and categories after deleting.
      await runScan();
    } catch (e) {
      error = String(e);
    } finally {
      cleaning = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="max-w-4xl mx-auto">
  <!-- Header -->
  <div class="flex items-center justify-between mb-8">
    <div class="flex items-center gap-3">
      <div class="text-3xl">🧹</div>
      <div>
        <h1 class="text-2xl font-bold" style="color: var(--text-primary)">Linux Cleaner</h1>
        <p class="text-sm" style="color: var(--text-secondary)">Storage scanner &amp; cleaner</p>
      </div>
    </div>
    <button
      onclick={handleScanClick}
      disabled={scanning || cleaning}
      class="px-5 py-2.5 rounded-lg font-medium text-white transition-all duration-200 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
      style="background-color: var(--accent);"
      onmouseenter={(e) => { if (!scanning && !cleaning) (e.currentTarget as HTMLElement).style.backgroundColor = 'var(--accent-hover)'; }}
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

  {#if cleanReport}
    <div class="rounded-lg p-4 mb-6 border" style="background-color: color-mix(in srgb, var(--safe-color) 10%, transparent); border-color: var(--safe-color); color: var(--text-primary);">
      <div class="flex items-start gap-2">
        <span>✅</span>
        <div class="text-sm">
          <strong style="color: var(--safe-color)">
            Freed {formatSize(cleanReport.freed_bytes)}
          </strong>
          · {cleanReport.deleted}
          {cleanReport.deleted === 1 ? "item" : "items"} cleaned.
          {#if cleanReport.failed > 0}
            <span style="color: var(--advanced-color)">
              {cleanReport.failed} {cleanReport.failed === 1 ? "item" : "items"} could not be cleaned.
            </span>
          {/if}
          {#if failedOutcomes.length > 0}
            <ul class="mt-2 space-y-1 text-xs" style="color: var(--text-secondary)">
              {#each failedOutcomes.slice(0, 5) as outcome}
                <li class="truncate">• {outcome.path}: {outcome.error}</li>
              {/each}
            </ul>
          {/if}
        </div>
      </div>
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

    <!-- Selection toolbar -->
    <div class="rounded-xl p-4 mb-6 border flex flex-wrap items-center gap-x-4 gap-y-3" style="background-color: var(--bg-card); border-color: var(--border-color);">
      <label class="inline-flex items-center gap-2 cursor-pointer select-none">
        <input
          type="checkbox"
          checked={allSelected}
          use:indeterminate={selectedItems.length > 0 && !allSelected}
          onchange={toggleAll}
          class="h-4 w-4 cursor-pointer"
          style="accent-color: var(--accent);"
          aria-label="Select all cleanable items"
        />
        <span class="font-medium" style="color: var(--text-primary)">Select all</span>
      </label>

      <div class="text-sm" style="color: var(--text-secondary)">
        {#if selectedItems.length > 0}
          <strong style="color: var(--accent)">{formatSize(selectedBytes)}</strong>
          selected · {selectedItems.length} of {allItems.length} items
        {:else}
          {allItems.length} {allItems.length === 1 ? "item" : "items"} · {formatSize(totalCleanable)} cleanable
        {/if}
      </div>

      <button
        onclick={requestClean}
        disabled={selectedItems.length === 0 || cleaning || scanning}
        class="ml-auto px-4 py-2 rounded-lg font-medium text-white transition-opacity duration-200 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed hover:opacity-90"
        style="background-color: var(--advanced-color);"
      >
        {#if cleaning}
          Cleaning…
        {:else}
          🗑️ Clean Selected ({selectedItems.length})
        {/if}
      </button>
    </div>

    <!-- Category Cards -->
    {#each scanResult.categories as category}
      <div class="rounded-xl p-6 mb-6 border" style="background-color: var(--bg-card); border-color: var(--border-color);">
        <div class="flex items-center justify-between mb-4 gap-3">
          <div class="flex items-center gap-2 min-w-0">
            <input
              type="checkbox"
              checked={categoryAllSelected(category)}
              use:indeterminate={categorySomeSelected(category)}
              onchange={() => toggleCategory(category)}
              class="h-4 w-4 cursor-pointer shrink-0"
              style="accent-color: var(--accent);"
              aria-label={`Select all ${category.category} items`}
            />
            <span class="text-xl">{category.category === "User Cache" ? "📁" : "🗑️"}</span>
            <h2 class="text-lg font-semibold truncate" style="color: var(--text-primary)">{category.category}</h2>
            {#if categorySelectedCount(category) > 0}
              <span class="text-xs px-2 py-0.5 rounded-full shrink-0" style="background-color: color-mix(in srgb, var(--accent) 15%, transparent); color: var(--accent);">
                {categorySelectedCount(category)} selected
              </span>
            {/if}
          </div>
          <div class="px-3 py-1 rounded-full text-sm font-medium shrink-0" style="background-color: color-mix(in srgb, var(--accent) 15%, transparent); color: var(--accent);">
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
                  <th class="w-8 py-2"></th>
                  <th class="text-left py-2 pr-4 font-medium" style="color: var(--text-muted)">Name</th>
                  <th class="text-right py-2 px-4 font-medium" style="color: var(--text-muted)">Size</th>
                  <th class="text-right py-2 pl-4 font-medium" style="color: var(--text-muted)">Risk</th>
                </tr>
              </thead>
              <tbody>
                {#each category.items.slice(0, 20) as item}
                  <tr
                    style="border-bottom: 1px solid var(--border-color); background-color: {isSelected(item.id) ? 'color-mix(in srgb, var(--accent) 8%, transparent)' : 'transparent'};"
                  >
                    <td class="py-2">
                      <input
                        type="checkbox"
                        checked={isSelected(item.id)}
                        onchange={() => toggleItem(item.id)}
                        class="h-4 w-4 cursor-pointer"
                        style="accent-color: var(--accent);"
                        aria-label={`Select ${item.label}`}
                      />
                    </td>
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
                    <td colspan="4" class="py-2 text-center text-sm italic" style="color: var(--text-muted)">
                      Showing 20 of {category.items.length} items — "Select all" still selects every item
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

<!-- Clean confirmation dialog -->
{#if confirmOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4" role="dialog" aria-modal="true" aria-labelledby="clean-dialog-title">
    <button
      type="button"
      class="absolute inset-0 cursor-default"
      style="background-color: rgba(0, 0, 0, 0.6); border: 0;"
      aria-label="Cancel cleaning"
      onclick={() => (confirmOpen = false)}
    ></button>

    <div class="relative w-full max-w-md rounded-xl border p-6 shadow-xl" style="background-color: var(--bg-card); border-color: var(--border-color);">
      <h2 id="clean-dialog-title" class="text-lg font-semibold mb-2" style="color: var(--text-primary)">
        Clean selected items?
      </h2>
      <p class="text-sm mb-4" style="color: var(--text-secondary)">
        This permanently deletes <strong>{selectedItems.length}</strong>
        {selectedItems.length === 1 ? "item" : "items"} and frees about
        <strong>{formatSize(selectedBytes)}</strong>. This cannot be undone.
      </p>

      <ul class="text-xs mb-5 max-h-32 overflow-y-auto rounded-lg p-3 space-y-1" style="background-color: var(--bg-primary); color: var(--text-secondary);">
        {#each selectedItems.slice(0, 8) as item}
          <li class="truncate">• {item.label}</li>
        {/each}
        {#if selectedItems.length > 8}
          <li class="italic">…and {selectedItems.length - 8} more</li>
        {/if}
      </ul>

      <div class="flex justify-end gap-2">
        <button
          onclick={() => (confirmOpen = false)}
          class="px-4 py-2 rounded-lg font-medium border transition-opacity duration-200 cursor-pointer hover:opacity-80"
          style="background-color: var(--bg-card); border-color: var(--border-color); color: var(--text-primary);"
        >
          Cancel
        </button>
        <button
          onclick={runClean}
          disabled={cleaning}
          class="px-4 py-2 rounded-lg font-medium text-white transition-opacity duration-200 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed hover:opacity-90"
          style="background-color: var(--advanced-color);"
        >
          {cleaning ? "Cleaning…" : "Delete permanently"}
        </button>
      </div>
    </div>
  </div>
{/if}
