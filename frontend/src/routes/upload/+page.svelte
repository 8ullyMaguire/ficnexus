<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  import { auth } from '$lib/stores/auth.svelte';
  import { authHeaders } from '$lib/api/social';
  import RepGainToast from '$lib/components/RepGainToast.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let title = $state('');
  let author = $state('');
  let summary = $state('');
  let tags = $state('');
  let status = $state('');
  let file: File | null = $state(null);
  let submitting = $state(false);
  let error = $state('');
  let success = $state(false);
  let returnedUrlId: string | null = $state(null);
  let repGained = $state(0);
  let repTotal = $state(0);
  let showRepToast = $state(false);

  // Honeypot + form-timing trap: the backend's honeypot::inspect_submission
  // silently rejects submissions without a JS-set form_opened_at (bots script
  // the API directly and skip it). Real users open the form seconds before
  // submitting. The hidden `website` field stays empty for humans.
  let formOpenedAt = $state(0);
  let website = $state('');

  onMount(async () => {
    if (!auth.initialized) await auth.init();
    formOpenedAt = Date.now() - 5000; // account for initial page load
  });

  function onFileChange(e: Event) {
    const input = e.target as HTMLInputElement;
    file = input.files?.[0] ?? null;
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!auth.isLoggedIn) {
      error = 'Please log in to upload.';
      return;
    }
    // Honeypot: if the hidden `website` field was filled, silently abort —
    // this is a bot. We don't show an error; we just do nothing.
    if (website) return;
    if (!title.trim() || !author.trim() || !file) {
      error = 'Title, author, and file are required.';
      return;
    }
    submitting = true;
    error = '';
    success = false;
    returnedUrlId = null;
    try {
      const formData = new FormData();
      formData.append('title', title.trim());
      formData.append('author', author.trim());
      formData.append('summary', summary.trim());
      formData.append('tags', tags.trim());
      formData.append('status', status);
      // Honeypot + timing trap fields consumed by the backend.
      formData.append('website', website);
      formData.append('form_opened_at', String(formOpenedAt));
      formData.append('file', file);

      const res = await fetch('/api/upload', {
        method: 'POST',
        body: formData,
        headers: { ...authHeaders() }
      });

      if (!res.ok) {
        const txt = await res.text().catch(() => '');
        let msg = txt;
        try {
          const j = JSON.parse(txt);
          msg = j.msg ?? j.message ?? txt;
        } catch {}
        throw new Error(msg || `Upload failed (${res.status})`);
      }

      const data = await res.json().catch(() => ({}));
      // backend returns { err:0, url_id, work_id } on success; some errors use err !=0
      if (data.err !== undefined && data.err !== 0) {
        throw new Error(data.msg ?? data.message ?? 'Upload failed');
      }

      success = true;
      returnedUrlId = data.url_id ?? data.urlId ?? null;
      // Reputation toast: show "+N reputation" on every productive action.
      if (typeof data.rep_gained === 'number' && data.rep_gained > 0) {
        repGained = data.rep_gained;
        repTotal = typeof data.reputation === 'number' ? data.reputation : repTotal;
        showRepToast = true;
      }
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      submitting = false;
    }
  }
</script>

