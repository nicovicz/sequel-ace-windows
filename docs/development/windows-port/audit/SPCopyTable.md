# Audit: `SPCopyTable.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Views/TableViews/SPCopyTable.m`. Baseline commit `02c910932`.

- File: 1807 baris; header `SPCopyTable.h` 250 baris.
- Peran: `SPCopyTable` adalah subclass `SPTableView` (NSTableView) yang menyusun
  payload clipboard tab / CSV / SQL-INSERT, menggerakkan drag-out dan context
  menu sel, menangani traversal tombol editing in-cell, dan menjalankan bundle
  Data-Table.
- Total: 29 definisi method + 4 `@synthesize`.

## 1. Inventaris ivar/property & konstanta

### Ivar (`.h:55–74`)
| `.h` | Ivar |
| --- | --- |
| 56 | `SPTableContent* tableInstance` |
| 57 | `id mySQLConnection` |
| 58 | `NSArray* columnDefinitions` |
| 59–60 | `NSString* selectedTable`, `selectedDatabase` |
| 61 | `SPDataStorage* tableStorage` |
| 63 | `NSUserDefaults *prefs` |
| 65 | `NSRange fieldEditorSelectedRange` |
| 66 | `NSString *tmpBlobFileDirectory` |
| 72–73 | `NSInteger mouseDownRow`, `mouseDownColumn` |

Property (`.h:76–81`): readonly `mouseDownRow`/`mouseDownColumn`; readwrite `tmpBlobFileDirectory`;
readwrite `fieldEditorSelectedRange`. Synthesize `.m:80–81, 1787–1788`.

### Konstanta/macro
| Baris | Simbol | Catatan |
| --- | --- | --- |
| `.m:57–60` | `SPEditMenuCopy=2001`, `SPEditMenuCopyWithColumns=2002`, `SPEditMenuCopyAsSQL=2003`, `SPEditMenuCopyAsSQLNoAutoInc=2004` | global (`extern` di `.h:41–44`) |
| `.m:62–65` | `kBlobExclude=1`, `kBlobInclude=2`, `kBlobAsFile=3`, `kBlobAsImageFile=4` | mode blob |
| `.m:66` | `SACellFilterMenuTag=1945001` | tag item filter submenu |
| `.m:68–72` | `kColType=@"TYPE"`, `kColMapping=@"MAPPING"`, `kHeader=@"HEADER"`, `kFieldType=@"FIELD_TYPE"`, `kFieldTypeGroup=@"FIELDGROUP"` | key dict builder SQL-insert |
| `.h:38–39` | `SP_MAX_CELL_WIDTH_MULTICOLUMN 200`, `SP_MAX_CELL_WIDTH 400` | cap lebar |

## 2. Semua `#pragma mark` (urut baris)

`.m:184, 944, 969, 1137, 1431/1432 (Field editing checks), 1534/1535 (Bundle Command Support), 1775`.

## 3. Tabel method (urut baris)

### Blok A — helper privat / state cell (`@implementation` 74 → 184)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 83 | 95 | `_removeCellFilterMenuItemFromMenu:` | Hapus item cell-filter bertag + separator sebelumnya |
| 97 | 104 | `_storageColumnIndexForVisibleColumn:` | Map urutan kolom terlihat → index storage |
| 106 | 151 | `_appendCellFilterMenuToMenu:forEvent:` | Bangun submenu "Filter by Selected Value" dari deskriptor sel |
| 156 | 168 | `isCellEditingMode` | YES untuk SPCustomQuery atau SPTableContent-on-a-view |
| 174 | 182 | `isCellComplex` | YES bila sel teredit bukan `SPTextAndLinkCell` |

### Blok B — copy/serialisasi baris (184 → 944)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 189 | 220 | `copy:` | Rutekan tag menu ke copy SQL-insert atau tab ke pasteboard |
| 226 | 357 | `rowsAsTabStringWithHeaders:onlySelectedRows:blobHandling:` | Bangun grid tab/newline (NULL/blob/geometry) |
| 363 | 495 | `rowsAsCsvStringWithHeaders:onlySelectedRows:blobHandling:` | Idem tab tapi koma, cell dibungkus quote |
| 501 | 505 | `rowsAsSqlInsertsOnlySelectedRows:` | Wrapper convenience |
| 507 | 760 | `rowsAsSqlInsertsOnlySelectedRows:skipAutoIncrementColumn:skipGeneratedColumn:` | Bangun `INSERT INTO ... VALUES (...)`, batch ~250k |
| 762 | 777 | `componentsJoinedAndBacktickQuoted:` | Join header kolom backtick-quoted, koma |
| 782 | 785 | `draggingSourceOperationMaskForLocal:` | Selalu `NSDragOperationCopy` |
| 791 | 857 | `draggedRowsAsTabString` | Payload drag-out: baris terpilih sebagai baris tab-separated |
| 865 | 928 | `displayStringForRow:column:` | Format satu sel terlihat (NULL/blob/geometry/formatter) |
| 930 | 942 | `isNullAtRow:column:` | YES bila sel storage mentah SQL NULL |

