<script>
  import { ICON } from '../lib/icons.js';
  import { fmtTime, fmtCountdown } from '../lib/prayer.js';
  import { settings } from '../lib/store.svelte.js';

  let {
    next,
    progress = 0,
    remaining = 0,
    period,
    expanded = false,
    editing = false,
    status = 'loading',
    width = 246,
    height = 40,
    onclick,
  } = $props();

  const C = 2 * Math.PI * 15.5;
  const urgent = $derived(next && remaining > 0 && remaining <= 10 * 60000);

  let flip = $state(false);
  $effect(() => {
    const t = setInterval(() => (flip = !flip), 4000);
    return () => clearInterval(t);
  });
  const showCountdown = $derived(urgent || flip);

  const title = $derived(
    status === 'error' && !next ? 'Luar Talian' : status === 'loading' && !next ? 'Memuatkan' : next?.name ?? '—',
  );
</script>

<button
  class="mini gold-frame"
  class:expanded
  class:editing
  class:urgent
  style:--bar-w="{width}px"
  style:--bar-h="{height}px"
  {onclick}
  title="W-Solat — klik untuk jadual"
>
  <span class="glow"></span>
  <span class="pattern"></span>

  <span class="ring">
    <svg viewBox="0 0 36 36" aria-hidden="true">
      <circle class="track" cx="18" cy="18" r="15.5" />
      <circle
        class="bar"
        cx="18"
        cy="18"
        r="15.5"
        stroke-dasharray={C}
        stroke-dashoffset={C * (1 - progress)}
      />
    </svg>
    <span class="ic">{@html ICON[period] ?? ICON.isyak}</span>
  </span>

  <span class="name">{title}</span>
  <span class="ticker">
    <span class="track" style:transform="translateY({showCountdown && next ? '-50%' : '0'})">
      <span class="num">{next ? fmtTime(next.time, settings.format24) : '--:--'}</span>
      <span class="num cd">-{fmtCountdown(remaining)}</span>
    </span>
  </span>
  <span class="ar mini-ar">{next?.ar ?? ''}</span>

  {#if status === 'offline' || status === 'error'}
    <span class="offline" title="Data luar talian (cache)">{@html ICON.wifiOff}</span>
  {/if}
</button>

<style>
  .mini {
    --bar-w: 246px;
    --bar-h: 40px;
    --ring-s: calc(var(--bar-h) * 0.775);
    --ticker-h: clamp(14px, calc(var(--bar-h) * 0.375), 18px);
    position: relative;
    flex: none;
    width: var(--bar-w);
    height: var(--bar-h);
    border-radius: 12px;
    display: grid;
    grid-template-columns: var(--ring-s) minmax(58px, 1.25fr) minmax(56px, 1.15fr) minmax(40px, 0.9fr) auto;
    align-items: center;
    column-gap: clamp(6px, 2.5%, 12px);
    padding: 0 10px;
    overflow: hidden;
    background:
      linear-gradient(105deg, color-mix(in srgb, var(--sky-2) 85%, transparent), color-mix(in srgb, var(--sky-1) 92%, transparent) 70%),
      rgba(12, 14, 18, 0.88);
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.45), inset 0 1px 0 rgba(255, 255, 255, 0.05);
    transition: transform 0.25s var(--spring), box-shadow 0.25s ease, background 1.2s ease;
    text-align: left;
  }
  .mini.editing { transition: box-shadow 0.25s ease, background 1.2s ease; }
  .mini:hover {
    transform: translateY(-1px);
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(233, 200, 115, 0.12);
  }
  .mini.editing:hover, .mini.editing:active { transform: none; }
  .mini:active { transform: scale(0.97); transition-duration: 0.08s; }
  .mini.expanded { box-shadow: 0 0 0 1px rgba(233, 200, 115, 0.25), 0 6px 18px rgba(0, 0, 0, 0.55); }

  .pattern {
    position: absolute;
    inset: 0;
    background-image: var(--pattern);
    background-size: 22px 22px;
    opacity: 0.08;
    mask-image: linear-gradient(90deg, transparent 20%, #000 100%);
    pointer-events: none;
  }
  .glow {
    position: absolute;
    left: -10px;
    top: 50%;
    width: 60px;
    height: 60px;
    transform: translateY(-50%);
    background: radial-gradient(circle, color-mix(in srgb, var(--accent) 35%, transparent), transparent 70%);
    pointer-events: none;
  }

  .ring {
    position: relative;
    flex: none;
    width: var(--ring-s);
    height: var(--ring-s);
    display: grid;
    place-items: center;
  }
  .ring > svg { position: absolute; inset: 0; transform: rotate(-90deg); }
  .ring circle { fill: none; stroke-width: 2.2; }
  .ring .track { stroke: rgba(255, 255, 255, 0.1); }
  .ring .bar {
    stroke: var(--gold);
    stroke-linecap: round;
    transition: stroke-dashoffset 1s linear;
    filter: drop-shadow(0 0 2px rgba(233, 200, 115, 0.6));
  }
  .ic { width: calc(var(--ring-s) * 0.48); height: calc(var(--ring-s) * 0.48); color: var(--accent); display: grid; }
  .ic :global(svg) { width: 100%; height: 100%; }

  .name, .ticker, .mini-ar { position: relative; text-align: center; white-space: nowrap; }
  .name { font-family: var(--font-display); font-weight: 650; font-size: clamp(11.5px, calc(var(--bar-h) * 0.31), 15px); }
  .mini-ar { font-size: clamp(15px, calc(var(--bar-h) * 0.425), 21px); line-height: 1.3; color: var(--gold); opacity: 0.92; }
  .ticker { height: var(--ticker-h); overflow: hidden; display: block; }
  .ticker .track { display: flex; flex-direction: column; transition: transform 0.5s var(--ease); }
  .ticker .num { height: var(--ticker-h); line-height: var(--ticker-h); font-size: clamp(10.5px, calc(var(--bar-h) * 0.285), 14px); color: var(--ink-dim); }
  .ticker .cd { color: var(--gold-soft); }

  .offline { position: relative; width: 12px; height: 12px; color: #f59e0b; display: grid; }
  .offline :global(svg) { width: 100%; height: 100%; }

  .urgent .ring .bar { stroke: var(--accent); animation: pulse 1.6s ease-in-out infinite; }
  .urgent .ticker .cd { color: var(--accent); font-weight: 600; }
  @keyframes pulse {
    50% { filter: drop-shadow(0 0 6px var(--accent)); }
  }
</style>
