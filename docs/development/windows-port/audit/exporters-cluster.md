# Audit: Exporter cluster

Audit method-by-method klaster `Source/Controllers/DataExport/`. `Start`/`End` = nomor
baris di file masing-masing. Baseline commit `02c910932`.

File: `Exporters/SPCSVExporter.m` (406), `Exporters/SPDotExporter.m` (240),
`Exporters/SPXMLExporter.m` (342), `Model/SPExportFile.m` (238).

---

## 1. `SPCSVExporter.m` (406)

Serialisasi tabel atau array ke output pemisah-delimiter (CSV/TSV/kustom), dengan deteksi kolom numerik untuk memutuskan quoting per-sel.

### Ivar/properti
`.h:45–57` ivar: `csvDataArray`, `csvTableName`, `csvFieldSeparatorString`, `csvEnclosingCharacterString`, `csvEscapeString`, `csvLineEndingString`, `csvNULLString`, `csvOutputFieldNames`, `csvTableData`. Property `.h:62–107`. `@synthesize` 43–52. **Tak ada konstanta.**

### `#pragma mark`
**Tidak ada.**

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 61 | 70 | `initWithDelegate:` | Simpan delegate, assert conformance protokol |
| 75 | 404 | `exportOperation` | Loop export utuh: validasi, streaming query, escaping per-sel, write |

### Temuan
- **Method raksasa tunggal.** `exportOperation` ~330 baris (75–404); semua state lokal di stack frame (77–92). Tak ada helper.
- **Urutan cek guard**: (a) ada tabel-atau-array + nama database (95–100); (b) separator/escape/line-ending non-nil (103–108); (c) ketiganya bukan string kosong (111–116).
- **Aturan quoting**: `quoteFieldSeparators` diset saat enclosing string kosong (89) — mode delimiter-only. Sel dibungkus `csvEnclosingCharacterString` kecuali numerik (358–365); sel kosong emit enclosing ganda (302–305).
- **Escaping**: precompute salinan (escape→escaped escape, escape+separator, escape+enclosing, escape+line-end) (200–203); ganti escape (332–335); ganti enclosing hanya bila escape ≠ enclosing (338–343); ganti separator + line-ending hanya bila `quoteFieldSeparators || csvCellIsNumeric` (346–355). Semua `NSLiteralSearch`.
- **Un-mangling** separator/line-ending: literal `\t`/`\n`/`\r` → kontrol nyata sebelum dipakai (162–178, 180–197).
- **Deteksi numerik**: dari array `tableColumnNumericStatus` (typegrouping `bit`/`integer`/`float` dari SPTableData) (143–152); else per-sel: NSData non-numerik (316–318), else `NSScanner scanFloat` + `isAtEnd` + aturan leading-zero (321–329). Baris header dipaksa non-numerik via `forceNonNumericRow` (308–310).
- **NULL**: `isNSNull` emit `csvNULLString` mentah tanpa enclosing + separator (276–282). ⚠️ Pakai `[csvRow count]` (bukan `csvCellCount`) untuk cek separator akhir (368) — potensi edge **UNKNOWN**.
- **Encoding**: sel NSData decode dengan `exportOutputEncoding`, fallback ASCII (285–292); geometry via `wktString` (294–296).
- **File writing**: akumulasi per-baris di `csvString`, tulis via inherited `writeString:` (375), `[[exportOutputFile exportFileHandle] synchronizeFile]` di akhir (397).
- **Cancellation/error**: loop cek `exportOutputFile.fileHandleError` (dispatch `cancelExportForFile:` via `SPMainQSync` di main, 219–224, 262–267) + `isCancelled` (226–233, 269–271) → cancel streaming query.
- **UNKNOWN**: apakah `connection` dijamin non-nil di 133/157.

---

## 2. `SPDotExporter.m` (240)

Emit diagram ER Graphviz DOT/HTML-record (tabel sebagai record node + edge FK dashed).

### Ivar/properti
`.h:45–55` ivar: `dotExportTables`, `dotExportCurrentTable`, `dotDatabaseHost`, `dotDatabaseName`, `dotDatabaseVersion`, `dotForceLowerTableNames`, `dotTableData`. Property `.h:61–96`. `@synthesize` 42–49.

### `#pragma mark`
`232`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 58 | 68 | `initWithDelegate:` | Simpan delegate, assert protokol, bersihkan current table |
| 70 | 230 | `exportOperation` | Bangun/tulis preamble DOT, node tabel, edge FK |
| 234 | 238 | `dealloc` | Nil `delegate` |

