# Export data dan printing

Kontrak source untuk export (SQL/CSV/XML/Dot) dan printing. Sitasi `file:line`
merujuk checkout `02c910932`.

File: `Source/Controllers/DataExport/SPExportController.m/.h` (3982 baris);
`Exporters/` (`SPExporter`, `SPSQLExporter`, `SPCSVExporter`, `SPXMLExporter`,
`SPDotExporter`, `SPHTMLExporter`, `SPPDFExporter` + protokol);
`Model/SPExportFile.m/.h`; `SPExportUtilities.m/.h`, `SPExportFileNameTokenObject.m/.h`,
`SAExportOutputEncoding.swift`; `Source/Other/Utility/SAPrintUtility.swift`;
`Source/Views/SAWebView.swift`; `ExportDialog.xib`.

## 1. Format, sumber, status aktif

**Tab format atas `exportTypeTabBar`** — hanya empat (`ExportDialog.xib:82-107`): `SQL`, `CSV`, `XML`, `Dot`. Tidak ada tab HTML/PDF/Excel di sini.

**Tab opsi per-format `exportOptionsTabBar`** (`ExportDialog.xib:377-381`): `SQL`, `Excel`, `CSV`, `HTML`, `XML`, `PDF`, `Dot`. Panel HTML dan PDF **kosong** (`ExportDialog.xib:710-716`, `793-799`).

**Format aktif**: SQL, CSV, XML, Dot.
**HTML/PDF/Excel tidak aktif**: exporter-nya stub — `SPHTMLExporter.m:45-54` dan `SPPDFExporter.m:45-54` hanya implement `initWithDelegate:` (tanpa `exportOperation`). Controller menaikkan exception: `[NSException raise:NSInvalidArgumentException format:@"unsupported exportType=%lu",exportType];` (`1447-1452`, juga `2411-2416, 3333-3340, 3354-3361, 3492-3499, 3513-3520`). Flag HTML/PDF di-comment out di logika enable: `1030-1034`.

**Enum tipe export** (`SPConstants.h:66-75`): `SPSQLExport=0, SPCSVExport=1, SPXMLExport=2, SPDotExport=3, SPPDFExport=4, SPHTMLExport=5, SPExcelExport=6, SPAnyExportType=NSUIntegerMax`.

**Sumber input** — `ExportDialog.xib:954-971`: `Filtered Results`, `Query Results`, `Tables`
(enum `SPConstants.h:78-82`: `SPFilteredExport=0, SPQueryExport=1, SPTableExport=2`).

Seleksi data (`1388-1430`): `SPFilteredExport` → `[tableContentInstance currentDataResultWithNULLs:YES hideBLOBs:NO]`;
`SPQueryExport` → `[customQueryInstance currentDataResultWithNULLs:YES truncateDataFields:NO]`;
`SPTableExport` → array tabel (SQL termasuk proc/func, `740-770`).

Aturan enable/disable (`_switchTab` `1014-1094`): SQL memaksa hanya `SPTableExport` (`1047-1055`); Dot selalu `SPTableExport` (`1023`, `setExportInput:` `471-477`); Filtered/Query aktif hanya bila `currentResult.count > 1` (`1059-1060`).

Default saat dibuka dari main window (`exportData` `384-414`): Content → CSV + `SPFilteredExport`;
Custom Query → CSV + `SPQueryExport`; else SQL + `SPTableExport`.

## 2. Sintaks output per format

### 2.1 SQL (`SPSQLExporter.m`) — ekstensi `"sql"` (`SPConstants.m:77`)

Header (`156-252`): BOM UTF-8 bila dipilih `"\xef\xbb\xbf"` (`152-154`); koneksi dipaksa `utf8mb4` (`159`); banner `"# Sequel Ace SQL dump"`, `# <Version> <CFBundleVersion>`, `# <Host>: %@ (MySQL %@)`, `# <Database>: %@`, `# <Generation Time>: %@` (`164-168`); `SET NAMES utf8mb4;` (`172-175`); `/*!40014 SET …FOREIGN_KEY_CHECKS=0 */;` (`177`); `/*!40101 SET @OLD_SQL_MODE='NO_AUTO_VALUE_ON_ZERO', SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;` (`251`); `/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;` (`252`). `sql_mode` lama via `SHOW VARIABLES LIKE 'sql_mode'` (`213`); `SET SQL_MODE=''` (`227`); restore di `endCleanup:` `SET SQL_MODE=%@` tickQuoted (`913-916`).

