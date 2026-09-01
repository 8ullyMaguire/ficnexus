// Translate-PageButton: collects untranslated visible items via the
// pending-registry in $lib/api/translate (TranslatedText registers its key
// there when a fetch resolves to null) and enqueues them for the machine
// tier. AO3 .action button look to match the archive skin.
<script lang="ts">
  import { targetReadingLocale } from '$lib/stores/readingLocale.svelte';
  import { enqueueTranslations, takePendingItems } from '$lib/api/translate';
  import { t } from '$lib/i18n/index.svelte';

  let busy = $state(false);
  let msg = $state('');
  let locale = $derived(targetReadingLocale());

  async function enqueuePage() {
    if (busy || !locale) return;
    busy = true;
    msg = '';
    try {
      const items = takePendingItems(locale);
      if (items.length === 0) {
        msg = t('translate.nothingToQueue');
      } else {
        const queued = await enqueueTranslations(items, locale);
        msg = queued > 0
          ? t('translate.queuedToast', { count: queued })
          : t('translate.queueFailed');
      }
    } catch {
      msg = t('translate.queueFailed');
    } finally {
      busy = false;
    }
  }
</script>

{#if locale}
  <div class="tpb">
    <button class="tpb-btn" disabled={busy} onclick={enqueuePage}
      title={t('translate.pageButtonTitle')}>
      {#if busy}…{:else}🌐{/if} {t('translate.pageButton')}
    </button>
    {#if msg}<span class="tpb-msg">{msg}</span>{/if}
  </div>
{/if}

<style>
  .tpb { display: inline-flex; align-items: center; gap: .5em; margin: .3em 0; }
  .tpb-btn {
    background: #eee; background-image: linear-gradient(#fff 2%, #ddd 95%, #bbb 100%);
    color: #444; border: 1px solid #bbb; border-bottom: 1px solid #aaa; border-radius: .25em;
    padding: .2em .7em; cursor: pointer; font: inherit; font-size: .85em;
  }
  .tpb-btn:hover { color: #900; border-top-color: #999; border-left-color: #999; box-shadow: inset 2px 2px 2px #bbb; }
  .tpb-msg { font-size: .8em; color: #666; font-style: italic; }
</style>
