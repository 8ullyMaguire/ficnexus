<script lang="ts">
  /**
   * Trust badge — a compact chip showing a member's trust level (0-6) with
   * one star per level. Color escalates with trust so regulars are visible
   * at a glance in comments and forum threads.
   */
  interface Props {
    level: number;
    levelName: string;
    compact?: boolean;
  }

  let { level, levelName, compact = true }: Props = $props();

  const TRUST_COLORS: Record<number, string> = {
    0: 'bg-gray-300 text-gray-800',
    1: 'bg-sky-100 text-sky-800',
    2: 'bg-emerald-100 text-emerald-800',
    3: 'bg-indigo-100 text-indigo-800',
    4: 'bg-amber-100 text-amber-800',
    5: 'bg-purple-100 text-purple-800',
    6: 'bg-pink-100 text-pink-800',
  };
  const color = TRUST_COLORS[level] ?? TRUST_COLORS[0];
  const label = compact
    ? `${levelName} (TL${level})`
    : `${levelName} — Trust Level ${level}`;

  // Star count mirrors the trust level (TL0 = 0 stars, TL6 = 6).
  const stars = Array.from({ length: Math.max(0, Math.min(6, level)) }, () => 1);
</script>

<span
  class="inline-flex items-center gap-1 rounded px-2.5 py-0.5 text-xs font-semibold {color}"
  title={label}
>
  {#each stars as _ (1)}
    <svg class="h-3 w-3 fill-current" viewBox="0 0 20 20" aria-hidden="true">
      <path
        d="M10 1.6l2.59 5.22L18 7.42l-5 4.88 1.18 6.92L10 15.67l-6.18 3.25L5 12.3 1 7.42l5.41-1.6z"
      />
    </svg>
  {/each}
  {label}
</span>











