<script lang="ts">
  /** Props for the ArchivistCall component.
   *
   * `body` holds the stored LLM answer text — typically a search URL or a
   * natural-language recommendation produced by the archivist. */
  export interface ArchivistCallProps {
    body: string;
  }

  let { body }: ArchivistCallProps = $props();

  /** Render a stored search URL as a friendly "Try this search" summary.
   * Falls back to the raw body for non-URL text. */
  function summary(): string {
    const raw = body ?? '';
    const m = raw.match(/^\/search[?/]/);
    if (m) {
      const qs = raw.slice(m[0].length).replace(/^\?/, '');
      const params = new URLSearchParams(qs);
      const tags = params.get('include_tags');
      const words = params.get('words');
      const complete = params.get('complete');
      const parts: string[] = [];
      if (tags) parts.push(`tags: ${tags}`);
      if (words) parts.push(`≥ ${words} words`);
      if (complete === 'true') parts.push('complete');
      if (parts.length) return `Try this search: ${parts.join(', ')}`;
    }
    return raw;
  }
</script>

<div class="archivist-call">
  <span class="archivist-attribution">FicNexus Archivist</span>
  <span class="archivist-label">Archivist answer</span>
  <p class="archivist-lipsum">{summary()}</p>
</div>

<style>
  .archivist-call {
    border: 1px solid var(--color-link, #2b4bd7);
    background: #eef4ff;
    padding: 0.75rem 1rem;
    border-radius: 8px;
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.95rem;
    line-height: 1.5;
    margin: 0.4rem 0;
  }
  .archivist-attribution {
    font-weight: 700;
    color: var(--color-link, #2b4bd7);
    display: block;
    margin-bottom: 0.2rem;
  }
  .archivist-label {
    font-size: 0.75rem;
    color: var(--color-text-muted, #888);
    background: var(--color-bg, #fff);
    border: 1px solid var(--color-border, #ddd);
    border-radius: 999px;
    padding: 0.1rem 0.5rem;
    display: inline-block;
    margin-bottom: 0.4rem;
  }
  .archivist-lipsum {
    margin: 0;
    white-space: pre-wrap;
    color: var(--color-text, #444);
  }
  /* Archive skin variant */
  :global(.ui-mode-archive) .archivist-call {
    border-color: var(--archive-link, #990000);
    background: var(--archive-bg-raised, #f5f5f5);
  }
  :global(.ui-mode-archive) .archivist-attribution {
    color: var(--archive-link, #990000);
  }
  :global(.ui-mode-archive) .archivist-lipsum {
    color: var(--archive-text, #2a2a2a);
  }
</style>