### Blok C — wiring data (944 → 969)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 949 | 959 | `setTableInstance:withTableData:withColumns:withTableName:withDatabaseName:withConnection:` | Simpan table instance, storage, kolom, nama, koneksi |
| 964 | 967 | `setTableData:` | Ganti referensi table storage |

### Blok D — autosize kolom (969 → 1137)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 974 | 1020 | `autodetectColumnWidths` | Hitung lebar per kolom, lalu fit proporsional ke lebar tabel |
| 1026 | 1135 | `autodetectWidthForColumnDefinition:maxRows:` | Ukur lebar sel/header tersampel untuk satu kolom |

### Blok E — menu/seleksi/tombol editing (1137 → 1431)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1139 | 1213 | `menuForEvent:` | Bangun context menu: bundle, copy nilai sel, submenu filter |
| 1215 | 1237 | `selectTableRows:` | Pilih baris dari array string index, bounds-checked |
| 1243 | 1272 | `validateMenuItem:` | Enable item copy/find berdasar seleksi & tipe delegate |
| 1278 | 1404 | `control:textView:doCommandBySelector:` | Tangkap tab/backtab/enter/down/up untuk navigasi + simpan edit sel |
| 1406 | 1429 | `keyDown:` | Enter edit baris terpilih; Escape cancel; spasi edit |

### Blok F — Field editing checks (1432 → 1534)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1438 | 1513 | `shouldUseFieldEditorForRow:column:checkWithLock:` | Putuskan sheet vs in-cell editing (blob/text/newline/panjang) |
| 1518 | 1532 | `isColumnEditable:` | NO untuk kolom generated, else YES |

### Blok G — Bundle Command Support (1535 → 1775)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1537 | 1773 | `executeBundleItemForDataTable:` | Jalankan bundle scope=datatable: tulis file input, env, output |

### Blok H — lifecycle/event (1775 → `@end` 1807)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1777 | 1785 | `awakeFromNib` | Reset columnDefinitions, cache prefs, reset cache mouseDown |
| 1799 | 1805 | `mouseDown:` | Cache row/kolom di bawah pointer untuk payload drag |

## 4. Temuan menonjol / UNKNOWN

**Perakitan copy-as-SQL — `rowsAsSqlInsertsOnlySelectedRows:skipAutoIncrementColumn:skipGeneratedColumn:` (507–760)**
- Desain dua pass: **Part 1** (539–619) filter kolom ke `tbColumns`, catat `kColMapping/kColType/kHeader/kFieldType/kFieldTypeGroup`. Deteksi auto-inc mencoba key `autoincrement` dan `AUTO_INCREMENT_FLAG` (556–562); nama auto-inc nil → alert + abort (569–577). Kolom `generatedalways` dibuang bila `skipGeneratedColumn` (550).
- Kode tipe kolom: 0=numeric/unquoted, 1=string, 2=blob/text, 3=geometry (600–612), digerakkan `SPFieldTypeClassifier shouldBeUnquotedWithFieldTypeGroup:fieldType:` (605).
- **Part 2** (625–759): emit `INSERT INTO \`table\` (cols)\nVALUES\n` (628–629), loop baris. Sel not-loaded di-fetch ulang via `argumentForRow:` + `SELECT ... WHERE` (652–666) — string SQL dibangun inline, TODO "could be preloaded… rather than cell-by-cell" (660).
- Tipe 0 pakai `unquotedSQLLiteralForValue:...` (680); tipe 1/2 quote via `escapeAndQuoteData:`/`escapeAndQuoteString:` (701–705); UUID/textdata NTData dikoersi ke string UTF8 (692–699); geometry `[cellData data]` (710).
- Batching: starter `INSERT` baru saat akumulasi `value` > 250 000 char (732–738). `,\n` akhir dipangkas (752–755), `;\n` final (757).
- UNKNOWN: rasional ambang 250 000 tanpa komentar; penamaan `penultimateRowIndex` sebenarnya adalah *count* baris terpilih (520).

