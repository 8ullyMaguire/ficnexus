<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { createRequest } from '$lib/api/requests';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let title = $state('');
  let body = $state('');
  let seedWorkId = $state<number | null>(null);
  let seedTitle = $state('');
  let submitting = $state(false);
  let error = $state('');
  let ok = $state(false);

  // Honeypot: real form posts opened the form seconds ago (backend rejects
  // <500ms or missing form_opened_at).
  let formOpenedAt = $state(0);
  let website = $state(''); // honeypot trap field — humans never fill it

  // Live suggest: matches in the archive (populated on title/body blur).
  let matches = $state<Array<{ work_id: number; title: string; author: string; words: number; sources: string[] }>>([]);
  let matchesDismissed = $state(false);
  let suggestLoading = $state(false);
  let suggestRan = $state(false);

  onMount(() => {
    formOpenedAt = Date.now() - 5000;
    // $page is undefined in unit tests (no router) — guard the params.
    const seed = $page?.url?.searchParams?.get('seed');
    if (seed) {
      const parsed = Number(seed);
      if (Number.isFinite(parsed) && parsed > 0) seedWorkId = parsed;
    }
    // ?q= prefills title + body from an Ask-the-Archive query
    // (the ask page's "Turn this into a request" flow).
    const q = $page?.url?.searchParams?.get('q');
    if (q) {
      if (!title) title = q.slice(0, 200);
      if (!body) body = q;
    }
    // Prefill from the download input's "Request this fic" button (Task 3).
    const prefill = sessionStorage.getItem('fichub_request_prefill');
    if (prefill) {
      try {
        const p = JSON.parse(prefill) as { title?: string; author?: string; site?: string };
        const titleStr = p.author ? `${p.title ?? ''} by ${p.author}` : (p.title ?? '');
        if (titleStr && !title) title = titleStr.slice(0, 200);
        const detail = p.site ? `${titleStr} (${p.site})` : titleStr;
        if (detail && !body) body = detail;
      } catch {
        // ignore malformed prefill
      }
      sessionStorage.removeItem('fichub_request_prefill');
      // Auto-run suggest on prefill so the box shows up immediately.
      setTimeout(() => runSuggest(), 100);
    }
  });

  /** Best-effort live suggest: parse title+body and hit /api/find-fic. */
  async function runSuggest() {
    matchesDismissed = false;
    const text = `${title} ${body}`.trim();
    if (text.length < 4) {
      matches = [];
      suggestRan = false;
      return;
    }
    suggestLoading = true;
    suggestRan = true;
    try {
      const res = await fetch('/api/find-fic', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ query: text, limit: 3, per_page: 10 }),
      });
      if (res.ok) {
        const data = await res.json();
        matches = (data.results ?? []).slice(0, 3);
      } else {
        matches = [];
      }
    } catch {
      matches = [];
    } finally {
      suggestLoading = false;
    }
  }

  async function submit() {
    if (!title.trim()) { error = 'Title is required'; return; }
    if (website) return; // honeypot
    submitting = true;
    error = '';
    try {
      const res = await createRequest({
        title: title.trim(),
        body: body.trim(),
        seed_work_id: seedWorkId,
      });
      if (res.err === 0 && res.id) {
        ok = true;
        try {
          await goto(`/requests/${res.id}`);
        } catch {
          // jsdom/unit tests have no router — the POST succeeded, that's enough.
        }
      } else {
        error = res.msg ?? 'Failed to create request';
      }
    } catch {
      error = 'Failed to create request';
    } finally {
      submitting = false;
    }
  }
</script>

