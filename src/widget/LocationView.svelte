<script>
  import groups from '../data/groups.json';
  import { ICON } from '../lib/icons.js';
  import { detectDeviceZone } from '../lib/location.js';
  import { settings, saveSettings } from '../lib/store.svelte.js';
  import SubHeader from './ui/SubHeader.svelte';
  import Select from './ui/Select.svelte';

  let { onback, onsaved } = $props();

  const current = groups.find((g) => g.zone === settings.zone);
  let state = $state(current?.state ?? '');
  let zone = $state(current?.zone ?? '');
  let query = $state('');
  let detecting = $state(false);
  let detectError = $state('');

  const stateOptions = [...new Set(groups.map((g) => g.state))].sort().map((s) => ({ value: s, label: s }));
  const areaOptions = $derived(
    groups.filter((g) => g.state === state).map((g) => ({ value: g.zone, label: `${g.area.join(', ')} (${g.zone})` })),
  );

  const results = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (q.length < 2) return [];
    const out = [];
    for (const g of groups) {
      for (const a of g.area) {
        if (a.toLowerCase().includes(q) || g.zone.toLowerCase().includes(q)) {
          out.push({ zone: g.zone, state: g.state, area: a });
        }
      }
    }
    return out.slice(0, 30);
  });

  let chosenName = $state('');

  function choose(r) {
    state = r.state;
    zone = r.zone;
    chosenName = r.area;
    query = '';
  }

  function save(e) {
    e.stopPropagation();
    const g = groups.find((x) => x.zone === zone);
    if (!g) return;
    settings.zone = g.zone;
    settings.locationName = chosenName && g.area.includes(chosenName) ? chosenName : g.area[0];
    saveSettings();
    onsaved?.();
  }

  async function autoDetect(e) {
    e.stopPropagation();
    detecting = true;
    detectError = '';
    const res = await detectDeviceZone();
    detecting = false;
    if (!res) {
      detectError = 'Gagal mengesan lokasi automatik.';
      return;
    }
    state = res.state;
    zone = res.zone;
    chosenName = res.locationName;
    settings.zone = res.zone;
    settings.locationName = res.locationName;
    saveSettings();
    onsaved?.();
  }
</script>

<SubHeader title="Tetapan Lokasi" icon="pin" {onback} />

<div class="body">
  <button class="auto-btn" disabled={detecting} onclick={autoDetect}>
    <span class="bic">{@html ICON.pin}</span>
    {detecting ? 'Mengesan lokasi…' : 'Kesan Lokasi Automatik'}
  </button>
  {#if detectError}<p class="err">{detectError}</p>{/if}

  <div class="search">
    <span class="sic">{@html ICON.search}</span>
    <input placeholder="Cari kawasan… (cth: Shah Alam)" bind:value={query} onclick={(e) => e.stopPropagation()} />
  </div>

  {#if results.length}
    <div class="results">
      {#each results as r (r.zone + r.area)}
        <button onclick={(e) => { e.stopPropagation(); choose(r); }}>
          <span class="ra">{r.area}</span>
          <span class="rz">{r.state} · {r.zone}</span>
        </button>
      {/each}
    </div>
  {:else if query.trim().length >= 2}
    <p class="none">Tiada padanan.</p>
  {/if}

  <label class="lbl" for="state">Negeri</label>
  <Select options={stateOptions} bind:value={state} placeholder="— Pilih Negeri —" onchange={() => { zone = ''; chosenName = ''; }} />

  <label class="lbl" for="area">Kawasan / Zon</label>
  <Select
    options={areaOptions}
    bind:value={zone}
    disabled={!state}
    placeholder={state ? '— Pilih Kawasan —' : 'Pilih negeri dahulu'}
    onchange={() => (chosenName = '')}
  />

  <button class="primary" disabled={!zone} onclick={save}>
    <span class="bic">{@html ICON.save}</span> Simpan Lokasi
  </button>
</div>

<style>
  .body { padding: 6px 12px 12px; display: flex; flex-direction: column; gap: 6px; overflow-y: auto; }
  .auto-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 7px 10px;
    border-radius: 10px;
    font-size: 11px;
    font-weight: 600;
    color: var(--gold-soft);
    background: rgba(233, 200, 115, 0.12);
    border: 1px solid rgba(233, 200, 115, 0.3);
    transition: background 0.2s, transform 0.2s var(--spring);
  }
  .auto-btn:hover:not(:disabled) { background: rgba(233, 200, 115, 0.2); transform: translateY(-1px); }
  .auto-btn:disabled { opacity: 0.6; cursor: default; }
  .err { font-size: 10px; color: #f87171; text-align: center; }
  .search {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 9px;
    border-radius: 10px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(233, 200, 115, 0.18);
  }
  .search:focus-within { border-color: var(--gold); }
  .sic { width: 12px; height: 12px; color: var(--gold); display: grid; flex: none; }
  .sic :global(svg) { width: 100%; height: 100%; }
  input {
    flex: 1;
    min-width: 0;
    background: none;
    border: none;
    outline: none;
    padding: 7px 0;
    font-size: 11.5px;
    user-select: text;
  }
  input::placeholder { color: var(--ink-faint); }

  .results {
    max-height: 150px;
    overflow-y: auto;
    border-radius: 10px;
    background: rgba(0, 0, 0, 0.3);
    padding: 3px;
  }
  .results button {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 5px 8px;
    border-radius: 7px;
    text-align: left;
  }
  .results button:hover { background: rgba(233, 200, 115, 0.12); }
  .ra { font-size: 11.5px; }
  .rz { font-size: 9.5px; color: var(--ink-faint); }
  .none { font-size: 10.5px; color: var(--ink-faint); text-align: center; padding: 4px; }

  .lbl { font-size: 9.5px; letter-spacing: 1.2px; text-transform: uppercase; color: var(--ink-faint); margin-top: 4px; }

  .primary {
    margin-top: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 8px;
    border-radius: 10px;
    font-size: 11.5px;
    font-weight: 650;
    color: #1a1406;
    background: linear-gradient(135deg, var(--gold-soft), var(--gold) 50%, var(--gold-deep));
    box-shadow: 0 4px 14px rgba(185, 141, 60, 0.35);
    transition: transform 0.2s var(--spring), filter 0.2s;
  }
  .primary:hover:not(:disabled) { transform: translateY(-1px); filter: brightness(1.06); }
  .primary:disabled { opacity: 0.4; cursor: default; }
  .bic { width: 13px; height: 13px; display: grid; }
  .bic :global(svg) { width: 100%; height: 100%; }
</style>
