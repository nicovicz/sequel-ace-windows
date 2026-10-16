# Peta perilaku Sequel Ace untuk port Windows

Baseline yang dipakai di sini adalah source commit `02c910932` (versi aplikasi
6.0.3). Dokumen ini mencatat perilaku yang dapat dibuktikan dari source dan
selisih implementasi Rust yang ada. Ini **bukan** bukti bahwa aplikasi Windows
sudah setara saat dijalankan. Salinan di `/Applications/Sequel Ace.app` masih
5.4.0 (20109), sedangkan salinan yang baru di
`~/Downloads/Sequel Ace.app` adalah 6.0.3 (20116). Xcode penuh belum
tersedia untuk membangun checkout ini.

Rilis resmi 6.0.3 (20116) memakai tag `production/6.0.3-20116` pada commit
`08fc8596`, sedangkan checkout ini `02c910932`, 24 commit lebih baru. Setelah
salinan 6.0.3 dipakai sebagai pembanding, gunakan source tag rilis untuk
perbandingan runtime yang tepat; perubahan setelah tag perlu diaudit terpisah
sebelum dianggap mewakili salinan rilis. Nomor versi/build yang sama saja
tidak membuktikan commit biner identik dengan checkout ini.

## Perbandingan paket aplikasi 6.0.3

Pemeriksaan statis pada `~/Downloads/Sequel Ace.app` menunjukkan:

| Bukti dari paket rilis | Hasil | Batas kesimpulan |
| --- | --- | --- |
| `Contents/Info.plist` (REA `ev_688b57082d352ffdd6ca7ecbbe85f98eb3dc374b586fa0713aa85f9633036a17`) | Bundle `com.sequel-ace.sequel-ace`, versi 6.0.3, build 20116, executable `Sequel Ace`. | Mengidentifikasi paket, bukan membuktikan jalur UI tertentu. |
| `Contents/Resources/PreferenceDefaults.plist` (REA `ev_c501d9d2dd88c18819cbabf7068e54170569e5bc3c9a41a160a1b50a51472849`) | `LimitResults=true`, `LimitResultsValue=1000`, `ProcessListShowFullProcessList=false`, `ProcessListEnableAutoRefresh=false`, interval refresh 10 detik. | Ini nilai bawaan; preference pengguna yang pernah disimpan dapat berbeda. |
| Perbandingan dengan `Resources/Plists/PreferenceDefaults.plist` di checkout (REA `ev_2a1bed241371622ea10d23203c08991114281bd7166c094ba04445cade48f9db`) | Semua 112 kunci dan nilainya sama. | Kesamaan preference tidak membuktikan keseluruhan perilaku atau tampilan sama. |

Executable rilis yang dibuka REA adalah Mach-O arm64 SHA-256
`f1d27acc5c50cfb76f88af09983bfb1053208ff906aa7408294839d244fc38bc`.
Perbedaan source antara tag rilis dan checkout terpusat pada tiga XIB,
penempatan tombol popup dan drop zone filter Content, serta string terkait;
query dan preference yang dipetakan di bawah tidak berubah. Tampilan runtime,
koneksi database, dan respons kesalahan belum diamati langsung. Paket di
Downloads dapat menjadi pembanding rilis 6.0.3, sedangkan klaim kesetaraan
checkout terbaru tetap memerlukan build dari commit yang sama.

Snapshot UI akhirnya berhasil pada jendela Sequel Ace 6.0.3 (REA
`ev_4da1941c4070767598243a65ca8d91e375856fbf2d1a8aaab95fd4788478da65`,
window 1470×828). Sidebar menampilkan pencarian dan Favorites/Quick Connect;
form Quick Connect menyediakan TCP/IP, Socket, SSH, AWS IAM, dan Vault, lalu
Name, Host, Username, Password, Database opsional, Port, Time Zone, opsi
LOCAL_DATA_INFILE, cleartext plugin, Require SSL, dan Get Public Key. Toolbar
menampilkan pemilih database serta Structure, Content, Relations, Triggers,
Table Info, Query, Table History, Users, dan Console.

