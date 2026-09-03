<script lang="ts">
  // 5-star click-to-rate widget (feedback rework).
  // - `value`: current rating (0 = nothing selected).
  // - `hoverValue`: transient preview while the pointer is over a star.
  // - Emits a `rate` custom event with the clicked rating (1..=5) via the
  //   `onrate` callback prop so parents stay in control of the POST.
  let {
    value = $bindable(0),
    readonly = false,
    size = '1.3rem',
    onrate,
  }: {
    value?: number;
    readonly?: boolean;
    size?: string;
    onrate?: (rating: number) => void;
  } = $props();

  let hoverValue = $state(0);

  const displayValue = $derived(hoverValue > 0 ? hoverValue : value);

  function setHover(n: number) {
    if (!readonly) hoverValue = n;
  }

  function clearHover() {
    hoverValue = 0;
  }

  function click(n: number) {
    if (readonly) return;
    value = n;
    onrate?.(n);
  }
</script>

<div class="star-rating" role="group" aria-label="Star rating">
  {#each [1, 2, 3, 4, 5] as n (n)}
    <button
      type="button"
      class="star"
      class:filled={n <= displayValue}
      class:active={n <= value}
      aria-label={readonly ? `${n} star${n > 1 ? 's' : ''}` : `Rate ${n} star${n > 1 ? 's' : ''}`}
      aria-pressed={n <= value}
      disabled={readonly}
      onmouseenter={() => setHover(n)}
      onmouseleave={clearHover}
      onclick={() => click(n)}
      style={`font-size: ${size}`}
    >{n <= displayValue ? '★' : '☆'}</button>
  {/each}
</div>

<style>
  .star-rating {
    display: inline-flex;
    gap: 0.15rem;
    align-items: center;
  }
  .star {
    background: none;
    border: none;
    padding: 0 0.05rem;
    cursor: pointer;
    line-height: 1;
    color: var(--color-muted);
    transition: color 0.12s, transform 0.12s;
  }
  .star:hover:not(:disabled) {
    transform: scale(1.15);
  }
  .star.filled {
    color: #f5b301; /* warm star gold */
  }
  .star.active {
    color: #f5b301;
  }
  .star:disabled {
    cursor: default;
  }
</style>
