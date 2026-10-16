# Audit: `SPDataImport.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/DataImport/SPDataImport.m`. Baseline commit `02c910932`.

- File: 1760 baris; header `SPDataImport.h` 151 baris.
- Peran: controller yang menjalankan import file & clipboard SQL/CSV untuk
  accessory view import/export — stream file, parse SQL/CSV, jalankan query yang
  dibentuk di background thread, dan agregasi error.
- Total: **22 implementasi method Obj-C, tanpa fungsi C static** (bertentangan
  dengan dugaan "banyak fungsi static"; `grep '^static'` kosong). Angka "~27
  method" termasuk 5 deklarasi privat di class-extension (58–62).

## 1. Header/import

`1–31` header MIT ("Created by Lorenz Textor … 2002"). `32–52` import (termasuk
`SPSQLParser`, `SPCSVParser`, `SPFieldMapperController`, `SPFileHandle`,
`SPEncodingPopupAccessory`, `SPQueryController`, `sequel-ace-Swift.h`).

## 2. Inventaris ivar/property

### Konstanta/macro lokal (`.m`)
Baris 54: `#define SP_FILE_READ_ERROR_STRING NSLocalizedString(@"File read error", …)` — dipakai di 473, 522, 896, 923. Tak ada `#define`/`static` lain, tak ada singleton `dispatch_once`.

### Class extension (`.m:56–66`)
`@interface SPDataImport () <SATextViewDelegate>` (56) — conformance tapi **tak ada method `SATextViewDelegate` yang diimplementasi** (lihat §5). 5 deklarasi privat (`_startBackgroundImportTaskForFilename:`, `_importBackgroundProcess:`, `_closeAndStopProgressSheet`, `_updateProgressForBytesProcessed:…`, `_getLineEndingForFile:`) (58–62). `@property fileManager` (64). `@synthesize fileManager`, `tableDocumentInstance`, `tablesListInstance` (70–72).

### Ivar `.h` (48–123)
| Baris | Grup |
| --- | --- |
| 51–55 | outlet dari `DBView.xib`: `tableDocumentInstance` (weak), `tablesListInstance`, `tableSourceInstance`, `tableDataInstance`, `customQueryInstance` |
| 57–63 | outlet sheet error + progress |
| 69–77 | outlet accessory (dimuat `awakeFromNib` via `ImportAccessory.xib`): `importView`, `importTabView`, `importFieldNamesSwitch`, field terminator/enclosed/escaped/lines, `importFormatPopup`, `importEncodingPopup` |
| 80–85 | `importSQLErrorHandlingPopup`; sheet clipboard |
| 87 | `mySQLConnection` |
| 90–108 | state import/field-mapper: `fieldMappingImportArray`, `…IsPreview`, `fieldMappingTableColumnNames`, `fieldMappingArray`, `fieldMappingGlobalValueArray`, `fieldMappingTableDefaultValues`, `fieldMapperOperator`, `selectedTableTarget`, `selectedImportMethod`, `lastFilename`, `csvImportHeaderString`, `csvImportTailString`, `fieldMapperSheetStatus`, `numberOfImportDataColumns`, `fieldMappingArrayHasGlobalVariables`, `csvImportMethodHasTail`, `insertRemainingRowsAfterUpdate`, `importMethodIsUpdate`, `importIntoNewTable` |
| 110–115 | `prefs`, `_Atomic(BOOL) progressCancelled`, `importProgressReporter`, `mainNibLoaded` |
| 117–122 | array/index-set kolom khusus: `geometryFields`/`…MapIndex`, `bitFields`/`…MapIndex`, `nullableNumericFields`/`…MapIndex` |
| 42–46 | `typedef enum SPFieldMapperSheetStatus` (`InProgress=1`, `Completed=2`, `Cancelled=3`) |

## 3. `#pragma mark` (urut baris)

74/75 (Initialisation), 138/139 (IBAction methods), 166/167 (Import construction methods), 337/338 (SQL import), 758/759 (CSV import), 1543/1544 (Import delegate notifications), 1592/1593 (Other), 1643/1644 (Private API). Tak ada grup fungsi static.

## 4. Tabel method (22, urut baris)

| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 77 | 110 | `init` | Alokasi ivar, index set, default; set `fileManager` |
| 112 | 136 | `awakeFromNib` | Muat nib `ImportAccessory`, setup popup encoding, tab view tanpa tab |
| 144 | 147 | `changeFormat:` | Pindah tab accessory sesuai popup format |
| 152 | 155 | `cancelProgressBar:` | Set flag `progressCancelled` untuk abort loop import |
| 160 | 164 | `closeSheet:` | Akhiri sheet dengan tag sender |
| 172 | 247 | `importFromClipboard` | Setup text view clipboard; tulis pasteboard ke file temp; mulai import |
| 252 | 326 | `importFile` | NSOpenPanel, warn tabel existing, mulai import background |
| 331 | 335 | `startSQLImportProcessWithFile:` | Paksa format SQL + mulai import background untuk filename |
| 350 | 756 | `importSQLFile:` | Stream file, parse statement SQL, eksekusi query, agregasi error |
| 774 | 1205 | `importCSVFile:` | Stream file, parse baris CSV, bangun field mapping, jalankan INSERT/UPDATE |
| 1214 | 1317 | `buildFieldMappingArrayWithData:isPreview:ofSoureFile:databaseName:` | Tampilkan sheet field mapper, tangkap mapping, deteksi kolom geometry/bit/nullable |
| 1323 | 1456 | `mappedUpdateSetStatementStringForRowArray:` | Bangun klausa SET + WHERE untuk metode UPDATE |
| 1462 | 1541 | `mappedValueStringForRowArray:` | Bangun string `(...)` VALUES untuk satu baris CSV |
| 1549 | 1590 | `panelSelectionDidChange:` | Auto-pilih format SQL/CSV + line ending dari ekstensi file |
| 1598 | 1611 | `setConnection:` | Simpan koneksi, setup `prefs` + font error-view (init sekali) |
| 1616 | 1627 | `toolbarSelectableItemIdentifiers:` | Kembalikan identifier item toolbar sebagai selectable |
| 1632 | 1641 | `showErrorSheetWithMessage:` | Tampilkan sheet error import di main thread |
| 1651 | 1662 | `_startBackgroundImportTaskForFilename:` | Detach thread background memanggil `_importBackgroundProcess:` |
| 1667 | 1675 | `_importBackgroundProcess:` | Worker background; dispatch ke importer SQL/CSV |
| 1680 | 1689 | `_closeAndStopProgressSheet` | Akhiri sheet progress + reset progress bar |
| 1700 | 1714 | `_updateProgressForBytesProcessed:totalBytes:fileHandle:` | Update progress async throttled via `SAImportProgressReporter` |
| 1719 | 1758 | `_getLineEndingForFile:` | Shell `/usr/bin/file` untuk infer LF/CR/CRLF |

