<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";

  interface ThreadConfig {
    p_thread_enabled: boolean;
    b_thread_enabled: boolean;
    f_thread_enabled: boolean;
    c_thread_enabled: boolean;
    max_concurrent_workflows: number;
    max_concurrent_verifications: number;
    backpressure_threshold: number;
  }

  interface ThreadStats {
    active_workflows: number;
    active_verifications: number;
    pool_utilization: number;
    total_agents: number;
    system_load: number;
  }

  let config = $state<ThreadConfig>({
    p_thread_enabled: true,
    b_thread_enabled: true,
    f_thread_enabled: true,
    c_thread_enabled: true,
    max_concurrent_workflows: 10,
    max_concurrent_verifications: 3,
    backpressure_threshold: 0.9,
  });

  let stats = $state<ThreadStats>({
    active_workflows: 0,
    active_verifications: 0,
    pool_utilization: 0,
    total_agents: 0,
    system_load: 0,
  });

  let autoRefresh = $state(true);
  let refreshInterval: number;
  let showAdvanced = $state(false);

  async function fetchThreadConfig() {
    try {
      config = await invoke<ThreadConfig>("get_thread_config");
    } catch (err) {
      console.error("Failed to fetch thread config:", err);
    }
  }

  async function fetchThreadStats() {
    try {
      stats = await invoke<ThreadStats>("get_thread_stats");
    } catch (err) {
      console.error("Failed to fetch thread stats:", err);
    }
  }

  async function updateConfig() {
    try {
      await invoke("update_thread_config", { config });
      console.log("Thread config updated");
    } catch (err) {
      console.error("Failed to update thread config:", err);
      alert(`Failed to update config: ${err}`);
    }
  }

  async function emergencyShutdown() {
    if (!confirm("Emergency shutdown will stop all threads immediately. Continue?")) {
      return;
    }

    try {
      await invoke("emergency_shutdown_threads");
      alert("Emergency shutdown completed");
    } catch (err) {
      console.error("Emergency shutdown failed:", err);
      alert(`Emergency shutdown failed: ${err}`);
    }
  }

  onMount(() => {
    fetchThreadConfig();
    fetchThreadStats();
    refreshInterval = setInterval(() => {
      if (autoRefresh) {
        fetchThreadStats();
      }
    }, 2000);
  });

  onDestroy(() => {
    clearInterval(refreshInterval);
  });

  function getLoadColor(load: number): string {
    if (load < 0.5) return "var(--success)";
    if (load < 0.8) return "var(--warning)";
    return "var(--error)";
  }

  function getLoadStatus(load: number): string {
    if (load < 0.5) return "NORMAL";
    if (load < 0.8) return "HIGH";
    return "CRITICAL";
  }
</script>

