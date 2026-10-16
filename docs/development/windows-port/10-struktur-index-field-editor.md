# Struktur tabel, index, dan field editor

Kontrak source untuk tab **Structure**, sheet **Indexes**, dan sheet **Field
Editor**. Semua sitasi `file:line` merujuk pada checkout `02c910932`.

File yang terlibat:

| File | Peran |
| --- | --- |
| `Source/Controllers/MainViewControllers/TableStructure/SPTableStructure.m/.h` (3180 baris) | Grid struktur: baca kolom+index, bentuk semua `ALTER TABLE` field, aturan auto-increment |
| `Source/Controllers/MainViewControllers/TableStructure/SPTableFieldValidation.m/.h` | Klasifikasi tipe berdasarkan index ke array `typeSuggestions` |
| `Source/Controllers/MainViewControllers/TableStructure/SAAutoIncrementRuleSupport.swift` | Aturan AUTO_INCREMENT MySQL 8.4 |
| `Source/Controllers/MainViewControllers/TableStructure/TableSortHelper.swift` | State sort descriptor/indikator grid struktur |
| `Source/Controllers/SubviewControllers/SPIndexesController.m/.h` | Sheet index baru + SQL add/remove index |
| `Source/Controllers/SubviewControllers/SPFieldEditorController.m/.h` (+`+PHPSerialized.swift`) | Sheet field/BIT, QuickLook, editor tree PHP-serialized |
| `Source/Model/SAPHPSerializedValue.swift` | Parser/serializer PHP-serialize |
| `Source/Views/TableViews/SAFieldEditorCommitPolicy.swift` | Penekan inline-commit + tracker seleksi combo-box |
| `Source/Controllers/SubviewControllers/SPCharsetCollationHelper.m/.h` | Proxy popup charset/collation |
| `Source/Other/CategoryAdditions/SPDataAdditions.m/.h`, `SPDataBase64EncodingAdditions.m/.h` | Helper hex/format/enkripsi/base64 |
| `Source/Interfaces/IndexesView.xib`, `FieldEditorSheet.xib` | UI sheet + string statis |

## 1. Cara kolom dibaca dan model kolom

### 1.1 Sumber query

`SPTableStructure.loadTable:` **tidak** query kolom sendiri — ia menyalin
objek kolom hasil cache dan query index saja:

- `for (id col in [tableDataInstance columns]) { [theTableFields addObject:[col mutableCopy]]; }` — `SPTableStructure.m:1656-1659`
- `SHOW INDEX FROM %@` dengan `[aTable backtickQuotedString]` — `SPTableStructure.m:1662`

Objek kolom berasal dari `SPTableData`:

- Tabel: `SHOW CREATE TABLE %@.%@` atau `SHOW CREATE TABLE %@` — `SPTableData.m:556, 559`, diparse `parseCreateStatement:ofType:` — `SPTableData.m:737-1007`.
- View: `SHOW CREATE TABLE %@` — `SPTableData.m:1042`, lalu `SHOW COLUMNS FROM %@` — `SPTableData.m:1083`, diparse `SPTableData.m:1100-1126`.

Jalur printing query langsung: `SHOW COLUMNS FROM %@` dan `SHOW INDEXES FROM %@`
— `SPTableStructure.m:1468-1469`.

Encoding popup: `[databaseDataInstance getDatabaseCharacterSetEncodings]` —
`SPTableStructure.m:1705`.

### 1.2 Model field/kolom — setiap key yang dilacak

Dari `parseFieldDefinitionStringParts:` (`SPTableData.m:1385-1596`):

