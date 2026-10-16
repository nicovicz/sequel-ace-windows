# Import data (SQL dan CSV)

Kontrak source untuk subsistem import: sumber file/clipboard, import SQL, import
CSV, field mapper, dan penanganan encoding/Unicode. Sitasi `file:line` merujuk
checkout `02c910932`.

File: `Source/Controllers/DataImport/SPDataImport.m/.h` (1760 baris);
`SPFieldMapperController.m/.h` (2140 baris); `SACSVImportStreamDecoder.swift`;
`SAImportProgressReporter.swift`; `Source/Other/Parsing/SPCSVParser.m/.h`;
`ImportAccessory.xib`; `DataMigrationDialog.xib`.

Threading: kedua importer jalan di thread background terpisah
(`_startBackgroundImportTaskForFilename:` → `_importBackgroundProcess:`,
`SPDataImport.m:1651-1675`); kerja UI dimarshal via `SPMainQSync`/`SPMainLoopAsync`.

## 1. Sumber & format

- **File**: `-importFile` membuka `NSOpenPanel` dengan accessory `importView` (`252-326`). Dir/filename terakhir diingat via pref `exportPath`, `importFormatPopupValue`, `lastFilename` (`270-284, 291-292, 305`).
- **Clipboard**: `-importFromClipboard` (`172-247`). Baca `NSPasteboardTypeString`, tampilkan maks 4000 char + `"\n…"` (`186-193`), tulis teks penuh ke file temp bernama prefix `SPImportClipboardTempFileNamePrefix` + `HHmmss` (`224`). Prefix `@"~/tmp/_SP_ClipBoard_Import_File_"` (`SPConstants.m:233`). File temp dihapus setelah import.
- **Format** ditentukan popup "Format:" (`ImportAccessory.xib:37`), item `"CSV"`/`"SQL"` (`ImportAccessory.xib:302-303`), di-wire `changeFormat:` (`144-147`). Hanya **SQL dan CSV** yang didispatch (`1673-1674`).
- **Deteksi ekstensi** (`-panelSelectionDidChange:` `1549-1590`): `.gz`/`.bz2` di-strip (`1559-1567`); `"SQL"`→ SQL (`1569-1573`); `"CSV"`/`"TSV"`→ CSV + field terminator `,` atau `\t` (`1574-1584`). Line ending ditebak via `/usr/bin/file -L -b` + regex `"with ([A-Z]{2,4}) line terminators"` → `\n`/`\r`/`\r\n` (`1719-1758`).
- **Encoding clipboard** beda per format (`226-238`): SQL → `NSUTF8StringEncoding`; CSV → `[mySQLConnection stringEncoding]`. Popup encoding direset ke UTF-8 dan **disable** untuk clipboard (`203-204`).
- **Warning overwrite**: bila DB punya >1 tabel dan format SQL, alert mendahului import (`317-322`).

## 2. Import SQL (`-importSQLFile:`, `350-756`)