<div class="thread-controller">
  <header class="controller-header">
    <h2>Thread Controller</h2>
    <div class="load-badge" style="background: {getLoadColor(stats.system_load)};">
      {getLoadStatus(stats.system_load)} ({(stats.system_load * 100).toFixed(0)}%)
    </div>
  </header>

  <!-- Thread Status Cards -->
  <div class="thread-status">
    <div class="thread-card" class:enabled={config.p_thread_enabled}>
      <div class="thread-header">
        <span class="thread-name">P-Thread (Pool)</span>
        <label class="toggle">
          <input
            type="checkbox"
            bind:checked={config.p_thread_enabled}
            onchange={updateConfig}
          />
          <span class="slider"></span>
        </label>
      </div>
      <div class="thread-stats">
        <div class="stat-row">
          <span>Total Agents:</span>
          <span>{stats.total_agents}</span>
        </div>
        <div class="stat-row">
          <span>Utilization:</span>
          <span>{(stats.pool_utilization * 100).toFixed(0)}%</span>
        </div>
      </div>
    </div>

    <div class="thread-card" class:enabled={config.b_thread_enabled}>
      <div class="thread-header">
        <span class="thread-name">B-Thread (Orchestrator)</span>
        <label class="toggle">
          <input
            type="checkbox"
            bind:checked={config.b_thread_enabled}
            onchange={updateConfig}
          />
          <span class="slider"></span>
        </label>
      </div>
      <div class="thread-stats">
        <div class="stat-row">
          <span>Active Workflows:</span>
          <span>{stats.active_workflows}</span>
        </div>
        <div class="stat-row">
          <span>Max Concurrent:</span>
          <span>{config.max_concurrent_workflows}</span>
        </div>
      </div>
    </div>

    <div class="thread-card" class:enabled={config.f_thread_enabled}>
      <div class="thread-header">
        <span class="thread-name">F-Thread (Verification)</span>
        <label class="toggle">
          <input
            type="checkbox"
            bind:checked={config.f_thread_enabled}
            onchange={updateConfig}
          />
          <span class="slider"></span>
        </label>
      </div>
      <div class="thread-stats">
        <div class="stat-row">
          <span>Active Verifications:</span>
          <span>{stats.active_verifications}</span>
        </div>
        <div class="stat-row">
          <span>Max Concurrent:</span>
          <span>{config.max_concurrent_verifications}</span>
        </div>
      </div>
    </div>

    <div class="thread-card" class:enabled={config.c_thread_enabled}>
      <div class="thread-header">
        <span class="thread-name">C-Thread (Checkpoints)</span>
        <label class="toggle">
          <input
            type="checkbox"
            bind:checked={config.c_thread_enabled}
            onchange={updateConfig}
          />
          <span class="slider"></span>
        </label>
      </div>
      <div class="thread-stats">
        <div class="stat-row">
          <span>Status:</span>
          <span>Not Implemented</span>
        </div>
      </div>
    </div>
  </div>

  <!-- Advanced Settings -->
  <button class="advanced-toggle" onclick={() => showAdvanced = !showAdvanced}>
    {showAdvanced ? "▼" : "▶"} Advanced Settings
  </button>

  {#if showAdvanced}
    <div class="advanced-settings">
      <div class="setting-group">
        <label>
          Max Concurrent Workflows
          <input
            type="number"
            bind:value={config.max_concurrent_workflows}
            min="1"
            max="100"
            onchange={updateConfig}
          />
        </label>
      </div>

      <div class="setting-group">
        <label>
          Max Concurrent Verifications
          <input
            type="number"
            bind:value={config.max_concurrent_verifications}
            min="1"
            max="20"
            onchange={updateConfig}
          />
        </label>
      </div>

      <div class="setting-group">
        <label>
          Backpressure Threshold
          <input
            type="range"
            bind:value={config.backpressure_threshold}
            min="0.5"
            max="1.0"
            step="0.05"
            onchange={updateConfig}
          />
          <span class="threshold-value">{(config.backpressure_threshold * 100).toFixed(0)}%</span>
        </label>
      </div>
    </div>
  {/if}

  <!-- Emergency Controls -->
  <div class="emergency-zone">
    <button class="emergency-btn" onclick={emergencyShutdown}>
      🚨 Emergency Shutdown
    </button>
  </div>
</div>

<style>
  .thread-controller {
    padding: var(--space-lg);
    background: var(--bg-secondary);
    border-radius: 12px;
    border: 1px solid var(--border);
  }

  .controller-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-lg);
  }

  .controller-header h2 {
    margin: 0;
    font-size: 24px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .load-badge {
    padding: 8px 16px;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 700;
    color: white;
  }

  .thread-status {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: var(--space-md);
    margin-bottom: var(--space-lg);
  }

  .thread-card {
    padding: var(--space-md);
    background: var(--bg-tertiary);
    border: 2px solid var(--border);
    border-radius: 10px;
    opacity: 0.5;
    transition: all 0.2s ease;
  }

  .thread-card.enabled {
    opacity: 1;
    border-color: var(--accent);
  }

  .thread-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-md);
  }

  .thread-name {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
  }

  /* Toggle Switch */
  .toggle {
    position: relative;
    display: inline-block;
    width: 48px;
    height: 24px;
  }

  .toggle input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--border);
    transition: 0.3s;
    border-radius: 24px;
  }

  .slider:before {
    position: absolute;
    content: "";
    height: 18px;
    width: 18px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: 0.3s;
    border-radius: 50%;
  }

  input:checked + .slider {
    background-color: var(--accent);
  }

  input:checked + .slider:before {
    transform: translateX(24px);
  }

  .thread-stats {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .stat-row {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .stat-row span:last-child {
    font-weight: 600;
    color: var(--text-primary);
  }

  .advanced-toggle {
    width: 100%;
    padding: var(--space-md);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 8px;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
    transition: all 0.2s ease;
    margin-bottom: var(--space-md);
  }

  .advanced-toggle:hover {
    background: var(--accent-glow);
    border-color: var(--accent);
  }

  .advanced-settings {
    padding: var(--space-md);
    background: var(--bg-tertiary);
    border-radius: 8px;
    margin-bottom: var(--space-lg);
  }

  .setting-group {
    margin-bottom: var(--space-md);
  }

  .setting-group:last-child {
    margin-bottom: 0;
  }

  .setting-group label {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .setting-group input[type="number"],
  .setting-group input[type="range"] {
    padding: 8px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 14px;
  }

  .threshold-value {
    font-size: 12px;
    color: var(--text-muted);
  }

  .emergency-zone {
    padding: var(--space-md);
    background: rgba(239, 68, 68, 0.1);
    border: 2px solid var(--error);
    border-radius: 10px;
  }

  .emergency-btn {
    width: 100%;
    padding: var(--space-md);
    background: var(--error);
    border: none;
    border-radius: 8px;
    font-size: 16px;
    font-weight: 700;
    color: white;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .emergency-btn:hover {
    background: #dc2626;
    transform: scale(1.02);
  }
</style>
