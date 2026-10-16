# Audit: `SPSQLExporter.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/DataExport/Exporters/SPSQLExporter.m`. Baseline commit `02c910932`.

- File: 1028 baris; header 146 baris.
- Peran: serialisasi seleksi database/tabel ke satu file dump SQL (header,
  `CREATE TABLE`/placeholder view, `INSERT`, trigger, view, routine, footer) dan
  melaporkan error/progress ke delegate.
- `SPSQLExporter` subclass `SPExporter` (`.h:43`), implement `SPSQLExporterProtocol` via property `delegate` (`.h:32,70`).
- Total: hanya **5 definisi method** — `exportOperation` saja 822 baris.

## 1. Inventaris ivar/property + konstanta

Class extension `SPSQLExporter ()` (44–48) hanya mendeklarasikan helper privat `_createViewPlaceholderSyntaxForView:` (46).

### Ivar `.h:45–65`
`sqlExportTables` (45), `sqlDatabaseHost` (47), `sqlDatabaseName` (48), `sqlDatabaseVersion` (49),
`sqlExportCurrentTable` (50), `sqlExportErrors` (51), `sqlOutputIncludeUTF8BOM` (53),
`sqlOutputEncodeBLOBasHex` (54), `sqlOutputIncludeErrors` (55), `sqlOutputIncludeAutoIncrement` (56),
`sqlOutputIncludeGeneratedColumns` (57), `sqlInsertDivider` (`SPSQLExportInsertDivider`, 59),
`sqlCurrentTableExportIndex` (61), `sqlInsertAfterNValue` (62), `sqlTableDataInstance` (`SPTableData*`, 64, lazy `.m:107`).

### Property `.h:70–140`
`delegate` (weak, 70), `sqlExportTables` (75), `sqlDatabaseHost` (80), `sqlDatabaseName` (85),
`sqlDatabaseVersion` (90), `sqlExportCurrentTable` (95), `sqlExportErrors` (100),
`sqlOutputIncludeUTF8BOM` (105), `sqlOutputEncodeBLOBasHex` (110), `sqlOutputIncludeErrors` (115),
`sqlOutputIncludeAutoIncrement` (120), `sqlOutputIncludeGeneratedColumns` (125),
`sqlCurrentTableExportIndex` (130), `sqlInsertAfterNValue` (135), `sqlInsertDivider` (140).
Semua auto-synthesize (`.m:52–66`).

**Static/konstanta file-local: tidak ada** (`grep '^static'` kosong). `#include <stdlib.h>` (42) hanya untuk `calloc`/`free` (379–380, 440–441, 487–488, 619–620). `SPCurrentTimestampPattern` (1006) dan `SPSQLExportInsertDivider`/`SPSQLInsertEveryNDataBytes` (83, 507) bukan dari file ini — **UNKNOWN** lokasi definisi `SPSQLExportInsertDivider`.

## 2. Semua `#pragma mark`

**Tidak ada.** `exportOperation` (90–911) tak dipisah mark; batas fase hanya dari komentar.

## 3. Tabel method (5 definisi, urut baris)

| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 75 | 88 | `initWithDelegate:` | Validasi protokol delegate; set default (divider, ambang 250000 byte) |
| 90 | 911 | `exportOperation` | Driver ekspor monolitik: header, tabel, content, trigger, view, routine, footer |
| 913 | 917 | `endCleanup:` | Restore `SQL_MODE` server sebelumnya bila tertangkap |
| 924 | 927 | `didExportErrorsOccur` | YES bila string error ekspor akumulasi non-kosong |
| 937 | 1026 | `_createViewPlaceholderSyntaxForView:` | Bangun placeholder `CREATE TABLE ... ENGINE=MyISAM` untuk kolom view |

## 4. Temuan menonjol / UNKNOWN

