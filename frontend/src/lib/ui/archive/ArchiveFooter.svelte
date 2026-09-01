<script lang="ts">
  import LocaleSelector from '$lib/components/LocaleSelector.svelte';
</script>

<footer class="archive-footer">
  <div class="archive-footer-columns">
    <div class="archive-footer-col">
      <h4 class="archive-footer-heading">Customize</h4>
      <a class="archive-footer-link" href="/settings">Settings</a>
      <a class="archive-footer-link" href="/settings">Interface style</a>
    </div>
    <div class="archive-footer-col">
      <h4 class="archive-footer-heading">About</h4>
      <a class="archive-footer-link" href="/docs">Docs</a>
      <a class="archive-footer-link" href="/roadmap">Roadmap</a>
      <a class="archive-footer-link" href="/privacy">Privacy</a>
      <a class="archive-footer-link" href="/copyright">Copyright / DMCA</a>
      <span class="archive-footer-link archive-footer-inert">Consensus</span>
    </div>
    <div class="archive-footer-col">
      <h4 class="archive-footer-heading">Contact</h4>
      <a class="archive-footer-link" href="/forum">Forum</a>
      <a class="archive-footer-link" href="/requests">Requests</a>
      <a class="archive-footer-link" href="/ask">Ask</a>
      <a class="archive-footer-link" href="https://github.com/ficnexus/ficnexus/issues">Report a bug</a>
    </div>
    <div class="archive-footer-col">
      <h4 class="archive-footer-heading">Development</h4>
      <a class="archive-footer-link" href="/modlog">Modlog</a>
      <a class="archive-footer-link" href="/opds">OPDS</a>
      <a class="archive-footer-link" href="/feed.xml">Atom Feed</a>
      <span class="archive-footer-link archive-footer-inert">API Docs</span>
    </div>
    <div class="archive-footer-col locale-selector-col">
      <h4 class="archive-footer-heading">Language</h4>
      <LocaleSelector />
    </div>
  </div>
  <div class="archive-footer-bottom">
  </div>
</footer>

<style>
  .archive-footer {
    background: var(--archive-accent-line, #990000) url('/images/red-ao3.png');
    color: #fff;
    padding: 1.2rem 1rem 0.8rem;
    margin-top: auto;
  }

  .archive-footer-columns {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 1rem 1.5rem;
    max-width: 1100px;
    margin: 0 auto;
  }

  .archive-footer-heading {
    font-size: 0.78em;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin: 0 0 0.3rem;
    color: #fff;
  }

  /* AO3 footer: #fff links, permanent underline, hover keeps #fff (no
     colour flip, no added decoration). */
  .archive-footer-link,
  .archive-footer-link:link,
  .archive-footer-link:visited {
    display: block;
    font-size: 0.78em;
    color: #fff;
    text-decoration: underline;
    text-underline-position: under;
    border-bottom: none;
    padding: 0.15em 0;
  }

  .archive-footer-link:hover {
    color: #fff;
  }

  .archive-footer-inert {
    cursor: default;
    opacity: 0.5;
  }

  .archive-footer-inert:hover {
    color: rgba(255, 255, 255, 0.85);
  }

  .locale-selector-col :global(.locale-selector) {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25em 0.5em;
  }

  .archive-footer-bottom {
    max-width: 1100px;
    margin: 0.8rem auto 0;
    padding-top: 0.6rem;
    border-top: 1px solid rgba(255, 255, 255, 0.25);
    text-align: center;
  }

  .archive-switch-btn {
    background: none;
    border: none;
    color: #fff;
    font-size: 0.78em;
    text-decoration: underline;
    text-underline-position: under;
    cursor: pointer;
    padding: 0;
    font-family: inherit;
    box-shadow: none;
  }

  .archive-switch-btn:hover {
    color: #fff;
    box-shadow: none;
  }

  @media (max-width: 600px) {
    .archive-footer-columns {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  @media (max-width: 380px) {
    .archive-footer-columns {
      grid-template-columns: 1fr;
    }
  }
</style>
