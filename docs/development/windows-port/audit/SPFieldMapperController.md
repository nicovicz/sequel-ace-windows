# Audit: `SPFieldMapperController.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/DataImport/SPFieldMapperController.m`. Baseline commit
`02c910932`.

- File: 2140 baris; header `SPFieldMapperController.h` 213 baris.
- Peran: NSWindowController sheet "Data Migration / field mapping" — memetakan
  kolom sumber CSV/import ke kolom tabel target, membangun statement
  INSERT/REPLACE/UPDATE, dan mengedit ekspresi global-value pada pipeline import.
- Total: 66 definisi method.

## 1. Skeleton berkas

| Baris | Isi |
| --- | --- |
| 1–29 | header MIT / author (Hans-Jörg Bibiko, 2010) |
| 31–43 | import (`SPTableData`, `SPDataImport`, `SPTablesList`, `SPTextView`, `SPTableView`, `SPCategoryAdditions`, `RegexKitLite`, `SPDatabaseData`, `SPFunctions`, `SPMySQL`, `sequel-ace-Swift.h`) |
| 46–54 | konstanta static (7 ID kolom + 2 enum tipe sumber) |
| 56–60 | class extension kosong `SPFieldMapperController () <NSMenuItemValidation, NSControlTextEditingDelegate>`; forward decl `_setupFieldMappingPopUpMenus` (59) |
| 62 | `@implementation` |
| 64–65 | `@synthesize sourcePath; @synthesize databaseName;` |
| 67–68 | `#pragma mark -` / Initialisation |
| 222/223 | Setter methods; 252/253 Getter methods; 384/385 IBAction methods; 1035/1036 Global Value Sheet; 1152/1153 Advanced Sheet; 1263/1264 Others; 1635/1636 Table view datasource methods; 2112/2113 NSTextField delegates; 2127/2128 NSComboBox delegates |
| 2140 | `@end` |

Static file-scope: kolom identifier `import_value, type, target_field, operator, value_index, global_value, sql` (46–52), `SPSourceColumnTypeText=0`, `SPSourceColumnTypeInteger=1` (53–54).

## 2. Inventaris ivar/property

`.h` ivar 39–149:

| Baris | Grup |
| --- | --- |
| 40–59 | IBOutlet: `fieldMapperTableView`, `fieldMapperTableScrollView`, `globalValuesTableView`, `tableTargetPopup`, `fileSourcePath`, `importMethodPopup`, tombol row up/down, `recordCountLabel`, `importFieldNamesHeaderSwitch`, `addRemainingDataSwitch`, `importButton`, `advancedBox`, `alignByPopup`, `alignByPopupLabel`, `importMethodLabel`, `advancedLabel`, `matchingNameMenuItem`, `addNewColumnMenuItem`(unused), `setAllTypesToMenuItem`(unused) |
| 61–75 | kontrol new-table + sheet global-value |
| 77–88 | checkbox advanced (`ignore`, `ignoreUpdate`, `delayed`, `delayedReplace`, `onupdate`, `lowPriority` ×3, `highPriority`, `skipexistingRows`), `onupdateTextView`, `gobackButton` |
| 90–94 | `advancedButton`, `advancedInsertView`/`ReplaceView`/`UpdateView` |
| 96 | `typeComboxBox` (`NSComboBoxCell`) |
| 98–102 | `theDelegate`, `customQueryInstance` (**tak pernah direferensikan di `.m`**), `fieldMappingImportArray`, `databaseDataInstance`, `tablesListInstance` |
| 104–113 | array model: `fieldMappingArray`, `fieldMappingTableColumnNames`, `fieldMappingTableTypes`, `fieldMappingButtonOptions`, `fieldMappingOperatorOptions`, `fieldMappingOperatorArray`, `fieldMappingGlobalValues`, `fieldMappingGlobalValuesSQLMarked`, `fieldMappingTableDefaultValues`, `defaultFieldTypesForComboBox` |
| 115–122 | key/string: `doImportKey`/`doNotImportKey`/`isEqualKey`, `doImportString`(=`―`)/`doNotImportString`(=`" "`)/`isEqualString`(=`=`), `newTableEncoding`, `newTableEngine` (dua terakhir unused) |
| 124–148 | `toBeEditedRowIndexes`, `primaryKeyFields`, `lastDisabledCSVFieldcolumn`, `mySQLConnection`, `databaseName`, `sourcePath`, `prefs`, dimensi window, `numberOfImportColumns`, `fieldMappingCurrentRow`, `firstDefaultItemOffset`, BOOL `fieldMappingImportArrayIsPreview`/`importFieldNamesHeader`/`showAdvancedView`/`targetTableHasPrimaryKey`/`newTableMode`/`addGlobalSheetIsOpen` |
| 151–152 | property `sourcePath`, `databaseName` (synthesize `.m:64-65`) |

