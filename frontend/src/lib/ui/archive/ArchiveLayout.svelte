<script lang="ts">
  import type { Snippet } from 'svelte';
  import ArchiveHeader from './ArchiveHeader.svelte';
  import ArchiveFooter from './ArchiveFooter.svelte';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import HelpModal from '$lib/components/HelpModal.svelte';
  import ForumBottomNav from '$lib/components/ForumBottomNav.svelte';
  import ConsentToast from '$lib/ui/consent/ConsentToast.svelte';
  import { onMount } from 'svelte';
  import { applyTheme, loadTheme } from '$lib/themes/apply.js';
  import { initI18n } from '$lib/i18n/index.svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { getPref, type ArchiveSkin } from '$lib/prefs';

  let { children }: { children: Snippet } = $props();

  onMount(() => {
    applyTheme(loadTheme());
    initI18n({ userLocale: auth.user?.locale ?? null });
    auth.init();
    // Apply the archive skin class to <body>
    const skin = getPref('archiveSkin');
    document.body.classList.add(`skin-${skin}`);
  });
</script>

<svelte:head>
  <link rel="stylesheet" href="/styles/zerafina-skin.css" />
</svelte:head>

<div class="archive-shell">
  <ArchiveHeader />

  <main class="archive-main">
    {@render children()}
  </main>

  <ArchiveFooter />
  <ForumBottomNav />
</div>

<HelpModal />
<CommandPalette />
<ConsentToast />

<style>
  .archive-shell {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--archive-bg, #ffffff);
    /* AO3 body text is plain black */
    color: #000;
    font-family: Georgia, 'Times New Roman', serif;
  }

  .archive-main {
    flex: 1;
    max-width: 1100px;
    margin: 0 auto;
    width: 100%;
    padding: 1rem;
    /* AO3 #main: 0.875em base, 1.286 line-height — the compact density */
    font-size: 0.875em;
    line-height: 1.286;
  }

  /* AO3 base-link contract (1_site_screen.css): body text is #000 (not the
     red heading colour); links are dark #111 with a solid bottom border as
     the underline — on hover the TEXT greys (#999) and the underline
     disappears; visited links grey to #666 with a dashed bottom border. */
  :global(.archive-shell) {
    color: #000;
  }

  :global(.archive-shell a) {
    color: #111;
    text-decoration: none;
    border-bottom: 1px solid;
  }

  :global(.archive-shell a:visited) {
    color: #666;
    text-decoration: none;
    border-bottom: 1px dashed;
  }

  :global(.archive-shell a:visited:hover) {
    color: #111;
  }

  /* AO3 hover = the text (and its currentcolor underline) fades grey; the
     underline is permanent via currentcolor, never added/removed on hover. */
  :global(.archive-shell a:hover) {
    color: #999;
    text-decoration: none;
  }

  :global(.archive-shell a:active),
  :global(.archive-shell a:focus),
  :global(.archive-shell button:focus) {
    outline: 1px dotted;
  }

  /* Red (#900) stays a heading/link colour only where AO3 keeps it:
     the brand, blurbs and the header. Body links use the dark scheme. */
  :global(.archive-shell .blurb h4 a:link),
  :global(.archive-shell .blurb h4 a:visited),
  :global(.archive-shell h4 .blurb-heading),
  :global(.archive-shell .rec-title),
  :global(.archive-shell .trending-title),
  :global(.archive-shell .request-title),
  :global(.archive-shell .arc-link),
  /* AO3 keeps the logo #900 even when visited (#ID specificity beats
     a:visited) — pin :link/:visited explicitly. */
  :global(.archive-shell .archive-brand),
  :global(.archive-shell a:link.archive-brand),
  :global(.archive-shell a:visited.archive-brand),
  :global(.archive-shell a:hover.archive-brand) {
    color: var(--archive-link, #990000);
    border-bottom: none;
  }

  /* AO3: #header / nav / footer keep their own regimes — undo the global
     border-underline inside them. */
  :global(.archive-shell #header a),
  :global(.archive-shell .archive-header a),
  :global(.archive-shell .archive-nav-link),
  :global(.archive-shell .forum-bottomnav a),
  :global(.archive-shell .forum-bottom-nav a) {
    border-bottom: none;
  }

  :global(.archive-shell .archive-footer a),
  :global(.archive-shell .archive-footer a:link),
  :global(.archive-shell .archive-footer a:visited) {
    text-decoration: underline;
    border-bottom: 0;
  }

  :global(.archive-shell .archive-footer a:hover) {
    color: #fff;
    border-bottom: 0;
  }

  /* AO3 tags: dotted underline → #900 box with #fff text on hover. */
  /* AO3 tags (commas lists): dotted underline -> #900 box, #fff text hover.
     Exclude the skin's category chips (.tag-warning etc.) which keep their
     chip styling. */
  :global(.archive-shell a.tag:not([class*='tag-'])) {
    color: #111;
    line-height: 1.5;
    text-decoration: none;
    padding: 0;
    border-bottom: 1px dotted;
  }

  :global(.archive-shell a.tag:not([class*='tag-']):hover) {
    background: #900;
    color: #fff;
    border-bottom: 1px dotted;
  }

  /* AO3 button look for real buttons (actions/submit): gradient box, #444
     text; hover lifts the ink (#900) + inset shadow. */
  :global(.archive-shell button),
  :global(.archive-shell input[type="submit"]) {
    background: #eee;
    background-image: linear-gradient(#fff 2%, #ddd 95%, #bbb 100%);
    color: #444;
    font-weight: 400;
    line-height: 1.286;
    display: inline-block;
    padding: 0.25em 0.75em;
    white-space: nowrap;
    text-decoration: none;
    border: 1px solid #bbb;
    border-bottom: 1px solid #aaa;
    border-radius: 0.25em;
    cursor: pointer;
    font-family: inherit;
  }

  :global(.archive-shell button:hover),
  :global(.archive-shell button:focus),
  :global(.archive-shell input[type="submit"]:hover) {
    color: #900;
    border-top: 1px solid #999;
    border-left: 1px solid #999;
    box-shadow: inset 2px 2px 2px #bbb;
  }

  :global(.archive-shell input:focus),
  :global(.archive-shell select:focus),
  :global(.archive-shell textarea:focus) {
    background: #f3efec;
  }
</style>