### Temuan
- **Doc header salah label**: `SPDotExporter.h:37` bilang `@class SPXMLExporter SPXMLExporter.m` (copy-paste), padahal kelas `SPDotExporter` (43).
- **Output**: butuh encoding `utf8mb4` (86); banner (88–95); `digraph "Database Structure" {` + label/labelloc/compound/node/fontname/ranksep/ratio/rankdir (97–105).
- **Node**: tiap tabel `subgraph "table_<name>"` (144) + HTML record `<TABLE BORDER=... >` (146–148); baris kolom iterasi `tableInfo[@"columns"]` embed `name`/`type` (153–156). Header color `#DDDDDD`, atau `#DDDDFF` untuk View (138–142).
- **FK/edge**: constraint dari `tableInfo[@"constraints"]`; origin columns + `ref_columns` dibaca tapi hanya **yang pertama** dipakai (186–187, komentar 183–185). Multi-kolom origin tambah `[ arrowhead=crow, arrowtail=odiamond ]` (191–193). Edge dikumpulkan di `fkInfo` lalu flush setelah semua node, `edge [ arrowhead=inv, arrowtail=normal, style=dashed, color="#444444" ]` (209–215). `ref_table` dipakai verbatim sedangkan kolom referensi di-lowercase (195).
- ⚠️ **Escaping minimal** — nama tabel/kolom diinterpolasi langsung ke string DOT/HTML dengan `stringWithFormat`; **tak ada** escaping quote maupun `<>`. Risiko injection/output rusak untuk nama memuat `"` atau metakarakter HTML (144–155).
- **File writing**: `writeString:` untuk preamble + tiap node (108, 162, 220); akhir `[[self exportOutputFile] close]` (223) — beda dari CSV/XML yang `synchronizeFile`.
- Cancellation/error sama pola CSV (115–120, 171–176; 123–126, 178–181). Tanpa streaming query. Progress `i * (exportMaxProgress / count)` (200–202), delegate `dotExportProcessProgressUpdated:` (203).

---

## 3. `SPXMLExporter.m` (342)

XML dalam dua dialek — MySQL (`mysqldump --xml`-like) atau "Plain" — opsional sertakan struktur dan/atau konten baris.

### Ivar/properti
`.h:44–52` ivar: `xmlDataArray`, `xmlTableName`, `xmlNULLString`, `xmlOutputIncludeStructure`, `xmlOutputIncludeContent`, `xmlFormat`. Property `.h:58–88`. `@synthesize` 41–47. Enum `SPXMLExportMySQLFormat`/`SPXMLExportPlainFormat` (92–93, dst).

### `#pragma mark`
**Tidak ada.**

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 56 | 65 | `initWithDelegate:` | Simpan delegate, assert protokol |
| 67 | 340 | `exportOperation` | Export XML utuh: guard, struktur, baris, escaping, write |

### Temuan
- **Single giant method** (~270 baris).
- **Dialek/guard**: return dini bila tak ada tabel/array, atau (MySQL + bukan struktur & bukan konten), atau (Plain + tanpa NULL string) (89–96).
- **Struktur (MySQL saja)**: `SHOW COLUMNS` untuk fields (116) + `SHOW TABLE STATUS LIKE` untuk options (119); nama tabel di-escape ganda untuk literal LIKE (117–118). Emit `<table_structure name=...>` + baris `<field .../>` (123–134) + `<options .../>` (138–155). Konten dibuka `<table_data ...>` (159–161).
- **Dialek**: MySQL emit `<field name="..." xsi:nil="true" />` untuk NULL (284–286) atau `>value</field>` (288). Plain emit wrapper `<tableName>`/`<custom>` (183–185, 327–329) + pasangan tag buka/tutup precomputed (176–180, 293–295).
- **Escaping**: nama kolom/tag pakai `HTMLEscapeString` (escape `& < > "`) (178–179, 184, 282, 328); konten sel pakai `XMLEscapeStringForContent` (escape `& < >`, pertahankan quote) (288, 294). Def: `SPStringAdditions.m:182` + `:208`. Distingsi atribut vs konten disengaja.
- **NULL**: perbandingan pointer `data == [NSNull null]` (266–268); NULL string hanya diPlain (270–272).
- **Encoding**: file ditulis encoding output (UTF-8 per prolog); NSData decode dengan `[connection stringEncoding]`, fallback ASCII (256–261); geometry via `wktString` (274–276).
- **File writing**: `writeString:` (163, 302, 325, 328); akhir `synchronizeFile` (333). **UNKNOWN**: tak ada prolog `<?xml ...?>` ditulis di sini — diasumsikan di tempat lain (lihat `SPExportController.m:1961`).
- Cancellation/error sama (199–204, 235–240; 207–214, 243–249). Progress seperti CSV (308–315), delegate `xmlExportProcessProgressUpdated:` (318).

