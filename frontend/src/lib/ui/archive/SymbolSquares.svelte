<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { ratingSquare, categorySquare, warningSquare, statusSquare } from './symbols';
  import SymbolsGuideModal from './SymbolsGuideModal.svelte';

  interface Props {
    rating?: string | null;
    categories?: string[] | null;
    warnings?: string[] | null;
    status?: string | null;
  }

  let { rating = null, categories = null, warnings = null, status = null }: Props = $props();

  let showModal = $state(false);

  function openModal() { showModal = true; }
  function closeModal() { showModal = false; }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      openModal();
    }
    if (e.key === 'Escape' && showModal) closeModal();
  }

  const ratingGlyph = $derived(ratingSquare(rating));
  const categoryGlyph = $derived(categorySquare(categories));
  const warningGlyph = $derived(warningSquare(warnings));
  const statusGlyph = $derived(statusSquare(status));

  const tooltip = $derived(['R: ' + ratingGlyph, 'C: ' + categoryGlyph, 'W: ' + warningGlyph, 'S: ' + statusGlyph].join(' \u00b7 '));
</script>

<svelte:window onkeydown={(e) => { if (e.key === 'Escape' && showModal) closeModal(); }} />

{#if showModal}
  <SymbolsGuideModal close={closeModal} />
{/if}
<div
  class="symbol-squares"
  role="button"
  tabindex="0"
  title={tooltip}
  aria-label={t('symbols.guideAria')}
  onclick={openModal}
  onkeydown={onKeydown}
>
  <span class="symbol-square symbol-rating" title={t('symbols.rating')}>{ratingGlyph}</span>
  <span class="symbol-square symbol-category" title={t('symbols.category')}>{categoryGlyph}</span>
  <span class="symbol-square symbol-warning" title={t('symbols.warnings')}>{warningGlyph}</span>
  <span class="symbol-square symbol-status" title={t('symbols.status')}>{statusGlyph}</span>
</div>

<style>
  .symbol-squares {
    display: inline-flex;
    gap: 3px;
    vertical-align: middle;
    cursor: help;
    user-select: none;
  }
  .symbol-square {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    min-width: 18px;
    font-size: 11px;
    font-weight: 700;
    line-height: 1;
    text-align: center;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-text, #2a2a2a);
    border-radius: 0;
  }
  .symbol-rating { font-size: 10px; color: var(--archive-link, #990000); }
  .symbol-category { font-size: 10px; }
  .symbol-warning { font-size: 11px; color: #b00; }
  .symbol-status { font-size: 11px; color: #070; }
</style>