## 3. Semua `#pragma mark` (urut baris)

`67/68 (Initialisation), 222/223 (Setter methods), 252/253 (Getter methods), 384/385 (IBAction methods), 1035/1036 (Global Value Sheet), 1152/1153 (Advanced Sheet), 1263/1264 (Others), 1635/1636 (Table view datasource methods), 2112/2113 (NSTextField delegates), 2127/2128 (NSComboBox delegates)`.

## 4. Tabel method (66 definisi, urut baris)

### Initialisation (67–68)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 70 | 117 | `initWithDelegate:` | Muat nib DataMigrationDialog, alokasi array model, set sentinel key/string, ambil tables list delegate |
| 119 | 220 | `awakeFromNib` | Wire context menu, path control, popup target tabel, list tipe, state checkbox default, prime mapping |

### Setter methods (222–223)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 225 | 228 | `setConnection:` | Simpan `SPMySQLConnection` untuk query ALTER/CREATE |
| 230 | 250 | `setImportDataArray:hasHeader:isPreview:` | Simpan baris import, turunkan `numberOfImportColumns`, reset global value ke placeholder `…` |

### Getter methods (252–253)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 255 | 262 | `selectedTableTarget` | Nama tabel baru atau pilihan popup |
| 264 | 267 | `fieldMapperOperator` | Salinan `fieldMappingOperatorArray` |
| 269 | 272 | `selectedImportMethod` | Judul popup: INSERT/REPLACE/UPDATE |
| 274 | 277 | `fieldMappingArray` | Array index kolom sumber |
| 279 | 291 | `fieldMappingGlobalValueArray` | Global value, di-quote+escape SQL kecuali flagged/NULL |
| 293 | 302 | `globalValuesInUsage` | YES bila ada mapping merujuk global value |
| 304 | 307 | `importIntoNewTable` | Kembalikan `newTableMode` |
| 309 | 312 | `fieldMappingTableColumnNames` | Nama kolom target |
| 314 | 317 | `fieldMappingTableDefaultValues` | Nilai DEFAULT kolom target |
| 319 | 328 | `importFieldNamesHeader` | State checkbox, fallback ke ivar provisional |
| 330 | 333 | `hasContentRows` | Ada baris selain header? |
| 335 | 338 | `insertRemainingRowsAfterUpdate` | State switch "Add remaining rows" |
| 340 | 364 | `importHeaderString` | Bangun prefiks `INSERT/REPLACE/UPDATE` dengan LOW_PRIORITY/DELAYED dll |
| 366 | 372 | `onupdateString` | Klausa `ON DUPLICATE KEY UPDATE …` atau kosong |
| 374 | 377 | `canBeClosed` | YES bila tombol import enabled |
| 379 | 382 | `isGlobalValueSheetOpen` | Accessor flag sheet terbuka |

### IBAction methods (384–385)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 387 | 397 | `closeInfoSheet:` | Persist pref engine/encoding lalu akhiri sheet info |
| 399 | 476 | `closeSheet:` | Saat Import: ALTER TABLE untuk kolom baru atau CREATE TABLE; sembunyikan view advanced, akhiri sheet |
| 478 | 615 | `changeTableTarget:` | Refresh list tabel, muat kolom/tipe/default/PK, reset operator+popup |
| 617 | 679 | `changeImportMethod:` | Reset checkbox advanced, tampilkan view advanced cocok, buang operator `=` |
| 681 | 721 | `changeFieldAlignment:` | Terapkan alignment file order/reversed/header-match, ingat pref |
| 725 | 737 | `stepRow:` | Gerak `fieldMappingCurrentRow` naik/turun + refresh display |
| 739 | 771 | `changeHasHeaderCheckbox:` | Regenerasi nama kolom (`col_N` vs baris header) di mode new-table |
| 773 | 776 | `goBackToFileChooserFromPathControl:` | Double-click path-control → tombol goback |
| 778 | 788 | `goBackToFileChooser:` | Akhiri sheet + panggil ulang import file/clipboard |
| 790 | 902 | `newTable:` | Sniff tipe/panjang per kolom + usulkan skema tabel baru |
| 907 | 925 | `addNewColumn:` | Append kolom placeholder, tandai editable |
| 930 | 959 | `removeNewColumn:` | Hapus kolom ditambah, renumber `toBeEditedRowIndexes` |
| 964 | 977 | `setAllTypesTo:` | Copy tipe baris terpilih ke semua kolom target |
| 982 | 1033 | `newTableInfo:` | Isi popup engine/encoding + buka sheet info |