---

## 4. `SPExportFile.m` (238)

Model satu file export abstrak — path, `SPFileHandle`, logika status pembuatan handle, write/close/delete, kompresi.

### Ivar/properti
`.h:45–52` ivar: `exportFilePath`, `exportFileNeedsCSVHeader`, `exportFileNeedsXMLHeader`, `exportFileHandle`, `exportFileHandleStatus`. Property `.h:58–89` (+`exportFileNeedsUserChosenDir` tanpa backing ivar + `fileHandleError`). `@synthesize` 44–50. Class extension 36–40.

### `#pragma mark`
`52/53 (Initialisation), 91/92 (General Methods), 207/208 (Private API)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 38 | 38 | `_createFileHandle` (deklarasi) | Forward declaration privat |
| 62 | 65 | `exportFileAtPath:` (class) | Factory autoreleased |
| 74 | 89 | `initWithFilePath:` | Set path, status=-1, error nil, header flags NO |
| 97–125 | | `close` / `delete` | Tutup/hapus file |
| 134 | 145 | `writeData:` | Tulis NSData via handle; log + set `fileHandleError` saat gagal |
| 154 | 189 | `createExportFileHandle:` | Resolve exists/overwrite/writable + buat handle |
| 197 | 205 | `setCompressionFormat:` | Forward format kompresi ke handle, else raise |
| 215 | 236 | `_createFileHandle` | Buat file kosong, buka `SPFileHandle`, bersihkan saat gagal |

### Temuan
- **Jalur write**: `writeData:` → `[[self exportFileHandle] writeData:data]` (144); handle hilang → log + simpan path di `fileHandleError` (136–142) — flag yang di-poll tiga exporter.
- **Pembuatan handle**: `createExportFileHandle:` raise `NSInternalInconsistencyException` bila path kosong (157–162); file ada + overwrite NO → `SPExportFileHandleExists` (179–181); overwrite tapi tak writable → `Failed` (171–177); else buat. `_createFileHandle` pakai `NSFileManager createFileAtPath:contents:[NSData data]` (219) lalu `SPFileHandle fileHandleForWritingAtPath:` (226); hapus file bila handle gagal dibuka (228–233).
- Status awal sentinel `-1` (= `SPExportFileHandleInvalid`, `SPConstants.h:229`) (80).
- `exportFileNeedsCSVHeader`/`XMLHeader`/`UserChosenDir` default NO (83–85). **UNKNOWN**: di mana `exportFileNeedsUserChosenDir` di-set (property tanpa ivar).
- `delete` butuh path + handle hidup (113), tutup dulu (116), `removeItemAtPath:error:nil` (121) — error diabaikan.
- **UNKNOWN**: `fileHandleError` tak di-reset setelah write sukses; tak ada dealloc; apakah caller bergantung pada `exportFileHandleStatus` vs `fileHandleError` untuk abort tak dapat ditentukan dari klaster ini.

## Ringkasan lintas-file
- **Hanya dua exporter nyata** (CSV, XML) melakukan streaming baris; Dot menulis skema/graf ER. Ketiganya menyatukan logika di satu `exportOperation` tanpa helper.
- **Idiom cancellation bersama** (identik): cek `exportOutputFile.fileHandleError` → `SPMainQSync` `cancelExportForFile:` → cek `isCancelled` + cancel streaming query (`SPCSVExporter.m:219–233`, `SPXMLExporter.m:199–214`, `SPDotExporter.m:115–126`).
- **Divergensi escaping**: CSV escaping multi-pass literal terkonfigurasi; XML pakai `HTMLEscapeString` (nama/atribut) + `XMLEscapeStringForContent` (konten); Dot **tanpa** escaping (risiko).
- **Divergensi finalisasi**: CSV/XML `synchronizeFile` handle mentah (`:397`, `:333`); Dot `SPExportFile close` (`:223`).
- **Base bersama** `SPExporter` → `writeString:` (`SPExporter.m:119–124`), `exportMaxProgress`, `exportOutputFile`, `exportOutputEncoding`, `setExportProgressValue:`, `isCancelled`.