Per tabel (`259-691`):
- Header komentar tabel: `# <Dump of table> %@\n# ------------------------------------------------------------\n\n` (`335`).
- DROP: `DROP TABLE IF EXISTS `tbl`;\n\n` (selalu `"TABLE"`) (`339-341`).
- Struktur = output `SHOW CREATE TABLE` (`299, 317`); strip `AUTO_INCREMENT=[0-9]+ ` opsional (`347-349`); lalu `createTableSyntax ;\n\n` (`351-352`).
- Content (`356-629`): `LOCK TABLES `tbl` WRITE;\n/*!40000 ALTER TABLE `tbl` DISABLE KEYS */;\n\n` (`458`); `INSERT INTO `tbl` (`c1`, `c2`)\nVALUES` (`463`); baris pertama `\n\t(`, berikutnya `,\n\t(` (`519-524`); statement baru `;\n\nINSERT INTO `tbl` (…)\nVALUES\n\t(` (`510-514`); terminator `;\n\n` (`608`); `/*!40000 ALTER TABLE `tbl` ENABLE KEYS */;\nUNLOCK TABLES;\n` (`611-612`).
  - Bentuk nilai: `NULL` (`534`); integer/float/bit mentah (`407-415, 538-540`); BIT → `b'%@'` (`545`); kolom hex → `HEX(col)` di-fetch, ditulis `X'%@'` (`422, 550`); geometry → `escapeAndQuoteData:[object data]` (`554-555`); nol-panjang → `''` (`559-560`); BLOB hex bila opsi else decode string (`564-586`) else `escapeAndQuoteString:` (`590`).
  - Insert divider (`85-88`): `SPSQLInsertEveryNDataBytes=0` (default, `83`) vs `SPSQLInsertEveryNRows=1`; ambang `sqlInsertAfterNValue` default `250000` (`84`), dibanding sebagai `queryLength >= (sqlInsertAfterNValue*1024)` byte atau `rowsWrittenForCurrentStmt == sqlInsertAfterNValue` (`507-508`). UI default `250` KiB (`261`); popup `KiB of data`/`Rows` (`ExportDialog.xib:483-489`).
- Trigger (bila structure) (`632-686`): `DELIMITER ;;\n` (`640`), per trigger `/*!50003 SET SESSION SQL_MODE="…" */;;\n/*!50003 CREATE */ /*!50017 DEFINER=`u`@`h` */ /*!50003 TRIGGER `…` <Timing> <Event> ON `tbl` FOR EACH ROW <Statement> */;;\n` (`663-671`), footer `DELIMITER ;\n/*!50003 SET SESSION SQL_MODE=@OLD_SQL_MODE */;\n` (`674`).
- View: ditunda; `DROP TABLE IF EXISTS `v`; DROP VIEW IF EXISTS `v`;\n\n` + `Create View` yang dipretty (`306-312`); placeholder `CREATE TABLE `v` (… ) ENGINE=MyISAM` (`950-1021`); diemit kemudian (`713-716`).
- Proc/func (`721-882`): header `--\n-- Dumping routines (PROCEDURE|FUNCTION) for database 'db'\n--\nDELIMITER ;;\n\n` (`754-755`); per item `# Dump of <PROCEDURE|FUNCTION> name\n# ---…\n\n` (`808`), `/*!50003 DROP <TYPE> IF EXISTS `name` */;;\n` (`812-813`), `/*!50003 SET SESSION SQL_MODE="…"*/;;\n` (`842`), `/*!50003 CREATE*/ /*!50020 DEFINER=`u`@`h`*/ /*!50003 <body> */;;\n\n/*!50003 SET SESSION SQL_MODE=@OLD_SQL_MODE */;;\n` (`865`); footer `DELIMITER ;\n` (`870`).
- Footer restore (`885-896`): `/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;`, `/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;`, `/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;`, `/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;`, `…_RESULTS…`, `…COLLATION_CONNECTION…`.
- Error: `# Error: %@\n\n\n` (`325, 439`) dan `# Error: %@\n` bila "include errors" (`626, 683, 835, 851, 879`).