## 5. Temuan menonjol / UNKNOWN

**Loop import SQL — `importSQLFile:` (350–756)**
- Streaming reader: chunk 1 MiB (`fileChunkMaxLength = 1024 * 1024`, 370), baca hingga `allDataRead` (454–680). Scan line-ending manual pada `0x0A`/`0x0D` (491–536) diumpan `SPSQLParser` (449–452, 529).
- Ekstraksi query `trimAndReturnStringToCharacter:';'` (558); query ber-CR dinormalisasi via `SPSQLParser normaliseQueryForExecution:` (563–567).
- **Agregasi error:** ditambah ke satu `NSMutableString *errors` (369) via `appendFormat:@"[ERROR in query %ld] %@\n"` (594, 703). Popup non-blocking menawarkan Continue/Ignore All/Stop (628–653). Penanganan khusus utf8mb4/ER 1115 (598–625).
- Restore encoding + sql_mode di exit dan tiap jalur error (464–469, 506–511, 718–723).
- Lookup `lower_case_table_names` via `SASQLDatabaseContext` (575–584, 686–695); `databaseName` diupdate setelah query sukses (656–662, 705–711).
- Cleanup: refresh DB list, update tabel, re-query struktur background, notifikasi sukses (735–748).

**Loop import CSV — `importCSVFile:` (774–1205)**
- **Tiga tanggung jawab terjalin dalam satu loop** (komentar 854–858): (1) baca+parse baris, (2) tampilkan field-mapper sekali saat ≥100 baris, (3) bangun/jalankan query INSERT atau UPDATE.
- `SACSVImportStreamDecoder` (777, 884) untuk hindari pemotongan mid-character (882–883).
- Batching: `csvRowsPerQuery = 1000` (789); cap panjang query 250000 (1045). `csvImportTailString` disisip untuk tail gaya `ON DUPLICATE KEY UPDATE` (1049–1050).
- Dua strategi query: INSERT (1039–1052, statement multi-row batch); UPDATE (1053–1112, per-baris `mappedUpdateSetStatementStringForRowArray:` (1071), fallback INSERT baris sisa saat `rowsAffectedByLastQuery == 0` (1090–1105)).
- **Agregasi error:** saat batch gagal, jalankan ulang baris individual untuk pinpoint baris gagal (1115–1144), append `[ERROR in row %ld]` (1082, 1102, 1134) ke `errors` (782). Logging console opsional digerbangi `SPConsoleEnableImportExportLogging` (1085–1087, 1118–1120, 1136–1138).
- Post-import: pilih tabel baru atau reload content (1179–1203).

**Field mapping — `buildFieldMappingArrayWithData:…` (1214–1317)**
- Busy-wait **poll 1-based `usleep(100000)`** sampai sheet field-mapper kembali (1255–1257) — pola non-ideal. Komentar 1239: objek UI tak boleh tertangkap autoreleasepool background.
- Tolak `> 512` kolom untuk cegah hang (1224–1229).
- Deteksi nama kolom geometry/bit/nullable-numeric dari `SPTableData` (1300–1309).

**`mappedValueStringForRowArray:` / `mappedUpdateSetStatementStringForRowArray:`**
- Logika substitusi variabel global `$N` (regex `(?<!\\\\)\\$(\d+)`) **terduplikasi tiga kali** (1333/1352–1381, 1404–1434, 1469/1482–1511) — target refactor jelas.
- Placeholder out-of-range mengemit literal `GLOBAL_SQL_EXPRESSION_ERROR` (1377, 1429, 1507).

**UNKNOWN**
1. Conformance `<SATextViewDelegate>` (56) **tanpa method delegate yang diimplementasi** di file ini — conformance mati atau disediakan di tempat lain.
2. Doc comment `buildFieldMappingArrayWithData:` berkata "returns false on failure" (1210), tapi jalur data kosong **return YES** (1221) sementara `>512 kolom` return NO (1228) dan cancellation return NO (1263) — **semantik return tidak konsisten**.
3. `toolbarSelectableItemIdentifiers:` (1616) menduplikasi identifier `[toolbar items]` verbatim; **UNKNOWN** NSToolbar mana yang dilayani.
4. `importFile` (303–305) ada baris kosong nyasar + cek ulang `importFileName == nil` setelah assert non-nil (305, 309, 314) — guard redundan.
5. `SP_FILE_READ_ERROR_STRING` dipakai untuk error baca maupun encoding (4 lokasi).
