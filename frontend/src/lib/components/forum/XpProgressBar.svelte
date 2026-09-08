<script lang="ts">
  /** XP progress bar showing current level progress. */
  let { percent = 0, level = 0 }: { percent: number; level: number } = $props();
</script>

<div class="xp-bar-container">
  <div class="xp-bar-header">
    <span class="xp-level">Lv.{level}</span>
    <span class="xp-percent">{percent}%</span>
  </div>
  <div class="xp-bar-track">
    <div class="xp-bar-fill" style="width: {Math.min(100, Math.max(0, percent))}%"></div>
  </div>
</div>

<style>
  .xp-bar-container {
    width: 100%;
  }
  .xp-bar-header {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    margin-bottom: 0.2rem;
  }
  .xp-level {
    font-weight: 600;
    color: #58a6ff;
  }
  .xp-percent {
    color: #8b949e;
  }
  .xp-bar-track {
    width: 100%;
    height: 6px;
    background: #21262d;
    border-radius: 3px;
    overflow: hidden;
  }
  .xp-bar-fill {
    height: 100%;
    background: linear-gradient(90deg, #1f6feb, #58a6ff);
    border-radius: 3px;
    transition: width 0.3s ease;
  }
</style>
