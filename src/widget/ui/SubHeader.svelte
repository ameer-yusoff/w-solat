<script>
  import { ICON } from '../../lib/icons.js';
  let { title, icon, onback, children } = $props();
</script>

<header class="sub">
  <button class="back" onclick={(e) => { e.stopPropagation(); onback?.(); }} title="Kembali">{@html ICON.back}</button>
  {#if icon}<span class="ic">{@html ICON[icon]}</span>{/if}
  <span class="title">{title}</span>
  {@render children?.()}
</header>
<div class="rule"><span></span></div>

<style>
  .sub { display: flex; align-items: center; gap: 7px; padding: 10px 12px 8px; }
  .back {
    width: 24px;
    height: 24px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    background: rgba(255, 255, 255, 0.06);
    color: var(--ink-dim);
    transition: background 0.2s, color 0.2s;
  }
  .back:hover { background: rgba(233, 200, 115, 0.15); color: var(--gold); }
  .back :global(svg) { width: 14px; height: 14px; }
  .ic { width: 14px; height: 14px; color: var(--gold); display: grid; }
  .ic :global(svg) { width: 100%; height: 100%; }
  .title { font-family: var(--font-display); font-weight: 600; font-size: 12.5px; flex: 1; }

  /* Garis pemisah dengan permata di tengah */
  .rule { position: relative; height: 1px; margin: 0 12px 4px; background: linear-gradient(90deg, transparent, var(--gold-line), transparent); }
  .rule span {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 5px;
    height: 5px;
    transform: translate(-50%, -50%) rotate(45deg);
    background: var(--gold);
    box-shadow: 0 0 6px var(--gold);
  }
</style>