Opsi SQL UI (`SPExportController.h:102-110`; xib `382-514`): Structure / Content / DROP TABLE syntax / Errors / Add UTF-8 BOM / Output BLOB as hex / Include GENERATED columns / New INSERT statement every: + KiB|Rows + Auto increment value.

### 2.2 CSV (`SPCSVExporter.m`) — `csv`/`tsv` (delimiter tab → `tsv`, `2403`)

Default opsi dari xib: delimiter `,` (alternatif `;`, `\t`) (`ExportDialog.xib:609-616`); enclosure `"` atau kosong (`626-632`); escape `"` atau `\` (`639-645`); line terminator `\n`, `\r\n`, `\r` (`661-668`); "Put field names in first row" on; NULL default `NULL` via `SPNullValue` (`291`).

Escape-sequence `\\t`→tab, `\\n`→LF, `\\r`→CR diterapkan ke separator dan line ending (`162-197`). Aturan quoting: NULL → `csvNULLString` tanpa enclosure (`276-282`); sel kosong → enclosure dua kali (`302-305`); sel numerik (integer/float/bit) tanpa quote, else di-enclose (`312-329, 358-365`); escape char digandakan (`200, 332-335`); enclosure di-escape hanya bila `escape != enclosure` (`338-343`); separator/line-end di-escape saat `quoteFieldSeparators` (enclosure kosong) atau numerik (`346-355`). Query `SELECT COUNT(1) FROM `tbl`` lalu `SELECT * FROM `tbl`` streaming (`157-158`).

### 2.3 XML (`SPXMLExporter.m`) — `xml`; `SPXMLExportMySQLFormat=0`, `SPXMLExportPlainFormat=1` (`SPConstants.h:91-94`)

Header (`SPExportController.m:1942-1976`): prolog `<?xml version="1.0" encoding="utf-8" ?>\n\n` (`1946`); komentar `<!--\n-\n- Sequel Ace XML dump\n- <Version> …\n- <Host>: %@ (MySQL %@)\n- <Database>: %@\n- <Generation Time> Time: %@\n-\n-->\n\n` (`1947-1954`); MySQL format table → `<mysqldump xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">\n<database name="db">\n\n` (`1961`), query/filtered → `<resultset statement="<escaped query>" xmlns:xsi="…">\n\n` (`1966`); Plain → `<dbName>\n\n` (`1972`). Ditulis `dataUsingEncoding:NSUTF8StringEncoding` (`1975`).

Body: struktur MySQL `<table_structure name="tbl">` (`123`), per kolom `<field field="…" type="…" null="…" key="…" default="…" extra="…" />` (`127-133`), `<options name= engine= version= row_format= rows= … />` (`138-153`), `</table_structure>` (`155`); content `<table_data name="tbl">` (`160`), baris `\t<row>\n` (`231`) dengan `<field name="col">value</field>` (`282-288`; NULL → ` xsi:nil="true" />`), `\t</row>\n\n` (`299`), `\t</table_data>\n\n` (`325`). Plain: `\t<tbl>\n` (`184`), field `\t\t<col>`…`</col>\n` (`178-179`). Penutup (`3816, 3838`): `</database>\n</mysqldump>\n`, `</resultset>\n`, atau `</dbName>\n`.

### 2.4 Dot (`SPDotExporter.m`) — `dot`

