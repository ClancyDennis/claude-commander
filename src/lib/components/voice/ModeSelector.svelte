<script lang="ts">
  import { Mic, MessageCircle } from "$lib/components/ui/icons";

  interface Props {
    onSelectDictate: () => void;
    onSelectDiscuss: () => void;
  }

  let { onSelectDictate, onSelectDiscuss }: Props = $props();
</script>

<div class="mode-select-grid">
  <button
    type="button"
    class="mode-card-half"
    onclick={onSelectDictate}
  >
    <div class="mode-icon dictate">
      <Mic size={32} />
    </div>
    <h3>Dictate</h3>
    <p>Speak to type</p>
  </button>

  <button
    type="button"
    class="mode-card-half"
    onclick={onSelectDiscuss}
  >
    <div class="mode-icon discuss">
      <MessageCircle size={32} />
    </div>
    <h3>Discuss</h3>
    <p>Voice conversation</p>
  </button>
</div>

<style>
  .mode-select-grid {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    height: 100%;
    padding: 0.5rem 0.5rem;
  }

  .mode-card-half {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 2rem 1rem;
    border: 2px solid var(--border-hex);
    border-radius: 0.75rem;
    background: var(--bg-secondary);
    text-align: center;
    cursor: pointer;
    transition: all 0.15s ease;
    flex: 1;
  }

  .mode-card-half:hover {
    border-color: var(--accent-hex);
    background: var(--bg-tertiary);
    transform: translateY(-2px);
  }

  .mode-card-half h3 {
    font-weight: 600;
    font-size: 1rem;
    margin: 0;
  }

  .mode-card-half p {
    font-size: 0.75rem;
    color: var(--text-secondary);
    margin: 0;
  }

  .mode-icon {
    width: 56px;
    height: 56px;
    border-radius: 0.75rem;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .mode-icon.dictate {
    background: rgba(59, 130, 246, 0.1);
    color: rgb(59, 130, 246);
  }

  .mode-icon.discuss {
    background: rgba(34, 197, 94, 0.1);
    color: rgb(34, 197, 94);
  }
</style>
