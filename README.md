# W-Solat

Widget waktu solat Malaysia (**JAKIM e-Solat**) untuk taskbar Windows 11 yang dibina menggunakan **Tauri v2**, **Rust**, dan **Svelte 5** dengan reka bentuk berinspirasikan seni bina Islam.

## Ciri-Ciri Utama

- **Bar Mini Taskbar Windows 11**: Paparan waktu solat seterusnya, kiraan detik secara langsung, nama waktu dalam tulisan Arab (fon **Amiri**), dan lengkung kemajuan pada taskbar.
- **Panel Jadual Waktu Solat**: Paparan penuh waktu **Imsak, Subuh, Syuruk, Zohor, Asar, Maghrib, dan Isyak** bersama tarikh Hijrah serta masihi.
- **Pengesanan Lokasi Automatik & Manual**: Mengesan lokasi semasa peranti secara automatik dan memadankannya dengan zon JAKIM e-Solat, serta menyokong pilihan zon secara manual di seluruh Malaysia.
- **Notifikasi Custom Berinspirasikan Islam**:
  - Tetingkap notifikasi khas mengikut suasana waktu solat dengan rekaan bingkai emas, siluet masjid, dan kaligrafi Arab.
  - Empat nada terbina dalam (**Irama**, **Chime**, **Melodi**, dan **Beep**) serta pilihan **Sunyi**.
  - Tujuh pilihan azan (**Azan Bayati**, **Azan Hijaz**, **Azan Jiharkah**, **Azan Nahawand**, **Azan Rast**, **Azan Sikah**, dan **Azan Soba**). Setiap pilihan menggunakan rakaman Subuh untuk waktu Subuh dan rakaman biasa untuk waktu lain.
  - Tetapan **Azan** daripada versi terdahulu dipadankan kepada **Azan Bayati**.
  - Tutup automatik **3 saat** selepas bunyi notifikasi tamat, dan berhenti serta-merta apabila pengguna menutup notifikasi.
  - Sokongan peringatan awal (*reminder*) dan *snooze* 5 minit.
- **Auto-Mula Windows**: Pilihan untuk menjalankan aplikasi secara automatik apabila Windows dimulakan, lengkap dengan ikon *system tray*.

## Pemasangan (Pengguna Windows)

1. Muat turun pemasang Windows terkini daripada halaman **[Releases](https://github.com/ameer-yusoff/w-solat/releases/latest)**.
2. Jalankan fail pemasang tersebut dan ikut arahan di skrin.
3. Widget akan muncul di taskbar dan mengesan zon waktu solat anda secara automatik.

## Pembangunan (Developer)

### Prasyarat

- [Node.js](https://nodejs.org/) (v18+ disyorkan)
- [Rust & Cargo](https://www.rust-lang.org/tools/install)
- WebView2 Runtime (tersedia secara lalai pada Windows 11)

### Langkah Menjalankan Secara Tempatan

```bash
# 1. Klon repositori
git clone https://github.com/ameer-yusoff/w-solat.git
cd w-solat

# 2. Pasang kebergantungan
npm install

# 3. Jalankan mod pembangunan
npm start

# 4. Bina aplikasi & pemasang NSIS (release)
npm run release
```

## Struktur Projek

```text
w-solat/
├── public/
│   ├── audio/          # Tujuh pasangan MP3 azan biasa dan Subuh
│   └── fonts/          # Fon Amiri Arabic (Regular & Bold)
├── src/
│   ├── data/           # Data zon waktu solat Malaysia (JAKIM)
│   ├── lib/            # Logik waktu solat, lokasi, bunyi, ikon & state
│   ├── notify/         # Antaramuka tetingkap notifikasi custom
│   ├── styles/         # Gaya global dan tema Islam
│   └── widget/         # Antaramuka bar mini, jadual, lokasi & tetapan
├── src-tauri/
│   ├── icons/          # Ikon aplikasi & tray
│   ├── capabilities/   # Konfigurasi kebenaran Tauri v2
│   ├── src/            # Backend Rust, Win32 taskbar & tray
│   └── tauri.conf.json # Konfigurasi Tauri v2
├── index.html          # Titik masuk widget utama
├── notify.html         # Titik masuk tetingkap notifikasi
└── vite.config.js      # Konfigurasi Vite + Svelte 5
```

## Sumber Data

Data waktu solat diperoleh daripada API rasmi **e-Solat JAKIM** bagi semua zon di Malaysia.

## Kredit Audio

Rakaman azan oleh **Ustaz Husaini Mahmur**, daripada album [Azan 7 Tarannum di YouTube Music](https://music.youtube.com/playlist?list=OLAK5uy_ldnXdmREHvQ-lQ8eZf4FLtRfSgk4L7mso). Album diterbitkan oleh Inteam Records.

Pakej aplikasi menggunakan 14 fail MP3 dalam `public/audio/`. Untuk setiap tarannum, gunakan nama `azan-<tarannum>.mp3` bagi rakaman biasa dan `azan-<tarannum>-subuh.mp3` bagi rakaman Subuh. Contohnya, `azan-bayati.mp3` dan `azan-bayati-subuh.mp3`.

## Lesen

Projek ini dilesenkan di bawah [MIT License](LICENSE).