Header (`88-105`): `// ************************************************************\n`, `// Generated by: Sequel Ace\n`, `// Version <CFBundleVersion>\n//\n`, `// <homepage>\n// <devURL>\n//\n`, `// Host: %@ (MySQL %@)\n`, `// Database: %@\n`, `// Generation Time: %@\n`, `// *******************************…`. Koneksi dipaksa `utf8mb4` (`86`).
Graph: `digraph "Database Structure" {\n` (`97`); `\tlabel = "ER Diagram: %@";\n` (`98`); `\tlabelloc = t;\n`, `\tcompound = true;\n`, `\tnode [ shape = record ];\n`, `\tfontname = "Helvetica";\n`, `\tranksep = 1.25;\n`, `\tratio = 0.7;\n`, `\trankdir = LR;\n` (`99-105`).
Per tabel (`144-160`): `\tsubgraph "table_%@" {\n`, `node [ shape = "plaintext" ]`, `"link" [ label=<\n<TABLE BORDER="0" …>\n<TR><TD COLSPAN="3" BGCOLOR="%s">%@</TD></TR>\n` (header `#DDDDDD`, atau `#DDDDFF` untuk view, `138-142`); kolom `col:<FONT FACE="Helvetica-Oblique" POINT-SIZE="10">type</FONT>` (`155`). Edge (`195, 209-215`): `edge [ arrowhead=inv, arrowtail=normal, style=dashed, color="#444444" ];`; FK `%@:%@ -> %@:%@ %@;` dengan ekstra ` [ arrowhead=crow, arrowtail=odiamond ]` (`192`); tutup `}\n` (`217`).

## 3. Token nama file, encoding, penulisan, progress/cancel

### 3.1 Sistem token nama file
Token (`SPConstants.m:242-251`): `database, host, date, year, month, day, time, time24, favorite, table`. Nama tampil lokal (`222-233`). Pooled token (`updateAvailableExportFilenameTokens` `2304-2341`); `table` dapat dihapus via `isTableTokenAllowed` (`2270-2299`). Ekspansi (`expandCustomFilenameFormatUsingTableName:` `2443-2515`): host, database, table, `date`→`shortStyleNoTimeFormatter`, year `yyyy`, month `MM`, day `dd`, time→`shortStyleNoDateFormatter`, time24 `HH:mm:ss`, favorite→nama koneksi; lalu `:`→`-`, `/`→`-`, `.`→`-` (`2494-2509`); kosong→default.
Default nama (`generateDefaultExportFilename` `2360-2386`): Filtered → `"%@_view"`; Query → `"query_result"`; Table → `"%@_%@"` (db_`yyyy-MM-dd`). Ekstensi (`2393-2432`): sql/xml/dot/csv(atau tsv); kompresi menambah `.gz`/`.bz2`.
Serialisasi field token: plist type `SPExportCustomFileNameTokenPlist` (`SPConstants.m:72`); baca/tulis (`2693-2747`).

### 3.2 Encoding output
`SAExportOutputEncoding.outputEncoding(for:connectionEncoding:)` (`SAExportOutputEncoding.swift:41-49`): `.sql/.dot/.xml` → UTF-8; `.csv` → encoding koneksi. Base `writeString:` encode dengan `exportOutputEncoding` (`SPExporter.m:119-124`; default UTF-8 `64`). Set via `[exporter setExportOutputEncoding:[self outputEncodingForCurrentExportType]]` (`1701`).

### 3.3 Penulisan/streaming
`SPExportFile`: buat file + `SPFileHandle` (`215-236`); status `SPExportFileHandleInvalid=-1/Created=0/Failed=1/Exists=2` (`SPConstants.h:227-233`); `writeData:` no-op + catat `fileHandleError` bila tak ada handle (`134-145`); kompresi via `setCompressionFormat:` (`197-205`); `SPNoCompression/SPGzipCompression/SPBzip2Compression` (`SPConstants.h:212-217`). Query streaming `streamingQueryString:...useLowMemoryBlockingStreaming:` (`SPSQLExporter.m:449`, `SPCSVExporter.m:158`, `SPXMLExporter.m:111`). File kosong dibuat sebelum run (`1711-1735`); header CSV/XML ditulis per file (`1717-1722, 1904-1935, 1942-1976`); `synchronizeFile` (`SPCSVExporter.m:397`, `SPXMLExporter.m:333`).

