// ==========================================================
// Logik waktu solat: API JAKIM e-Solat (melalui Rust), cache
// bulanan untuk mod luar talian, tarikh Hijri & tema mengikut waktu.
// ==========================================================
import { invoke } from '@tauri-apps/api/core';

export const PRAYERS = [
  { key: 'imsak',   name: 'Imsak',   ar: 'الإمساك', minor: true },
  { key: 'fajr',    name: 'Subuh',   ar: 'الفجر',   short: 'S' },
  { key: 'syuruk',  name: 'Syuruk',  ar: 'الشروق',  minor: true },
  { key: 'dhuhr',   name: 'Zohor',   ar: 'الظهر',   short: 'Z' },
  { key: 'asr',     name: 'Asar',    ar: 'العصر',   short: 'A' },
  { key: 'maghrib', name: 'Maghrib', ar: 'المغرب',  short: 'M' },
  { key: 'isha',    name: 'Isyak',   ar: 'العشاء',  short: 'I' },
];
export const MAIN_PRAYERS = PRAYERS.filter((p) => !p.minor);
export const byKey = Object.fromEntries(PRAYERS.map((p) => [p.key, p]));

/** Tema visual yang dipadankan dengan setiap waktu solat. */
export const PRAYER_PERIOD = { fajr: 'subuh', dhuhr: 'zohor', asr: 'asar', maghrib: 'maghrib', isha: 'isyak' };

const EN_MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
const MS_DAYS = ['Ahad', 'Isnin', 'Selasa', 'Rabu', 'Khamis', 'Jumaat', 'Sabtu'];
const MS_MONTHS = ['Jan', 'Feb', 'Mac', 'Apr', 'Mei', 'Jun', 'Jul', 'Ogo', 'Sep', 'Okt', 'Nov', 'Dis'];
const HIJRI_MONTHS = [
  'Muharram', 'Safar', 'Rabiulawal', 'Rabiulakhir', 'Jamadilawal', 'Jamadilakhir',
  'Rejab', 'Syaaban', 'Ramadan', 'Syawal', 'Zulkaedah', 'Zulhijjah',
];

const pad = (n) => String(n).padStart(2, '0');
export const localDateKey = (d) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
const esolatDate = (d) => `${pad(d.getDate())}-${EN_MONTHS[d.getMonth()]}-${d.getFullYear()}`;

export function timeToDate(t, base = new Date()) {
  const [h, m, s] = String(t).split(':').map(Number);
  const d = new Date(base);
  d.setHours(h || 0, m || 0, s || 0, 0);
  return d;
}

export function fmtTime(t, h24 = false) {
  if (!t) return '--:--';
  const [hs, m] = String(t).split(':');
  let h = parseInt(hs, 10);
  if (h24) return `${pad(h)}:${m}`;
  const ap = h >= 12 ? 'PM' : 'AM';
  h = h % 12 || 12;
  return `${h}:${m} ${ap}`;
}

export function fmtCountdown(ms) {
  ms = Math.max(0, ms);
  const h = Math.floor(ms / 3600000);
  const m = Math.floor((ms % 3600000) / 60000);
  const s = Math.floor((ms % 60000) / 1000);
  return `${pad(h)}:${pad(m)}:${pad(s)}`;
}

export function fmtGregorian(d) {
  return `${MS_DAYS[d.getDay()]}, ${d.getDate()} ${MS_MONTHS[d.getMonth()]} ${d.getFullYear()}`;
}

export function fmtHijri(hijri, date = new Date()) {
  if (hijri && /^\d{4}-\d{1,2}-\d{1,2}$/.test(hijri)) {
    const [y, m, d] = hijri.split('-').map(Number);
    return `${d} ${HIJRI_MONTHS[m - 1]} ${y}H`;
  }
  try {
    const parts = new Intl.DateTimeFormat('en-u-ca-islamic-umalqura', {
      day: 'numeric', month: 'numeric', year: 'numeric',
    }).formatToParts(date);
    const get = (t) => parseInt(parts.find((p) => p.type === t)?.value, 10);
    return `${get('day')} ${HIJRI_MONTHS[get('month') - 1]} ${get('year')}H`;
  } catch {
    return '';
  }
}

// ---------------- Pengambilan data + cache ----------------
const cacheKey = (zone, d) => `wsolat.cache.${zone}.${d.getFullYear()}-${pad(d.getMonth() + 1)}`;

function findEntry(list, date) {
  if (!Array.isArray(list)) return null;
  const want = esolatDate(date).toLowerCase();
  const hit = list.find((e) => String(e.date || '').toLowerCase() === want);
  return hit || null;
}

