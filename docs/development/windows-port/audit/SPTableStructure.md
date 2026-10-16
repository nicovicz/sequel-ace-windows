# Audit: `SPTableStructure.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/MainViewControllers/TableStructure/SPTableStructure.m`.
Baseline commit `02c910932`.

- File: 3180 baris; header `SPTableStructure.h` 159 baris.
- Peran: controller tab Structure — load/edit kolom tabel, membangun+eksekusi
  `ALTER TABLE` kolom, reset AUTO_INCREMENT, wiring sheet index, collation/encoding,
  dan UI help tipe field.
- Total: 73 definisi method (+1 class method) + 1 fungsi C `_BuildMenuWithPills`.

## 1. Inventaris ivar/property

### `SPTableStructure.h`
| Baris | Deklarasi |
| --- | --- |
| 32–42 | forward `@class` (SPDatabaseDocument, SPTableFieldValidation, SPTableData, SPDatabaseData, SPTablesList, SPIndexesController, SPTableView, SPMySQLConnection, SPMySQLResult, SPExtendedTableInfo, SPTableInfo) |
| 44–57 | `@interface SPFieldTypeHelp : NSObject` — ivar `typeName/typeDefinition/typeRange/typeDescription` (46–49); `@property(readonly)` (52–55) |
| 59 | `@interface SPTableStructure` conform `NSTableViewDelegate, NSTableViewDataSource, NSComboBoxCellDataSource` |
| 61–67 | IBOutlet model: tablesListInstance, tableDataInstance, tableDocumentInstance (weak), tableInfoInstance, extendedTableInfoInstance, indexesController, databaseDataInstance |
| 69–70 | IBOutlet help panel: structureHelpPanel, structureHelpText |
| 72–93 | IBOutlet kontrol: keySheet, resetAutoIncrement*, tableSourceView, tombol add/duplicate/remove/reload field, chooseKeyButton, structureGrabber, editTableButton, tombol index, indexesTableView, tablesIndexesSplitView, indexesShowButton, filterSearchField; viewColumnsMenu, encodingPopupCell |
| 95–111 | `mySQLConnection`, `fieldValidation`, `selectedTable`, `tableFields`/`filteredTableFields`, `oldRow`/`enumFields`, `defaultValues`, `currentlyEditingRow`, `prefs`, `collations`, `typeSuggestions`/`extraFieldSuggestions`, `isCurrentExtraAutoIncrement`, `autoIncrementIndex`, `isEditingRow`/`isEditingNewRow`/`isSavingRow` |

### Tambahan `.m`
`@interface SPFieldTypeHelp ()` (56–63) + `@synthesize` (67–70);
class extension `@interface SPTableStructure () <NSMenuItemValidation>` (114–135) dengan
ivar privat **`fieldsSortHelper` (TableSortHelper *)** (115) dan prototipe method privat (118–129),
`#pragma mark - SPTableStructureDelegate` (131), `_displayFieldTypeHelpIfPossible:` (133).
Non-method file-scope: `MakeFieldTypeHelp` (84–93), `struct _cmpMap` (95–99),
`_BuildMenuWithPills` (110, 3138–3180).

## 2. Semua `#pragma mark` (urut baris)

`131 (SPTableStructureDelegate, di extension), 139/140 (Initialisation), 342/343 (Edit methods), 809/810 (Other IB action methods), 817/818 (Index sheet methods), 829/830 (Additional methods), 1351/1352 (Alert sheet methods), 1363/1364 (KVO methods), 1390/1391 (Accessors), 1522/1523 (Task interaction), 1576/1577 (Private API), 1634/1635 (Table loading), 1976 (SPTableStructureDelegate), 1978 (Table view datasource methods), 2340/2341 (Table view delegate methods), 2543/2544 (Split view delegate methods), 2581/2582 (Combo box delegate methods), 2710/2711 (Menu delegate methods), 2788 (tanpa judul), 3136 (setelah @end, sebelum C function)`. Header: `.h:152 #pragma mark - SPTableStructureLoading`.

