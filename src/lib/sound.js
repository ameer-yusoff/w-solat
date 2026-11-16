// ==========================================================
// Enjin bunyi: menyokong audio Azan sebenar (.mp3) serta
// Web Audio synthesizer, dengan hentian segera & callback tamat.
// ==========================================================
const azanSounds = ['Bayati', 'Hijaz', 'Jiharkah', 'Nahawand', 'Rast', 'Sikah', 'Soba']
  .map((name) => ({ value: `azan-${name.toLowerCase()}`, label: `Azan ${name}`, icon: 'music' }));

export const SOUNDS = [
  ...azanSounds,
  { value: 'takbir',   label: 'Takbir', icon: 'music' },
  { value: 'chime',    label: 'Chime',  icon: 'bell' },
  { value: 'hijaz',    label: 'Hijaz',  icon: 'music' },
  { value: 'softbeep', label: 'Beep',   icon: 'volume' },
  { value: 'silent',   label: 'Sunyi',  icon: 'mute' },
];

export class SoundEngine {
  constructor() {
    this.ctx = null;
    this.master = null;
    this.audio = null;
    this.endTimer = null;
  }

  init(volume = 0.8) {
    if (!this.ctx) {
      const Ctx = window.AudioContext || window.webkitAudioContext;
      if (!Ctx) return false;
      this.ctx = new Ctx();
      this.master = this.ctx.createGain();
      const delay = this.ctx.createDelay();
      const fb = this.ctx.createGain();
      const wet = this.ctx.createGain();
      delay.delayTime.value = 0.18;
      fb.gain.value = 0.28;
      wet.gain.value = 0.22;
      this.master.connect(this.ctx.destination);
      this.master.connect(delay);
      delay.connect(fb).connect(delay);
      delay.connect(wet).connect(this.ctx.destination);
    }
    this.master.gain.value = Math.max(0, Math.min(1, volume));
    if (this.ctx.state === 'suspended') this.ctx.resume();
    return true;
  }

  note(freq, start, dur, type = 'sine', gain = 0.25) {
    const osc = this.ctx.createOscillator();
    const g = this.ctx.createGain();
    osc.type = type;
    osc.frequency.setValueAtTime(freq, start);
    g.gain.setValueAtTime(0.0001, start);
    g.gain.linearRampToValueAtTime(gain, start + 0.04);
    g.gain.exponentialRampToValueAtTime(0.0001, start + dur);
    osc.connect(g).connect(this.master);
    osc.start(start);
    osc.stop(start + dur + 0.05);
  }

  chime(t) {
    [523.25, 659.25, 783.99, 1046.5].forEach((f, i) => {
      this.note(f, t + i * 0.22, 1.4, 'sine', 0.28);
      this.note(f * 2, t + i * 0.22, 0.8, 'triangle', 0.07);
    });
    return 2.2;
  }

  takbir(t) {
    const melody = [
      [293.66, 0.35, 0], [349.23, 0.35, 0.38], [392.0, 0.55, 0.76], [440.0, 0.45, 1.35],
      [392.0, 0.4, 1.82], [349.23, 0.4, 2.24], [293.66, 0.8, 2.66],
    ];
    melody.forEach(([f, d, delay]) => {
      this.note(f, t + delay, d + 0.4, 'sine', 0.3);
      this.note(f * 1.5, t + delay, d + 0.2, 'triangle', 0.06);
    });
    return 3.9;
  }

  hijaz(t) {
    const seq = [
      [293.66, 0.5], [311.13, 0.35], [369.99, 0.6], [392.0, 0.45], [440.0, 0.9],
      [392.0, 0.4], [369.99, 0.45], [311.13, 0.4], [293.66, 1.2],
    ];
    let at = 0;
    seq.forEach(([f, d]) => {
      this.note(f, t + at, d + 0.5, 'sine', 0.3);
      this.note(f / 2, t + at, d + 0.6, 'sine', 0.08);
      this.note(f * 2, t + at, d * 0.6, 'triangle', 0.03);
      at += d * 0.82;
    });
    return at + 1.4;
  }

  softbeep(t) {
    this.note(587.33, t, 0.35, 'sine', 0.25);
    this.note(880.0, t + 0.2, 0.65, 'sine', 0.28);
    return 1;
  }

  /**
   * Mainkan nada pilihan.
   * @param {string} type
   * @param {number} volume
   * @param {{ prayerKey?: string, onEnded?: () => void }} opts
   * @returns {boolean} true jika bunyi sedang dimainkan
   */
  play(type, volume = 0.8, { prayerKey = '', onEnded } = {}) {
    this.stop();
    if (type === 'silent') {
      onEnded?.();
      return false;
    }

    if (azanSounds.some(({ value }) => value === type)) {
      const variant = prayerKey === 'fajr' ? '-subuh' : '';
      const src = `/audio/${type}${variant}.mp3`;
      const el = new Audio(src);
      el.volume = Math.max(0, Math.min(1, volume));
      el.onended = () => {
        if (this.audio === el) {
          this.audio = null;
          onEnded?.();
        }
      };
      el.onerror = () => {
        if (this.audio === el) {
          this.audio = null;
          onEnded?.();
        }
      };
      this.audio = el;
      el.play().catch((err) => {
        console.warn('Gagal mainkan audio azan:', err);
        if (this.audio === el) {
          this.audio = null;
          onEnded?.();
        }
      });
      return true;
    }

    if (!this.init(volume)) {
      onEnded?.();
      return false;
    }
    const t = this.ctx.currentTime + 0.05;
    const fn = this[type] ? type : 'chime';
    const secs = this[fn](t);
    this.endTimer = setTimeout(() => {
      this.endTimer = null;
      onEnded?.();
    }, secs * 1000);
    return true;
  }

  stop() {
    if (this.endTimer) {
      clearTimeout(this.endTimer);
      this.endTimer = null;
    }
    if (this.audio) {
      this.audio.onended = null;
      this.audio.onerror = null;
      this.audio.pause();
      this.audio.currentTime = 0;
      this.audio = null;
    }
    if (this.ctx) {
      this.ctx.close();
      this.ctx = null;
      this.master = null;
    }
  }
}