function cleanupOldCache(keep) {
  for (let i = localStorage.length - 1; i >= 0; i--) {
    const k = localStorage.key(i);
    if (k && k.startsWith('wsolat.cache.') && !keep.includes(k)) localStorage.removeItem(k);
  }
}

async function fetchMonth(zone, date) {
  const data = await invoke('fetch_prayer_times', { zone, period: 'month' });
  if (data?.prayerTime?.length) {
    const key = cacheKey(zone, date);
    localStorage.setItem(key, JSON.stringify(data.prayerTime));
    cleanupOldCache([key]);
    return data.prayerTime;
  }
  return null;
}

/**
 * Dapatkan waktu solat bagi tarikh tertentu.
 * Keutamaan: cache bulanan → API bulanan → API harian.
 * @returns {{ entry: object|null, offline: boolean }}
 */
export async function loadDay(zone, date = new Date(), { force = false } = {}) {
  let list = null;
  if (!force) {
    try { list = JSON.parse(localStorage.getItem(cacheKey(zone, date)) || 'null'); } catch { list = null; }
  }
  let entry = findEntry(list, date);
  if (entry) return { entry, offline: false };

  try {
    list = await fetchMonth(zone, date);
    entry = findEntry(list, date);
    if (entry) return { entry, offline: false };
  } catch (e) {
    console.warn('Gagal ambil data bulanan:', e);
  }

  try {
    const data = await invoke('fetch_prayer_times', { zone, period: 'today' });
    if (data?.prayerTime?.[0]) return { entry: data.prayerTime[0], offline: false };
  } catch (e) {
    console.warn('Gagal ambil data harian:', e);
  }

  // Luar talian: guna sebarang cache zon ini sebagai anggaran
  for (let i = 0; i < localStorage.length; i++) {
    const k = localStorage.key(i);
    if (k && k.startsWith(`wsolat.cache.${zone}.`)) {
      try {
        const l = JSON.parse(localStorage.getItem(k));
        const near = l[Math.min(date.getDate() - 1, l.length - 1)];
        if (near) return { entry: near, offline: true };
      } catch { /* abaikan */ }
    }
  }
  return { entry: null, offline: true };
}

/** Cari entri esok daripada cache (untuk Subuh esok). */
export function cachedDay(zone, date) {
  try { return findEntry(JSON.parse(localStorage.getItem(cacheKey(zone, date)) || 'null'), date); } catch { return null; }
}

// ---------------- Pengiraan masa ----------------
export function getNext(entry, now, tomorrowEntry = null) {
  if (!entry) return null;
  for (const p of MAIN_PRAYERS) {
    if (!entry[p.key]) continue;
    const date = timeToDate(entry[p.key], now);
    if (now < date) return { ...p, time: entry[p.key], date };
  }
  const src = tomorrowEntry?.fajr ? tomorrowEntry : entry;
  const t = new Date(now);
  t.setDate(t.getDate() + 1);
  return { ...byKey.fajr, time: src.fajr, date: timeToDate(src.fajr, t) };
}

export function getPrev(entry, now) {
  if (!entry) return null;
  for (let i = MAIN_PRAYERS.length - 1; i >= 0; i--) {
    const p = MAIN_PRAYERS[i];
    if (!entry[p.key]) continue;
    const date = timeToDate(entry[p.key], now);
    if (date <= now) return { ...p, time: entry[p.key], date };
  }
  const y = new Date(now);
  y.setDate(y.getDate() - 1);
  return { ...byKey.isha, time: entry.isha, date: timeToDate(entry.isha, y) };
}

/** Tema berdasarkan waktu semasa: subuh | dhuha | zohor | asar | maghrib | isyak */
export function getPeriod(entry, now) {
  if (!entry) {
    const h = now.getHours();
    if (h < 6) return 'isyak';
    if (h < 7) return 'subuh';
    if (h < 13) return 'dhuha';
    if (h < 16) return 'zohor';
    if (h < 19) return 'asar';
    if (h < 20) return 'maghrib';
    return 'isyak';
  }
  const at = (k) => (entry[k] ? timeToDate(entry[k], now) : null);
  const steps = [
    ['isha', 'isyak'], ['maghrib', 'maghrib'], ['asr', 'asar'],
    ['dhuhr', 'zohor'], ['syuruk', 'dhuha'], ['fajr', 'subuh'],
  ];
  for (const [k, period] of steps) {
    const d = at(k);
    if (d && now >= d) return period;
  }
  return 'isyak';
}

export const PERIOD_LABEL = {
  subuh: 'Fajar', dhuha: 'Dhuha', zohor: 'Tengah hari', asar: 'Petang', maghrib: 'Senja', isyak: 'Malam',
};
