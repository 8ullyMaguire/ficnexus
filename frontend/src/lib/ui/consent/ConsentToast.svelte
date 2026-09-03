<script lang="ts">
  // Cookie-consent toast — first-visit gate for non-essential cookies.
  // Approve → 1-year `fh_consent` cookie; reject → sessionStorage only.
  // Shown inside ArchiveLayout (fixed bottom bar) until a choice is made.
  import { consentToastVisible, grantConsent, rejectConsent } from '$lib/stores/consent.svelte';

  const visible = $derived(consentToastVisible());
</script>

{#if visible}
  <div class="consent-toast" role="dialog" aria-label="Cookie consent" aria-live="polite">
    <p class="consent-text">
      FicHub uses cookies for essential features. With your permission we also
      store a device cookie that keeps your anonymous library (bookmarks and
      follows) between visits — no account needed. See the
      <a href="/privacy">privacy policy</a>.
    </p>
    <div class="consent-actions">
      <button class="consent-accept" onclick={grantConsent}>Accept</button>
      <button class="consent-reject" onclick={rejectConsent}>Reject</button>
    </div>
  </div>
{/if}

<style>
  .consent-toast {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 1000;
    display: flex;
    gap: 1rem;
    align-items: center;
    justify-content: center;
    flex-wrap: wrap;
    padding: 0.75rem 1rem;
    background: #222;
    color: #eee;
    font-family: Verdana, sans-serif;
    font-size: 0.8125rem;
    box-shadow: 0 -2px 6px rgba(0, 0, 0, 0.4);
  }

  .consent-text {
    margin: 0;
    max-width: 46rem;
    line-height: 1.4;
  }

  .consent-text a {
    color: #bbb;
    text-decoration: underline;
  }

  .consent-actions {
    display: flex;
    gap: 0.5rem;
  }

  .consent-actions button {
    padding: 0.3rem 0.9rem;
    border-radius: 0.25em;
    border: 1px solid #888;
    cursor: pointer;
    font: inherit;
  }

  .consent-accept {
    background: #990000;
    border-color: #990000;
    color: #fff;
  }

  .consent-accept:hover {
    background: #b30000;
  }

  .consent-reject {
    background: transparent;
    color: #eee;
  }

  .consent-reject:hover {
    border-color: #ccc;
  }
</style>