## 3. Tabel method (urut baris)

### `@implementation SPFieldTypeHelp` (65–82)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 72 | 80 | `dealloc` | Nil keempat string; log `Dealloc called` (sisa debug) |

### Initialisation (140–340)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 142 | 166 | `init` | Alokasi `tableFields/oldRow/enumFields`, default, `fieldValidation`, `prefs` |
| 168 | 308 | `awakeFromNib` | Bangun sort helper, gridline, observer, list saran tipe/extra, state menu |
| 310 | 313 | `_serverUsesMySQL84AutoIncrementRules` | TRUE untuk server non-MariaDB ≥ 8.4 |
| 315 | 318 | `_isAutoIncrementExtraValue:` | Delegasi ke `SAAutoIncrementRuleSupport` |
| 320 | 323 | `_isMySQL84AutoIncrementRuleExtraValue:` | Delegasi ke `SAAutoIncrementRuleSupport` |
| 325 | 330 | `_fieldTypeAllowsAutoIncrement:` | Pra-8.4 selalu YES; else delegasi Swift |
| 332 | 340 | `_showInvalidAutoIncrementAlertForFieldType:` | Alert MySQL 8.4 AUTO_INCREMENT hanya integer |

### Edit methods (343–807)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 348 | 373 | `addField:` | Sisip baris kosong, masuk mode edit, pilih kolom name |
| 378 | 448 | `showOptimizedFieldType:` | Jalankan `PROCEDURE ANALYSE(0,8192)`; fallback ke estimator di MySQL 8/error |
| 450 | 471 | `_showOptimizedFieldTypeFallbackTask:` | Task background menampilkan tipe estimasi fallback |
| 473 | 493 | `_columnStatsForFieldName:lengthFunction:` | Query statistik COUNT/NULL/min/max kolom |
| 495 | 600 | `_estimatedOptimizedFieldTypeForField:failureReason:` | Estimasi tipe int/binary/string dari statistik |
| 605 | 638 | `toggleColumnView:` | Show/hide kolom Key/encoding/collation/comment per tag menu |
| 643 | 670 | `duplicateField:` | Salin baris terpilih, tambah sufiks "Copy", masuk edit mode |
| 675 | 742 | `removeField:` | Konfirmasi + drop field (dan FK) via `SAFieldRemovalTask` background |
| 747 | 766 | `resetAutoIncrement:` | Buka sheet (tag 1) atau set AUTO_INCREMENT=1 (tag 2) |
| 768 | 777 | `takeAutoIncrementFrom:` | Validasi `objectValue` NSTextField adalah NSNumber |
| 784 | 807 | `cancelRowEditing` | Revert/hapus baris sedang diedit, reset state edit |

### Other IB action methods (810–815)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 812 | 815 | `unhideIndexesView:` | Geser divider splitview untuk menampilkan indexes |

### Index sheet methods (818–827)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 823 | 827 | `closeSheet:` | Akhiri sheet dengan tag sender sebagai return code |

### Additional methods (830–1349)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 837 | 866 | `setAutoIncrementTo:` | Issue `ALTER TABLE … AUTO_INCREMENT = n`, atau reload bila nil |
| 871 | 906 | `convertIndexResultToArray:` | Konversi baris index ke dict mutable, normalisasi NULL/Extra/Null |
| 914 | 937 | `saveRowOnDeselect` | Akhiri editing + commit baris aktif |
| 944 | 1071 | `addRowToDB` | Bangun/eksekusi `ALTER TABLE ADD|CHANGE` untuk baris teredit |
| 1078 | 1301 | `_buildPartialColumnDefinitionString:` | Builder SQL definisi kolom inti (tipe, charset/collate, null, default, extra, comment) |
| 1308 | 1321 | `showErrorSheetWith:` | Delay 0.3s lalu alert warning dari dict |
| 1326 | 1349 | `validateMenuItem:` | Enable/disable item menu remove/duplicate/optimize/AUTO_INCREMENT |