| Key | Sumber | Sitasi |
| --- | --- | --- |
| `datacolumnindex` | index kolom dalam tabel (string) | `SPTableData.m:797, 1104` |
| `name` | nama field tanpa quote | `SPTableData.m:798, 1105` |
| `type` | tipe dasar uppercase | `SPTableData.m:1409, 1414` |
| `length` | panjang dalam kurung | `SPTableData.m:1431, 1440` |
| `decimals` | skala desimal untuk REAL/DOUBLE/FLOAT/DECIMAL/NUMERIC | `SPTableData.m:1434` |
| `values` | anggota ENUM/SET | `SPTableData.m:1424` |
| `typegrouping` | `bit/integer/float/date/string/binary/enum/textdata/geometry/blobdata` | `SPTableData.m:1447-1476` |
| `null` | BOOL (default YES) | `SPTableData.m:1480, 1545, 1550` |
| `unsigned` | BOOL (default NO) | `SPTableData.m:1481, 1497` |
| `binary` | BOOL (default NO) | `SPTableData.m:1482, 1505` |
| `zerofill` | BOOL (default NO) | `SPTableData.m:1483, 1501` |
| `autoincrement` | BOOL (default NO) | `SPTableData.m:1484, 1554` |
| `onupdatetimestamp` | BOOL (default NO) | `SPTableData.m:1485, 1578` |
| `comment` | string (default "") | `SPTableData.m:1486, 1584` |
| `unparsed` | token definisi yang tak dikenali (dipertahankan) | `SPTableData.m:1487, 1591` |
| `encoding` | CHARSET/CHARACTER SET eksplisit | `SPTableData.m:1510, 1516` |
| `collation` | COLLATE eksplisit | `SPTableData.m:1523` |
| `generatedalways` | `VIRTUAL`/`STORED` | `SPTableData.m:1532, 1536` |
| `default` | default tanpa quote, atau `NSNull` untuk NULL | `SPTableData.m:1566, 1569` |
| `isfunction` | YES kalau default bukan string ter-quote | `SPTableData.m:1563` |
| `isprimarykey` | `@1` bila bagian PRIMARY KEY | `SPTableData.m:941` |
| `unique` | `@1` bila UNIQUE key | `SPTableData.m:959` |
| `primarykeyfield` | (level tabel) array nama field PK | `SPTableData.m:946` |

Ditambahkan `SPTableStructure.loadTable:` / `setTableDetails:`:

- `Key`: `PRI` untuk PRIMARY (`M:1686`), `SPA` untuk geometry + SPATIAL index (`M:1690-1692`), selain itu `MUL`/`UNI` dari `Non_unique` (`M:1695`).
- `encodingName`/`collationName`: diselesaikan via aturan 4-kasus MySQL (`M:1747-1809`) atau override JSON (`M:1755-1757`).
- `Extra`: default `None` (`M:1833`); `auto_increment` (`M:1836-1838`); `VIRTUAL GENERATED`/`STORED GENERATED` (`M:1841-1847`); `on update CURRENT_TIMESTAMP` untuk TIMESTAMP/DATETIME (`M:1850-1859`).
- `binary` = `@1` bila collation berakhir `_bin` (`M:1806-1808`); untuk ENUM/SET `length` ditulis ulang jadi daftar anggota ter-quote (`M:1814`).

### 1.3 Normalisasi hasil index

`convertIndexResultToArray:` (`SPTableStructure.m:871-906`): `NSNull` → placeholder
NULL milik user (`886-891`); `Extra` `""` → `"None"` (`894-896`); `Null` `"YES"` →
`"1"`, selain itu `"0"` (`897-901`).

## 2. ALTER TABLE untuk field dan aturan quoting

### 2.1 Statement yang dibentuk

`addRowToDB` (`SPTableStructure.m:944-1071`):

- Prefix `ALTER TABLE %@` dengan `[selectedTable backtickQuotedString]` (`970`).
- Baris baru → `ADD` (`972-974`).
- Baris lama (modify **dan** rename) → `CHANGE %@` dengan **nama field lama**
  backtick-quoted (`975-977`):
  `ALTER TABLE `t` CHANGE `oldname` <definisi kolom parsial>`
- Lalu `_buildPartialColumnDefinitionString:theRow` (`979`).
- Bila `autoIncrementIndex == @"PRIMARY KEY"`: tambah `\n PRIMARY KEY` (`984-985`);
  bila baris baru juga `\n AFTER `prevcol`` (`988-990`).
- Selain itu, bila `autoIncrementIndex` jenis index lain: (`AFTER` dulu untuk baris
  baru) lalu `\n, ADD %@ (%@)` dengan kind index + nama field backtick-quoted (`992-999`).
- Selain itu, baris baru: `\n AFTER `prevcol`` (`1002-1004`).

**Tidak ada statement RENAME COLUMN terpisah** — rename diekspresikan lewat
`CHANGE` (`976`).