- **Baca streaming**: chunk `fileChunkMaxLength = 1024 * 1024` (`370`). File dibuka via `SPFileHandle fileHandleForReadingAtPath:` (`385`) yang transparan mendekompres gzip/bzip2.
- **Encoding (`420-432`)**: index 0 = "Autodetect" (`ImportAccessory.xib:130`): `detectEncodingforFileAtPath:`; bila terpetakan via `[SPMySQLConnection mySQLCharsetForStringEncoding:]`, issue `@"SET NAMES '%@'"` + simpan encoding lama untuk restore (`423-427`). Else pakai `selectedTag` (`431`). Default `sqlEncoding = NSUTF8StringEncoding` (`380`).
- `sql_mode` disimpan via `@"SELECT @@sql_mode"` (`438-441`) dan direstore `@"SET SQL_MODE=%@"` dengan `tickQuotedString` (`468, 721-723`). `noBackslashEscapes` dari server status, re-sync tiap query (`444-452, 590`).
- **Pemisahan statement/delimiter**: file diparse baris-per-baris pada byte mentah `0x0A`/`0x0D` (`492`), line ending berurutan digabung (`495-500`), tiap segmen jadi `NSString` dengan `sqlEncoding` (`503-504`) dan diumpan ke `SPSQLParser` (`529`). Parser `[sqlParser setDelimiterSupport:YES]` (`449-450`) — `DELIMITER` dihormati. Statement lengkap ditarik `trimAndReturnStringToCharacter:';' trimmingInclusively:YES returningInclusively:NO` (`558`). Normalisasi `+normaliseQueryForExecution:` bila ada CR (`562-567`). Query kosong dilewati (`570`). Query tak terterminasi di EOF dieksekusi (`682-715`). DROP case-only: lazy `@"SELECT @@lower_case_table_names"` (`580, 691`) via `SASQLDatabaseContext`.
- **Eksekusi**: `[mySQLConnection queryString:query usingEncoding:sqlEncoding withResultType:SPMySQLResultAsResult assertingDatabaseContext:databaseName]` (`587, 698`).
- **Error**: popup tag `SPSQLImportAskOnError = 0`, `SPSQLImportIgnoreErrors = 1` (`SPConstants.h:222-223`). Error ditambah `@"[ERROR in query %ld] %@"` (`594, 703`). `"Query was empty"` diabaikan eksplisit (`593, 702`). Error utf8mb4 (ID 1115) → alert "Incompatible encoding in SQL file" (`599-624`). Else, kecuali ignore, alert Continue/Ignore All Errors/Stop (`628-654`).
- **Progress**: `SAImportProgressReporter` format `NSLocalizedString(@"Imported %@ of SQL", …)` (`397`); bar `setMaxValue:fileTotalLength` (`404`).
- **Cancel**: `-cancelProgressBar:` set atomic `progressCancelled = YES` (`152-155`); dicek di `457, 559, 684`.
- **Lost connection**: pesan `"The connection to the server was lost during the import.  The import is only partially complete."` (`547-555`).
- **Cleanup**: restore `SET NAMES`/`SET SQL_MODE`, `setQueryMode:SPInterfaceQueryMode`, hapus temp, tutup sheet, tampilkan error, refresh DB list/structure (`717-745`), post `"Import Finished"` / `"Finished importing %@"` (`748`).

## 3. Import CSV (`-importCSVFile:`, `774-1205`)