### Alert sheet methods (1352–1361)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1354 | 1361 | `addRowSheetPrimaryAction` | Masuk ulang edit mode pada baris gagal (alert "Edit row") |

### KVO methods (1364–1388)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1369 | 1388 | `observeValueForKeyPath:ofObject:change:context:` | Reaksi perubahan pref gridline/global-font |

### Accessors (1391–1520)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1396 | 1405 | `setConnection:` | Simpan koneksi, wire indexes controller, daftarkan drag type |
| 1410 | 1419 | `defaultValueForField:` | Kembalikan default tersimpan atau placeholder NULL |
| 1424 | 1442 | `fieldNames` | Load tabel bila perlu; kembalikan array nama field |
| 1447 | 1450 | `enumFields` | Kembalikan salinan list nilai enum/set |
| 1458 | 1516 | `tableSourceForPrinting` | Bangun array struktur+index untuk print via SHOW COLUMNS/INDEX |
| 1518 | 1520 | `activeFieldsSource` | Kembalikan list terfilter bila memfilter, else list penuh |

### Task interaction (1523–1574)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1528 | 1544 | `startDocumentTaskForTab:` | Disable kontrol tabel/index selama task |
| 1549 | 1574 | `endDocumentTaskForTab:` | Enable kembali kontrol, hormati editability tabel/view |

### Private API (1577–1632)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1582 | 1632 | `_removeFieldAndForeignKey:` | Jalankan drop FK lalu drop field, sheet error, refresh skema |

### Table loading (1635–1974)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1640 | 1877 | `loadTable:` | SHOW INDEX, bangun dict field, normalisasi encoding/collation/default/Extra |
| 1882 | 1894 | `reloadTable:` | Simpan baris, reset cache, jalankan ulang `loadTable:` |
| 1901 | 1974 | `setTableDetails:` | Terapkan detail ke store/UI, sortir + filter, reload view |

### Table view datasource (1976–2338)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1980 | 1983 | `numberOfRowsInTableView:` | Jumlah field aktif (terfilter) |
| 1985 | 2070 | `tableView:objectValueForTableColumn:row:` | Isi sel collation/encoding/Extra + kembalikan nilai sel |
| 2072 | 2224 | `tableView:setObjectValue:forTableColumn:row:` | Tulis edit; tangani side-effect encoding/collation/binary/Extra/null/type |
| 2229 | 2237 | `tableView:shouldEditTableColumn:row:` | Blokir editing saat working atau view terpilih |
| 2242 | 2258 | `tableView:pasteboardWriterForRow:` | Mulai drag satu baris untuk reorder field |
| 2265 | 2287 | `tableView:validateDrop:proposedRow:proposedDropOperation:` | Izinkan hanya posisi reorder tabel yang sama |
| 2292 | 2338 | `tableView:acceptDrop:row:dropOperation:` | Jalankan `ALTER TABLE … MODIFY COLUMN … FIRST/AFTER` |

### Table view delegate (2341–2541)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2343 | 2349 | `tableView:shouldSelectRow:` | Commit baris teredit sebelum pindah seleksi |
| 2354 | 2379 | `tableViewSelectionDidChange:` | Enable tombol remove/duplicate per seleksi & jumlah field |
| 2384 | 2457 | `control:textView:doCommandBySelector:` | Tangkap tab/backtab/enter/escape untuk navigasi & simpan/batal |
| 2463 | 2527 | `tableView:willDisplayCell:forTableColumn:row:` | Disable sel menurut tipe field/tabel |
| 2529 | 2533 | `tableView:didClickTableColumn:` | Sortir list field by descriptor kolom |
| 2535 | 2541 | `sort:withDescriptor:` | Terapkan satu sort descriptor |