### 3.4 Progress & cancel
`NSOperationQueue` (`SPExportController.h:183`); exporter subclass `NSOperation` (`SPExporter.m:74-84`). Progress `exportProgressValue`/`exportMaxProgress` (`SPExporter.h:95-103`). Cancel (`cancelExport:` `530-569`): set `exportCancelled`, `cancelAllOperations`, `[connection cancelCurrentQuery]`, hapus file cache + restore encoding (`577-599`). Exporter polling `[self isCancelled]` + `exportOutputFile.fileHandleError` (`520-524`). Label: `Exporting %@`/`Initializing...`/`Writing...`/`Table %lu of %lu (%@): Fetching data...` (`1317-1318, 3659, 3713, 3765, 3775, 3794, 3903, 3910`).

Persistensi: plist content type `export settings` (`SPConstants.m:83`), versi 1 (`3188-3189`), `exportCurrentSettings:`/`importCurrentSettings:` (`3097-3095`).

## 4. Printing

### 4.1 View yang mendukung print
`printDocument` (`SPDatabaseDocument.m:6070`) → `generateHTMLForPrinting` (`6155`) bercabang per `currentlySelectedView` (`6161-6262`):
- Structure → heading `Table Structure`/`View Structure` (`6177-6180`); termasuk index (`6189-6200`).
- Content → `Table Content` (`6207`); pakai `currentDataResultWithNULLs:NO hideBLOBs:YES` (`6205`).
- Custom Query → `Query Result` (`6222`); `customQueryInstance currentResult` (`6220`).
- Relations → `Table Relations` (`6237`); Triggers → `Table Triggers` (`6251`).
- Status → `generateTableInfoHTMLForPrinting` (`6099, 6289-6321`), heading `Table Information` (`6300`).

Validasi menu (`3030-3038`): aktif bila db + satu tabel terpilih, atau Custom Query aktif.
`SPTextView printDocument:` (`SPTextView.m:1205-1239`) pakai `NSPrintOperation printOperationWithView:`.

### 4.2 Warning limit baris
`SPDatabaseDocument.m:6070-6090` — hanya `SPTableViewContent`: `rowLimit = [prefs integerForKey:SPPrintWarningRowLimit]` (`6074`); key `PrintWarningRowLimit` (`SPConstants.m:220`), **default 1000** (`PreferenceDefaults.plist:180-181`); `resultRows = currentResult.count - 1` (`6077`); bila `resultRows > rowLimit` tampilkan warning (`6079-6085`).

### 4.3 HTML untuk print
`MGTemplateEngine` + `ICUTemplateMatcher` (`6265-6267`); template `SPPrintTemplate` (`SPConstants.m:90`) untuk view, `SPTableInfoPrintTemplate` (`SPConstants.m:91`) untuk status. Variabel: `c` (username/hostname/database/port/table/version `6366-6394`), `columns`, `rows`, `indexes`, `indexColumns`, `hasIndexes`, `heading`, `font` (`Lucida Grande` atau font editor `6306-6309`), `gridlines` = `1px solid #CCCCCC` atau `none` (`6272`).

### 4.4 Pipeline print
1. `printDocument` → cek row-limit → `startPrintDocumentOperation` (`6095`): `startTaskWithDescription:@"Generating print document..."`, spawn `printThread` (`6102-6107`), aktifkan cancellation `Cancel` (`6105`).
2. HTML dibangun off-main; cancel dicek (`6277-6280`).
3. `loadPrintWebViewWithHTMLString:` (`6129`) → `SAHTMLPrintRenderer printHTMLString:completionHandler:` (`6133-6149`).
4. `SAHTMLPrintRenderer` (`SAPrintUtility.swift:156-219`) memuat `WKWebView` offscreen 800×1100 (`177`), pada `didFinish` bangun operation via `SAPrintUtility.printOperation(for:)` (`194`); `invalidate` (`186`).
5. `SAPrintUtility.printOperation(for:)` (`59-80`): `shouldPrintBackgrounds` dari pref `PrintBackground` (`63`, default true `PreferenceDefaults.plist:176-177`); `configuredPrintInfo()` (`30-57`) margin simetris, `horizontalPagination = .fit`, `verticalPagination = .automatic`, `isVerticallyCentered = false`; panel opsi `[.showsOrientation, .showsScaling, .showsPaperSize]` (`75-76`).
6. `runOperationModalForWindow:` (`6148`); `endTask` dulu (`6144`); renderer di-invalidate saat tutup (`6418`).

