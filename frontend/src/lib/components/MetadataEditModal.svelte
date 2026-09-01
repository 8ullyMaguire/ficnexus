<script lang="ts">
  // MetadataEditModal — curator/mod/admin proposes a metadata change for a
  // work. The proposal goes to the curator vote queue (peer-voted, same as
  // body fixes); it is applied only after a quorum approves.
  import { auth } from '$lib/stores/auth.svelte';

  let {
    workId,
    urlId,
    currentTitle = '',
    currentAuthor = '',
    currentStatus = '',
    currentDescription = '',
    onClose = () => {},
  } = $props<{
    workId: number;
    urlId: string;
    currentTitle?: string;
    currentAuthor?: string;
    currentStatus?: string;
    currentDescription?: string;
    onClose?: () => void;
  }>();

  let field = $state<'title' | 'author' | 'status' | 'description'>('title');
  let newValue = $state('');
  let reason = $state('');
  let msg = $state('');
  let err = $state('');
  let submitting = $state(false);

  const FIELDS: { key: 'title' | 'author' | 'status' | 'description'; label: string; current: string }[] = [
    { key: 'title', label: 'Title', current: currentTitle },
    { key: 'author', label: 'Author', current: currentAuthor },
    { key: 'status', label: 'Status', current: currentStatus },
    { key: 'description', label: 'Description', current: currentDescription },
  ];

  function selectField(f: (typeof FIELDS)[number]) {
    field = f.key;
    newValue = '';
    reason = '';
    msg = '';
    err = '';
  }

  async function submit() {
    if (!newValue.trim()) { err = 'New value required.'; return; }
    submitting = true;
    msg = '';
    err = '';
    try {
      const res = await fetch('/api/curator/metadata/propose', {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          work_id: workId,
          url_id: urlId,
          field,
          old_value: FIELDS.find((f) => f.key === field)?.current ?? '',
          new_value: newValue.trim(),
          reason: reason.trim(),
        }),
      });
      const body = await res.json();
      if (body.err !== 0) throw new Error(body.msg ?? `HTTP ${res.status}`);
      msg = `Proposed — pending curator vote (proposal #${body.proposal_id}).`;
      newValue = '';
      reason = '';
    } catch (e) {
      err = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

{#if auth.isLoggedIn && auth.level >= 100}
  <div class="md-backdrop" role="presentation" onclick={(e) => { if (e.target === e.currentTarget) onClose(); }}>
    <div class="md-modal" role="dialog" aria-modal="true" aria-label="Propose metadata change">
      <header class="md-head">
        <h2>✏️ Propose metadata change</h2>
        <button class="close" aria-label="Close" onclick={onClose}>✕</button>
      </header>
      <div class="md-body">
        <p class="muted">Curators propose changes; other curators vote. Applied only after a quorum approves.</p>

        <div class="field-tabs">
          {#each FIELDS as f (f.key)}
            <button class="field-tab" class:active={field === f.key} onclick={() => selectField(f)}>
              {f.label}
            </button>
          {/each}
        </div>

        <div class="current-value">
          <span class="label">Current:</span>
          <span class="val">{FIELDS.find((f) => f.key === field)?.current || '—'}</span>
        </div>

        <label class="md-field">
          <span>New value</span>
          {#if field === 'description'}
            <textarea bind:value={newValue} rows="3" placeholder="Corrected description…"></textarea>
          {:else}
            <input type="text" bind:value={newValue} placeholder={`New ${field}…`} />
          {/if}
        </label>

        <label class="md-field">
          <span>Reason (optional)</span>
          <input type="text" bind:value={reason} placeholder="Why is this wrong?" />
        </label>

        {#if err}<p class="error">{err}</p>{/if}
        {#if msg}<p class="ok">{msg}</p>{/if}
      </div>
      <footer class="md-foot">
        <button class="btn" onclick={onClose}>Cancel</button>
        <button class="btn btn-primary" onclick={submit} disabled={submitting || !newValue.trim()}>
          {submitting ? 'Proposing…' : 'Send to vote queue'}
        </button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .md-backdrop {
    position: fixed; inset: 0; background: rgba(0,0,0,0.45);
    display: flex; align-items: center; justify-content: center; z-index: 1100; padding: 1rem;
  }
  .md-modal {
    background: var(--color-bg, #fff); border-radius: 12px; max-width: 520px; width: 100%;
    max-height: 85vh; display: flex; flex-direction: column; box-shadow: 0 12px 40px rgba(0,0,0,0.25);
  }
  .md-head { display: flex; align-items: center; justify-content: space-between; padding: 0.9rem 1.2rem; border-bottom: 1px solid var(--color-border, #eee); }
  .md-head h2 { margin: 0; font-size: 1.05rem; }
  .close { background: none; border: none; font-size: 1.1rem; cursor: pointer; color: var(--color-text-muted, #888); }
  .md-body { padding: 1rem 1.2rem; overflow-y: auto; flex: 1; }
  .muted { color: var(--color-text-muted, #888); font-size: 0.85rem; }
  .field-tabs { display: flex; gap: 0.4rem; flex-wrap: wrap; margin: 0.6rem 0; }
  .field-tab { padding: 0.3rem 0.7rem; border: 1px solid var(--color-border, #ddd); border-radius: 999px; background: transparent; cursor: pointer; font-size: 0.85rem; }
  .field-tab.active { background: var(--color-accent, #4a6cf7); color: #fff; border-color: var(--color-accent, #4a6cf7); }
  .current-value { font-size: 0.9rem; margin-bottom: 0.6rem; }
  .current-value .label { color: var(--color-text-muted, #888); margin-right: 0.4rem; }
  .current-value .val { font-style: italic; }
  .md-field { display: block; margin-bottom: 0.7rem; }
  .md-field span { display: block; font-size: 0.85rem; margin-bottom: 0.2rem; color: var(--color-text-muted, #888); }
  .md-field input, .md-field textarea {
    width: 100%; padding: 0.45rem 0.6rem; border: 1px solid var(--color-border, #ddd);
    border-radius: 8px; font-size: 0.9rem; background: var(--color-bg, #fff); color: inherit;
  }
  .error { color: #c0392b; font-size: 0.85rem; }
  .ok { color: var(--color-success, #27ae60); font-size: 0.85rem; }
  .md-foot { padding: 0.7rem 1.2rem; border-top: 1px solid var(--color-border, #eee); display: flex; justify-content: flex-end; gap: 0.5rem; }
</style>