**Copy-as-tab/CSV (226–357, 363–495, 791–857)**
- Tubuh nyaris identik di ketiganya: bangun `columnMappings` via `calloc` dari identifier integer tiap kolom (252–254, 389–391, 800–802), iterasi baris terpilih/semua, tiap sel bercabang pada `isNSNull`→pref `SPNullValue`, `isSPNotLoaded`→"(not loaded)", `SABaseFormatter` (hanya tab/CSV), `NSData` blob (hex via `SPDisplayBinaryDataAsHex` atau decode dengan connection/ISO-Latin1), `SPMySQLGeometryData`→`wktString`, else `description`.
- Jalur tab meng-escape `\n`→`↵` dan `\t`→`⇥` hanya di cabang string polos (334) — jalur **CSV tidak** meng-escape quote/newline internal (472).
- Blob-as-file: tulis `%ld_%ld.dat` / `.tif` (LZW TIFF) / `.pdf` (via `SPGeometryDataView`) ke `tmpBlobFileDirectory` (296–312, 434–449). Direktori dihapus+dibuat ulang dulu (261–265, 399–403).
- UNKNOWN: output CSV tanpa escaping RFC-4180 untuk koma/quote di luar string description; kemungkinan disengaja tapi tak terdokumentasi.

**Bundle menu (`menuForEvent:` 1139–1213) & payload drag**
- `menuForEvent:` menghapus submenu Bundles existing (tag `10000000`, 1149–1153) dan membangun ulang dari `SPBundleManager.shared` scope DataTable, menambah submenu kategori, key equivalent, tooltip, tag `1000000+i` (1160–1206); lalu append item `SACellValueCopyCoordinator` (1208) dan submenu cell-filter (1209).
- Payload drag keyboard: `mouseDown:` cache row/col (1799–1805); dokumen header (`.h:68–73`) + komentar 1790–1798: payload dipublikasikan di `-[SPTableContent tableView:writeRowsWithIndexes:toPasteboard:]` memakai `displayStringForRow:column:` + `isNullAtRow:column:`. `draggedRowsAsTabString` (791) adalah string tab seleksi-penuh yang lebih lama. `draggingSourceOperationMaskForLocal:` paksa copy (782–785).
- UNKNOWN: tipe pasteboard persis untuk payload drag satu-sel ada di `SPTableContent`, bukan file ini.

**Eksekusi bundle — `executeBundleItemForDataTable:` (1537–1773)**
- Resolve path bundle dari tag (`1000000+i`) atau fallback tooltip (1539–1548), muat file command `.spBundle` (1556), bangun env unik dengan file input, file metadata tabel, index baris terpilih (tab-joined), `usedQuery`, blob dir (1574–1625). Blob handling memetakan `SPBundleInputSourceBlobHandling*` → `kBlob*` (1608–1617).
- Switch aksi input (1627–1644) pilih tab/CSV/SQL untuk terpilih vs semua baris.
- Baris metadata tabel beda per delegate: `SPCustomQuery` menulis type/typegrouping/char_length/UNSIGNED_FLAG/AUTO_INCREMENT_FLAG/PRI_KEY_FLAG (1667–1683); `SPTableContent` menulis type/typegrouping/length/unsigned/autoincrement/isprimarykey/comment (1684–1700).
- Jalankan bash via `SPBundleCommandRunner runBashCommand:...` (1715–1722), lalu cabang output: text tooltip, HTML tooltip, atau window output HTML (memakai ulang window by UUID) (1747–1772). Error code 9 ditekan ("command was killed", 1735).

**Heuristik field-editing — `shouldUseFieldEditorForRow:` (1438–1513)**
- Early-YES via pref `SPEditInSheetEnabled` (1441); hormati `SPEditInSheetForLongText`+threshold dan `SPEditInSheetForMultiLineText` (1450–1456); kolom blobdata paksa sheet kecuali enum (1463–1464). Lock `pthread_mutex_t` opsional sekitar baca sel (1470–1477); NULL selalu inline (1494–1498).
- **Kemungkinan bug/UNKNOWN**: `isBlob && ![columnType isEqualToString:@"enum"]` (1464) tak akan pernah mengecualikan enum karena `isBlob` sudah mensyaratkan `typegrouping == "blobdata"`; guard enum adalah dead code.

**Diagnostik**
- `isCellComplex` membungkus `preparedCellAtColumn:` deprecated dalam `#pragma clang diagnostic` push/pop + catatan migrasi cell-vs-view (175–181).
- Instrumentasi `SPLog` berat di jalur SQL (565, 571, 579, 584–585, 1526); dict error `errorDict` dikumpulkan tapi hanya di-log (534, 570, 621–623).
