<!--
  Tier 1 (Simple) "Interpreted as" chip row.

  Splits the free-text q on whitespace into editable, removable chips.
  Each chip is an inline input so the user can tweak a single clause
  without retyping the whole query; the remove button drops that token.
  Any change rebuilds `q` and pushes it back up via `onChange`.
-->
<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';

  let {
    q,
    onChange,
  }: {
    q: string;
    onChange: (q: string) => void;
  } = $props();

  // Local, editable token list. We manage it here rather than deriving it
  // purely from `q` so typing inside a chip never fights the controlled
  // input. Re-sync only when `q` changes from the outside (main box,
  // "Upgrade to Power", a saved query, etc.). Seeded by the $effect below.
  let tokens = $state<string[]>([]);

  $effect(() => {
    const next = splitTokens(q);
    if (next.join(' ') !== tokens.join(' ')) tokens = next;
  });

  function splitTokens(value: string): string[] {
    return value.split(/\s+/).filter(Boolean);
  }

  function commit() {
    onChange(tokens.join(' '));
  }

  function updateToken(i: number, text: string) {
    tokens[i] = text;
    commit();
  }

  function removeToken(i: number) {
    tokens.splice(i, 1);
    commit();
  }

  function onChipKeydown(e: KeyboardEvent, i: number) {
    const input = e.currentTarget as HTMLInputElement;
    if (e.key === 'Enter') {
      e.preventDefault();
      input.blur();
    } else if (e.key === 'Backspace' && input.value === '' && tokens.length > 1) {
      e.preventDefault();
      removeToken(i);
    }
  }
</script>

{#if tokens.length > 0}
  <div class="interpreted-row" aria-label={t('search2.interpretedAs')}>
    <span class="interpreted-label">{t('search2.interpretedAs')}</span>
    {#each tokens as token, i}
      <span class="chip interpreted-chip">
        <input
          class="chip-edit"
          value={token}
          aria-label={t('search2.chipLabel', { token })}
          oninput={(e) => updateToken(i, (e.currentTarget as HTMLInputElement).value)}
          onkeydown={(e) => onChipKeydown(e, i)}
        />
        <button
          class="chip-remove"
          type="button"
          aria-label={t('search2.removeChip', { token })}
          onclick={() => removeToken(i)}
        >
          ×
        </button>
      </span>
    {/each}
  </div>
{/if}

<style>
  .interpreted-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem;
    margin-bottom: 0.8rem;
    padding: 0.5rem 0.7rem;
    background: var(--color-surface);
    border: 1px dashed var(--color-border);
    border-radius: var(--radius);
  }
  .interpreted-label {
    font-size: 0.78rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-muted);
  }
  .interpreted-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
    padding: 0.15rem 0.35rem 0.15rem 0.5rem;
  }
  .chip-edit {
    min-width: 3.2rem;
    border: none;
    background: transparent;
    color: var(--color-text);
    font: inherit;
    font-size: 0.82rem;
    padding: 0;
    outline: none;
  }
  .chip-edit:focus {
    color: var(--color-primary);
  }
  .chip-remove {
    background: none;
    border: none;
    color: var(--color-muted);
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
    padding: 0 0.1rem;
  }
  .chip-remove:hover {
    color: var(--color-primary);
  }
</style>
