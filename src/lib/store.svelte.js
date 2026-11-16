// Tetapan pengguna (disimpan dalam localStorage WebView2)
const KEY = 'wsolat.settings';

export const DEFAULTS = {
  zone: 'WLY01',
  locationName: 'Kuala Lumpur, Putrajaya',
  format24: false,
  autostart: true,
  notif: {
    enabled: true,
    sound: 'takbir',
    volume: 0.8,
    offsetMinutes: 0, // peringatan awal (0 = tiada)
    prayers: { fajr: true, dhuhr: true, asr: true, maghrib: true, isha: true },
    position: 'widget', // 'widget' | 'corner'
    duration: 60, // saat; 0 = kekal sehingga ditutup
    overFullscreen: true,
  },
};

function merge(base, extra) {
  if (!extra || typeof extra !== 'object') return base;
  for (const k of Object.keys(base)) {
    if (!(k in extra)) continue;
    const b = base[k];
    base[k] = b && typeof b === 'object' && !Array.isArray(b) ? merge(b, extra[k]) : extra[k];
  }
  return base;
}

function load() {
  try {
    const saved = merge(structuredClone(DEFAULTS), JSON.parse(localStorage.getItem(KEY) || 'null'));
    if (saved.notif?.sound === 'azan') saved.notif.sound = 'azan-bayati';
    return saved;
  } catch {
    return structuredClone(DEFAULTS);
  }
}

export const isFirstRun = !localStorage.getItem(KEY);
export const settings = $state(load());

export function saveSettings() {
  localStorage.setItem(KEY, JSON.stringify($state.snapshot(settings)));
}

/** Keadaan masa jalan (tidak disimpan). */
export const runtime = $state({
  now: new Date(),
  entry: null,
  tomorrow: null,
  entryDate: '',
  status: 'loading', // loading | ok | offline | error
});
