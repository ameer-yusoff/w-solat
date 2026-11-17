<script>
  import { ICON, STAR8_PATH } from '../lib/icons.js';
  import { PRAYERS, fmtTime, fmtCountdown, fmtGregorian, timeToDate } from '../lib/prayer.js';
  import { settings, saveSettings, runtime } from '../lib/store.svelte.js';

  let { next, progress = 0, remaining = 0, hijri = '', onnav, onmove } = $props();

  const rows = $derived(
    runtime.entry
      ? PRAYERS.filter((p) => runtime.entry[p.key]).map((p) => {
          const date = timeToDate(runtime.entry[p.key], runtime.now);
          return { ...p, time: runtime.entry[p.key], passed: date <= runtime.now, active: next?.key === p.key };
        })
      : [],
  );

  function toggleBell(e, key) {
    e.stopPropagation();
    settings.notif.prayers[key] = !settings.notif.prayers[key];
    saveSettings();
  }
  const nav = (e, v) => { e.stopPropagation(); onnav?.(v); };
</script>

<header class="head">
  <button class="loc" onclick={(e) => nav(e, 'location')} title="Tukar kawasan">
    <span class="ic">{@html ICON.pin}</span>
    <span class="loc-name">{settings.locationName}</span>
    <span class="ic chev">{@html ICON.chevron}</span>
  </button>
  <button class="icon-btn" onclick={onmove} title="Ubah kedudukan & saiz widget" aria-label="Ubah kedudukan & saiz widget">{@html ICON.hand}</button>
  <button class="icon-btn" class:off={!settings.notif.enabled} onclick={(e) => nav(e, 'notif')} title="Tetapan notifikasi">
    {@html settings.notif.enabled ? ICON.bell : ICON.bellOff}
  </button>
  <button class="icon-btn" onclick={(e) => nav(e, 'settings')} title="Tetapan">{@html ICON.gear}</button>
</header>

<div class="dates">
  <span class="hijri">{hijri}</span>
  <span class="dot">◆</span>
  <span class="greg">{fmtGregorian(runtime.now)}</span>
</div>

