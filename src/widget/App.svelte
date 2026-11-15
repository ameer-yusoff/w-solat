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
  import { SoundEngine } from '../lib/sound.js';
  import MiniBar from './MiniBar.svelte';
  import Schedule from './Schedule.svelte';
  import LocationView from './LocationView.svelte';
  import NotifView from './NotifView.svelte';
  import SettingsView from './SettingsView.svelte';

  const sound = new SoundEngine();
  let expanded = $state(false);
  let view = $state('schedule');

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
  const toggle = () => (expanded ? closePanel() : openPanel());

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
    listen('close-panel', () => expanded && closePanel()).then((u) => unlisten.push(u));
    listen('toggle-panel', () => toggle()).then((u) => unlisten.push(u));
    listen('refresh', () => refresh(true)).then((u) => unlisten.push(u));
    listen('test-notif', () => testNotification()).then((u) => unlisten.push(u));

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
      unlisten.forEach((u) => u());
    };
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="root period-{period}" onclick={(e) => e.target === e.currentTarget && expanded && closePanel()}>
  {#if expanded}
    <section class="panel gold-frame" in:panelIn out:panelOut>
      <div class="panel-sky"></div>
      <div class="panel-pattern"></div>
      {#key view}
        <div class="view" in:viewIn>
          {#if view === 'schedule'}
            <Schedule {next} {progress} {remaining} {hijri} onnav={(v) => (view = v)} />
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

  <MiniBar {next} {progress} {remaining} {period} {expanded} status={runtime.status} onclick={toggle} />
</div>

<style>
  .root {
    position: relative;
    width: 100vw;
    height: 100vh;
  }

  .panel {
    position: absolute;
    left: 2px;
    right: 2px;
    bottom: 50px;
    max-height: calc(100vh - 54px);
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