- Hapus field: `ALTER TABLE %@ DROP %@` (tabel + nama field backtick-quoted) —
  `SPTableStructure.m:1599-1602`. Opsional hapus FK dulu: `ALTER TABLE %@ DROP FOREIGN KEY %@` (`1593-1596`).
- Reorder (drag): `ALTER TABLE %@ MODIFY COLUMN %@` + ` FIRST` atau ` AFTER `col`` (`2304-2315`).
- Reset AUTO_INCREMENT: `ALTER TABLE %@ AUTO_INCREMENT = %llu` (`852`).

### 2.2 `_buildPartialColumnDefinitionString:` (`SPTableStructure.m:1078-1301`)

Menghasilkan `` `name` `` + ` ` + tipe uppercase (`1098-1101`). Kasus khusus:

- `SERIAL`: tanpa panjang (`specialFieldTypes`) (`1104-1106`).
- `BOOL(EAN)`: NULL/NOT NULL + DEFAULT inline, lalu blok generik dilewati (`1109-1131`).
- Panjang: `(%@)` bila non-kosong (`1134-1137`).
- Tipe string (`isFieldTypeString:`), kecuali JSON/UUID: `\n CHARACTER SET %@` (`1148`),
  `\n BINARY` bila binary==1 selain itu `\n COLLATE %@` (`1152-1160`).
- Numeric non-BIT: `\n UNSIGNED` (`1167`), `\n ZEROFILL` (`1171`).
- NULL/NOT NULL dilewati untuk field generated (`1176-1183`); `NOT NULL` juga dipaksa
  saat `Extra == "SERIAL DEFAULT VALUE"` (`1177-1179`).
- DEFAULT dilewati untuk `AUTO_INCREMENT` dan field generated (`1186`). Cabang:
  - default kosong → tidak ada (`1225-1227`);
  - placeholder NULL + null diizinkan → `\n DEFAULT NULL` (`1230-1235`);
  - TIMESTAMP/DATETIME cocok `SPCurrentTimestampPattern` → `\n DEFAULT CURRENT_TIMESTAMP`
    (opsional `(len)`) (`1237-1247`). Pattern: `(?i)^ OWS CURRENT_TIMESTAMP (?: OWS \( OWS (\d*) OWS \) )? OWS $` (`SPConstants.m:526`);
  - BIT → `\n DEFAULT %@` tanpa quote (`1250-1252`);
  - `*CHAR`, `*TEXT`, `*ENUM`, TIMESTAMP, DATETIME, DATE, INET4, INET6 → di-quote via
    `[mySQLConnection escapeAndQuoteString:]` kecuali ekspresi/literal string (`1254-1260`);
  - ENUM kosong → spasi (`1262-1264`);
  - selain itu `\n DEFAULT %@` mentah (`1266-1269`).
- `UNIQUE`/`GENERATED ALWAYS AS` + extra lain (`1273-1282`).
- `unparsed` ditambahkan (`1286-1288`), `generatedalways` (`1291-1293`),
  `COMMENT` via `escapeAndQuoteString:` (`1296-1298`).

### 2.3 Aturan quoting