<!-- Mihrab: kad utama berbentuk gerbang -->
<div class="mihrab gold-frame">
  <svg class="arch-line" viewBox="0 0 220 30" preserveAspectRatio="none" aria-hidden="true">
    <path d="M6 30 C6 12 60 4 110 2 C160 4 214 12 214 30" />
  </svg>
  {#if next}
    <div class="kicker">Menuju waktu</div>
    <div class="ar big">{next.ar}</div>
    <div class="who"><b>{next.name}</b> · <span class="num">{fmtTime(next.time, settings.format24)}</span></div>
    <div class="count num">−{fmtCountdown(remaining)}</div>
    <div class="bar"><span style:width="{progress * 100}%"></span></div>
  {:else}
    <div class="kicker">{runtime.status === 'error' ? 'Tiada sambungan' : 'Memuatkan data…'}</div>
    <div class="ar big">الصلاة</div>
    <div class="who">{runtime.status === 'error' ? 'Cuba semula sebentar lagi' : 'JAKIM e-Solat'}</div>
  {/if}
</div>

<ul class="list">
  {#each rows as r (r.key)}
    <li class:active={r.active} class:passed={r.passed && !r.active} class:minor={r.minor}>
      <svg class="star" viewBox="0 0 24 24" aria-hidden="true"><path d={STAR8_PATH} /></svg>
      <span class="pname">{r.name}</span>
      <span class="ar par">{r.ar}</span>
      <span class="ptime num">{fmtTime(r.time, settings.format24)}</span>
      {#if !r.minor}
        <button
          class="bell"
          class:on={settings.notif.prayers[r.key] && settings.notif.enabled}
          onclick={(e) => toggleBell(e, r.key)}
          title={settings.notif.prayers[r.key] ? 'Matikan notifikasi' : 'Hidupkan notifikasi'}
        >{@html settings.notif.prayers[r.key] ? ICON.bell : ICON.bellOff}</button>
      {:else}
        <span class="bell-spacer"></span>
      {/if}
    </li>
  {:else}
    <li class="empty">{runtime.status === 'error' ? 'Data tidak dapat dimuatkan.' : 'Memuatkan data…'}</li>
  {/each}
</ul>

<style>
  .head { display: flex; align-items: center; gap: 4px; padding: 9px 9px 2px 9px; }
  .loc {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 7px;
    border-radius: 9px;
    transition: background 0.2s;
  }
  .loc:hover { background: rgba(233, 200, 115, 0.1); }
  .loc-name { font-size: 11.5px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .ic { width: 13px; height: 13px; flex: none; display: grid; color: var(--gold); }
  .ic.chev { color: var(--ink-faint); width: 11px; height: 11px; }
  .ic :global(svg) { width: 100%; height: 100%; }
  .icon-btn {
    width: 26px;
    height: 26px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    color: var(--ink-dim);
    transition: background 0.2s, color 0.2s;
  }
  .icon-btn:hover { background: rgba(233, 200, 115, 0.12); color: var(--gold); }
  .icon-btn.off { color: #f87171; }
  .icon-btn :global(svg) { width: 14px; height: 14px; }

  .dates {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 6px;
    padding: 0 10px 7px;
    font-size: 10px;
    color: var(--ink-dim);
    white-space: nowrap;
  }
  .hijri { color: var(--gold-soft); font-weight: 600; }
  .dot { font-size: 5px; color: var(--gold); }

  .mihrab {
    position: relative;
    margin: 0 10px;
    padding: 18px 12px 11px;
    border-radius: 110px 110px 14px 14px / 34px 34px 14px 14px;
    text-align: center;
    background:
      radial-gradient(90% 80% at 50% 0%, color-mix(in srgb, var(--accent) 22%, transparent), transparent 70%),
      rgba(0, 0, 0, 0.28);
    overflow: hidden;
  }
  .mihrab::after {
    content: '';
    position: absolute;
    inset: 0;
    background-image: var(--pattern);
    background-size: 26px 26px;
    opacity: 0.06;
    pointer-events: none;
  }
  .arch-line {
    position: absolute;
    top: 5px;
    left: 8px;
    right: 8px;
    width: calc(100% - 16px);
    height: 18px;
    fill: none;
    stroke: var(--gold);
    stroke-width: 0.8;
    opacity: 0.45;
  }
  .kicker { font-size: 9.5px; letter-spacing: 1.6px; text-transform: uppercase; color: var(--ink-faint); }
  .big {
    font-size: 34px;
    line-height: 1.05;
    color: var(--gold);
    text-shadow: 0 0 18px rgba(233, 200, 115, 0.35);
    margin: 1px 0 1px;
  }
  .who { font-size: 11px; color: var(--ink-dim); }
  .who b { color: var(--ink); font-weight: 650; }
  .count {
    margin-top: 3px;
    font-size: 21px;
    font-weight: 600;
    letter-spacing: 0.5px;
    color: var(--gold-soft);
  }
  .bar {
    position: relative;
    height: 3px;
    margin: 7px 14px 0;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.08);
    overflow: hidden;
  }
  .bar span {
    position: absolute;
    inset: 0 auto 0 0;
    border-radius: 3px;
    background: linear-gradient(90deg, var(--gold-deep), var(--gold-soft));
    box-shadow: 0 0 8px var(--gold);
    transition: width 1s linear;
  }

  .list { list-style: none; padding: 8px 8px 9px; display: flex; flex-direction: column; gap: 1px; }
  li {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 26px;
    padding: 0 4px 0 8px;
    border-radius: 9px;
    font-size: 11.5px;
    transition: background 0.3s;
  }
  li:hover { background: rgba(255, 255, 255, 0.04); }
  .star { width: 8px; height: 8px; flex: none; fill: none; stroke: var(--gold); stroke-width: 2; opacity: 0.5; }
  .pname { flex: 1; font-weight: 550; }
  .par { font-size: 14px; color: var(--ink-faint); line-height: 1; }
  .ptime { width: 58px; text-align: right; color: var(--ink-dim); font-size: 11px; }
  .bell, .bell-spacer { width: 20px; height: 20px; flex: none; }
  .bell { display: grid; place-items: center; border-radius: 6px; color: var(--ink-faint); opacity: 0.6; transition: all 0.2s; }
  .bell:hover { opacity: 1; background: rgba(233, 200, 115, 0.12); }
  .bell.on { color: var(--gold); opacity: 0.85; }
  .bell :global(svg) { width: 11px; height: 11px; }

  li.minor { height: 22px; font-size: 10.5px; }
  li.minor .pname, li.minor .ptime { color: var(--ink-faint); font-weight: 400; }
  li.minor .star { opacity: 0.25; }
  li.passed .pname, li.passed .ptime { color: var(--ink-faint); }
  li.passed .star { opacity: 0.25; }

  li.active {
    background: linear-gradient(90deg, rgba(233, 200, 115, 0.2), rgba(233, 200, 115, 0.04));
    box-shadow: inset 0 0 0 1px rgba(233, 200, 115, 0.3);
  }
  li.active .star { fill: var(--gold); opacity: 1; filter: drop-shadow(0 0 4px var(--gold)); animation: spin 12s linear infinite; }
  li.active .pname { color: var(--gold-soft); }
  li.active .par { color: var(--gold); }
  li.active .ptime { color: var(--ink); font-weight: 600; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .empty { justify-content: center; color: var(--ink-faint); font-size: 11px; }
</style>