<svelte:head><title>New request — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
  <div class="new-request">
    <header class="page-head">
      <h1>{t('requests.newTitle')}</h1>
      <p class="subtitle">{t('requests.newSubtitle')}</p>
    </header>

    <form class="archive-form" onsubmit={(e) => { e.preventDefault(); submit(); }}>
      <dl class="archive-dl">
        <dt>{t('requests.formTitle')}</dt>
        <dd>
          <input
            type="text"
            bind:value={title}
            maxlength="200"
            placeholder={t('requests.formTitlePlaceholder')}
            onblur={runSuggest}
            required
          />
        </dd>
        <dt>{t('requests.formDetails')}</dt>
        <dd>
          <textarea
            bind:value={body}
            maxlength="4000"
            rows="4"
            placeholder={t('requests.formDetailsPlaceholder')}
            onblur={runSuggest}
          ></textarea>
        </dd>
      </dl>

      {#if seedWorkId}
        <div class="seed-chip">Seeded from a fic (work #{seedWorkId}){seedTitle ? ` — ${seedTitle}` : ''}
          <button type="button" onclick={() => { seedWorkId = null; seedTitle = ''; }}>remove</button>
        </div>
      {/if}

      <!-- "Already in the archive?" dismissible box -->
      {#if matches.length > 0 && !matchesDismissed}
        <aside class="archive-matches" data-testid="matches-box">
          <header>
            <strong>📚 {matches.length} fic{matches.length === 1 ? '' : 's'} already in the archive</strong>
            <button type="button" onclick={() => matchesDismissed = true} aria-label="Dismiss">✕</button>
          </header>
          <ul>
            {#each matches as m}
              <li>
                <a href="/works/{m.url_id ?? m.work_id}">{m.title}</a>
                <span class="by">by {m.author}</span>
                <span class="words">{m.words.toLocaleString('en-US')} words</span>
              </li>
            {/each}
          </ul>
        </aside>
      {/if}

      <!-- Honeypot (hidden from humans) -->
      <input type="text" name="website" bind:value={website} style="display:none" tabindex="-1" autocomplete="off" />

      {#if error}<p class="archive-error">{error}</p>{/if}
      {#if ok}<p class="archive-ok">{t('requests.created')}</p>{/if}

      <ArchiveButton type="submit" disabled={submitting || !title.trim()}>
        {submitting ? t('requests.creating') : t('requests.postRequest')}
      </ArchiveButton>
    </form>
  </div>
{:else}
  <div class="new-request">
    <header class="page-head">
      <h1>{t('requests.newTitle')}</h1>
      <p class="subtitle">{t('requests.newSubtitle')}</p>
    </header>

    <form onsubmit={(e) => { e.preventDefault(); submit(); }}>
      <label>
        {t('requests.formTitle')}
        <input
          type="text"
          bind:value={title}
          maxlength="200"
          placeholder={t('requests.formTitlePlaceholder')}
          onblur={runSuggest}
          required
        />
      </label>

      <label>
        {t('requests.formDetails')}
        <textarea
          bind:value={body}
          maxlength="4000"
          rows="4"
          placeholder={t('requests.formDetailsPlaceholder')}
          onblur={runSuggest}
        ></textarea>
      </label>

      {#if seedWorkId}
        <div class="seed-chip">📌 Seeded from a fic (work #{seedWorkId}){seedTitle ? ` — ${seedTitle}` : ''}
          <button type="button" onclick={() => { seedWorkId = null; seedTitle = ''; }}>✕ remove</button>
        </div>
      {/if}

      {#if matches.length > 0 && !matchesDismissed}
        <aside class="matches-box" data-testid="matches-box">
          <header>
            <strong>📚 {matches.length} fic{matches.length === 1 ? '' : 's'} already in the archive</strong>
            <button type="button" onclick={() => matchesDismissed = true} aria-label="Dismiss">✕</button>
          </header>
          <ul>
            {#each matches as m}
              <li>
                <a href="/works/{m.url_id ?? m.work_id}">{m.title}</a>
                <span class="by">by {m.author}</span>
                <span class="words">{m.words.toLocaleString('en-US')} words</span>
              </li>
            {/each}
          </ul>
        </aside>
      {/if}

      <input type="text" name="website" bind:value={website} style="display:none" tabindex="-1" autocomplete="off" />

      {#if error}<div class="error-card"><strong>⚠️ {error}</strong></div>{/if}
      {#if ok}<p class="ok">{t('requests.created')}</p>{/if}

      <button class="btn btn-primary" type="submit" disabled={submitting || !title.trim()}>
        {submitting ? t('requests.creating') : t('requests.postRequest')}
      </button>
    </form>
  </div>
{/if}

{#if uiMode === 'archive'}
<style>
  .new-request {
    max-width: var(--archive-max-width, 620px);
    margin: 0 auto;
    padding: 1.5rem;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .page-head {
    margin-bottom: 1.25rem;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    padding-bottom: 0.5em;
  }
  .page-head h1 {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.7em;
    font-weight: normal;
    color: var(--archive-heading, #990000);
    margin: 0 0 0.15em;
  }
  .subtitle {
    color: var(--archive-muted, #666666);
    font-size: 0.92em;
    margin: 0;
  }
  .archive-form {
    display: flex;
    flex-direction: column;
    gap: 1em;
  }
  .archive-dl { margin: 0; padding: 0; }
  .archive-dl dt {
    font-weight: 600;
    font-size: 0.9em;
    margin-bottom: 0.2em;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-dl dd {
    margin: 0 0 0.75em;
  }
  .archive-dl input,
  .archive-dl textarea {
    width: 100%;
    box-sizing: border-box;
    padding: 0.5em 0.6em;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.95em;
  }
  .archive-dl textarea { resize: vertical; min-height: 4em; }
  .seed-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.5em;
    background: var(--archive-bg-raised, #f5f5f5);
    padding: 0.3em 0.75em;
    border: 1px solid var(--archive-border, #dddddd);
    font-size: 0.9em;
  }
  .seed-chip button {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--archive-link, #990000);
    font-family: inherit;
  }
  .archive-error { color: var(--archive-heading, #990000); font-size: 0.9em; }
  .archive-ok { color: var(--archive-ok, #2e9e5b); }

  .archive-matches {
    background: var(--archive-bg-raised, #f8f4e8);
    border: 1px solid var(--archive-border, #dddddd);
    border-left: 3px solid var(--archive-link, #990000);
    padding: 0.75em 1em;
    font-size: 0.9em;
  }
  .archive-matches header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5em;
  }
  .archive-matches button {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--archive-muted, #666);
    font-family: inherit;
  }
  .archive-matches ul {
    margin: 0;
    padding-left: 1.25em;
  }
  .archive-matches li {
    margin-bottom: 0.25em;
  }
  .archive-matches .by { color: var(--archive-muted, #666); margin-left: 0.4em; }
  .archive-matches .words { color: var(--archive-muted, #666); margin-left: 0.4em; font-size: 0.85em; }
</style>
{/if}

<style>
  .new-request { max-width: 620px; margin: 0 auto; padding: 1.5rem; }
  .page-head { margin-bottom: 1.25rem; }
  .subtitle { color: var(--color-text-muted, #888); }
  form { display: flex; flex-direction: column; gap: 1rem; }
  label { display: flex; flex-direction: column; gap: 0.4rem; font-weight: 600; }
  input, textarea {
    padding: 0.6rem 0.75rem; border: 1px solid var(--color-border, #ddd); border-radius: 8px;
    font: inherit; font-weight: 400; width: 100%; box-sizing: border-box;
  }
  .seed-chip { display: inline-flex; align-items: center; gap: 0.5rem; background: var(--color-chip-bg, #eef);
    padding: 0.4rem 0.75rem; border-radius: 999px; font-size: 0.85rem; align-self: flex-start; }
  .seed-chip button { background: none; border: none; cursor: pointer; color: var(--color-link, #2b4bd7); }
  .btn-primary { align-self: flex-start; }

  .matches-box {
    background: #fffbe8;
    border: 1px solid #e0d8a8;
    border-left: 3px solid #b58a00;
    border-radius: 8px;
    padding: 0.75rem 1rem;
    font-size: 0.9rem;
  }
  .matches-box header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }
  .matches-box button {
    background: none;
    border: none;
    cursor: pointer;
    color: #666;
    font: inherit;
  }
  .matches-box ul {
    margin: 0;
    padding-left: 1.25rem;
  }
  .matches-box li {
    margin-bottom: 0.25rem;
  }
  .matches-box .by { color: #666; margin-left: 0.4rem; }
  .matches-box .words { color: #666; margin-left: 0.4rem; font-size: 0.85rem; }
</style>