### Split view delegate (2544–2579)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2546 | 2549 | `splitView:canCollapseSubview:` | Selalu YES |
| 2551 | 2554 | `splitView:constrainMaxCoordinate:ofSubviewAt:` | Max = proposedMax − 130 |
| 2556 | 2559 | `splitView:constrainMinCoordinate:ofSubviewAt:` | Min = proposedMin + 130 |
| 2561 | 2564 | `splitView:additionalEffectiveRectOfDividerAtIndex:` | Rect grabber divider |
| 2566 | 2579 | `splitViewDidResizeSubviews:` | Show/hide tombol indexes berdasar state collapse |

### Combo box delegate (2582–2708)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2584 | 2587 | `comboBoxCell:objectValueForItemAtIndex:` | Saran tipe di index |
| 2589 | 2592 | `numberOfItemsInComboBoxCell:` | Jumlah saran tipe |
| 2597 | 2607 | `comboBoxCell:completedString:` | Completion prefiks case-insensitive tipe field |
| 2609 | 2626 | `comboBoxCell:willPopUpWindow:` | Pilih tipe aktif, posisi help panel, tampilkan help |
| 2628 | 2632 | `comboBoxCell:willDismissWindow:` | Sembunyikan help panel struktur |
| 2634 | 2637 | `comboBoxCellSelectionDidChange:` | Refresh help panel saat seleksi berubah |
| 2639 | 2708 | `_displayFieldTypeHelpIfPossible:` | Render teks/range/deskripsi help + posisi panel kiri/kanan popup |

### Menu delegate (2711–2786)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2713 | 2786 | `menuNeedsUpdate:` | Tambah marker/tooltip "pill" Table/Default ke menu encoding/collation |

### Blok tanpa judul (2788–3132)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2790 | 2797 | `dealloc` | Hapus observer KVO/notifikasi; log `Dealloc called` |
| 2799 | 3095 | `+helpForFieldType:` | Bangun+return `SPFieldTypeHelp` untuk nama tipe (lazy) |
| 3097 | 3101 | `filterChanged:` | Re-filter + reload saat search field berubah |
| 3103 | 3132 | `filterFieldsWithString:` | Hitung `filteredTableFields` by nama case-insensitive |

### C function setelah `@end`
| Start | End | Function | Tujuan |
| --- | --- | --- | --- |
| 3138 | 3180 | `_BuildMenuWithPills` | Tambah pill attachment + tooltip ke item menu cocok representedObject |

## 4. Temuan menonjol / UNKNOWN

**Generasi ALTER**
- `addRowToDB` (944–1071) penulis utama: `ALTER TABLE <t> ADD|CHANGE <col> <def>` (970–979) lalu append index/AFTER (982–1004).
- `_buildPartialColumnDefinitionString:` (1078–1301) builder definisi kolom dipakai `addRowToDB` dan `acceptDrop:` (2306); cabang SERIAL (1104), BOOL/BOOLEAN (1109), length (1134), charset/binary/collate (1144–1163), UNSIGNED/ZEROFILL (1164–1173), NULL/NOT NULL (1176–1183), default (1186–1270), GENERATED ALWAYS/Extra (1273–1282), `unparsed` (1286–1288), `generatedalways` (1291–1293), COMMENT (1296–1297).
- `acceptDrop:` (2292–2338) → `ALTER TABLE <t> MODIFY COLUMN <def> FIRST|AFTER <col>`; `_removeFieldAndForeignKey:` (1582–1632) → `DROP FOREIGN KEY` lalu `DROP <field>`; `setAutoIncrementTo:` (837–866) → `ALTER TABLE <t> AUTO_INCREMENT = <n>` (`%llu` 852).
- ⚠️ **Bug laten (code smell) di 1219–1220**: `((firstChar == '"') && (lastChar = '"'))` memakai **assignment `=`** (bukan `==`); kondisi kedua selalu truthy dan memutasi `lastChar`. Bandingkan dengan `==` yang benar di cabang single-quote pada ekspresi sama.
- ⚠️ **Offset AFTER `addRowToDB` (989, 995, 1003)** memakai `currentlyEditingRow - 1` dengan `safeObjectAtIndex:`; untuk field pertama index = −1 (**UNKNOWN** apakah `safeObjectAtIndex:` menangani negatif).
- `autoIncrementIndex`: hanya "PRIMARY KEY" dapat inline `PRIMARY KEY`; "INDEX"/"UNIQUE" jatuh ke `else` (992–999).

