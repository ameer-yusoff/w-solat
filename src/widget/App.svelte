<script>
  import { onMount } from 'svelte';
  import { backOut } from 'svelte/easing';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { settings, saveSettings, runtime, isFirstRun } from '../lib/store.svelte.js';
  import {
    loadDay, cachedDay, localDateKey, getNext, getPrev, getPeriod,
    MAIN_PRAYERS, timeToDate, fmtTime, fmtHijri, fmtCountdown,
  } from '../lib/prayer.js';
  import { detectDeviceZone } from '../lib/location.js';
  import { ICON } from '../lib/icons.js';
  import { SoundEngine } from '../lib/sound.js';
  import MiniBar from './MiniBar.svelte';
  import Schedule from './Schedule.svelte';
  import LocationView from './LocationView.svelte';
  import NotifView from './NotifView.svelte';
  import SettingsView from './SettingsView.svelte';

  const sound = new SoundEngine();
  let expanded = $state(false);
  let view = $state('schedule');
  let barAtTop = $state(false);
  let editingPosition = $state(false);
  let dragging = $state(false);
  let resizing = $state(false);
  let savingPosition = $state(false);
  let positionError = $state('');
  let tutorialStep = $state(0); // 0 = tiada, 1 = grab, 2 = confirm
  let tutorialTimers = [];
  let dragRequest = Promise.resolve();
  let movePending = false;
  let resizeStart = null;
  let sizeSyncTimer = null;

  const MINI_MIN_W = 240;
  const MINI_MAX_W = 320;
  const MINI_MIN_H = 36;
  const MINI_MAX_H = 56;

  const next = $derived(getNext(runtime.entry, runtime.now, runtime.tomorrow));
  const prev = $derived(getPrev(runtime.entry, runtime.now));
  const period = $derived(getPeriod(runtime.entry, runtime.now));
  const remaining = $derived(next ? next.date - runtime.now : 0);
  const progress = $derived(
    next && prev ? Math.min(1, Math.max(0, (runtime.now - prev.date) / (next.date - prev.date))) : 0,
  );
  const hijri = $derived(fmtHijri(runtime.entry?.hijri, runtime.now));

  // ---------------- Data ----------------
  let loading = false;
  async function refresh(force = false) {
    if (loading) return;
    loading = true;
    const today = new Date();
    if (!runtime.entry) runtime.status = 'loading';
    try {
      const { entry, offline } = await loadDay(settings.zone, today, { force });
      runtime.entry = entry;
      runtime.entryDate = localDateKey(today);
      const tm = new Date(today);
      tm.setDate(tm.getDate() + 1);
      runtime.tomorrow = cachedDay(settings.zone, tm);
      runtime.status = entry ? (offline ? 'offline' : 'ok') : 'error';
    } catch (e) {
      console.error(e);
      runtime.status = 'error';
    } finally {
      loading = false;
      updateTray();
    }
  }

  // ---------------- Notifikasi ----------------
  const fired = new Set();

  function buildPayload(p, t, kind, n = settings.notif) {
    return {
      kind,
      key: p.key,
      name: p.name,
      ar: p.ar,
      time: fmtTime(t, settings.format24),
      location: settings.locationName,
      minutes: n.offsetMinutes,
      sound: n.sound,
      volume: n.volume,
      duration: n.duration,
      position: n.position,
      overFullscreen: n.overFullscreen,
      hijri,
    };
  }

  async function sendNotification(payload) {
    if (expanded) closePanel();
    try {
      await invoke('show_notification', { payload });
    } catch (e) {
      console.error('Gagal papar notifikasi:', e);
    }
  }

  function checkNotif(now) {
    const n = settings.notif;
    if (!n.enabled || !runtime.entry || runtime.entryDate !== localDateKey(now)) return;
    for (const p of MAIN_PRAYERS) {
      if (!n.prayers[p.key]) continue;
      const t = runtime.entry[p.key];
      if (!t) continue;
      const at = timeToDate(t, now).getTime();
      const checks = [[at, 'adhan']];
      if (n.offsetMinutes > 0) checks.push([at - n.offsetMinutes * 60000, 'reminder']);
      for (const [target, kind] of checks) {
        const diff = now.getTime() - target;
        const key = `${runtime.entryDate}|${p.key}|${kind}|${n.offsetMinutes}`;
        // Tetingkap 90s supaya tidak terlepas walaupun pemasa diperlahankan
        if (diff >= 0 && diff <= 90000 && !fired.has(key)) {
          fired.add(key);
          sendNotification(buildPayload(p, t, kind));
        }
      }
    }
  }

  function testNotification(n = settings.notif) {
    const p = next || { ...MAIN_PRAYERS[0], time: '06:00:00' };
    sendNotification(buildPayload(p, p.time, 'test', n));
  }

  // ---------------- Tray ----------------
  function updateTray() {
    if (!next) return;
    const left = fmtCountdown(next.date - new Date()).slice(0, 5);
    invoke('set_tray_tooltip', {
      text: `W-Solat · ${next.name} ${fmtTime(next.time, settings.format24)} (${left} lagi)\n${settings.locationName}`,
    }).catch(() => {});
  }

  // ---------------- Panel ----------------
  function openPanel() {
    view = 'schedule';
    expanded = true;
    invoke('set_expanded', { expanded: true }).catch(() => {});
  }
  function closePanel() {
    expanded = false;
    invoke('set_expanded', { expanded: false }).catch(() => {});
  }
  const toggle = () => {
    if (!editingPosition) expanded ? closePanel() : openPanel();
  };

  function clearTutorialTimers() {
    tutorialTimers.forEach((t) => clearTimeout(t));
    tutorialTimers = [];
  }

  function startTutorial() {
    clearTutorialTimers();
    tutorialStep = 1;
    tutorialTimers.push(
      setTimeout(() => {
        if (editingPosition) tutorialStep = 2;
      }, 3000),
      setTimeout(() => {
        tutorialStep = 0;
      }, 6000),
    );
  }

  async function editPosition() {
    try {
      await invoke('edit_widget_position');
      editingPosition = true;
      closePanel();
      startTutorial();
    } catch (err) {
      console.error('Gagal buka mod ubah kedudukan:', err);
    }
  }

  function sendDrag(phase) {
    dragRequest = dragRequest
      .then(() => invoke('drag_widget', { phase }))
      .catch((err) => console.error('Gagal alihkan widget:', err));
    return dragRequest;
  }

  function startDrag(event) {
    if (event.button !== 0 || dragging || savingPosition) return;
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    dragging = true;
    sendDrag('start');
  }

  function moveDrag() {
    if (!dragging || movePending) return;
    movePending = true;
    sendDrag('move').finally(() => (movePending = false));
  }

  function endDrag(phase) {
    if (!dragging) return;
    dragging = false;
    sendDrag(phase);
  }

  function syncMiniBarSize() {
    clearTimeout(sizeSyncTimer);
    sizeSyncTimer = setTimeout(() => {
      invoke('set_minibar_size', {
        width: settings.miniWidth,
        height: settings.miniHeight,
      }).catch(() => {});
    }, 16);
  }

  function startResize(axis, event) {
    if (event.button !== 0 || dragging || savingPosition) return;
    event.preventDefault();
    event.stopPropagation();
    event.currentTarget.setPointerCapture(event.pointerId);
    resizing = true;
    resizeStart = {
      axis,
      x: event.clientX,
      y: event.clientY,
      w: settings.miniWidth,
      h: settings.miniHeight,
    };
  }

  function moveResize(event) {
    if (!resizing || !resizeStart) return;
    const dx = event.clientX - resizeStart.x;
    const dy = event.clientY - resizeStart.y;
    if (resizeStart.axis.includes('w')) {
      settings.miniWidth = Math.round(Math.min(MINI_MAX_W, Math.max(MINI_MIN_W, resizeStart.w + dx * 2)));
    }
    if (resizeStart.axis.includes('h')) {
      const deltaY = barAtTop ? dy : -dy;
      settings.miniHeight = Math.round(Math.min(MINI_MAX_H, Math.max(MINI_MIN_H, resizeStart.h + deltaY)));
    }
    syncMiniBarSize();
  }

  function endResize() {
    if (!resizing) return;
    resizing = false;
    resizeStart = null;
    invoke('set_minibar_size', {
      width: settings.miniWidth,
      height: settings.miniHeight,
    }).catch(() => {});
  }

  async function savePosition() {
    if (dragging || resizing || savingPosition) return;
    savingPosition = true;
    positionError = '';
    clearTutorialTimers();
    tutorialStep = 0;
    try {
      await dragRequest;
      saveSettings();
      await invoke('save_widget_position');
      editingPosition = false;
    } catch (err) {
      positionError = 'Gagal simpan kedudukan';
      console.error(positionError, err);
    } finally {
      savingPosition = false;
    }
  }

  function panelIn(node) {
    return {
      duration: 340,
      easing: backOut,
      css: (t, u) => `opacity:${Math.min(1, t * 1.6)};transform:translateY(${u * 18}px) scale(${0.92 + 0.08 * t})`,
    };
  }
  function panelOut(node) {
    return { duration: 160, css: (t, u) => `opacity:${t};transform:translateY(${u * 10}px) scale(${0.97 + 0.03 * t})` };
  }
  function viewIn(node) {
    return { duration: 260, easing: backOut, css: (t, u) => `opacity:${t};transform:translateX(${u * 16}px)` };
  }

  // ---------------- Kitaran hayat ----------------
  onMount(() => {
    const unlisten = [];
    listen('widget-edge', ({ payload }) => (barAtTop = payload))
      .then((u) => {
        unlisten.push(u);
        return invoke('widget_at_top');
      })
      .then((value) => (barAtTop = value))
      .catch(() => {});
    listen('close-panel', () => expanded && closePanel()).then((u) => unlisten.push(u));
    listen('toggle-panel', () => toggle()).then((u) => unlisten.push(u));
    listen('refresh', () => refresh(true)).then((u) => unlisten.push(u));
    listen('test-notif', () => testNotification()).then((u) => unlisten.push(u));
    invoke('set_minibar_size', {
      width: settings.miniWidth,
      height: settings.miniHeight,
    }).catch(() => {});

    (async () => {
      try {
        if (settings.autostart) {
          settings.autostart = await invoke('set_autostart', { enabled: true });
        } else {
          settings.autostart = await invoke('get_autostart');
        }
      } catch { /* mod dev / tiada kebenaran */ }

      if (isFirstRun) {
        const detected = await detectDeviceZone();
        if (detected) {
          settings.zone = detected.zone;
          settings.locationName = detected.locationName;
          saveSettings();
          refresh(true);
          return;
        }
        saveSettings();
      }
    })();

    refresh();
    let lastMinute = -1;
    const timer = setInterval(() => {
      const now = new Date();
      runtime.now = now;
      if (runtime.entryDate && localDateKey(now) !== runtime.entryDate) refresh();
      checkNotif(now);
      const minute = now.getMinutes();
      if (minute !== lastMinute) {
        lastMinute = minute;
        updateTray();
        if (runtime.status !== 'ok' && minute % 2 === 0) refresh();
      }
    }, 1000);

    return () => {
      clearInterval(timer);
      clearTutorialTimers();
      clearTimeout(sizeSyncTimer);
      unlisten.forEach((u) => u());
    };
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="root period-{period}"
  class:barAtTop
  style:--bar-offset="{settings.miniHeight + 10}px"
  onclick={(e) => e.target === e.currentTarget && expanded && closePanel()}
>
  {#if expanded}
    <section class="panel gold-frame" in:panelIn out:panelOut>
      <div class="panel-sky"></div>
      <div class="panel-pattern"></div>
      {#key view}
        <div class="view" in:viewIn>
          {#if view === 'schedule'}
            <Schedule {next} {progress} {remaining} {hijri} onnav={(v) => (view = v)} onmove={editPosition} />
          {:else if view === 'location'}
            <LocationView
              onback={() => (view = 'schedule')}
              onsaved={() => { view = 'schedule'; refresh(true); }}
            />
          {:else if view === 'notif'}
            <NotifView {sound} onback={() => (view = 'schedule')} ontest={testNotification} />
          {:else}
            <SettingsView onback={() => (view = 'schedule')} onrefresh={() => refresh(true)} />
          {/if}
        </div>
      {/key}
    </section>
  {/if}

  <div class="bar-wrap">
    {#if editingPosition}
      <div class="control-slot grip-slot">
        {#if tutorialStep === 1}
          <div class="tutorial-bubble left" role="status">
            <span>Seret di sini untuk ubah posisi</span>
          </div>
        {/if}
        <button
          class="move-control grip"
          class:dragging
          aria-label="Seret widget"
          title="Klik dan seret untuk mengalihkan widget"
          onpointerdown={startDrag}
          onpointermove={moveDrag}
          onpointerup={() => endDrag('end')}
          onpointercancel={() => endDrag('cancel')}
          onlostpointercapture={() => endDrag('cancel')}
        >{@html ICON.grip}</button>
      </div>
    {/if}

    <div class="mini-stage" class:editing={editingPosition}>
      {#if editingPosition && resizing}
        <div class="size-badge">{settings.miniWidth} × {settings.miniHeight}</div>
      {/if}
      <MiniBar
        {next}
        {progress}
        {remaining}
        {period}
        {expanded}
        editing={editingPosition}
        status={runtime.status}
        width={settings.miniWidth}
        height={settings.miniHeight}
        onclick={toggle}
      />
      {#if editingPosition}
        <div
          class="resize-handle resize-y"
          title="Seret untuk ubah tinggi"
          onpointerdown={(e) => startResize('h', e)}
          onpointermove={moveResize}
          onpointerup={endResize}
          onpointercancel={endResize}
        ></div>
        <div
          class="resize-handle resize-w"
          title="Seret untuk ubah lebar"
          onpointerdown={(e) => startResize('w', e)}
          onpointermove={moveResize}
          onpointerup={endResize}
          onpointercancel={endResize}
        ></div>
        <div
          class="resize-handle resize-wh"
          title="Seret untuk ubah lebar & tinggi"
          onpointerdown={(e) => startResize('wh', e)}
          onpointermove={moveResize}
          onpointerup={endResize}
          onpointercancel={endResize}
        ></div>
      {/if}
    </div>

    {#if editingPosition}
      <div class="control-slot confirm-slot">
        {#if tutorialStep === 2}
          <div class="tutorial-bubble right" role="status">
            <span>Klik sini untuk sahkan</span>
          </div>
        {/if}
        <button class="move-control confirm" disabled={dragging || resizing || savingPosition} onclick={savePosition}
          title={positionError || 'Simpan kedudukan & saiz widget'}
          aria-label={positionError || 'Simpan kedudukan & saiz widget'}>{@html ICON.check}</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .root {
    position: relative;
    width: 100vw;
    height: 100vh;
  }

  .bar-wrap {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    pointer-events: none;
  }
  .bar-wrap :global(button) { pointer-events: auto; }
  .barAtTop .bar-wrap { bottom: auto; top: 4px; }
  .control-slot { position: relative; flex: none; pointer-events: none; }
  .control-slot button { pointer-events: auto; }
  .move-control {
    width: 30px;
    height: 36px;
    flex: none;
    border: 1px solid var(--gold-line);
    border-radius: 9px;
    display: grid;
    place-items: center;
    background: var(--sky-1);
    color: var(--gold-soft);
  }
  .move-control :global(svg) { width: 16px; height: 16px; pointer-events: none; }
  .grip { cursor: grab; touch-action: none; }
  .grip.dragging { cursor: grabbing; }
  .confirm { background: var(--gold-soft); color: #1a1406; }
  .confirm:disabled { opacity: 0.65; }

  .tutorial-bubble {
    position: absolute;
    bottom: calc(100% + 9px);
    padding: 6px 10px;
    border-radius: 9px;
    background: rgba(12, 16, 24, 0.96);
    border: 1px solid var(--gold-line);
    color: var(--ink);
    font-size: 11px;
    font-weight: 500;
    line-height: 1.25;
    white-space: nowrap;
    box-shadow: 0 8px 20px rgba(0, 0, 0, 0.55);
    pointer-events: none;
    z-index: 20;
    animation: bubbleIn 0.22s var(--spring);
  }
  .tutorial-bubble.left { left: 0; }
  .tutorial-bubble.right { right: 0; }
  .tutorial-bubble::after {
    content: '';
    position: absolute;
    top: 100%;
    width: 8px;
    height: 8px;
    background: rgba(12, 16, 24, 0.96);
    border-right: 1px solid var(--gold-line);
    border-bottom: 1px solid var(--gold-line);
    transform: translateY(-4px) rotate(45deg);
  }
  .tutorial-bubble.left::after { left: 11px; }
  .tutorial-bubble.right::after { right: 11px; }

  .barAtTop .tutorial-bubble {
    bottom: auto;
    top: calc(100% + 9px);
  }
  .barAtTop .tutorial-bubble::after {
    top: auto;
    bottom: 100%;
    border-right: none;
    border-bottom: none;
    border-left: 1px solid var(--gold-line);
    border-top: 1px solid var(--gold-line);
    transform: translateY(4px) rotate(45deg);
  }

  @keyframes bubbleIn {
    from { opacity: 0; transform: translateY(4px) scale(0.95); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }

  .mini-stage {
    position: relative;
    flex: none;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
  }
  .mini-stage.editing {
    outline: 1px dashed rgba(233, 200, 115, 0.55);
    outline-offset: 2px;
    border-radius: 13px;
  }
  .size-badge {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
    padding: 3px 8px;
    border-radius: 6px;
    background: rgba(12, 16, 24, 0.92);
    border: 1px solid var(--gold-line);
    color: var(--gold-soft);
    font-size: 10.5px;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    pointer-events: none;
  }
  .barAtTop .size-badge { bottom: auto; top: calc(100% + 8px); }

  .resize-handle {
    position: absolute;
    pointer-events: auto;
    touch-action: none;
    z-index: 5;
  }
  .resize-w {
    top: 4px;
    bottom: 4px;
    right: -4px;
    width: 8px;
    cursor: ew-resize;
  }
  .resize-y {
    left: 8px;
    right: 8px;
    top: -4px;
    height: 8px;
    cursor: ns-resize;
  }
  .barAtTop .resize-y { top: auto; bottom: -4px; }
  .resize-wh {
    right: -5px;
    top: -5px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--gold-soft);
    border: 1.5px solid #1a1406;
    cursor: nesw-resize;
  }
  .barAtTop .resize-wh { top: auto; bottom: -5px; cursor: nwse-resize; }

  .barAtTop .panel { bottom: auto; top: var(--bar-offset, 50px); transform-origin: 30% 0%; }

  .panel {
    position: absolute;
    left: 0;
    right: 0;
    width: 316px;
    margin: 0 auto;
    bottom: var(--bar-offset, 50px);
    max-height: calc(100vh - var(--bar-offset, 50px) - 4px);
    border-radius: 18px;
    overflow: hidden;
    transform-origin: 30% 100%;
    background: var(--sky-1);
    box-shadow:
      0 18px 40px -8px rgba(0, 0, 0, 0.65),
      0 0 0 1px rgba(0, 0, 0, 0.4);
    display: flex;
    flex-direction: column;
  }
  .panel-sky {
    position: absolute;
    inset: 0;
    background:
      radial-gradient(120% 70% at 50% -10%, var(--sky-3) 0%, transparent 60%),
      linear-gradient(180deg, var(--sky-2) 0%, var(--sky-1) 55%, #03070a 100%);
    opacity: 0.95;
    transition: background 1.2s ease;
  }
  .panel-pattern {
    position: absolute;
    inset: 0;
    background-image: var(--pattern);
    background-size: 36px 36px;
    opacity: 0.07;
    mask-image: linear-gradient(180deg, #000 0%, transparent 70%);
  }
  .view {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    max-height: calc(100vh - 54px);
  }
</style>
