<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  interface Props { close: () => void; }
  let { close }: Props = $props();
  let dialogEl: HTMLDivElement | undefined = $state(undefined);

  function onBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) close();
  }
  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') { e.preventDefault(); close(); }
    if (e.key === 'Tab' && dialogEl) {
      const focusable = dialogEl.querySelectorAll<HTMLElement>('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])');
      if (focusable.length === 0) return;
      const first = focusable[0]; const last = focusable[focusable.length - 1];
      if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
      else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="symbols-guide-backdrop" onclick={onBackdropClick} role="presentation">
  <div
    class="symbols-guide"
    role="dialog"
    aria-modal="true"
    aria-label={t('symbols.guideTitle')}
    bind:this={dialogEl}
    tabindex="-1"
    onkeydown={onKeydown}
  >
    <header class="symbols-guide-header">
      <h2 class="symbols-guide-title">{t('symbols.guideTitle')}</h2>
      <button class="symbols-guide-close" type="button" aria-label={t('symbols.guideClose')} onclick={close}>✕</button>
    </header>
    <section class="symbols-guide-body">
      <h3>{t('symbols.legendRating')}</h3>
      <ul>
        <li>G — {t('symbols.legendGeneral')}</li>
        <li>T — {t('symbols.legendTeen')}</li>
        <li>M — {t('symbols.legendMature')}</li>
        <li>E — {t('symbols.legendExplicit')}</li>
      </ul>
      <h3>{t('symbols.legendCategory')}</h3>
      <ul>
        <li>Gen — {t('symbols.legendGen')}</li>
        <li>F/M — {t('symbols.legendFm')}</li>
        <li>F/F — {t('symbols.legendFf')}</li>
        <li>M/M — {t('symbols.legendMm')}</li>
        <li>🌐 — {t('symbols.legendNoCategory')}</li>
      </ul>
      <h3>{t('symbols.legendWarning')}</h3>
      <ul>
        <li>! — {t('symbols.legendWarningFlag')}</li>
        <li>✓ — {t('symbols.legendNoWarnings')}</li>
      </ul>
      <h3>{t('symbols.legendStatus')}</h3>
      <ul>
        <li>✓ — {t('symbols.legendComplete')}</li>
        <li>✗ — {t('symbols.legendIncomplete')}</li>
      </ul>
      <p class="symbols-guide-note">{t('symbols.guideNote')}</p>
    </section>
  </div>
</div>

<style>
  .symbols-guide-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000; }
  .symbols-guide { background: var(--archive-bg, #fff); border: 1px solid var(--archive-border, #ddd); max-width: 520px; width: 90vw; max-height: 80vh; overflow-y: auto; font-family: Georgia, 'Times New Roman', serif; color: var(--archive-text, #2a2a2a); }
  .symbols-guide-header { display: flex; align-items: center; justify-content: space-between; background: var(--archive-bg-raised, #f5f5f5); border-bottom: 1px solid var(--archive-border, #ddd); padding: 0.5em 0.8em; }
  .symbols-guide-title { font-size: 1em; font-weight: 700; margin: 0; color: var(--archive-link, #990000); }
  .symbols-guide-close { background: none; border: 1px solid var(--archive-border, #ddd); cursor: pointer; font-size: 0.9em; line-height: 1; padding: 0.2em 0.5em; }
  .symbols-guide-body { padding: 0.8em 1em; font-size: 0.9em; line-height: 1.5; }
  .symbols-guide-body h3 { font-size: 0.95em; font-weight: 700; margin: 0.8em 0 0.3em; color: var(--archive-link, #990000); }
  .symbols-guide-body ul { list-style: none; margin: 0.3em 0 0.5em 0; padding: 0; }
  .symbols-guide-body li { margin: 0.15em 0; }
  .symbols-guide-note { font-size: 0.82em; color: var(--archive-muted, #666); margin-top: 0.8em; }
</style>