### Global Value Sheet (1035–1036)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1038 | 1080 | `addGlobalSourceVariable:` | Bangun ulang pull-down insert + menu recent, buka sheet global-value |
| 1082 | 1089 | `addGlobalValue:` | Append baris global value kosong + mulai edit |
| 1091 | 1110 | `removeGlobalValue:` | Hapus global value terpilih + entri SQL-flag |
| 1112 | 1121 | `insertNULLValue:` | Ganti global value terpilih dengan NSNull |
| 1123 | 1150 | `closeGlobalValuesSheet:` | Commit edit, opsional reassign mapping aktif, akhiri sheet |

### Advanced Sheet (1152–1153)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1155 | 1169 | `openAdvancedSheet:` | Toggle box advanced + resize window |
| 1171 | 1241 | `advancedCheckboxValidation:` | Paksa checkbox opsi INSERT/UPDATE saling eksklusif |
| 1243 | 1252 | `insertPulldownValue:` | Sisip referensi kolom `$N` ke global value yang diedit |
| 1254 | 1261 | `insertRecentGlobalValue:` | Sisip teks global value yang pernah dipakai |

### Others (1263–1264)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1266 | 1321 | `resizeWindowByHeightDelta:` | Besarkan/kecilkan window untuk view advanced via tukar autoresizing mask |
| 1323 | 1412 | `matchHeaderNames` | Alignment nama header Levenshtein+fuzzy ke `fieldMappingArray` |
| 1417 | 1438 | `setupFieldMappingArray` | Init mapping index kolom sumber per baris (identity atau 0) |
| 1443 | 1469 | `updateFieldMappingButtonCell` | Bangun label popup import-value dari cell + global value |
| 1474 | 1481 | `updateFieldMappingOperatorOptions` | Item popup operator (tambah `=` hanya untuk UPDATE) |
| 1486 | 1510 | `updateFieldNameAlignment` | Restore pref alignment tersimpan + terapkan |
| 1512 | 1537 | `updateRowNavigation` | Enable tombol row, update label "N of M record(s)" |
| 1539 | 1593 | `validateImportButton` | Enable Import hanya untuk tabel baru valid atau mapping updatable |
| 1598 | 1633 | `validateMenuItem:` | Validasi/judul dinamis item menu context tabel |

### Table view datasource methods (1635–1636)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1638 | 1645 | `numberOfRowsInTableView:` | Jumlah baris tabel field-mapper dan global-values |
| 1647 | 1650 | `tableView:willDisplayCell:forTableColumn:row:` | Terapkan font pilihan ke tiap sel |
| 1652 | 1672 | `tableView:didClickTableColumn:` | Klik header kolom operator toggle semua operator |
| 1674 | 1726 | `tableView:toolTipForCell:…` | Tooltip kolom import value/operator/target field |
| 1728 | 1741 | `menuNeedsUpdate:` | Sembunyikan semua item menu DEFAULT kecuali baris aktif |
| 1743 | 1777 | `_setupFieldMappingPopUpMenus` | Bangun ulang menu cell import-value+operator dan `firstDefaultItemOffset` |
| 1779 | 1842 | `tableView:objectValueForTableColumn:row:` | Nilai sel, termasuk tukar data cell tipe (combo vs token field) |
| 1844 | 1854 | `tableView:shouldEditTableColumn:row:` | Aturan editability (global value, kolom ditambah/new-table) |
| 1856 | 1973 | `tableView:setObjectValue:forTableColumn:row:` | Tulis mapping/operator/tipe/global value, aksi sentinel menu, recent |
| 1975 | 1984 | `tableViewSelectionDidChange:` | Enable tombol sheet global-value by seleksi |
| 1990 | 2110 | `control:textView:doCommandBySelector:` | Tangkap tab/shift-tab/enter/arrow/escape saat editing sel |

### NSTextField delegates (2112–2113)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2118 | 2125 | `controlTextDidChange:` | Re-validasi tombol import saat mengetik nama tabel |

### NSComboBox delegates (2127–2128)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2130 | 2133 | `comboBoxCell:objectValueForItemAtIndex:` | Tipe field tersimpan di index |
| 2135 | 2138 | `comboBoxCell:numberOfItemsInComboBoxCell:` | Jumlah tipe field tersimpan |

## 5. Temuan menonjol / UNKNOWN

**Array mapping kolom (invariant paralel)**
- Empat array index-paralel = model: `fieldMappingArray` (index kolom sumber per baris target), `fieldMappingTableColumnNames`/`Types`/`DefaultValues` (per kolom target), `fieldMappingOperatorArray` (doImport/doNotImport/isEqual per kolom target) — alokasi 82–91, init di `changeTableTarget:` 585–594 dan `newTable:` 895–897.
- **Global value disimpan di array yang sama di-offset `numberOfImportColumns`**: `fieldMappingGlobalValues`/`fieldMappingGlobalValuesSQLMarked` di-seed satu `…` per kolom import (245–248); index baris `globalValuesTableView` selalu `numberOfImportColumns + rowIndex` (1087–1088, 1099–1100, 1633, 1829–1837, 1946–1970). Nilai `fieldMappingArray >= numberOfImportColumns` berarti "merujuk global value" (297, 1684, 1698).
- Sentinel index menu = `count+1..count+5/6` (Ignore field / ignore all / import all / match / add value) di 1861–1894, cocok dengan urutan konstruksi menu di `_setupFieldMappingPopUpMenus` 1745–1772 dengan `firstDefaultItemOffset` (1767).