<svelte:head><title>Import Work — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
{#if showRepToast}
  <RepGainToast
    amount={repGained}
    eventLabel="Upload"
    totalRep={repTotal}
    ondismiss={() => (showRepToast = false)}
  />
{/if}
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1>Import Work</h1>
      <p class="archive-summary">Upload a fic file with optional metadata. Files are held for curator approval before they appear publicly.</p>
    </header>

    {#if !auth.isLoggedIn}
      <div class="archive-login-prompt">
        <p>Please <a class="archive-link" href="/login">log in</a> to import works.</p>
        <p class="archive-muted">You need an account to upload files for curation.</p>
      </div>
    {:else}
      <form class="archive-form" onsubmit={handleSubmit}>
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Work Info</legend>
          <dl class="archive-dl">
            <dt class="archive-required"><label for="upload-title">Title *</label></dt>
            <dd><input id="upload-title" class="archive-input" type="text" bind:value={title} required maxlength="500" placeholder="Work title" /></dd>

            <dt class="archive-required"><label for="upload-author">Author *</label></dt>
            <dd><input id="upload-author" class="archive-input" type="text" bind:value={author} required maxlength="500" placeholder="Author name" /></dd>

            <dt class="archive-required"><label for="upload-file">File *</label></dt>
            <dd>
              <input id="upload-file" class="archive-input" type="file" accept=".txt,.epub,.html,.md" onchange={onFileChange} required />
              <p class="archive-note">Accepted: .txt, .epub, .html, .md</p>
            </dd>

            <dt><label for="upload-summary">Summary</label></dt>
            <dd><textarea id="upload-summary" class="archive-textarea" bind:value={summary} rows="4" maxlength="5000" placeholder="Optional summary / description"></textarea></dd>

            <dt><label for="upload-tags">Tags</label></dt>
            <dd>
              <input id="upload-tags" class="archive-input" type="text" bind:value={tags} placeholder="comma-separated tags" />
              <p class="archive-note">Comma-separated, e.g. "time travel, slow burn"</p>
            </dd>

            <dt><label for="upload-status">Status</label></dt>
            <dd>
              <select id="upload-status" class="archive-select" bind:value={status}>
                <option value="complete">Complete</option>
                <option value="ongoing">In Progress</option>
                <option value="">Unknown</option>
              </select>
            </dd>
          </dl>
        </fieldset>

        {#if error}
          <p class="archive-error" role="alert">{error}</p>
        {/if}
        {#if success}
          <p class="archive-ok" role="status">
            Uploaded — awaiting curator approval. Track at <a class="archive-link" href="/admin/moderation">/admin/moderation</a> or <a class="archive-link" href="/dashboard">Dashboard</a>.
            {#if returnedUrlId}
              <br /><a class="archive-link" href="/works/{returnedUrlId}">View at /works/{returnedUrlId}</a>
            {/if}
          </p>
        {/if}

        <div class="archive-form-actions">
          <button class="archive-btn archive-btn-primary" type="submit" disabled={submitting}>
            {submitting ? 'Uploading…' : 'Import Work'}
          </button>
        </div>
      </form>
    {/if}
  </div>
</main>
{:else}
{#if showRepToast}
  <RepGainToast
    amount={repGained}
    eventLabel="Upload"
    totalRep={repTotal}
    ondismiss={() => (showRepToast = false)}
  />
{/if}
<div class="upload-page">
  <h1>Import Work</h1>
  <p class="muted">Upload a fic file with optional metadata. Files are held for curator approval before they appear publicly.</p>

  {#if !auth.isLoggedIn}
    <div class="login-prompt">
      <p>Please <a href="/login">log in</a> to import works.</p>
    </div>
  {:else}
    <form onsubmit={handleSubmit}>
      <label>Title *<input type="text" bind:value={title} required maxlength="500" /></label>
      <label>Author *<input type="text" bind:value={author} required maxlength="500" /></label>
      <label>File *<input type="file" accept=".txt,.epub,.html,.md" onchange={onFileChange} required /></label>
      <p class="note">Accepted: .txt, .epub, .html, .md</p>
      <label>Summary<textarea bind:value={summary} rows="4" maxlength="5000"></textarea></label>
      <label>Tags (comma-separated)<input type="text" bind:value={tags} /></label>
      <label>Status
        <select bind:value={status}>
          <option value="complete">Complete</option>
          <option value="ongoing">In Progress</option>
          <option value="">Unknown</option>
        </select>
      </label>

      <!-- Honeypot (hidden from humans, mirrors requests/new form) -->
      <input type="text" name="website" bind:value={website} style="display:none" tabindex="-1" autocomplete="off" />

      {#if error}<p class="error" role="alert">{error}</p>{/if}
      {#if success}
        <p class="ok" role="status">
          Uploaded — awaiting curator approval. Track at <a href="/admin/moderation">/admin/moderation</a> or <a href="/dashboard">Dashboard</a>.
          {#if returnedUrlId} <a href="/works/{returnedUrlId}">View at /works/{returnedUrlId}</a>{/if}
        </p>
      {/if}

      <button class="btn btn-primary" type="submit" disabled={submitting}>
        {submitting ? 'Uploading…' : 'Import Work'}
      </button>
    </form>
  {/if}
</div>
{/if}

<style>
  /* Archive */
  .archive-content {
    max-width: 760px;
    margin: 0 auto;
    padding: 1rem;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-header {
    margin-bottom: 1.25rem;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    padding-bottom: 0.5em;
  }
  .archive-header h1 {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 2em;
    font-weight: 400;
    color: var(--archive-heading, #990000);
    margin: 0 0 0.15em;
  }
  .archive-summary {
    color: var(--archive-muted, #666666);
    font-size: 0.95em;
    margin: 0;
  }
  .archive-form {
    display: flex;
    flex-direction: column;
    gap: 1.25em;
  }
  .archive-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    padding: 1em 1.25em;
    margin: 0;
  }
  .archive-legend {
    font-family: Georgia, 'Times New Roman', serif;
    font-weight: 700;
    font-size: 1.05em;
    color: var(--archive-text, #2a2a2a);
    padding: 0 0.4em;
  }
  .archive-dl {
    margin: 0;
    padding: 0;
  }
  .archive-dl dt {
    font-weight: 700;
    font-size: 0.9em;
    margin: 0.85em 0 0.2em;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-dl dt:first-child { margin-top: 0; }
  .archive-required::after {
    content: ' *';
    color: #990000;
  }
  .archive-dl dd { margin: 0; }
  .archive-input,
  .archive-textarea,
  .archive-select {
    width: 100%;
    box-sizing: border-box;
    padding: 0.5em 0.6em;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.95em;
    background: var(--archive-bg, #ffffff);
  }
  .archive-textarea { resize: vertical; min-height: 6em; }
  .archive-note {
    font-size: 0.82em;
    color: var(--archive-muted, #666666);
    margin: 0.25em 0 0;
  }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-ok { color: #2e7d32; font-size: 0.92em; background: #f0f7f0; border: 1px solid #c8e6c9; padding: 0.6em 0.8em; }
  .archive-muted { color: var(--archive-muted, #666666); font-size: 0.9em; }
  .archive-link { color: var(--archive-link, #990000); }
  .archive-form-actions { margin-top: 0.5em; }
  .archive-btn {
    display: inline-block;
    padding: 0.45em 1.1em;
    font-size: 0.9em;
    font-weight: 700;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-btn-bg, #f5f5f5);
    color: var(--archive-text, #2a2a2a);
    cursor: pointer;
    font-family: inherit;
  }
  .archive-btn:hover:not(:disabled) { background: #eeeeee; }
  .archive-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .archive-btn-primary {
    background: var(--archive-link, #990000);
    color: #ffffff;
    border-color: var(--archive-link, #990000);
  }
  .archive-btn-primary:hover:not(:disabled) { background: #7a0000; }
  .archive-login-prompt {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 1em 1.25em;
    background: var(--archive-bg-raised, #fafafa);
  }
  /* Modern */
  .upload-page { max-width: 640px; margin: 0 auto; padding: 1.5rem; }
  .upload-page h1 { margin-bottom: 0.25em; }
  .muted { color: var(--color-text-muted, #666); }
  .upload-page form { display: flex; flex-direction: column; gap: 0.9rem; margin-top: 1rem; }
  .upload-page label { display: flex; flex-direction: column; gap: 0.35rem; font-weight: 600; font-size: 0.95em; }
  .upload-page input[type="text"], .upload-page textarea, .upload-page select {
    padding: 0.55rem 0.7rem; border: 1px solid var(--color-border, #ddd); border-radius: 6px; font: inherit; font-weight: 400;
  }
  .upload-page .note { font-size: 0.82em; color: #666; margin: -0.4rem 0 0; }
  .error { color: #c0392b; font-weight: 600; }
  .ok { color: #2e7d32; background: #f0f7f0; border: 1px solid #c8e6c9; padding: 0.6em 0.8em; }
  .login-prompt { border: 1px solid #ddd; padding: 1em; background: #fafafa; margin-top: 1em; }
</style>
