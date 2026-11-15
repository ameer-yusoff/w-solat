<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { ICON, MOSQUE_PATH } from '../lib/icons.js';
  import { PRAYER_PERIOD } from '../lib/prayer.js';
  import { SoundEngine } from '../lib/sound.js';

  const sound = new SoundEngine();

  let data = $state(null);
  let visible = $state(false);
  let playing = $state(false);
  let left = $state(1); // baki masa (0..1)
  let hovering = $state(false);

  let timer = null;
  let soundTimer = null;
  let snoozeTimer = null;
  let elapsed = 0;

  const period = $derived(data ? PRAYER_PERIOD[data.key] ?? 'isyak' : 'isyak');

  const copy = $derived.by(() => {
    if (!data) return {};
    const isFajr = data.key === 'fajr';
    switch (data.kind) {
      case 'reminder':
        return {
          kicker: `${data.minutes} minit lagi`,
          title: `Solat ${data.name}`,
          phraseAr: 'إِنَّ الصَّلَاةَ كَانَتْ عَلَى الْمُؤْمِنِينَ كِتَابًا مَوْقُوتًا',
          phrase: 'Sesungguhnya solat itu fardu yang ditentukan waktunya (An-Nisa’: 103)',
        };
      case 'snooze':
        return {
          kicker: 'Peringatan semula',
          title: `Sudah solat ${data.name}?`,
          phraseAr: 'حَيَّ عَلَى الْفَلَاحِ',
          phrase: 'Marilah menuju kejayaan',
        };
      case 'test':
        return {
          kicker: 'Ujian notifikasi',
          title: `Solat ${data.name}`,
          phraseAr: 'حَيَّ عَلَى الصَّلَاةِ',
          phrase: 'Marilah menunaikan solat',
        };
      default:
        return {
          kicker: 'Telah masuk waktu',
          title: `Solat ${data.name}`,
          phraseAr: isFajr ? 'الصَّلَاةُ خَيْرٌ مِنَ النَّوْمِ' : 'حَيَّ عَلَى الصَّلَاةِ',
          phrase: isFajr ? 'Solat itu lebih baik daripada tidur' : 'Marilah menunaikan solat',
        };
    }
  });

  function startAutoClose(ms) {
    clearInterval(timer);
    elapsed = 0;
    left = 1;
    if (!ms) return;
    timer = setInterval(() => {
      if (hovering) return;
      elapsed += 100;
      left = Math.max(0, 1 - elapsed / ms);
      if (elapsed >= ms) dismiss();
    }, 100);
  }

  function playSound() {
    clearInterval(timer);
    left = 1;
    if (data.sound === 'silent') {
      playing = false;
      startAutoClose((data.duration || 10) * 1000);
      return;
    }

    playing = sound.play(data.sound, data.volume ?? 0.8, {
      prayerKey: data.key,
      onEnded: () => {
        playing = false;
        // Tutup automatik 3 saat selepas bunyi notifikasi tamat
        startAutoClose(3000);
      },
    });

    if (!playing) {
      startAutoClose(3000);
    }
  }

  function silence(e) {
    e?.stopPropagation();
    sound.stop();
    playing = false;
    startAutoClose(3000);
  }

  async function dismiss() {
    clearInterval(timer);
    sound.stop();
    playing = false;
    visible = false;
    setTimeout(() => {
      if (!visible) invoke('hide_notification').catch(() => {});
    }, 320);
  }

  function snooze(e) {
    e.stopPropagation();
    const p = $state.snapshot(data);
    clearTimeout(snoozeTimer);
    snoozeTimer = setTimeout(() => {
      invoke('show_notification', { payload: { ...p, kind: 'snooze' } }).catch(() => {});
    }, 5 * 60000);
    dismiss();
  }

  function showAlert(payload) {
    if (!payload) return;
    data = payload;
    visible = false;
    requestAnimationFrame(() => {
      visible = true;
      playSound();
    });
  }

  onMount(() => {
    let un;
    listen('prayer-alert', (ev) => showAlert(ev.payload)).then((u) => (un = u));
    invoke('get_pending_notification')
      .then((pending) => {
        if (pending && !data) showAlert(pending);
      })
      .catch(() => {});
    return () => {
      un?.();
      clearInterval(timer);
      clearTimeout(soundTimer);
      clearTimeout(snoozeTimer);
      sound.stop();
    };
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="stage period-{period}">
  {#if data}
    <article
      class="card gold-frame"
      class:show={visible}
      onmouseenter={() => (hovering = true)}
      onmouseleave={() => (hovering = false)}
    >
      <div class="sky"></div>
      <div class="pattern"></div>
      <svg class="mosque" viewBox="0 0 200 80" aria-hidden="true"><path d={MOSQUE_PATH} /></svg>
      <div class="shine"></div>

      <header class="top">
        <span class="app">W-SOLAT</span>
        <span class="sep">•</span>
        <span class="loc">{data.location}</span>
        <button class="x" onclick={(e) => { e.stopPropagation(); dismiss(); }} title="Tutup">{@html ICON.close}</button>
      </header>

      <div class="main">
        <div class="medallion">
          <svg class="arch" viewBox="0 0 80 96" aria-hidden="true">
            <path class="outer" d="M4 94V44C4 22 20 8 40 2c20 6 36 20 36 42v50Z" />
            <path class="inner" d="M11 94V46c0-17 12-29 29-35 17 6 29 18 29 35v48" />
          </svg>
          <span class="ar mname">{data.ar}</span>
          {#if playing}
            <span class="wave"><i></i><i></i><i></i><i></i></span>
          {/if}
        </div>

        <div class="text">
          <div class="kicker">{copy.kicker}</div>
          <h1>{copy.title}</h1>
          <div class="meta">
            <span class="ic">{@html ICON.clock}</span>
            <span class="num">{data.time}</span>
            {#if data.hijri}<span class="dot">◆</span><span>{data.hijri}</span>{/if}
          </div>
          <div class="ar phrase-ar">{copy.phraseAr}</div>
          <div class="phrase">{copy.phrase}</div>
        </div>
      </div>

      <footer class="actions">
        {#if playing}
          <button class="ghost" onclick={silence}><span class="ic">{@html ICON.mute}</span>Senyap</button>
        {/if}
        {#if data.kind !== 'test'}
          <button class="ghost" onclick={snooze}><span class="ic">{@html ICON.clock}</span>5 minit</button>
        {/if}
        <button class="gold" onclick={(e) => { e.stopPropagation(); dismiss(); }}>
          <span class="ic">{@html ICON.check}</span>{data.kind === 'reminder' ? 'Baik' : 'Insya-Allah'}
        </button>
      </footer>

      {#if !playing}
        <div class="timebar"><span style:transform="scaleX({left})"></span></div>
      {/if}
    </article>
  {/if}
</div>

<style>
  .stage { width: 100vw; height: 100vh; padding: 10px; display: flex; align-items: flex-end; }

  .card {
    position: relative;
    width: 100%;
    border-radius: 20px;
    overflow: hidden;
    padding: 10px 14px 12px;
    background: var(--sky-1);
    box-shadow: 0 14px 34px -6px rgba(0, 0, 0, 0.7), 0 0 0 1px rgba(0, 0, 0, 0.45);
    opacity: 0;
    transform: translateY(26px) scale(0.94);
    transition: opacity 0.25s ease, transform 0.3s var(--ease);
  }
  .card.show {
    opacity: 1;
    transform: none;
    transition: opacity 0.35s ease, transform 0.6s var(--spring);
  }

  .sky {
    position: absolute;
    inset: 0;
    background:
      radial-gradient(70% 120% at 0% 100%, color-mix(in srgb, var(--accent) 22%, transparent), transparent 60%),
      radial-gradient(90% 90% at 100% 0%, var(--sky-3), transparent 65%),
      linear-gradient(135deg, var(--sky-2), var(--sky-1) 60%, #020407);
  }
  .pattern {
    position: absolute;
    inset: 0;
    background-image: var(--pattern);
    background-size: 34px 34px;
    opacity: 0.08;
    mask-image: linear-gradient(90deg, #000, transparent 75%);
  }
  .mosque {
    position: absolute;
    right: -6px;
    bottom: 0;
    width: 190px;
    height: 76px;
    fill: var(--gold);
    opacity: 0.09;
  }
  .shine {
    position: absolute;
    inset: 0;
    background: linear-gradient(105deg, transparent 35%, rgba(246, 227, 168, 0.18) 50%, transparent 65%);
    transform: translateX(-120%);
  }
  .show .shine { animation: shine 1.6s 0.35s var(--ease) both; }
  @keyframes shine { to { transform: translateX(120%); } }

  .top { position: relative; display: flex; align-items: center; gap: 6px; font-size: 10px; color: var(--ink-faint); }
  .brand { width: 12px; height: 12px; fill: var(--gold); filter: drop-shadow(0 0 4px var(--gold)); }
  .app { color: var(--gold); font-weight: 700; letter-spacing: 1.8px; font-size: 9.5px; }
  .sep { font-size: 8px; }
  .loc { flex: 1; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .x {
    width: 22px;
    height: 22px;
    border-radius: 7px;
    display: grid;
    place-items: center;
    color: var(--ink-faint);
    transition: background 0.2s, color 0.2s;
  }
  .x:hover { background: rgba(255, 255, 255, 0.1); color: var(--ink); }
  .x :global(svg) { width: 12px; height: 12px; }

  .main { position: relative; display: flex; gap: 13px; margin-top: 8px; align-items: stretch; }

  .medallion {
    position: relative;
    flex: none;
    width: 74px;
    height: 88px;
    display: grid;
    place-items: center;
  }
  .arch { position: absolute; inset: 0; width: 100%; height: 100%; }
  .arch .outer { fill: rgba(0, 0, 0, 0.3); stroke: var(--gold); stroke-width: 1.4; }
  .arch .inner { fill: none; stroke: var(--gold); stroke-width: 0.6; opacity: 0.5; }
  .show .arch .outer { stroke-dasharray: 260; stroke-dashoffset: 260; animation: draw 1.2s 0.15s var(--ease) forwards; }
  @keyframes draw { to { stroke-dashoffset: 0; } }
  .mname {
    position: relative;
    margin-top: 14px;
    font-size: 27px;
    color: var(--gold);
    text-shadow: 0 0 16px rgba(233, 200, 115, 0.55);
  }
  .show .mname { animation: glow 2.8s ease-in-out infinite; }
  @keyframes glow { 50% { text-shadow: 0 0 26px rgba(233, 200, 115, 0.9); } }

  .wave { position: absolute; bottom: 8px; display: flex; gap: 2px; align-items: flex-end; height: 10px; }
  .wave i { width: 2px; background: var(--gold); border-radius: 2px; animation: wave 0.9s ease-in-out infinite; }
  .wave i:nth-child(2) { animation-delay: 0.15s; }
  .wave i:nth-child(3) { animation-delay: 0.3s; }
  .wave i:nth-child(4) { animation-delay: 0.45s; }
  @keyframes wave { 0%, 100% { height: 3px; } 50% { height: 10px; } }

  .text { flex: 1; min-width: 0; display: flex; flex-direction: column; justify-content: center; }
  .kicker { font-size: 9.5px; letter-spacing: 1.6px; text-transform: uppercase; color: var(--accent); font-weight: 600; }
  h1 {
    font-family: var(--font-display);
    font-size: 20px;
    font-weight: 700;
    line-height: 1.15;
    letter-spacing: 0.2px;
    color: var(--ink);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .meta { display: flex; align-items: center; gap: 5px; font-size: 10.5px; color: var(--ink-dim); margin-top: 1px; white-space: nowrap; }
  .meta .ic { width: 11px; height: 11px; display: grid; color: var(--gold); }
  .meta .ic :global(svg) { width: 100%; height: 100%; }
  .dot { font-size: 5px; color: var(--gold); }
  .phrase-ar { margin-top: 5px; font-size: 17px; color: var(--gold-soft); line-height: 1.2; text-align: right; }
  .phrase { font-size: 9.5px; color: var(--ink-faint); font-style: italic; line-height: 1.3; text-align: right; }

  .actions { position: relative; display: flex; justify-content: flex-end; gap: 6px; margin-top: 10px; }
  .actions button {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 6px 11px;
    border-radius: 9px;
    font-size: 11px;
    font-weight: 600;
    transition: transform 0.2s var(--spring), background 0.2s, filter 0.2s;
  }
  .actions button:hover { transform: translateY(-1px); }
  .actions .ic { width: 12px; height: 12px; display: grid; }
  .actions .ic :global(svg) { width: 100%; height: 100%; }
  .ghost { background: rgba(255, 255, 255, 0.07); border: 1px solid rgba(233, 200, 115, 0.22); color: var(--ink-dim); }
  .ghost:hover { background: rgba(255, 255, 255, 0.12); color: var(--ink); }
  .gold {
    color: #1a1406;
    background: linear-gradient(135deg, var(--gold-soft), var(--gold) 50%, var(--gold-deep));
    box-shadow: 0 4px 14px rgba(185, 141, 60, 0.4);
  }
  .gold:hover { filter: brightness(1.07); }

  .timebar { position: absolute; left: 0; right: 0; bottom: 0; height: 2px; background: rgba(255, 255, 255, 0.06); }
  .timebar span {
    display: block;
    height: 100%;
    transform-origin: left;
    background: linear-gradient(90deg, var(--gold-deep), var(--gold-soft));
    transition: transform 0.1s linear;
  }
</style>
