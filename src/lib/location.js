import { invoke } from '@tauri-apps/api/core';
import groups from '../data/groups.json';

const CITY_ALIASES = {
  // Johor
  kulai: 'JHR02',
  senai: 'JHR02',
  skudai: 'JHR02',
  'pasir gudang': 'JHR02',
  'iskandar puteri': 'JHR02',
  nusajaya: 'JHR02',
  tangkak: 'JHR04',
  // Selangor & Lembah Klang
  'petaling jaya': 'SGR01',
  'subang jaya': 'SGR01',
  puchong: 'SGR01',
  damansara: 'SGR01',
  ampang: 'SGR01',
  cheras: 'SGR01',
  kajang: 'SGR01',
  bangi: 'SGR01',
  cyberjaya: 'SGR01',
  selayang: 'SGR01',
  rawang: 'SGR01',
  sungai: 'SGR01',
  'seri kembangan': 'SGR01',
  // Pulau Pinang
  'george town': 'PNG01',
  georgetown: 'PNG01',
  'bayan lepas': 'PNG01',
  butterworth: 'PNG01',
  'bukit mertajam': 'PNG01',
  // Perak
  kampar: 'PRK02',
  'seri iskandar': 'PRK03',
  taiping: 'PRK04',
  sitiawan: 'PRK05',
  // Kedah
  'sungai petani': 'KDH02',
  jitra: 'KDH01',
  // Pahang
  mentakab: 'PHG03',
  raub: 'PHG04',
  'cameron highlands': 'PHG06',
  'tanah rata': 'PHG06',
  // Negeri Sembilan
  nilai: 'NGS02',
  port: 'NGS02',
  // Terengganu
  dungun: 'TRG03',
  kemaman: 'TRG04',
  chukai: 'TRG04',
  // Sarawak
  samarahan: 'SWK08',
  // Sabah
  penampang: 'SBH07',
  putatan: 'SBH07',
};

const STATE_DEFAULT_ZONE = {
  johor: 'JHR02',
  kedah: 'KDH01',
  kelantan: 'KTN01',
  melaka: 'MLK01',
  'negeri sembilan': 'NGS02',
  pahang: 'PHG02',
  perlis: 'PLS01',
  'pulau pinang': 'PNG01',
  penang: 'PNG01',
  perak: 'PRK02',
  sabah: 'SBH07',
  sarawak: 'SWK08',
  selangor: 'SGR01',
  terengganu: 'TRG01',
  'kuala lumpur': 'WLY01',
  putrajaya: 'WLY01',
  labuan: 'WLY02',
};

function norm(s) {
  return String(s || '')
    .toLowerCase()
    .replace(/wilayah persekutuan|state of|federal territory of/g, '')
    .trim();
}

export function matchZoneFromGeo(geo) {
  if (!geo || (geo.countryCode && geo.countryCode !== 'MY')) return null;

  const city = norm(geo.city);
  const region = norm(geo.region);

  // 1. Semak alias bandar utama
  if (city) {
    for (const [alias, z] of Object.entries(CITY_ALIASES)) {
      if (city.includes(alias)) {
        const g = groups.find((x) => x.zone === z);
        if (g) return { zone: g.zone, locationName: geo.city.trim(), state: g.state };
      }
    }

    // 2. Padanan langsung dengan senarai kawasan JAKIM
    for (const g of groups) {
      for (const a of g.area) {
        const na = norm(a);
        if (city === na || city.includes(na) || na.includes(city)) {
          return { zone: g.zone, locationName: a, state: g.state };
        }
      }
    }
  }

  // 3. Padanan negeri
  for (const [st, z] of Object.entries(STATE_DEFAULT_ZONE)) {
    if (region.includes(st) || city.includes(st)) {
      const g = groups.find((x) => x.zone === z);
      if (g) return { zone: g.zone, locationName: geo.city?.trim() || g.area[0], state: g.state };
    }
  }

  return null;
}

export async function detectDeviceZone() {
  try {
    const geo = await invoke('detect_location');
    const matched = matchZoneFromGeo(geo);
    if (matched) return { ...matched, detected: true };
  } catch (e) {
    console.warn('Gagal mengesan lokasi peranti:', e);
  }
  return null;
}
