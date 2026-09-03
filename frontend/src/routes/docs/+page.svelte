<script lang="ts">
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  const docSections = [
    {
      heading: 'Start Here',
      items: [
        { title: 'What is FicNexus?', href: '/docs/what-is-fichub.html', desc: 'What the archive does, where its content comes from, and how it differs from other download tools.' },
        { title: 'Quick Start', href: '/docs/quickstart.html', desc: 'Download your first fic in under a minute — URL in, EPUB out.' },
        { title: 'FAQ', href: '/docs/faq.html', desc: 'Common questions about sources, formats, accounts, and privacy.' },
        { title: 'Feature Overview', href: '/docs/features.html', desc: 'Everything FicNexus can do: search, reader, social, progression, recipes.' },
      ],
    },
    {
      heading: 'User Guide',
      items: [
        { title: 'Searching', href: '/docs/searching.html', desc: 'Boolean operators, fielded filters, tag search, full-text quote search, and Ask-the-Archive.' },
        { title: 'Downloading & Formats', href: '/docs/downloading.html', desc: 'EPUB, MOBI, PDF, AZW3 and more; Send-to-Kindle; OPDS catalogs for e-readers.' },
        { title: 'Bookmarks & Shelves', href: '/docs/bookmarks.html', desc: 'Save works, organize shelves, import/export, and reading status tracking.' },
        { title: 'Ratings & Reviews', href: '/docs/ratings.html', desc: '5-star ratings, in-depth reviews, kudos, and how feedback feeds recommendations.' },
        { title: 'Comments', href: '/docs/comments.html', desc: 'Threaded comments on works and reactions.' },
        { title: 'Recommendations', href: '/docs/recommendations.html', desc: 'Personalized recs, Blind Date, trending, and similar-work suggestions.' },
        { title: 'Translations & the Curator Queue', href: '/docs/translations.html', desc: 'Machine + community translation of everything you read, consensus review, and reputation for helping.' },
        { title: 'Reading in Browser', href: '/docs/profile.html', desc: 'Reader preferences, reading history, stats, and cross-device sync.' },
      ],
    },
    {
      heading: 'Power Features',
      items: [
        { title: 'Recommendation Engines', href: '/docs/rec-engines.html', desc: 'The strategy registry behind recs — cooccur, embeddings, bandits — and the Recipe Builder.' },
      ],
    },
    {
      heading: 'Community & Moderation',
      items: [
        { title: 'Transparency', href: '/docs/transparency.html', desc: 'The public modlog, curator actions, and how moderation is audited.' },
        { title: 'Anti-Bot Defense', href: '/docs/anti-bot.html', desc: 'Rate limiting, proof-of-work, honeypots, and shadowbans explained.' },
        { title: 'Self-Healing Scrapers', href: '/docs/self-healing.html', desc: 'Failure classification, site-as-cache, and peer-voted curator fixes.' },
      ],
    },
    {
      heading: 'Contributing',
      items: [
        { title: 'Contributing Guide', href: '/docs/contributing.html', desc: 'Junior-dev onboarding: repo layout, testing conventions, and workflow.' },
        { title: 'Integrations', href: '/docs/integrations.html', desc: 'The Discord/Telegram bot, CLI, bookmarklet, and API clients.' },
      ],
    },
  ];
</script>