Snapshot visual port awal dari build host macOS (800×632) menunjukkan form
minimal tanpa sidebar atau toolbar. Preview terbaru dari build host macOS kini
berukuran 1200×720 dan menempatkan sidebar kiri, toolbar atas, serta form
koneksi di tengah seperti struktur Sequel Pro. Preview Mac ini memakai kode
Rust yang sama, tetapi belum membuktikan tampilan EXE di Windows 11.

Observasi hanya mencakup layar koneksi. Tidak ada koneksi database yang saya
jalankan sebagai bagian dari perbandingan ini; database browsing, perilaku
error yang terkontrol, dan kesetaraan visual pada Windows tetap belum
dibandingkan. Uji runtime yang dipakai sebagai baseline harus memakai data
koneksi uji tanpa kredensial pribadi.

Untuk permintaan mengikuti **Sequel Pro asli**, source publiknya memakai
`ConnectionView.xib` dengan tiga tab koneksi: TCP/IP, Socket, dan SSH; layar
memiliki split view Favorites di kiri dan detail koneksi di kanan. Lihat
[Sequel Pro ConnectionView.xib](https://github.com/sequelpro/sequelpro/blob/master/Interfaces/English.lproj/ConnectionView.xib).
Layar port Rust kini mengikuti susunan tersebut dan menampilkan Name/Color,
sidebar, serta toolbar. Tab TCP/IP memuat Host/Username/Password/Database/Port;
Socket memuat Socket/Database/Username/Password; SSH memisahkan kredensial SSH
dari Host/Username/Password/Database/Port MySQL. Nilai Host dan Port TCP/IP
dibiarkan kosong di form; `127.0.0.1` dan `3306` menjadi fallback saat koneksi
dibuat, sesuai model Sequel Ace. Hanya TCP/IP yang dapat dipakai; Connect pada
Socket/SSH dinonaktifkan dan ada keterangan bahwa koneksinya belum diterapkan.
Kontrol Time Zone/SSL masih nonaktif.
Name dan warna belum disimpan sebagai Favorite; sidebar hanya menampilkan
placeholder kosong dan tombol tambah/aksi masih nonaktif.
Preview dibuat dari build host macOS. EXE Windows telah dibangun ulang melalui
`WindowsPort/build-windows.sh`, tetapi belum dijalankan di Windows 11.

## Aturan untuk setiap fitur berikutnya

1. Catat entry point UI, state dan preference, controller/model, query atau
   panggilan protokol, hasil, kondisi kosong, kesalahan, serta efek samping
   dari source commit yang sama **sebelum** menulis port-nya.
2. Pisahkan fakta source, hasil pengamatan runtime, dan asumsi. Jika source
   belum menjawab suatu perilaku, beri status *belum diketahui* dan telusuri
   lebih lanjut. Jangan mengisi celah dengan angka atau UI buatan sendiri
   sambil menyebutnya setara.
3. Bandingkan kontrak itu dengan implementasi Rust secara per aksi. Selisih
   yang diketahui tetap tercatat sampai diperbaiki. Fitur baru tidak diberi
   label *parity* hanya karena query berhasil atau EXE terkompilasi.
4. Verifikasi pada MySQL dan MariaDB dengan database normal/kosong, objek
   table/view, hak akses terbatas, koneksi gagal/putus, data besar, NULL,
   teks Unicode, dan BLOB bila relevan. Bandingkan dengan build macOS **dari
   commit yang sama**. Preferensi pengguna sebelumnya adalah tidak menambah
   atau menjalankan tes otomatis, sehingga pemeriksaan itu belum dilakukan
   dalam pekerjaan ini; status runtime tetap terbuka.

## Kontrak source dan selisih port yang sudah ada

| Area | Perilaku Sequel Ace dari source | Port Rust saat ini dan selisih yang diketahui |
| --- | --- | --- |
| Koneksi TCP/IP | `SAConnectionInfo.swift` memodelkan TCP/IP, socket, SSH, AWS IAM, Vault serta pengaturan TLS, encoding, zona waktu, kompresi, dan autentikasi. `SAConnectionService.swift:350-507` memilih host/port, timeout preference, autentikasi, database awal, zona waktu, dan mengembalikan error khusus. Jika database awal gagal dipilih, koneksi MySQL tetap ada tetapi hasil koneksi bertanda gagal memilih database. | `WindowsPort/src/database/worker.rs` hanya TCP/IP dan memilih database awal. Timeout tetap 10 detik; TLS, encoding/zona waktu, kompresi, retry/cancel, dan pemetaan error belum setara. Kegagalan database awal dilaporkan sambil mempertahankan koneksi. |
| Daftar database | `SPDatabaseDocument.m:602-620` mengambil daftar dari driver. `SADatabaseListManager.swift:244-338` memisahkan empat database sistem, menjaga urutan, menyediakan placeholder/add/refresh, dan mencoba ulang sekali setelah popup diperbarui bila seleksi gagal atau daftar basi. | Pemisahan empat nama dan seleksi manual ada. Popup, aksi add, dan retry saat daftar basi belum ada. Perbedaan perilaku saat hak `SHOW DATABASES` terbatas belum diverifikasi. |
| Daftar objek | `SPTablesList.m:238-315` memakai `SHOW FULL TABLES`, atau `SHOW TABLE STATUS` jika komentar tabel diaktifkan; hasil dinormalisasi oleh `SATableListResultParser`. Query `information_schema.routines` menambah procedure dan function bila tersedia. Encoding identitas sementara diatur ke `utf8mb4`. | Hanya tabel dan view dari `SHOW FULL TABLES`; procedure/function, komentar tabel, normalisasi label server nonstandar, serta jalur encoding asli belum ada. |
| Struktur dan DDL | `SPDatabaseStructure.m:318` membaca `SHOW FULL COLUMNS`; `SPTableStructure.m:1469` membaca indeks; `SPDatabaseDocument.m:1509-1513` memilih `SHOW CREATE TABLE` atau `SHOW CREATE VIEW` untuk aksi SQL. UI asli memisahkan struktur dan aksi DDL. | Kolom, indeks, dan CREATE SQL ditampilkan pada satu panel detail. Data dasar ada, tetapi pengelompokan UI, opsi indeks, aksi struktur, dan penanganan hak akses belum dipetakan tuntas. |
| Info tabel | `SPTableData.m:1200-1247` membaca `SHOW TABLE STATUS`, menganggap row count MyISAM akurat, dan mencoba `COUNT(1)` saat nilai Rows NULL. `SPTableInfo.m:147-260` juga menunjukkan tanggal, ukuran, encoding/collation, auto increment, dan create options; nilai yang tidak tersedia dilewati. | Port memakai `information_schema.TABLES`, selalu melabeli jumlah baris sebagai estimasi, dan hanya menampilkan sebagian field. Ini **belum setara** bahkan jika angka tampak sama. |
| Relasi | `SPTableRelations.m:627-661` memakai cache constraint dari `SPTableData` dan menampilkan nama, kolom sumber, database/tabel/kolom tujuan, serta aksi ON UPDATE dan ON DELETE. View/non-InnoDB dapat memiliki status UI berbeda. | Port membaca `information_schema.KEY_COLUMN_USAGE` dan belum menampilkan aksi UPDATE/DELETE maupun status dukungan storage engine. |
| Content | `PreferenceDefaults.plist:164-167` mengaktifkan pembatasan dengan default 1000. `SPTableContent.m:970-1017` menambah filter, sort bila dipilih, lalu LIMIT/OFFSET hanya jika preference aktif. Preference dapat mengubah atau mematikan batas. `SPTableContent.m:1072-1080` kembali ke halaman pertama jika halaman akhir kosong. Hasil dimuat melalui streaming dan operasi dapat dibatalkan. | Content Rust kini memakai 1000 per halaman dan kembali ke halaman pertama jika halaman akhir kosong. Ukuran halaman tetap, tanpa pilihan tanpa batas, filter/sort, hitung total, streaming bertahap, cancel, editing, atau penanganan BLOB/NULL sesuai preference. Query membaca satu halaman ke memori. |
| Query SQL | `SPCustomQuery.m:192-282` memakai `SPSQLParser` untuk query lengkap, pilihan teks, atau statement di caret, termasuk delimiter. `SPCustomQuery.m:692-735` memberi konfirmasi untuk SQL destruktif bila preference peringatan aktif; default preference adalah **mati** (`PreferenceDefaults.plist:228-229`). `SPCustomQuery.m:760-1020` menjalankan statement berurutan, mendukung cancel dan hasil streaming; tidak ditemukan batas tampilan 1000 seperti port. | Port mengeksekusi teks editor melalui driver dan membatasi 20 result set serta 1000 baris per set. Pemilihan statement/teks, parser delimiter, cancel, hasil streaming, serta preference peringatan belum setara. Batas 1000 hasil SQL adalah batas **port**, bukan default Sequel Ace. |
| Riwayat query | `PreferenceDefaults.plist:100-101` menetapkan maksimum 20. `SPQueryController.m:989-1044` menggabungkan tanpa duplikat dan menyimpan riwayat per dokumen atau global ke preference/SQLite. | Port menyimpan 20 query hanya selama proses berjalan; cakupan dokumen, persistensi, dan aturan merge asli belum ada. |
| Variabel server | `SPServerVariablesController.m:160-195,275-320` membaca `SHOW VARIABLES` dalam sheet; pencarian mencocokkan nama **atau nilai**, dan ada aksi salin/simpan. | Port membaca semua variabel dan menampilkan mode ringkas/lengkap dalam panel, tanpa filter nama/nilai, pilihan baris, simpan, atau UI sheet asli. |
| Daftar proses | `PreferenceDefaults.plist:184-189` menetapkan mode *full* mati dan auto-refresh mati secara default. `SPProcessListController.m:558-584` memilih daftar biasa atau `SHOW FULL PROCESSLIST`, lalu menyediakan filter dan aksi proses. | Shortcut port kini memakai `SHOW PROCESSLIST` agar default tidak-full sesuai source. Belum ada toggle full, auto-refresh, filter, kill query/koneksi, dan hasil tetap terpotong oleh batas 1000 baris query port. |
| Salin hasil/CSV | Content dan query asli memiliki aksi salin/ekspor yang bergantung pada pilihan baris/kolom. Katalog aksi ada di `SPTableContent.m`, `SPCustomQuery.m`, dan `SPExportController.m`; kontrak format byte-per-byte belum dipetakan. | Port menyalin CSV untuk semua baris yang sedang ditampilkan. Ini utilitas port, belum boleh disebut sama dengan aksi ekspor/salin asli. |

## Keputusan dari audit ini

- Tombol `Preview Rows` yang sebelumnya ada di port dihapus: source asli
  menyediakan tampilan Content, bukan shortcut preview 1000 baris ke editor SQL.
- Angka 1000 hanya dibenarkan sebagai **default Content dengan pembatasan
  aktif**. Angka itu bukan batas bawaan editor SQL asli dan bukan ukuran tetap
  untuk setiap pengguna.
- `SHOW PROCESSLIST` dipakai pada shortcut port karena mode full pada aplikasi
  asli dimatikan secara default. Shortcut itu masih belum setara dengan jendela
  proses asli.
- Seluruh baris di atas berstatus **source dipetakan, runtime belum
  dibandingkan**. Detail yang belum tercantum perlu ditelusuri sebelum area
  itu diimplementasikan lebih lanjut.

## Urutan pengerjaan berdasarkan ketergantungan

1. Lengkapi kontrak dan perilaku koneksi, pemilihan database, dan daftar objek
   karena semua tampilan berikutnya bergantung pada state ini. Procedure dan
   function perlu model objek tersendiri sebelum detailnya dibuat.
2. Selesaikan kontrak Content (preference batas, count, sort/filter, representasi
   NULL/BLOB, loading/cancel) sebelum edit baris dan ekspor data penuh.
3. Samakan query editor (parser, pilihan/caret, cancel, hasil streaming,
   riwayat dan preference) sebelum query favorites atau import SQL.
4. Samakan metadata dan server tools per bagian, termasuk kegagalan izin dan
   perbedaan MySQL/MariaDB, sebelum menambah aksi perubahan schema/proses.

Setiap baris yang ditutup sebagai setara kelak harus mencatat build macOS dan
Windows dari commit yang sama, server dan hak akses yang digunakan, langkah
reproduksi, serta hasil untuk kondisi normal, kosong, dan error. Saat ini belum
ada baris yang memenuhi syarat itu.