- **Konstanta**: chunk `256 * 1024` (`788`); `csvRowsPerQuery = 1000` (`789`); cap per-statement `[query length] > 250000` memecah batch (`1045`).
- **Encoding (`839-852`)**: logic autodetect sama; `csvEncoding = 0` saat index 0.
- **Konfigurasi parser (`861-880`, tersimpan di pref)**: field terminator (`SPCSVImportFieldTerminator`), line terminator (`SPCSVImportLineTerminator`), enclosing quote (`SPCSVImportFieldEnclosedBy`) dengan `convertDisplayStrings:YES`. Escape: string literal `@"\\ or \""` memilih escape `@"\\"` (non-strict); selainnya set escape + `setEscapeStringsAreMatchedStrictly:YES` (`873-878`). Null replacement `setNullReplacementString:[prefs objectForKey:SPNullValue]` (`879`). Default parser (`501-514`): field `,`, line `\n`, quote `"`, escape `\`, non-strict.
- **Stream decoding**: chunk diumpan ke `SACSVImportStreamDecoder` (`884, 913`).
- **Ekstraksi baris**: `[csvParser getRowAsArrayAndTrimString:YES stringIsComplete:allDataRead]` (`938`); baris + posisi byte di `parsedRows`/`parsePositions` (`942-943`).
- **Header**: checkbox `importFieldNamesSwitch` (":80", `:200`/`:262`). Bila ON, baris pertama dibuang sebelum INSERT (`1014-1018`).
- **Ambang field mapping**: sheet mapper dibuka sekali ≥100 baris terparse, atau di EOF untuk file pendek (`948-957`).
- **Konstruksi SQL**:
  - `insertBaseString` mulai `csvImportHeaderString` (dari mapper); bila bukan UPDATE: `@"%@ ("` + tabel backtick (`988`), kolom backtick join koma (`990-1010`), lalu `@") VALUES\n"` (`1011`).
  - Nilai per baris dari `-mappedValueStringForRowArray:` (`1043`); baris join `@", \n"` (`1042`). Tail (`ON DUPLICATE KEY UPDATE …`) bila ada (`1050`).
  - **UPDATE** (`1053-1113`): tiap baris issue `insertBaseString` + `mappedUpdateSetStatementStringForRowArray:` (`1070-1071`); bila `insertRemainingRowsAfterUpdate`, `insertRemainingBaseString` = `@"INSERT INTO "` + kolom + `@") VALUES\n"` (`1055-1065`) dan baris dengan 0 affected di-insert (`1090-1105`).
- **Recovery error per-baris**: saat error batch, batch dijalankan ulang baris-per-baris untuk atribusi (`1115-1144`); error dicatat `@"[ERROR in row %ld] %@"` (`1082, 1134`).
- **Progress/cancel**: format `NSLocalizedString(@"Imported %@ of CSV data", …)` (`823`); cancel dicek di `887, 1037, 1068, 1122`.
- **Cleanup/finish (`1163-1203`)**: reset query mode, hapus temp, tampilkan error, "Import Finished", pilih tabel baru atau reload content.

## 4. Field mapper (`SPFieldMapperController`)

- **Init**: `-initWithDelegate:` (`70-117`); `-setImportDataArray:hasHeader:isPreview:` set `numberOfImportColumns` dari baris pertama (`230-250`).
- **Target tabel**: popup diisi `@"New Table"`, `@"Refresh List"`, separator, lalu `[tablesListInstance allTableNames]` (`148-168`). Default pilih tabel aktif, else index 3, else "New Table" (`157-166`).
- `-changeTableTarget:` muat metadata kolom via `SPTableData informationForTable:fromDatabase:` (`526-570`). Kolom PK → default `@"auto_increment"`/`@"0"`, set `targetTableHasPrimaryKey` (`544-552`); UNIQUE → `isReplacePossible` (`554-557`); else default kolom atau `[NSNull null]` (`561-565`). `REPLACE` aktif hanya bila ada PK/UNIQUE (`571-572`); `UPDATE` hanya bila tabel punya >1 field (`598-608`).
- **Mode "New Table"**: `-newTable:` (`790-902`) menurunkan nama kolom dari header (`col_1…` bila tanpa header) dan tipe (`INT(11)`, `BIGINT(11)`, `VARCHAR(255)`, `VARCHAR(32767)`, `TEXT`) dari sampel (`847-883`).
- **UI mapping**: kolom dengan konstanta (`46-52`): `import_value`, `type`, `target_field`, `operator`, `value_index`, `global_value`, `sql`. `fieldMappingArray` (per kolom target: index kolom sumber, `1417-1438`); `fieldMappingOperatorArray`: `doImportKey=@0` (`―`), `doNotImportKey=@1` (`" "`), `isEqualKey=@2` (`=`) (`95-100`). Alignment popup tag: 0 File order, 1 Reversed, 2 Matching names (Levenshtein), 3 Custom (`697-720`); `-matchHeaderNames` fuzzy via Levenshtein (`1323-1412`).
- **Kolom diabaikan & default**: operator `doNotImportKey` mengeluarkan kolom dari daftar kolom DAN VALUES (`SPDataImport.m:1473-1474`). `isEqualKey` jadi term WHERE untuk UPDATE (`1398-1451`). Saat sel sumber `SPNotLoaded`, default kolom tabel disubstitusi (`1387-1388, 1439-1440, 1517-1518`). Global value/SQL expression via sheet Global Values; placeholder `$1` disubstitusi (`1352-1382, 1482-1512`); SQL ditandai regex `"(?<!\\\\)\\$\\d+"` (`1949`); global non-SQL dibungkus `'…'` dengan `'`→`\'` (`287`).
- **Header string INSERT/REPLACE/UPDATE** (`-importHeaderString`, `340-364`): INSERT `@"INSERT %@%@%@%@INTO "` dengan opsional `LOW_PRIORITY `/`DELAYED `/`HIGH_PRIORITY `/`IGNORE ` (`343-348`); REPLACE `@"REPLACE %@%@INTO "` (`351-354`); UPDATE `@"UPDATE %@%@%@ SET "` (`357-361`); tail `@"ON DUPLICATE KEY UPDATE %@"` (`369`).
- **DDL dari mapper**: add column `@"ALTER TABLE %@ ADD %@ %@"` (`412-415`); create table `@"CREATE TABLE %@ (\n"` + definisi kolom indent tab + `@")"` + opsional `@" ENGINE=%@"` + `@" DEFAULT CHARACTER SET %@"` (`437-459`).
- **Validasi tombol Import** (`-validateImportButton`, `1539-1593`): mode new-table butuh nama valid, ≥1 kolom diimport bernama, semua tipe non-kosong; UPDATE butuh minimal satu kolom `=`.