- Identifier: `backtickQuotedString` → `` `%@` `` dengan backtick digandakan
  (``my`table`` → ```` `my``table` ````) — `SPStringAdditions.m:233-236`.
- Nilai: `escapeAndQuoteString:` → `SPMySQLConnectionEscapeString(self, theString, YES)`
  memakai encoding koneksi saat itu — `Querying & Preparation.m:62-65`.
- Daftar identifier: `componentsJoinedAndBacktickQuoted` / `componentsJoinedByCommas`
  — `SPArrayAdditions.m:39-67`.

## 3. Index: add/remove

**Tidak ada operasi "change index"** — hanya add dan remove (`SPIndexesController.m:147-330`).
`UNKNOWN`: modifikasi index in-place.

### 3.1 Tag tipe index

`SPConstants.h:204-208`: `SPPrimaryKeyMenuTag=0, SPIndexMenuTag=1, SPUniqueMenuTag=2, SPFullTextMenuTag=3, SPSpatialMenuTag=4`.

Default sheet: PRIMARY KEY terpilih, nama `PRIMARY` dan disabled
(`SPIndexesController.m:162-164`); bila PK sudah ada, item PK disembunyikan dan
INDEX dipilih (`169-185`). Tipe tambahan lewat `_addAdditionalIndexTypes`
(`665-694`): SPATIAL hanya MyISAM (`676-683`); FULLTEXT untuk MyISAM atau InnoDB
dengan `supportsFulltextOnInnoDB` (`686-693`). Popup storage disabled untuk
MyISAM/InnoDB dan SPATIAL (`223, 352`). String xib: `PRIMARY KEY`, `INDEX`,
`UNIQUE` (`IndexesView.xib:51-53`); storage `Default`/`BTREE`/`HASH` (`185-188`).

### 3.2 Add index

`_addIndexUsingDetails:` (`SPIndexesController.m:732-831`):

- PRIMARY KEY → nama dikosongkan (`750-752`); selain itu nama backtick-quoted atau kosong (`754`).
- Kolom: bila ada `Size` dan tipe di `supportsLength` dan bukan FULLTEXT → `` `col` (Size) ``
  (`771-773`), selain itu nama backtick-quoted (`775`). Tipe yang butuh panjang tapi
  belum diset akan dilewati (`769`).
- String tipe index non-INDEX/non-PRIMARY diberi `" INDEX"` (`781`) → `UNIQUE INDEX`,
  `FULLTEXT INDEX`, `SPATIAL INDEX`.
- Perakitan query (`784-804`):
  ```
  ALTER TABLE `t` ADD <indexType>[ `<name>`][ USING <storage>] (`col` [(`size`)], ...) [KEY_BLOCK_SIZE = <n>]
  ```
  Persis: `[NSMutableString stringWithFormat:@"ALTER TABLE %@ ADD %@", [table backtickQuotedString], indexType]` (`784`); `" "` + nama (`787-790`); `" USING "` + storage (`793-796`); `" (%@)"` dijoin koma (`799`); `" KEY_BLOCK_SIZE = %ld"` (`802-804`).
- `supportsLength` = CHAR, VARCHAR, TINYTEXT, TEXT, MEDIUMTEXT, LONGTEXT, BINARY,
  VARBINARY, TINYBLOB, BLOB, MEDIUMBLOB, LONGBLOB (`96-98`).
- `requiresLength` = TINYTEXT, TEXT, MEDIUMTEXT, LONGTEXT, TINYBLOB, BLOB,
  MEDIUMBLOB, LONGBLOB (`101-103`).
- Storage hanya dicatat bila popup index > 0 dan tipe ≠ SPATIAL (`282-284`).

### 3.3 Remove index

`_removeIndexUsingDetails:` (`SPIndexesController.m:838-896`): opsional hapus FK
`ALTER TABLE %@ DROP FOREIGN KEY %@` (`847`); lalu `ALTER TABLE %@ DROP PRIMARY KEY`
bila `PRIMARY` (`861`), selain itu `ALTER TABLE %@ DROP INDEX %@` (`864-865`).
Saat error 1553 dengan FK tak diketahui, `_removingIndexFailedWithForeignKeyError:`
mencari FK yang memakai kolom index itu dan menawarkan hapus keduanya
(`871-876, 904-954`).

### 3.4 Penetapan key / prompt auto-increment

Kolom `Key` diisi di `loadTable:` (`1686-1696`). Sel `Extra == auto_increment`
tanpa `Key` membuka `keySheet` dan memetakan pilihan ke `autoIncrementIndex` =
`PRIMARY KEY`/`INDEX`/`UNIQUE` (`SPTableStructure.m:2143-2166`).

## 4. Auto-increment dan validasinya

- `mysql84RestrictedExtraValues = ["AUTO_INCREMENT", "SERIAL DEFAULT VALUE"]` — `SAAutoIncrementRuleSupport.swift:7-10`.
- `integerFieldTypes = TINYINT, SMALLINT, MEDIUMINT, INT, INTEGER, BIGINT, BOOL, BOOLEAN` — `:12-21`.
- `isAutoIncrementExtraValue:` membandingkan nilai trim+uppercase dengan `AUTO_INCREMENT` — `:24-28`.
- `isMySQL84AutoIncrementRuleExtraValue:` cek keanggotaan set terbatas — `:30-35`.
- `fieldTypeAllowsAutoIncrement:` menormalkan dengan uppercase, trim, potong dari `(` pertama
  dan ambil token pertama — `:43-54`.
- Aturan diterapkan hanya pada non-MariaDB ≥ 8.4.0 (`SPTableStructure.m:310-313`);
  `_fieldTypeAllowsAutoIncrement:` mengembalikan YES bila server bukan 8.4 (`325-330`).

Penegakan:

- Saat simpan: bila Extra terbatas dan tipe tak diizinkan → alert, batalkan simpan (`956-959`).
- Saat sel Extra berubah: tolak, kembalikan state editing, alert, reload (`2128-2137`).
- Saat tipe berubah sementara Extra terbatas → reset Extra ke `None`, alert (`2199-2204`).
- Isi popup Extra: sembunyikan `auto_increment` bila tabel sudah punya field auto-increment,
  dan sembunyikan saran terbatas bila tipe baris tak diizinkan (`2060-2066`).
- Teks alert (`332-340`): judul `"AUTO_INCREMENT is unavailable"`; pesan
  `"MySQL 8.4 and newer only support AUTO_INCREMENT on integer columns. The %@ column type cannot use AUTO_INCREMENT on this server."`; label tipe fallback `"selected"`.

Perilaku lain:

- Set Extra = auto_increment memaksa `null = 0` (`2140`).
- Bila tak ada key, buka key sheet (`2143-2166`); cancel mereset Extra ke saran pertama
  dan membersihkan `autoIncrementIndex` (`2158-2164`).
- Bila `autoIncrementIndex == "PRIMARY KEY"`, field dipaksa unsigned saat simpan (`961-968`).
- `extraFieldSuggestions = ["None", "auto_increment", "on update CURRENT_TIMESTAMP", "SERIAL DEFAULT VALUE"]` (`209-214`).
- `SERIAL` diperlakukan sebagai tipe khusus tanpa panjang (`1104-1106`).

## 5. Charset/collation

### 5.1 Query server (`SPDatabaseData.m`)

- Charset: `SELECT * FROM `information_schema`.`character_sets` ORDER BY `character_set_name` ASC` (`300-309`), fallback `SHOW CHARACTER SET` (`313`), fallback hardcoded (`318`).
- Collation: `SELECT * FROM `information_schema`.`collations` WHERE character_set_name = '<escaped>' ORDER BY `collation_name` ASC` (`182`); retry alias utf8mb3→utf8 (`184-189`); fallback `SHOW COLLATION` (`192-197`); gagal → alert `"Unable to get database collations for given encoding"` (`205`).
- Default collation untuk encoding: baris pertama dengan `IS_DEFAULT` lowercase == `yes` (`237-241`).
- Charset dari collation: `getEncodingFromCollation:` (`256-265`).

### 5.2 Konstruksi popup

- `SPCharsetCollationHelper`: `promoteUTF8` default YES (`68`); format default `"Default (%@)"` untuk charset dan collation (`88-89`). `refreshCharsets` membuat item berlabel `"<DESCRIPTION> (<charset>)"`, mengapungkan `utf8*` ke atas, menyisipkan separator, dan menambahkan item Default di depan (`109-174`). `refreshCollations` menampilkan collation untuk charset terpilih, emit `Default`/`Default (%@)` dulu, menghormati `IS_DEFAULT == "Yes"` (`276-358`). Default warisan hanya berlaku selama charset diwarisi (`346-352`).
- `SPTableStructure.loadTable:` membuat item `encodingPopupCell` `"<DESCRIPTION> (<charset>)"` dengan encoding tabel ditampilkan abu-abu (`1718-1738`); list kosong → `"Not available"` (`1742`).
- Sel collation dibangun ulang per baris dari collation tabel untuk encoding baris itu (`1992-2039`).
- Resolusi pakai aturan 4-kasus MySQL 4.1 (`1760-1789`); JSON dipaksa `utf8mb4`/`utf8mb4_bin` dengan binary=1 (`1751-1757`); collation `_bin` menandai `binary` (`1806-1808`).

## 6. Sheet Field Editor

### 6.1 Pemilihan sheet dan klasifikasi data (`SPFieldEditorController.m`)

`editWithObject:...` (`225-440`):

- Tipe `BIT` → `bitSheet`; data diperlakukan sebagai bit string; tombol NULL sesuai `_allowNULL` (`243-280`).
- Selain itu `editSheet`: `_isBlob = (!_isJSON && isFieldBlob)` (`286`); `isBinary = BINARY|VARBINARY` (`287`).
- `NSData` → segmen hex, string `"0x%@"`, `dataToHexString` untuk binary (`343-357`); `SPMySQLGeometryData` → teks (WKT) + thumbnail (`358-366`); kolom JSON diformat via `SPJSONFormatter` (`374-382`); selain itu teks biasa (`367-388`).
- BINARY/VARBINARY string diberi prefix hex `0x` (`351-353`).
- Refresh ketersediaan PHP-serialized dan auto-open dijadwalkan setelah 0.15 detik (`428-429`).

### 6.2 Parsing / NULL / hex saat commit

- `closeEditSheet:` (`617-702`): validasi panjang maksimum (code point; toleransi titik desimal FLOAT `629-631`; placeholder NULL dikecualikan `640`), tampilkan `"Text is too long. Maximum text length is set to %llu."` (`646`) dan pilih overflow (`643-645`); return code 1 untuk OK (`663`) / bit OK (`665-667`); untuk JSON unformat via `SPJSONFormatter stringByUnformattingString:` (`688-691`); selain itu teruskan `returnData` ke `processFieldEditorResult:contextInfo:` (`698-700`).
- Konsumsi hasil ada di `SPTableContent.processFieldEditorResult:` (`SPTableContent.m:3543-3593`): string sama dengan placeholder `SPNullValue` **dan** kolom mengizinkan NULL → `NSNull` (`3563-3567`).
- Jalur hex (inline + validasi): `sp_dataWithHexString:` (`SPTableContent.m:4516`). Error: judul `"Invalid hexadecimal value"`, isi `"A valid hex string may only contain the numbers 0-9 and letters A-F (a-f). It can optionally begin with „0x“ and spaces will be ignored.\nAlternatively the syntax X'val' is supported, too."` (`5267-5274`).
- `cellValueIsDisplayedAsHexForColumn:` hanya untuk `typegrouping` `binary`/`blobdata` dan pref `SPDisplayBinaryDataAsHex` (`4533-4551`).
- `sp_dataWithHexString:` (`SPDataAdditions.m:392-`): terima spasi awal opsional, `0x` opsional, `X'val'`/`x'val'` MySQL (panjang genap, tanpa spasi internal, spasi akhir OK), panjang ganjil di-pad kiri `'0'`; invalid → `nil`.
- BIT sheet: `updateBitSheet` menyusun integer/hex/oktal + bit string `sheetEditData`; NULL menyetel ketiga field ke placeholder (`1055-1091`). `setToNull:` menonaktifkan/mengaktifkan kontrol (`1157-1177`). Operator tag 0-6 (all-1/all-0/negate/shift-L/shift-R/rotate-L/rotate-R) (`1096-1143`).

### 6.3 Deteksi & dekode PHP-serialized (`SAPhPSerializedValue.swift`)

- Byte tipe (ASCII): `N=78, b=98, i=105, d=100, s=115, a=97, O=79, C=67, E=69, r=114, R=82` (`14-31`).
- `phpSerializedParserMaximumDepth = 512` (`8`).
- Entry `parseString:encoding:error:` (`428-462`): kosong → `"No serialized data was provided."` (`448`); tak bisa dienkode → `"Serialized data cannot be encoded using the field encoding."` (`453`); sisa byte → `"Unexpected trailing characters after serialized value."` (`459`).
- Grammar: `N;`, `b:0|1;`, `i:<int>;`, `d:<float>;`, `s:<len>:"…";`, `a:<n>:{k v …}`, `O:<clen>:"Class":<n>:{…}`, `C:<clen>:"Class":<plen>:{payload}`, `E:<len>:"Case";`, `r/R:<n>;` (`496-670`).
- Float menerima `INF`/`-INF`/`NAN` persis (case-sensitive) + grammar desimal/eksponen (`110-176`).
- Dekode string: coba encoding primer, lalu utf8, isoLatin1 (`599-611`).
- Serializer `serializedString(using:errorMessage:)` (`325-420`) memakai marker sama, default `stdClass` untuk objek (`355`).

`SPFieldEditorController+PHPSerialized.swift`:

- Menu `"Edit PHP Serialized Data as Tree"`, ditambah dengan separator, awalnya disabled (`160-172`).
- Diaktifkan hanya bila parse berhasil **dan** `isEditable` **dan** bukan JSON **dan** bukan geometry (`refreshPHPSerializedEditorAvailability`).
- Editor sheet dibangun programatik (820×520, min 650×420) berjudul `"PHP Serialized Data"`; kolom `Key`/`Type`/`Value`; popup tipe `String, Integer, Float, Boolean, Array, Null`; tombol `Update Selected`, `Add Child`, `Delete`, `Cancel`, `Apply to Field`.
- Validasi commit: `"Integer values may only contain digits and an optional leading minus sign."`, `"Float values cannot be empty."`, `"Float values must be a valid number, INF, -INF, or NAN."`, `"Boolean values must be true/false or 1/0."`, `"Cannot change serialized structure while PHP references are present."`, `"Cannot add entries while PHP references are present."`, `"Cannot delete entries while PHP references are present."`, `"Select a scalar value to edit it. Arrays and objects can be expanded in the tree."`

### 6.4 Commit policy (`SAFieldEditorCommitPolicy.swift`)

- `SAComboBoxSelectionState`: `notTracked, current, deferred, invalidated` (`14-19`).
- `SAComboBoxSelectionTracker` menjaga callback combo AppKit terhadap generasi tabel/reload; `consumeSelection(matching:tableColumn:row:)` mengembalikan deferred/invalidated/current (`35-206`).
- `SAFieldEditorCommitPolicy.shouldIgnoreInlineCommit(...)` (`208-241`): abaikan deferred/invalidated selalu; abaikan bila field editor diperlukan kecuali sel `NSComboBoxCell` dengan state `current`; selain itu bandingkan nilai yang diusulkan dengan yang tersimpan/terdisplay.
- Wire-up di `SPTableContent.m`: properti `:109`, init `:168`, bracket reload `:550/:929/:961/:1248/:1497/:1572/:1588`, callback combo `:4347/:4352/:4357/:4450/:4454`, pemakaian policy `:4473`.

## 7. Tipe field yang ditawarkan dan validasinya

`typeSuggestions` (`SPTableStructure.m:218-270`) `→ SPTableFieldValidation.fieldTypes` (`272`).
Nilai dari `SPMySQLDataTypes.m:33-79`:
`TINYINT, SMALLINT, MEDIUMINT, INT, BIGINT, FLOAT, DOUBLE, DOUBLE PRECISION, REAL,
DECIMAL, BIT, SERIAL, BOOL, BOOLEAN, DEC, FIXED, NUMERIC` | separator |
`CHAR, VARCHAR, TINYTEXT, TEXT, MEDIUMTEXT, LONGTEXT, TINYBLOB, MEDIUMBLOB, BLOB,
LONGBLOB, BINARY, VARBINARY, JSON, ENUM, SET` | separator |
`DATE, DATETIME, TIMESTAMP, TIME, YEAR` | separator | geometry | separator | `INET4, INET6`.

`SPTableFieldValidation`: `_formatType:` trim/uppercase (`155-159`); tipe tak dikenal →
semua predikat YES. Ambang index: numeric `< 17` (`57`); string `> 17 && < 24` atau
`> 29 && < 32` (`122`); allowBinary `> 17 && < 24` (`144`); date `> 32 && < 38` (`78`);
geometry `> 38 && < 48` (`100`).

Sort descriptor KVC di `awakeFromNib` untuk `datacolumnindex,name,type,length,unsigned,
zerofill,binary,null,Key,default,Extra,comment,encodingName,collationName` dengan alias
`collation→collationName`, `encoding→encodingName` (`SPTableStructure.m:176-191`).

## 8. Referensi cepat: semua SQL yang dibentuk

| Aksi | Format string persis | Sitasi |
| --- | --- | --- |
| Add field | `ALTER TABLE %@ ADD <partialdef>` | `SPTableStructure.m:970-979` |
| Modify/rename field | `ALTER TABLE %@ CHANGE %@ <partialdef>` (`CHANGE <oldname>`) | `SPTableStructure.m:976-979` |
| Reorder field | `ALTER TABLE %@ MODIFY COLUMN <partialdef> FIRST\|AFTER %@` | `SPTableStructure.m:2304-2315` |
| Drop field | `ALTER TABLE %@ DROP %@` | `SPTableStructure.m:1599` |
| Drop FK (jalur field) | `ALTER TABLE %@ DROP FOREIGN KEY %@` | `SPTableStructure.m:1593` |
| Add index | `ALTER TABLE %@ ADD <type>[ <name>][ USING <st>] (<cols>)[ KEY_BLOCK_SIZE = <n>]` | `SPIndexesController.m:784-804` |
| Drop primary | `ALTER TABLE %@ DROP PRIMARY KEY` | `SPIndexesController.m:861` |
| Drop index | `ALTER TABLE %@ DROP INDEX %@` | `SPIndexesController.m:864` |
| Drop FK (jalur index) | `ALTER TABLE %@ DROP FOREIGN KEY %@` | `SPIndexesController.m:847` |
| Reset auto-increment | `ALTER TABLE %@ AUTO_INCREMENT = %llu` | `SPTableStructure.m:852` |
| Baca kolom (tabel) | `SHOW CREATE TABLE %@.%@` / `SHOW CREATE TABLE %@` | `SPTableData.m:556,559` |
| Baca kolom (view) | `SHOW COLUMNS FROM %@` | `SPTableData.m:1083` |
| Baca index | `SHOW INDEX FROM %@` | `SPTableStructure.m:1662` |
| Charset | `SELECT * FROM `information_schema`.`character_sets` ORDER BY `character_set_name` ASC` | `SPDatabaseData.m:309` |
| Collation | `SELECT * FROM `information_schema`.`collations` WHERE character_set_name = '%@' ORDER BY `collation_name` ASC` | `SPDatabaseData.m:182` |
| Optimized type | `SELECT %@ FROM %@ PROCEDURE ANALYSE(0,8192)` | `SPTableStructure.m:388` |

## 9. String yang terlihat pengguna (ringkas)

`SPTableStructure.m`: `"You cannot delete the last field in a table. Delete the table instead."` (`689`); `"Are you sure you want to delete the field '%@'? This action cannot be undone."` (`717`); `"Delete field '%@'?"` (`719`); `"Error while fetching the optimized field type"` (`403`); `"No optimized field type found."` (`419,463`); `"Optimized type for field '%@'"` (`422,468`); `"Estimating optimized field type..."` (`433`); `"An error occurred while trying to reset AUTO_INCREMENT of table '%@'.\n\nMySQL said: %@"` (`855`); `"An error occurred when trying to add the field '%@' via\n\n%@\n\nMySQL said: %@"` (`1051`); `"An error occurred when trying to change the field '%@' via\n\n%@\n\nMySQL said: %@"` (`1060`); `"Unable to delete relation"` (`1606`); `"Error moving field"` (`2321`); `"Not available"` (`1742`).

`SPIndexesController.m`: `"Adding index..."` (`231`); `"Removing index..."` (`320,944`); `"Delete index '%@'?"` / `"Are you sure you want to delete the index '%@'? This action cannot be undone."` (`319`); `"Unable to add index"` + `"An error occurred while trying to add the index.\n\nMySQL said: %@"` (`811`); `"Unable to delete index"` + `"...delete the index...MySQL said: %@"` (`881-882`); `"A foreign key needs this index"` (`932,939,943`); `"The foreign key relationship '%@' has a dependency on index '%@'. ..."` (`943`).

`SPFieldEditorController.m`: `"Interpret data as:"` (`130`); `"Text is too long. Maximum text length is set to %llu."` (`646`); `"Invalid JSON"` (`517,529,542,567`).

## 10. UNKNOWN

- **UNKNOWN**: tidak ada jalur modifikasi index in-place (hanya add/remove).
- **UNKNOWN**: isi `EditorQuickLookTypes.plist` belum dibaca; hanya kode pemuatannya (`SPFieldEditorController.m:136-187`).
- FieldEditor sheet tidak melakukan coercion tipe sendiri — klasifikasi/NULL/hex terjadi di `SPTableContent.m`; **UNKNOWN** apakah `SPCustomQuery.m:3499` menyimpang.