**Levenshtein / alignment `matchHeaderNames` (1323–1412)**
- Jarak = Levenshtein **minus** panjang prefix bersama, **minus** suffix bersama (via `commonPrefixWithString:…|NSBackwardsSearch` 1349 — **UNKNOWN**: kombinasi opsi itu tidak terdokumentasi/aneh; maksudnya tampak longest common suffix), **minus** panjang match regex fuzzy `.*?c` per karakter (1351–1361) via RegexKitLite.
- Exact match (`dist == 0`) dipaksa `-1e6` agar urut pertama (1363–1367).
- Dua index dipack ke satu `NSRange` via `NSStringFromRange(NSMakeRange(i, j))` dengan `i` = index **tabel** dan `j` = index **file** (1371); unpack `match.location` menulis `fieldMappingArray` dan `match.length` adalah kolom sumber (1390–1399) — urutan argumen terbalik relatif nama key `@"match"`. Guard `match.location >= fieldMappingArray.count` (1393–1397).
- Assignment greedy satu-satu dengan set `matchedFile`/`matchedTable` (1384–1411).

**Substitusi global value**
- SQL-ness diputuskan per nilai via regex `(?<!\\)\$\d+` saat edit (1949–1950, set `fieldMappingGlobalValuesSQLMarked`); nilai ber-flag dan `NSNull` lewat mentah, sisanya dibungkus `'…'` dengan `'`→`\'` (279–291, 287).
- Token `$N` hanya *ditulis* di sini (`insertPulldownValue:` 1250); ekspansi nyata terjadi di `SPDataImport.m` (di luar scope file ini).
- **UNKNOWN/bug**: `insertNULLValue:` set `fieldMappingGlobalValues` ke `NSNull` tapi tak memperbarui entri `fieldMappingGlobalValuesSQLMarked` paralel (1117 vs `removeGlobalValue:` 1099–1100).

**Temuan lain**
- **N1 (1891, kemungkinan bug precedence):** `if([anObject integerValue] == ([[self selectedImportMethod] isEqualToString:@"UPDATE"]) ? A : B)` — `==` mengikat lebih kuat dari `?:`, jadi nilai diuji adalah *boolean* perbandingan import-method, bukan index menu. Maksud hampir pasti `integerValue == (isUPDATE ? count+6 : count+5)` (bandingkan saudara `+1..+4` di 1863/1869/1878/1886).
- **N2 (1990–2006):** tabel tetap cell-based; `preparedCellAtColumn:` dibungkus `#pragma clang diagnostic ignored "-Wdeprecated-declarations"` + TODO (2000–2002).
- **N3 unused:** `customQueryInstance` (`.h:99`), `addNewColumnMenuItem` (`.h:58`), `setAllTypesToMenuItem` (`.h:59`), `newTableEncoding`/`newTableEngine` (`.h:121-122`) tanpa referensi di `.m`.
- **N4 stray:** semicolon deklarator di definisi 1112 (`insertNULLValue:`), 1638 (`numberOfRowsInTableView:`), juga `.h:191`.
- **N5 (1201–1205):** simpan/restore `isEditable` lalu append `NSAttributedString` **kosong** — tampak dimaksudkan membersihkan teks `onupdateTextView`; **UNKNOWN/no-op**.
- **N6 (813–814):** VLA C99 berukuran `numberOfImportColumns` (nol saat tanpa kolom) untuk heuristik sniff tipe di `newTable:`.
- **N7 (847):** variabel `serverGreaterThanVersion4` di-set dari `serverMajorVersion >= 5`, jadi cabang `else` (876–881, tanpa `VARCHAR(32767)`) dead di server modern — misnomer.
- **N8 (751):** `changeHasHeaderCheckbox:` iterasi `numberOfImportColumns` bukan `[headerRow count]`, pakai `safeObjectAtIndex:`.
- **N10 (1603):** `validateMenuItem:` hide/show by **index menu keras 3** ("Remove New Column").
- **N12:** default operator beda per jalur — `changeTableTarget:` set semua `doNotImportKey` lalu N pertama `doImportKey` (585–594), sedangkan `newTable:` set **semua** `doImportKey` (895–897).
- **N13:** sentinel tampilan tidak intuitif: `doImportString = @"―"`, `doNotImportString = @" "` (satu spasi), `isEqualString = @"="`.
