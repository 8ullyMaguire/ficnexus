<script lang="ts">
  import { docHelpState, closeDoc, loadDocSection } from '$lib/stores/doc-help.svelte';

  // Ask-the-Docs
  let askQ = $state('');
  let askResults = $state<AskResult[] | null>(null);
  let askErr = $state('');
  let askLoading = $state(false);

  interface AskResult {
    slug: string;
    page: string;
    anchor: string | null;
    title: string;
    feature: string;
    snippet: string;
    score: number;
  }

  async function ask() {
    const q = askQ.trim();
    if (!q || askLoading) return;
    askLoading = true;
    askErr = '';
    askResults = null;
    try {
      const res = await fetch(`/api/docs/ask?q=${encodeURIComponent(q)}&n=4`, { credentials: 'include' });
      const body = await res.json();
      if (body.err !== 0) throw new Error(body.msg ?? `HTTP ${res.status}`);
      askResults = body.results ?? [];
      if ((askResults ?? []).length === 0) askErr = 'No docs sections matched — try rewording, or browse the full docs.';
    } catch (e) {
      askErr = e instanceof Error ? e.message : String(e);
    } finally {
      askLoading = false;
    }
  }

  function openResult(r: AskResult) {
    void loadDocSection(r.slug);
    askResults = null;
    askQ = '';
  }
</script>

{#if docHelpState.active}
  <div class="doc-modal-backdrop" role="presentation" onclick={(e) => { if (e.target === e.currentTarget) closeDoc(); }}>
    <div class="doc-modal" role="dialog" aria-modal="true" aria-label="Help: {docHelpState.active.title}">
      <header class="doc-modal-head">
        <h2>{docHelpState.active.title}</h2>
        <button class="close" aria-label="Close help" onclick={closeDoc}>✕</button>
      </header>

      <!-- Ask the Docs -->
      <div class="ask-row">
        <input
          type="search"
          placeholder="Ask the docs: e.g. “how do I find dark harry fics?”"
          bind:value={askQ}
          onkeydown={(e) => { if (e.key === 'Enter') void ask(); }}
          aria-label="Ask the docs"
        />
        <button class="btn btn-small" onclick={() => void ask()} disabled={askLoading}>
          {askLoading ? '…' : 'Ask'}
        </button>
        {#if askErr}<p class="error ask-msg">{askErr}</p>{/if}
      </div>
      {#if askResults && askResults.length > 0}
        <ul class="ask-results">
          {#each askResults as r (r.slug)}
            <li>
              <button class="ask-result" onclick={() => openResult(r)}>
                <span class="ask-title">📖 {r.title}</span>
                <span class="ask-snippet">{r.snippet}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}

      <div class="doc-modal-body">
        {#if docHelpState.loading}
          <p class="muted"><span class="spinner"></span> Loading docs…</p>
        {:else if docHelpState.error}
          <p class="error">⚠️ {docHelpState.error}</p>
        {:else}
          <div class="mdbook-embed">
            {@html docHelpState.active.html}
          </div>
        {/if}
      </div>
      <footer class="doc-modal-foot">
        <a class="btn btn-small" href={`/docs/${docHelpState.active.page}${docHelpState.active.anchor ? `#${docHelpState.active.anchor}` : ''}`} target="_blank" rel="noopener">
          Open full docs ↗
        </a>
      </footer>
    </div>
  </div>
{/if}

<style>
  .doc-modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1rem;
  }
  .doc-modal {
    background: var(--color-bg, #fff);
    border-radius: 12px;
    max-width: 640px;
    width: 100%;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.25);
    overflow: hidden;
  }
  .doc-modal-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.9rem 1.2rem;
    border-bottom: 1px solid var(--color-border, #eee);
  }
  .doc-modal-head h2 { margin: 0; font-size: 1.05rem; }
  .close {
    background: none;
    border: none;
    font-size: 1.1rem;
    cursor: pointer;
    color: var(--color-text-muted, #888);
  }
  .ask-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 1.2rem;
    border-bottom: 1px solid var(--color-border, #eee);
  }
  .ask-row input {
    flex: 1;
    padding: 0.4rem 0.6rem;
    border: 1px solid var(--color-border, #ddd);
    border-radius: 8px;
    font-size: 0.9rem;
    background: var(--color-bg, #fff);
    color: inherit;
  }
  .ask-msg { font-size: 0.8rem; }
  .ask-results {
    list-style: none;
    margin: 0;
    padding: 0.4rem 1.2rem;
    border-bottom: 1px solid var(--color-border, #eee);
    max-height: 180px;
    overflow-y: auto;
  }
  .ask-results li { margin: 0.2rem 0; }
  .ask-result {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.15rem;
    width: 100%;
    padding: 0.45rem 0.6rem;
    border: none;
    background: var(--color-accent-bg, #eef1ff);
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    color: inherit;
  }
  .ask-result:hover { filter: brightness(0.97); }
  .ask-title { font-weight: 600; font-size: 0.9rem; }
  .ask-snippet { font-size: 0.8rem; color: var(--color-text-muted, #888); display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
  .doc-modal-body {
    padding: 1rem 1.2rem;
    overflow-y: auto;
    flex: 1;
  }
  .doc-modal-foot {
    padding: 0.7rem 1.2rem;
    border-top: 1px solid var(--color-border, #eee);
    display: flex;
    justify-content: flex-end;
  }
  .muted { color: var(--color-text-muted, #888); }
  .error { color: #c0392b; }
</style>