**Aturan AUTO_INCREMENT MySQL 8.4**
- `_serverUsesMySQL84AutoIncrementRules` (310–313), `_isAutoIncrementExtraValue:` (315–318), `_isMySQL84AutoIncrementRuleExtraValue:` (320–323), `_fieldTypeAllowsAutoIncrement:` (325–330), `_showInvalidAutoIncrementAlertForFieldType:` (332–340) melalu `SAAutoIncrementRuleSupport` (Swift). **UNKNOWN**: allowlist tipe integer ada di Swift di luar file ini.
- Titik penegakan: `addRowToDB` (956–959), `setObjectValue:` cabang Extra (2128–2137), cabang tipe (2199–2204), filtering saran Extra di `objectValueFor…` (2060–2066).
- `isCurrentExtraAutoIncrement` dari `tableDataInstance tableHasAutoIncrementField` (797, 1872) dan diturunkan ulang saat Extra berubah (2126).

**Jalur PROCEDURE ANALYSE (fallback MySQL 8)**
- `showOptimizedFieldType:` (378–448): di MySQL 8 (`serverSupport isMySQL8` 385) lewati `PROCEDURE ANALYSE` → fallback; selain itu jalankan query (388) dan perlakukan error 1064/1305 atau "PROCEDURE ANALYSE" sebagai "unavailable" → fallback (393–401), error lain alert (403–409), sukses tampilkan `Optimal_fieldtype` (411–425).
- Rantai fallback: `_showOptimizedFieldTypeFallbackTask:` (450–471) → `_estimatedOptimizedFieldTypeForField:failureReason:` (495–600) → `_columnStatsForFieldName:lengthFunction:` (473–493). Estimasi (integer min/max 547–557; tier BINARY/VARBINARY/BLOB 569–580; tier CHAR/VARCHAR/TEXT 582–599) delegasi ke `SPOptimizedFieldTypeEstimator` (Swift). **UNKNOWN**: threshold tipe persis ada di Swift.

**Lain-lain**
- `cancelRowEditing` (784–807) mereset `isCurrentExtraAutoIncrement` dari `tableDataInstance` (797), bukan restore nilai pra-edit — **UNKNOWN** apakah sengaja.
- `filterFieldsWithString:` (3103–3132): `NSLog(@"%@", entry)` debug tertinggal di **3120**; `filteredTableFields` hanya mengganti sumber bila jumlah terfilter lebih kecil (3124–3127).
- Sisa log debug: `NSLog(@"Dealloc called %s", __FILE_NAME__)` di `SPFieldTypeHelp dealloc` (79) dan `SPTableStructure dealloc` (2796).
- `tableSourceForPrinting` (1458–1516) menjalankan SHOW COLUMNS/SHOW INDEXES sendiri (1468–1469), bukan pakai cache.
- `objectValueForTableColumn:` cabang collation (1992–2040) punya TODO perf eksplisit (issue #2608, 1997) — membangun ulang menu collation tiap draw.
- Bridging Swift/ObjC: `SAAutoIncrementRuleSupport`, `SAFieldRemovalTask`/`SAFieldRemovalQueryResult`, `SADragPasteboard`, `TableSortHelper`, `SPOptimizedFieldTypeEstimator`, `SPComboBoxCell`, `SPIdMenu`, `SPPillAttachmentCell` semuanya eksternal.