## 5. Unicode/encoding & stream decoder

- **SQL path** melewati decoder Swift: mengiris byte mentah pada line ending dan membangun `NSString` dengan `sqlEncoding` (`503-504`). Gagal decode → alert "could not be read in the encoding you selected" (`522`).
- **CSV path** pakai `SACSVImportStreamDecoder` (`initWithEncoding:lineTerminator:` `44-51`), karena baca 256 KB tetap bisa memotong karakter multi-byte; decoder hanya mengemit baris lengkap.
- **Menu encoder** (`-awakeFromNib` `127-135`): item 0 = `NSLocalizedString(@"Autodetect", …)`; lalu `[SPEncodingPopupAccessory enabledEncodings]` dengan tag = `NSStringEncoding`.
- **`SACSVImportStreamDecoder`**:
  - Buffer byte; hanya scan pada batas code-unit, jadi byte terminator di dalam unit UTF-16/32 tidak dianggap line end (`codeUnitWidth(for:)` `201-210`; `scanForTerminators` `101-129`).
  - `text(byAppending:endOfInput:)` mengembalikan teks hingga terminator lengkap terakhir; `""` hingga terminator terlihat; `endOfInput` men-decode sisa (`59-93`).
  - Gagal decode → `NSError` domain `"SACSVImportStreamDecoder"`, code `1`, deskripsi `"The CSV data could not be decoded using the selected encoding."` (`22-23, 83-87`).
  - **BOM/byte order** (`resolvePreamble`/`byteOrder` `152-197`): BOM UTF-8 `EF BB BF` dibuang; BOM UTF-16 `FF FE`/`FE FF` memaku little/big endian; BOM UTF-32 `FF FE 00 00`/`00 00 FE FF`. Tanpa BOM, encoding byte-order-agnostic default **big-endian** (`181, 189`). Terminator diturunkan ulang pada encoding hasil (`161`).

`NSStringEncoding` autodetect didelegasikan ke `detectEncodingforFileAtPath:` (`SPFileManagerAdditions.m:171`), default UTF-8 bila file tak bisa dibuka.

## 6. String yang terlihat pengguna (dikutip persis)