**Header/banner (149–254)**
- BOM UTF-8 opsional bila `sqlOutputIncludeUTF8BOM` (152–154). Koneksi dipaksa `utf8mb4` (159).
- Banner: `Sequel Ace SQL dump`, versi dari `CFBundleVersion`, homepage/dev URL, host+versi MySQL, database, generation time (162–169).
- Restore sesi import dalam bentuk komentar kondisional MySQL: character set client/results/collation (172–174), `SET NAMES utf8mb4` (175), `FOREIGN_KEY_CHECKS` (177), lalu komentar historis panjang soal `SQL_MODE` (179–209).
- `sql_mode` lama via `SHOW VARIABLES LIKE 'sql_mode'` (213–219); bila sudah kosong dibiarkan nil (222–225), else `SET SQL_MODE=''` dipaksa (227, error dicatat 229–232).
- `@OLD_SQL_MODE` memuat `NO_AUTO_VALUE_ON_ZERO` (251, rasional issue #865 di 243–250); `SQL_NOTES=0` (252). Header di-flush via `writeString:metaString` (254).

**Struktur per tabel (297–353)**
- Tipe item dari index 4 elemen `sqlExportTables` → proc/func/tabel (133–144).
- `SHOW CREATE TABLE` (299); kolom `Create View` merutekan ke view handling (306–319).
- `DROP TABLE/VIEW IF EXISTS` bila drop syntax diminta (339–341). Struktur diemit bila diminta, `AUTO_INCREMENT=N` dibuang kecuali `sqlOutputIncludeAutoIncrement` (347–352). View ditunda: syntax disimpan di `viewSyntaxes`, `createTableSyntax` jadi placeholder (307–314).

**Content per tabel (356–629)**
- Metadata kolom dari `sqlTableDataInstance` (358); kolom GENERATED dihitung/dikecualikan kecuali `sqlOutputIncludeGeneratedColumns` (364–373).
- Array keputusan raw/hex/bit per-kolom via `calloc` (379–432); kolom BIT diekspor numerik (`CAST ... AS UNSIGNED` 413–425); string binary hex-encoded bila `sqlOutputEncodeBLOBasHex` (399–404).
- Row count untuk progress (435–445); query streaming low-memory bila `exportUsingLowMemoryBlockingStreaming` (449).
- `LOCK TABLES ... WRITE` + `DISABLE KEYS` (458); header `INSERT INTO ... VALUES` (463); `ENABLE KEYS` + `UNLOCK TABLES` (612). Content view di-skip via guard `tableType == SPTableTypeTable` (333, 339, 344, 356).

**Insert divider / bentuk nilai (505–604)**
- Dua mode `SPSQLInsertDivider`: `INSERT` baru saat panjang byte ≥ `sqlInsertAfterNValue * 1024` (`SPSQLInsertEveryNDataBytes`) atau setelah N baris (`SPSQLInsertEveryNRows`) (507–508). Default `SPSQLInsertEveryNDataBytes`, N = 250000 (83–84).
- Bentuk nilai (533–591): `NULL` (534); raw terpercaya (539); BIT → `b'..'` (543–545); hex pra-encode → `X'..'` (549–550); GEOMETRY via `escapeAndQuoteData` (554–555); string kosong `''` (559–560); `NSData` → hex atau decode string ter-cek-lossy (564–586); else escaped quoted string (589–590).

**Trigger (631–686, digerbangi `sqlOutputIncludeStructure`)**: `SHOW TRIGGERS WHERE Table = ...` (633); `DELIMITER ;;` (640); per trigger tulis `SQL_MODE` sesi, `DEFINER` dipecah `` `user`@`host` `` (661, 664–671); restore `DELIMITER ;` + `SQL_MODE=@OLD_SQL_MODE` (674).

**View (256, 693–719)**: pass tertunda setelah semua tabel; tulis banner `Dump of view` + `CREATE VIEW` tersimpan (dipretty di 311).

**Routine (721–882)**: loop `@[@"PROCEDURE", @"FUNCTION"]` (722); skip bila seleksi kosong (743); `SHOW PROCEDURE|FUNCTION STATUS WHERE Db = ...` (746); item dicocokkan by nama, hormati flag structure/drop (780–805); `DROP ... IF EXISTS` (811–814), `SHOW CREATE ...` (828), permission `NULL` ditangani (847–854), `DEFINER` di-escape (822–826), body dengan komentar berversi `/*!50003 ... */` (865).

**Footer (884–896)**: restore `SQL_NOTES`, `SQL_MODE`, `FOREIGN_KEY_CHECKS`, character set client/results/collation (886–893).

**Cleanup/completion**: error disimpan via `setSqlExportErrors:` (899), file ditutup (902), flag running dibersihkan (905), delegate diberi tahu `sqlExportProcessComplete:` (908), SQL mode direstore (910 → `endCleanup:` 913).

**Pola error**: `exportOutputFile.fileHandleError` memicu `SPMainQSync` → `cancelExportForFile:` + early `return` di banyak titik (262–267, 476–481, 645–650, 697–702, 725–730, 760–765, 783–788). Duplikasi panggilan delegate `sqlExportProcessWillBeginWritingData:` di 452 dan 470.

**UNKNOWN**
- Lokasi definisi `SPSQLExportInsertDivider` (hanya dirujuk di sini) dan `SPCurrentTimestampPattern` (1006).
- Kenapa `sqlExportProcessWillBeginWritingData:` dikirim **dua kali** (452, 470).