<svelte:head><title>Documentation — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Documentation</h1>
        <p class="archive-summary">Guides for readers, curators, and contributors — all readable right here in the app.</p>
      </header>

      {#each docSections as section}
        <section class="archive-section">
          <h2 class="archive-subhead">{section.heading}</h2>
          <dl class="archive-dl">
            {#each section.items as doc}
              <div class="archive-dl-row">
                <dt><a class="archive-link" href={doc.href} target={doc.href.startsWith('http') ? '_blank' : '_self'} rel={doc.href.startsWith('http') ? 'noopener noreferrer' : undefined}>{doc.title}</a></dt>
                <dd class="archive-muted">{doc.desc}</dd>
              </div>
            {/each}
          </dl>
        </section>
      {/each}

      <section class="archive-section">
        <h2 class="archive-subhead">Quick links</h2>
        <ul class="archive-list">
          <li><a class="archive-link" href="/docs/toc.html">Full table of contents</a> <span class="archive-muted">— every docs page, one list</span></li>
          <li><a class="archive-link" href="/docs/print.html">Single-page edition</a> <span class="archive-muted">— the whole book on one page (printable)</span></li>
        </ul>
      </section>
    </div>
  </main>
{:else}
  <div class="doc-page">
    <h1>Documentation</h1>
    <p class="subtitle">Guides for readers, curators, and contributors.</p>
    {#each docSections as section}
      <section class="doc-section">
        <h3>{section.heading}</h3>
        <dl>
          {#each section.items as doc}
            <div class="doc-row">
              <dt><a href={doc.href} target={doc.href.startsWith('http') ? '_blank' : '_self'}>{doc.title}</a></dt>
              <dd>{doc.desc}</dd>
            </div>
          {/each}
        </dl>
      </section>
    {/each}
  </div>
{/if}

<style>
  .archive-main {
    max-width: var(--archive-max-width, 1100px);
    margin: 0 auto;
    padding: 1rem;
  }
  .archive-content {
    max-width: 900px;
    margin: 0 auto;
    width: 100%;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-header {
    padding-bottom: 0.6em;
    border-bottom: 2px solid var(--archive-link, #990000);
    margin-bottom: 1em;
  }
  .archive-page-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.8em;
    font-weight: normal;
    margin: 0;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-summary {
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.75em 1em;
    margin: 0.5em 0 1em;
    font-style: italic;
    color: var(--archive-muted, #666666);
  }
  .archive-summary a { color: var(--archive-link, #990000); }
  .archive-subhead {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.15em;
    font-weight: 700;
    color: var(--archive-link, #990000);
    margin: 1em 0 0.5em;
  }
  .archive-section {
    margin-bottom: 1.2em;
    padding-bottom: 1em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }
  .archive-section:last-child { border-bottom: none; }
  .archive-dl {
    margin: 0;
    padding: 0;
  }
  .archive-dl-row {
    display: grid;
    grid-template-columns: 220px 1fr;
    gap: 0.3em 1em;
    padding: 0.4em 0;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
  }
  .archive-dl-row:last-child { border-bottom: none; }
  .archive-dl-row dt {
    font-weight: 700;
    font-size: 0.95em;
  }
  .archive-dl-row dt a { color: var(--archive-link, #990000); text-decoration: none; }
  .archive-dl-row dt a:hover { text-decoration: underline; }
  .archive-dl-row dd {
    margin: 0;
    font-size: 0.9em;
    color: var(--archive-muted, #666666);
    font-style: italic;
  }
  .archive-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .archive-list li {
    padding: 0.3em 0;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
  }
  .archive-list li:last-child { border-bottom: none; }
  .archive-link { color: var(--archive-link, #990000); text-decoration: none; }
  .archive-link:hover { text-decoration: underline; }
  .archive-muted { color: var(--archive-muted, #666666); font-style: italic; font-size: 0.9em; }

  @media (max-width: 600px) {
    .archive-dl-row { grid-template-columns: 1fr; }
  }

  .doc-page {
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem 1rem;
  }
  .doc-page h1 {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.8rem;
  }
  .subtitle { color: #666; margin-bottom: 1.5rem; }
  .doc-section h3 {
    font-size: 1.1rem;
    font-weight: 700;
    margin: 0 0 0.5rem;
  }
  .doc-section dl { margin: 0 0 1.5rem; }
  .doc-row { display: grid; grid-template-columns: 220px 1fr; gap: 0.5em 1em; padding: 0.3em 0; }
  .doc-row dt a { font-weight: 600; color: #990000; text-decoration: none; }
  .doc-row dt a:hover { text-decoration: underline; }
  .doc-row dd { margin: 0; color: #666; font-size: 0.9em; }
  @media (max-width: 600px) { .doc-row { grid-template-columns: 1fr; } }
</style>