**SPDataImport.m**: `@"File read error"` (`54`); `@"Import Error"` (`387, 812`); `@"The SQL file you selected could not be found or read."` (`387`); `@"The CSV file you selected could not be found or read."` (`812`); `@"An error occurred when reading the file.\n\nOnly %ld queries were executed.\n\n(%@)"` (`473`); `@"An error occurred when reading the file, as it could not be read in the encoding you selected (%@).\n\nOnly %ld queries were executed."` (`522`); `@"An error occurred when reading the file.\n\nOnly %ld rows were imported.\n\n(%@)"` (`896`); `@"The connection to the server was lost during the import.  The import is only partially complete."` (`551`); `@"[ERROR in query %ld] %@"` (`594, 634, 703`); `@"Incompatible encoding in SQL file"` (`604`); `@"The SQL file uses utf8mb4 encoding, but your MySQL version only supports the limited utf8 subset.…"` (`605`); `@"Import Anyway"` (`608`); `@"An error occurred while importing SQL"` (`633`); `@"Ignore All Errors"` (`638`); `@"Import cancelled!\n"` (`621, 651`); `@"[ERROR in row %ld] %@"` (`1082, 1102, 1134`); `@"Could not parse file as CSV"` (`1220`); `@"The CSV was read as containing more than 512 columns, more than the maximum columns permitted for speed reasons by Sequel Ace.…"` (`1227`); `@"The current database already has existing tables, importing may overwrite data. Are you sure you want to continue?"` (`318`); `@"Importing SQL"` (`401`), `@"Importing CSV"` (`826`), `@"Reading..."` (`402, 827`), `@"Imported %@ of SQL"` (`397`), `@"Imported %@ of CSV data"` (`823`); `@"Import Finished"`/`@"Finished importing %@"` (`748, 1177`).

**SACSVImportStreamDecoder.swift**: `"The CSV data could not be decoded using the selected encoding."` (`:86`).

**SPFieldMapperController.m**: `@"New Table"` (`149`), `@"Refresh List"` (`150`); `doImportString @"―"`, `doNotImportString @" "`, `isEqualString @"="` (`98-100`); `@"New Column Name"` (`911`); `@"Error adding new column"` (`420`); `@"Error adding new table"` (`464`); `@"Ignore Field"` (`1753`), `@"Ignore all Fields"` (`1754`), `@"Import all Fields"` (`1755`), `@"Match Field"` (`1757`), `@"Add Value or Expression…"` (`1761`), `@"Default: %@"` (`1769`).

**ImportAccessory.xib**: `"Format:"` (`:37`), `"Encoding:"` (`:48`), tab `"CSV"`/`"SQL"` (`:73, :245`), `"First line contains field names"` (`:80`), `"Fields enclosed by:"` (`:112`), `"Lines terminated by:"` (`:163`), `"Fields terminated by:"` (`:195`), `"On SQL Error:"` (`:252`), menu `"Ask me what to do"`/`"Ignore and continue"` (`:268-269`), judul window `"Import from Clipboard"` (`:399`), tombol `"Next"` (`:362`).

**DataMigrationDialog.xib**: `"CSV Field Mapping"` (`:66`); `"CSV Import Field Mapping"` (`:96`); header `"CSV Fields"` (`:161`), `"Table Target Fields"` (`:195`); `"Import Method:"` (`:351`); menu `INSERT`/`REPLACE`/`UPDATE` (`:332-338`); `"Align field names by:"` (`:401`), `"File order"`/`"Reversed file order"`/`"Matching names"`/`"Custom order"` (`:383-388`); checkboxes `DELAYED`, `LOW_PRIORITY`, `IGNORE`, `ON DUPLICATE KEY UPDATE:`, `HIGH_PRIORITY`, `Skip existing rows`, `Insert remaining rows` (`:440-583`); sheet Global Values `"Global Source Values or SQL Expressions"` (`:692`); context menu `"Set all Field Types to:"` (`:832`), `"Add Column to Target Table…"` (`:838`), `"Remove New Column:"` (`:844`), `"Edit Field Type for Column:"` (`:852`); sheet New Table `"New Table"` (`:858`), `"Table Encoding:"` (`:872`), `"Table Type:"` (`:904`).

## 7. UNKNOWN

- Set encoding yang ditawarkan runtime `[SPEncodingPopupAccessory enabledEncodings]` tak dienumerasi.
- Semantik dekompresi `SPFileHandle` di luar "gz/bz2" (progress pakai `realDataReadLength`/`compressionFormat`, `SPDataImport.m:1700-1706`); internal `SPFileHandle` tak diperiksa.
- Aturan tokenizing `DELIMITER` `SPSQLParser` persis — hanya bahwa `setDelimiterSupport:YES` aktif dan `';'` split char (`450, 558`).