`SAWebView.swift` menyediakan `WKWebView` hosted (zoom, find, navigasi) untuk window output bundle HTML; printing-nya dilakukan controller via `SAPrintUtility` (`SABundleHTMLOutputWindowController.swift:381`).

## 5. String yang terlihat pengguna (terpilih, dikutip persis)

**ExportDialog.xib**: tab `SQL`/`CSV`/`XML`/`Dot` (`:83-101`); opsi `SQL Excel CSV HTML XML PDF Dot` (`:382,516,565,710,716,793,799`); `Export` (`:115`); `Customize Filename` (`:887`); `New file per table` (`:831`); input `Filtered Results`/`Query Results`/`Tables` (`:962-964`); SQL `Structure`/`Content`/`Errors`/`Add UTF-8 BOM to output`/`Output BLOB fields as hex`/`Include GENERATED columns`/`DROP TABLE syntax` (`:390-505`); CSV `Fields:`/`Lines:`/`Wrap`/`Terminate`/`Escape`/`Put field names in first row`/`NULL values:` (`:573-692`); XML `MySQL Schema`/`Plain Schema` (`:772-773`); Dot `Use case-insensitive links` (`:807`); `Low memory export (may block server)` (`:162`); `Output compression:` (`:170`) + `None`/`Gzip (Very fast, good compression)`/`Bzip2 (Slower, very good compression)` (`:184-186`); `The following export errors occurred:` (`:307`).

**SPExportController.m**: `"Export Finished"`/`Finished exporting to %@` (`368`); `No directory selected.` (`433,680`); `Error while writing to the export file. Could not open file: %@` (`534`), `Export Error` (`535`); `Cancelling...` (`545`); `The selected file is not a valid file.\n\nPlease try again.\n\nClass: %@` (`661`); `The list of tables has changed` (`1115`); `“%@” already exists. Do you want to replace it?` (`2043`) + `Replace`/`Skip existing` (`2055, 2065`); `“%@” could not be created` (`2074`); `Invalid file supplied!` (`3077`)/`Wrong SPF content type!` (`3084`)/`Unsupported version for export settings!` (`3254`); `unsupported exportType=%lu` (`1451`); `exportType not implemented!` (`3338`).

**Printing**: `Continue to print?` (`6082`); `Are you sure you want to print the current content view of the table '%@'?\n\nIt currently contains %@ rows, which may take a significant amount of time to print.` (`6081`); `Generating print document...` (`6097`); heading `Table Structure`/`View Structure`/`Table Content`/`Query Result`/`Table Relations`/`Table Triggers`/`Table Information` (`6177-6300`); print accessory `Sequel Ace` (`SAPrintUtility.swift:103`), `Print Backgrounds` (`:113,136`), `On`/`Off` (`:138-139`).

## 6. UNKNOWN

- **Excel** ada sebagai tipe (`6`, tab opsi) tapi tanpa exporter, tanpa tab atas, dan controller menaikkan exception — praktis **tidak aktif/UNKNOWN**.
- Default escape CSV **UNKNOWN**: combo (`ExportDialog.xib:639-645`) mendaftar `"` dan `\` tanpa `selectedItem`.
- Markup HTML print ada di resource `SPPrintTemplate.html`/`SPTableInfoPrintTemplate.html` (disebut `6274, 6311`) — tak diperiksa.
