<script>
  import { invoke } from '@tauri-apps/api/core';
  import { ICON, MOSQUE_PATH } from '../lib/icons.js';
  import { settings, saveSettings, runtime } from '../lib/store.svelte.js';
  import SubHeader from './ui/SubHeader.svelte';
  import Toggle from './ui/Toggle.svelte';

  let { onback, onrefresh } = $props();
  let busy = $state(false);

  async function setAutostart(on) {
    try {
      settings.autostart = await invoke('set_autostart', { enabled: on });
    } catch (e) {
      console.error(e);
      settings.autostart = !on;
    }
    saveSettings();
  }

  async function refresh(e) {
    e.stopPropagation();
    busy = true;
    await onrefresh?.();
    setTimeout(() => (busy = false), 600);
  }

  const statusText = $derived(
    { ok: 'Dalam talian', offline: 'Luar talian (cache)', error: 'Tiada data', loading: 'Memuatkan…' }[runtime.status],
  );
</script>

<SubHeader title="Tetapan" icon="gear" {onback} />

<div class="body">
  <div class="row card">
    <div>
      <div class="rt">Mula bersama Windows</div>
      <div class="rs">Buka widget secara automatik</div>
    </div>
    <Toggle bind:checked={settings.autostart} onchange={setAutostart} label="Autostart" />
  </div>

  <div class="row card">
    <div>
      <div class="rt">Format 24 jam</div>
      <div class="rs">Contoh: 13:15 berbanding 1:15 PM</div>
    </div>
    <Toggle bind:checked={settings.format24} onchange={saveSettings} label="24 jam" />
  </div>

  <div class="row card">
    <div>
      <div class="rt">Data waktu solat</div>
      <div class="rs">
        <span class="st st-{runtime.status}"></span>{statusText} · Zon {settings.zone}
      </div>
    </div>
    <button class="mini-btn" class:spin={busy} onclick={refresh} title="Muat semula">{@html ICON.refresh}</button>
  </div>

  <div class="about">
    <svg viewBox="0 0 200 80" aria-hidden="true"><path d={MOSQUE_PATH} /></svg>
    <div class="ar">بِسْمِ اللهِ الرَّحْمٰنِ الرَّحِيْمِ</div>
    <div class="ver">W-Solat 2.0 · Tauri v2</div>
    <div class="src">Sumber data: JAKIM e-Solat</div>
  </div>
</div>

<style>
  .body { padding: 6px 12px 12px; display: flex; flex-direction: column; gap: 6px; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .card { padding: 8px 10px; border-radius: 11px; background: rgba(0, 0, 0, 0.25); border: 1px solid rgba(233, 200, 115, 0.14); }
  .rt { font-size: 11.5px; font-weight: 650; }
  .rs { font-size: 9.5px; color: var(--ink-faint); display: flex; align-items: center; gap: 4px; }
  .st { width: 6px; height: 6px; border-radius: 50%; background: #9ca3af; }
  .st-ok { background: #34d399; box-shadow: 0 0 6px #34d399; }
  .st-offline { background: #f59e0b; }
  .st-error { background: #f87171; }

  .mini-btn {
    width: 28px;
    height: 28px;
    flex: none;
    border-radius: 9px;
    display: grid;
    place-items: center;
    color: var(--gold);
    background: rgba(0, 0, 0, 0.28);
    border: 1px solid rgba(233, 200, 115, 0.2);
  }
  .mini-btn :global(svg) { width: 13px; height: 13px; }
  .spin :global(svg) { animation: spin 0.6s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .about { margin-top: 8px; text-align: center; display: flex; flex-direction: column; align-items: center; gap: 2px; }
  .about svg { width: 110px; height: 44px; fill: var(--gold); opacity: 0.22; }
  .about .ar { font-size: 17px; color: var(--gold); margin-top: 2px; }
  .ver { font-size: 10.5px; color: var(--ink-dim); font-weight: 600; }
  .src { font-size: 9.5px; color: var(--ink-faint); }
</style>
