<script>
  import { ICON } from '../lib/icons.js';
  import { MAIN_PRAYERS } from '../lib/prayer.js';
  import { SOUNDS } from '../lib/sound.js';
  import { settings, saveSettings } from '../lib/store.svelte.js';
  import SubHeader from './ui/SubHeader.svelte';
  import Select from './ui/Select.svelte';
  import Toggle from './ui/Toggle.svelte';
  import Pills from './ui/Pills.svelte';

  let { sound, onback, ontest } = $props();

  let temp = $state(structuredClone($state.snapshot(settings.notif)));
  let previewing = $state(false);

  const offsetOptions = [
    { value: 0, label: 'Tiada' },
    { value: 5, label: '5 min' },
    { value: 10, label: '10 min' },
    { value: 15, label: '15 min' },
  ];
  const durationOptions = [
    { value: 30, label: '30 saat' },
    { value: 60, label: '1 minit' },
    { value: 0, label: 'Kekal' },
  ];
  const positionOptions = [
    { value: 'widget', label: 'Atas widget' },
    { value: 'corner', label: 'Penjuru kanan' },
  ];

  function preview() {
    sound.stop();
    previewing = sound.play(temp.sound, temp.volume, {
      onEnded: () => (previewing = false),
    });
  }

  function togglePreview(e) {
    e.stopPropagation();
    if (previewing) {
      sound.stop();
      previewing = false;
    } else {
      preview();
    }
  }

  function save(e) {
    e.stopPropagation();
    Object.assign(settings.notif, $state.snapshot(temp));
    saveSettings();
    sound.stop();
    previewing = false;
    onback?.();
  }

  function test(e) {
    e.stopPropagation();
    sound.stop();
    previewing = false;
    ontest?.($state.snapshot(temp));
  }
</script>

<SubHeader title="Tetapan Notifikasi" icon="bell" onback={() => { sound.stop(); previewing = false; onback?.(); }} />

<div class="body">
  <div class="row card">
    <div>
      <div class="rt">Notifikasi</div>
      <div class="rs">Paparan khas W-Solat + nada</div>
    </div>
    <Toggle bind:checked={temp.enabled} label="Notifikasi" />
  </div>

  <div class="group" class:dim={!temp.enabled}>
    <span class="lbl">Nada</span>
    <div class="sound-row">
      <div class="grow"><Select options={SOUNDS} bind:value={temp.sound} onchange={preview} /></div>
      <button class="mini-btn" title={previewing ? 'Henti' : 'Dengar'} onclick={togglePreview}>
        {@html previewing ? ICON.mute : ICON.play}
      </button>
    </div>

    <span class="lbl">Volum <b class="num">{Math.round(temp.volume * 100)}%</b></span>
    <input
      class="range"
      type="range"
      min="0"
      max="1"
      step="0.05"
      bind:value={temp.volume}
      style:--p="{temp.volume * 100}%"
      onchange={preview}
      onclick={(e) => e.stopPropagation()}
    />

    <span class="lbl">Peringatan awal</span>
    <Pills options={offsetOptions} bind:value={temp.offsetMinutes} />

    <span class="lbl">Waktu solat</span>
    <div class="chips">
      {#each MAIN_PRAYERS as p (p.key)}
        <button
          class:on={temp.prayers[p.key]}
          title={p.name}
          onclick={(e) => { e.stopPropagation(); temp.prayers[p.key] = !temp.prayers[p.key]; }}
        >
          <span class="ar">{p.ar}</span>
          <span class="cn">{p.name}</span>
        </button>
      {/each}
    </div>

    <span class="lbl">Tempoh paparan</span>
    <Pills options={durationOptions} bind:value={temp.duration} />

    <span class="lbl">Kedudukan</span>
    <Pills options={positionOptions} bind:value={temp.position} />

    <div class="row">
      <div class="rs2">Papar ketika skrin penuh</div>
      <Toggle bind:checked={temp.overFullscreen} label="Skrin penuh" />
    </div>
  </div>

  <div class="actions">
    <button class="secondary" onclick={test}><span class="bic">{@html ICON.play}</span> Uji</button>
    <button class="primary" onclick={save}><span class="bic">{@html ICON.save}</span> Simpan</button>
  </div>
</div>

<style>
  .body { padding: 6px 12px 12px; display: flex; flex-direction: column; gap: 6px; overflow-y: auto; min-height: 0; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .card { padding: 8px 10px; border-radius: 11px; background: rgba(0, 0, 0, 0.25); border: 1px solid rgba(233, 200, 115, 0.14); }
  .rt { font-size: 12px; font-weight: 650; }
  .rs { font-size: 9.5px; color: var(--ink-faint); }
  .rs2 { font-size: 11px; color: var(--ink-dim); }
  .group { display: flex; flex-direction: column; gap: 6px; transition: opacity 0.25s; }
  .group.dim { opacity: 0.4; pointer-events: none; }
  .lbl { font-size: 9.5px; letter-spacing: 1.2px; text-transform: uppercase; color: var(--ink-faint); margin-top: 3px; display: flex; justify-content: space-between; }
  .lbl b { color: var(--gold); font-weight: 600; letter-spacing: 0; }

  .sound-row { display: flex; gap: 5px; }
  .grow { flex: 1; min-width: 0; }
  .mini-btn {
    width: 32px;
    flex: none;
    border-radius: 10px;
    display: grid;
    place-items: center;
    color: var(--gold);
    background: rgba(0, 0, 0, 0.28);
    border: 1px solid rgba(233, 200, 115, 0.18);
  }
  .mini-btn:hover { border-color: var(--gold); }
  .mini-btn :global(svg) { width: 12px; height: 12px; }

  .range {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 4px;
    border-radius: 4px;
    background: linear-gradient(90deg, var(--gold) var(--p), rgba(255, 255, 255, 0.12) var(--p));
    outline: none;
  }
  .range::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--gold-soft);
    border: 2px solid var(--gold-deep);
    box-shadow: 0 0 8px rgba(233, 200, 115, 0.5);
    cursor: pointer;
  }

  .chips { display: grid; grid-template-columns: repeat(5, 1fr); gap: 4px; }
  .chips button {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 4px 0 3px;
    border-radius: 9px;
    background: rgba(0, 0, 0, 0.28);
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: var(--ink-faint);
    transition: all 0.2s;
  }
  .chips .ar { font-size: 15px; line-height: 1.1; }
  .chips .cn { font-size: 8.5px; }
  .chips button.on {
    color: var(--gold-soft);
    border-color: rgba(233, 200, 115, 0.55);
    background: linear-gradient(180deg, rgba(233, 200, 115, 0.22), rgba(233, 200, 115, 0.06));
    box-shadow: 0 0 10px rgba(233, 200, 115, 0.15);
  }

  .actions { display: flex; gap: 6px; margin-top: 8px; }
  .actions button {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    padding: 8px;
    border-radius: 10px;
    font-size: 11.5px;
    font-weight: 650;
    transition: transform 0.2s var(--spring), filter 0.2s;
  }
  .actions button:hover { transform: translateY(-1px); }
  .secondary { background: rgba(255, 255, 255, 0.07); border: 1px solid rgba(233, 200, 115, 0.25); color: var(--gold-soft); }
  .primary {
    color: #1a1406;
    background: linear-gradient(135deg, var(--gold-soft), var(--gold) 50%, var(--gold-deep));
    box-shadow: 0 4px 14px rgba(185, 141, 60, 0.35);
  }
  .bic { width: 12px; height: 12px; display: grid; }
  .bic :global(svg) { width: 100%; height: 100%; }
</style>
