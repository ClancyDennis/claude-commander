<script lang="ts">
  import HelpTip from "../new-agent/HelpTip.svelte";

  let {
    attentionEnabled,
    onAttentionChange,
  }: {
    attentionEnabled: boolean;
    onAttentionChange: (value: boolean) => void;
  } = $props();
</script>

<section class="config-section">
  <h3>Voice</h3>
  <div class="advanced-setting-item">
    <div class="advanced-setting-header">
      <span class="advanced-setting-name">Voice Notifications on Task Completion</span>
      <HelpTip text="When enabled, a voice announcement plays when agents complete their tasks. You can respond by voice or stay silent to auto-dismiss." placement="top" />
    </div>
    <p class="advanced-setting-description">
      Automatically open a voice session to announce task completion results. The session will auto-close after 10 seconds of inactivity.
    </p>
    <div class="toggle-wrapper">
      <label class="toggle">
        <input
          type="checkbox"
          checked={attentionEnabled}
          onchange={(e) => onAttentionChange(e.currentTarget.checked)}
        />
        <span class="toggle-slider"></span>
      </label>
      <span class="toggle-label">{attentionEnabled ? "Enabled" : "Disabled"}</span>
    </div>
  </div>
</section>

<style>
  .config-section {
    background: var(--bg-secondary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-lg);
    padding: var(--space-6);
    margin-bottom: var(--space-6);
  }

  .config-section h3 {
    margin: 0 0 var(--space-4) 0;
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .advanced-setting-item {
    padding: var(--space-4);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-hex);
  }

  .advanced-setting-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-1);
  }

  .advanced-setting-name {
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .advanced-setting-description {
    font-size: var(--text-sm);
    color: var(--text-muted);
    margin: 0 0 var(--space-4) 0;
    line-height: var(--leading-relaxed);
  }

  /* Toggle Switch */
  .toggle-wrapper {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .toggle {
    position: relative;
    display: inline-block;
    width: 44px;
    height: 24px;
  }

  .toggle input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .toggle-slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--bg-primary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-full);
    transition: all var(--transition-fast);
  }

  .toggle-slider:before {
    position: absolute;
    content: "";
    height: 18px;
    width: 18px;
    left: 2px;
    bottom: 2px;
    background-color: var(--text-muted);
    border-radius: var(--radius-full);
    transition: all var(--transition-fast);
  }

  .toggle input:checked + .toggle-slider {
    background-color: var(--accent-hex);
    border-color: var(--accent-hex);
  }

  .toggle input:checked + .toggle-slider:before {
    transform: translateX(20px);
    background-color: white;
  }

  .toggle-label {
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }
</style>
