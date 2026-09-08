<script lang="ts">
  import { streamAuthorDownload } from '$lib/api/social';

  interface Props {
    authorUrl: string;
    authorName: string;
    siteName: string;
  }

  let { authorUrl, authorName, siteName }: Props = $props();

  let downloading = $state(false);
  let current = $state(0);
  let total = $state(0);
  let currentTitle = $state('');
  let completed = $state<string[]>([]);
  let error = $state('');
  let progress = $derived(total > 0 ? Math.round((current / total) * 100) : 0);

  function startDownload() {
    downloading = true;
    current = 0;
    total = 0;
    currentTitle = '';
    completed = [];
    error = '';

    streamAuthorDownload(
      authorUrl,
      (c, t, title) => {
        current = c;
        total = t;
        currentTitle = title;
      },
      (filename, blob) => {
        // Trigger browser download
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        downloading = false;
      },
      (msg) => {
        error = msg;
        downloading = false;
      }
    );
  }
</script>

<div class="batch-download">
  {#if !downloading && !error}
    <button class="btn-download" onclick={startDownload}>
      ⬇ Download all works by {authorName}
      <span class="site-badge">{siteName}</span>
    </button>
  {/if}

  {#if downloading}
    <div class="progress-container">
      <div class="progress-header">
        <span class="progress-label">Downloading works...</span>
        <span class="progress-count">{current} / {total}</span>
      </div>
      <div class="progress-bar-track">
        <div class="progress-bar-fill" style="width: {progress}%"></div>
      </div>
      <div class="progress-percent">{progress}%</div>
      {#if currentTitle}
        <div class="progress-current">{currentTitle}</div>
      {/if}
    </div>
  {/if}

  {#if error}
    <div class="error-box">
      <span class="error-icon">⚠</span>
      <span class="error-text">{error}</span>
      <button class="retry-btn" onclick={startDownload}>Retry</button>
    </div>
  {/if}
</div>

<style>
  .batch-download {
    margin: 0.75rem 0;
  }

  .btn-download {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border: 1px solid var(--color-primary, #6366f1);
    border-radius: 0.375rem;
    background: transparent;
    color: var(--color-primary, #6366f1);
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-download:hover {
    background: var(--color-primary, #6366f1);
    color: white;
  }

  .site-badge {
    font-size: 0.75rem;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    background: rgba(99, 102, 241, 0.1);
    color: var(--color-primary, #6366f1);
  }

  .progress-container {
    padding: 0.75rem;
    border: 1px solid var(--color-border, #333);
    border-radius: 0.375rem;
    background: var(--color-surface, #1a1a2e);
  }

  .progress-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.375rem;
    font-size: 0.8125rem;
  }

  .progress-label {
    color: var(--color-text, #e0e0e0);
    font-weight: 500;
  }

  .progress-count {
    color: var(--color-text-muted, #888);
  }

  .progress-bar-track {
    height: 0.5rem;
    border-radius: 0.25rem;
    background: var(--color-bg, #0a0a1a);
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    border-radius: 0.25rem;
    background: var(--color-primary, #6366f1);
    transition: width 0.3s ease;
  }

  .progress-percent {
    margin-top: 0.25rem;
    font-size: 0.75rem;
    color: var(--color-text-muted, #888);
    text-align: right;
  }

  .progress-current {
    margin-top: 0.25rem;
    font-size: 0.75rem;
    color: var(--color-text, #e0e0e0);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .error-box {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--color-error, #ef4444);
    border-radius: 0.375rem;
    background: rgba(239, 68, 68, 0.1);
    font-size: 0.8125rem;
  }

  .error-icon {
    color: var(--color-error, #ef4444);
  }

  .error-text {
    flex: 1;
    color: var(--color-error, #ef4444);
  }

  .retry-btn {
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--color-error, #ef4444);
    border-radius: 0.25rem;
    background: transparent;
    color: var(--color-error, #ef4444);
    font-size: 0.75rem;
    cursor: pointer;
  }
</style>
