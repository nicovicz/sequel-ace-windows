# Audit: `SPTablesList.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/SubviewControllers/SPTablesList.m`. Baseline commit `02c910932`.

- File: 3043 baris; header `SPTablesList.h` 207 baris.
- Peran: memiliki daftar objek database (table/view/proc/func) di kiri window
  `SPDatabaseDocument` — refresh list, filter, section pinned, state seleksi,
  judul+enablement menu context/gear/main-menu, dan seluruh operasi DDL
  (add/remove/truncate/duplicate/rename).
- Total: 92 definisi method.

## 1. Struktur

| Region | Baris |
| --- | --- |
| Header lisensi | 1–30 |
| Import | 32–58 |
| Static key (`SPNewTableName/Type/CharacterSet/Collation`) | 60–64 |
| Class extension `SPTablesList ()` | 66–81 |
| `@implementation SPTablesList` | 83 |
| `@end` | 3043 |

Class extension (`.m:68`) menyatakan `<NSSplitViewDelegate, NSTableViewDataSource,
NSMenuItemValidation>` dan forward-declare `_removeTable:`, `_truncateTable`,
`_addTable`, `_addTableWithDetails:`, `_copyTable`, `_renameTableOfType:from:to:inDatabase:`,
`_allSchemaObjectsOfType:`, `_databaseHasObjectOfType:`, `_pinnedTablesConnectionIdentifier`
(69–77), plus property `_SQLitePinnedTableManager` (79). Header conformance
`SPTablesList.h:48`: `<NSTextFieldDelegate, NSTableViewDelegate>`.

## 2. Inventaris ivar/property

| Lokasi | Baris | Isi |
| --- | --- | --- |
| `.h` | 50–60 | IBOutlet sub-controller (`tableDocumentInstance` __weak, tableSource/Content/Dump/Data/ExtendedTableInfo/TableInfo/Triggers, databaseDataInstance, spHistoryControllerInstance, customQueryInstance) |
| `.h` | 62–85 | outlet window/kontrol: `copyTableSheet`, `tablesListView` (`@public`), field/tombol copy-sheet, `tableSheet`, `tableNameField`, `tableEncodingButton`, `tableCollationButton`, `tableTypeButton`, `toolbarAddButton`, `chooseDatabaseButton`, `copyTableToNewDatabaseField`, `copyTableToNewDatabaseMessageField`, `toolbarActionsButton`, `toolbarReloadButton`, `addTableButton`, `truncateTableButton`, `tableListSplitView`, `tableListFilterSplitView`, `listFilterField` |
| `.h` | 87–98 | item menu gear (`removeTableMenuItem`…`separatorTableMenuItem3`) |
| `.h` | 100 | `mySQLConnection` |
| `.h` | 102–115 | item context menu |
| `.h` | 117–125 | model state: `tables`, `filteredTables`, `pinnedTables`, `tableTypes`, `tableComments`, `filteredTableTypes`, `selectedTableType`, `selectedTableName`, `pinnedTableNotificationName` |
| `.h` | 127 | `prefs` |
| `.h` | 129–131 | flag BOOL: `isTableListFiltered`, `tableListIsSelectable`, `tableListContainsViews` |
| `.h` | 133 | `addTableCharsetHelper` |
| `.h` | 136 | `@property (readonly, strong, nullable) SPDatabaseData *databaseDataInstance;` |
| `.m` | 61–64 | static key `SPNewTableName` dll |
| `.m` | 79, 88–89 | `@property` + `@synthesize _SQLitePinnedTableManager`/`databaseDataInstance` |

Catatan: `copyTableToNewDatabaseField`/`copyTableToNewDatabaseMessageField` (`.h:76–77`)
dideklarasikan tapi **tak direferensikan** di `.m` (grep 0) — **UNKNOWN** apakah ter-wire di XIB.

## 3. Semua `#pragma mark` (urut baris)

`85/86 (Initialisation), 167/168 (KVO methods), 190/191 (IBAction methods), 815/816 (Additional methods), 1267/1268 (Getter methods), 1485/1486 (Setter methods), 1589/1590 (Data validation), 1652/1653 (Datasource methods), 1754/1755 (TableView delegate methods), 2017/2018 (Interface validation), 2046/2047 (Table list filter interaction), 2347/2348 (Task interaction), 2380/2381 (Private API), 3033 (tanpa judul)`.

## 4. Tabel method (urut baris)

### Initialisation (85–165)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 91 | 117 | `init` | Alokasi array/dict, set flag, seed header "TABLES", ambil manager |
| 119 | 165 | `awakeFromNib` | Konfigurasi splitview, drag type, font, observer KVO/task, charset helper |

### KVO methods (167–188)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 173 | 188 | `observeValueForKeyPath:ofObject:change:context:` | Reaksi `SPGlobalFontSettings`: reset row height/font + reload |

### IBAction methods (190–813)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 196 | 374 | `updateTables:` | Refresh list penuh: `SHOW TABLE STATUS`/`FULL TABLES` + routines, reselect, filter, pinned |
| 379 | 421 | `addTable:` | Isi popup engine/charset, buka sheet New Table |
| 423 | 485 | `tableEncodingButtonChanged:` | Bangun ulang popup collation untuk charset terpilih |
| 490 | 494 | `closeSheet:` | Akhiri sheet modal dengan tag sender |
| 499 | 580 | `removeTable:` | Bangun alert delete type-aware (dengan Force delete) lalu `_removeTable:` |
| 585 | 634 | `copyTable:` | Isi sheet duplicate (DB target, nama, switch content) dan tampilkan |
| 637 | 667 | `setDatabases` | Bangun ulang popup database copy-sheet dari `[mySQLConnection databases]` |
| 671 | 683 | `renameTable:` | Mulai editing in-place baris tabel terpilih |
| 688 | 709 | `truncateTable:` | Alert konfirmasi truncate tabel terpilih |
| 711 | 745 | `togglePinTable:` | Pin/unpin tabel terpilih via manager SQLite, post notifikasi |
| 748 | 764 | `copyTableName:` | Copy string `db.table` seleksi ke pasteboard |
| 768 | 783 | `openTableInNewTab:` | Post notifikasi duplicate-tab dengan state koneksi |
| 785 | 803 | `openTableInNewWindow:` | Buka window baru + terapkan state koneksi/dokumen |
| 808 | 813 | `togglePaneCollapse:` | Toggle collapse pane info, persist `SPTableInformationPanelCollapsed` |

### Additional methods (815–1265)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 821 | 826 | `setConnection:` | Simpan `mySQLConnection`, picu `updateTables:` awal |
| 831 | 842 | `controlTextDidChange:` | Enable tombol add/duplicate per validitas nama |
| 847 | 862 | `controlTextDidEndEditing:` | Tombol Return di field nama = klik tombol |
| 872 | 1260 | `setSelectionState:` | Split besar: seleksi multi/kosong vs tunggal; tulis ulang judul/hidden menu semua tipe |
| 1262 | 1265 | `deselectAllTables` | Bersihkan seleksi table-view di main thread |

### Getter methods (1267–1483)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1270 | 1283 | `selectedTableAndViewNames` | Nama baris terpilih yang table/view saja |
| 1285 | 1296 | `selectedTableItems` | Nama semua baris terpilih |
| 1298 | 1309 | `selectedTableTypes` | Tipe semua baris terpilih |
| 1314 | 1317 | `tableName` | `selectedTableName` |
| 1322 | 1325 | `tableType` | `selectedTableType` |
| 1330 | 1333 | `tables` | Accessor array `tables` |
| 1338 | 1341 | `pinnedTables` | Accessor nama tabel pinned |
| 1346 | 1360 | `allTableAndViewNames` | List nama table+view terfilter |
| 1365 | 1368 | `allTableNames` | Delegasi `_allSchemaObjectsOfType:` Table |
| 1373 | 1380 | `allViewNames` | Nama view, urut alfabetis |
| 1385 | 1388 | `allProcedureNames` | Nama procedure |
| 1393 | 1396 | `allFunctionNames` | Nama function |
| 1401 | 1404 | `allEventNames` | Nama event |
| 1409 | 1412 | `allDatabaseNames` | Proxy list database dokumen |
| 1414 | 1417 | `selectedDatabase` | Proxy DB aktif dokumen |
| 1422 | 1425 | `allSystemDatabaseNames` | Proxy list DB sistem dokumen |
| 1430 | 1433 | `tableTypes` | Accessor array `tableTypes` |
| 1438 | 1443 | `tableTypeAtRow:` | Lookup tipe bounds-checked di list terfilter |
| 1448 | 1451 | `hasViews` | Tes keberadaan view |
| 1456 | 1459 | `hasFunctions` | Tes keberadaan function |
| 1464 | 1467 | `hasProcedures` | Tes keberadaan procedure |
| 1472 | 1475 | `hasEvents` | Tes keberadaan event |
| 1480 | 1483 | `hasNonTableObjects` | OR view/proc/func/event |

### Setter methods (1485–1587)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1494 | 1543 | `selectItemWithName:` | Pilih (atau re-filter agar muncul) item by nama, fallback case-insensitive |
| 1549 | 1587 | `selectItemsWithNames:` | Multi-select by nama; bersihkan filter bila perlu |

### Data validation (1589–1650)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1596 | 1599 | `isTableNameValid:forType:` | Wrapper convenience dengan `NO` |
| 1605 | 1650 | `isTableNameValid:forType:ignoringSelectedTable:` | Tolak trailing whitespace/kosong/duplikat per tipe |

### Datasource methods (1652–1752)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1658 | 1661 | `numberOfRowsInTableView:` | Jumlah baris = count `filteredTables` |
| 1666 | 1674 | `tableView:objectValueForTableColumn:row:` | Kembalikan string item; guard over-request saat import |
| 1679 | 1682 | `tableView:shouldEditTableColumn:row:` | Blokir editing rename saat task berjalan |
| 1687 | 1736 | `tableView:setObjectValue:forTableColumn:row:` | Commit rename: trim, validasi, konfirmasi opsional, panggil rename |
| 1738 | 1752 | `renameTableOfType:from:to:` | Bungkus `_renameTableOfType:` + fixup pin + refresh di `@try/@catch` |

### TableView delegate methods (1754–2015)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1760 | 1798 | `control:textView:doCommandBySelector:` | Panah di filter field memindah seleksi; enter/esc |
| 1803 | 1813 | `selectionShouldChangeInTableView:` | Blokir perubahan seleksi saat task; akhiri editing |
| 1818 | 1875 | `tableViewSelectionDidChange:` | Load objek terpilih, sinkron path navigator, clear state saat deselect |
| 1880 | 1906 | `tableView:shouldSelectRow:` | Larang pilih header/group row atau saat working |
| 1911 | 1917 | `tableView:isGroupRow:` | Group row = entri `SPTableTypeNone` |
| 1922 | 1974 | `tableView:willDisplayCell:forTableColumn:row:` | Set ikon per-tipe, indentasi, note komentar |
| 1979 | 1987 | `tableView:heightOfRow:` | Header 25pt; lain sesuai font |
| 1989 | 2008 | `tableView:acceptDrop:row:dropOperation:` | Eksekusi SQL tabel navigator yang di-drop, lalu refresh |
| 2010 | 2015 | `tableView:validateDrop:proposedRow:proposedDropOperation:` | Terima drop-above sebagai copy |

### Interface validation (2017–2044)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2023 | 2044 | `validateMenuItem:` | Enable copy/rename/open (1 baris) dan remove/truncate (>0 baris) |

### Table list filter interaction (2046–2332) — juga memuat method pinned
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2053 | 2058 | `showFilter` | Expand splitview filter bila collapse |
| 2064 | 2069 | `hideFilter` | Collapse splitview filter bila expand |
| 2074 | 2077 | `clearFilter` | Kosongkan field filter list |
| 2082 | 2088 | `refreshPinnedTables` | Bangun ulang section pinned + reload |
| 2090 | 2099 | `_pinnedTablesConnectionIdentifier` | Connection ID, fallback host name |
| 2105 | 2115 | `removePinnedTablesSection` | Buang header PINNED + entri dari model, kosongkan `pinnedTables` |
| 2121 | 2147 | `initPinnedTables` | Migrasi pin legacy, load pin manager, sisip section PINNED |
| 2149 | 2162 | `handlePinnedTableRenameFrom:To:` | Pin ulang tabel yang di-rename |
| 2164 | 2174 | `unpinDeletedTableIfPinned:` | Buang tabel terhapus dari himpunan pinned |
| 2177 | 2200 | `subscribeToTablePinningNotifications` | Registrasi ulang observer notifikasi pin per koneksi+database |
| 2202 | 2211 | `processPinTableNotification:` | Refresh pin, restore seleksi, terapkan ulang filter |
| 2218 | 2223 | `makeTableListFilterHaveFocus` | First-responder ke filter field pada loop berikut |
| 2228 | 2231 | `makeTableListHaveFocus` | First-responder ke table list view |
| 2236 | 2332 | `updateFilter:` | Bangun ulang `filteredTables`/`filteredTableTypes` dengan regex+substring, header, no-match, current-selection |
| 2338 | 2345 | `selectTableAtIndex:` | Pilih baris by NSNumber |

### Task interaction (2347–2378)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2353 | 2359 | `startDocumentTaskForTab:` | Disable selectability + tombol toolbar selama task |
| 2364 | 2370 | `endDocumentTaskForTab:` | Enable kembali setelah task |
| 2375 | 2378 | `setTableListSelectability:` | Set langsung `tableListIsSelectable` |

### Private API (2380–3031)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2386 | 2490 | `_removeTable:` | DROP objek terpilih (opsional `FOREIGN_KEY_CHECKS=0`), prune model, unpin, refresh |
| 2495 | 2515 | `_truncateTable` | TRUNCATE baris terpilih terbalik, warning error, reset status |
| 2522 | 2554 | `_addTable` | Kumpulkan nilai sheet, mulai task, detach `_addTableWithDetails:` |
| 2559 | 2661 | `_addTableWithDetails:` | Thread worker: bangun/eksekusi CREATE TABLE, sisip+pilih baris, switch ke Structure |
| 2666 | 2876 | `_copyTable` | SHOW CREATE + recreate untuk table/view/proc/func, opsional copy content, cross-DB move, sisip list |
| 2879 | 2899 | `_moveTable:from:to:tempTable:` | ALTER TABLE ... RENAME antar-database pakai nama temp |
| 2906 | 2998 | `_renameTableOfType:from:to:inDatabase:` | RENAME TABLE / recreate proc-func; rename beda-kapital via nama temp; raise saat error |
| 3000 | 3014 | `_allSchemaObjectsOfType:` | Snapshot-filter nama by SPTableType |
| 3016 | 3031 | `_databaseHasObjectOfType:` | Tes keberadaan by tipe (snapshot terfilter) |

### Blok tanpa judul (3033)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3035 | 3041 | `dealloc` | Hapus observer notifikasi + KVO; log dealloc (ARC, tanpa super) |

## 5. Temuan menonjol / UNKNOWN

**Legacy `setDatabases` vs manager modern**
- `SPTablesList` mendefinisikan `setDatabases` sendiri (`:637`) yang membangun ulang `chooseDatabaseButton` **copy-sheet** memakai panggilan legacy `[mySQLConnection databases]` (`:649`); hanya dipanggil dari `copyTable:` (`:590`).
- Tak dideklarasikan di `.h` maupun extension (`.m:68–81`) — method tanpa deklarasi.
- Item menu di `:646` memakai `action:@selector(setDatabases:)` (**dengan colon**), yang **tidak** diimplementasi `SPTablesList` (ia implement bentuk tanpa argumen). Aksi jatuh ke responder chain, kemungkinan ke `SPDatabaseDocument -setDatabases:` (`SPDatabaseDocument.m:598`). **UNKNOWN** apakah "Refresh Databases" di popup itu benar-benar me-refresh `chooseDatabaseButton` — kemungkinan me-refresh list dokumen, meninggalkan popup basi.
- Sisa aplikasi memakai jalur Swift `SADatabaseListManager`/`SPDatabaseDocument`; popup copy-sheet adalah pengecualian yang masih membaca langsung dari `mySQLConnection`.

**Pinned tables — nama host legacy**
- `initPinnedTables` (`:2121`) selalu menghitung `legacyHostName = [mySQLConnection host]` (`:2123`) hanya untuk memanggil `migratePinnedTablesFromLegacyHost:toConnectionIdentifier:databaseName:` (`:2126`). Shim kompatibilitas yang jalan tiap init; praktis beban mati setelah semua user migrasi, tapi tak bisa dihapus tanpa membuang key lama.

**Penempatan pragma tidak konsisten**
- Blok `Table list filter interaction` (`:2047`) juga memuat semua method pinned (`:2082–2211`) + focus helper (`:2218–2231`); tak ada heading `Pinned tables interaction` meski header mengelompokkannya (`SPTablesList.h:195`).
- `deselectAllTables` (`:1262`) ada di *Additional methods* padahal header mengelompokkan dengan setter (`:158`).
- `#pragma mark -` di `:3033` tanpa nama.

**Method tanpa deklarasi (drift permukaan publik)**
- `tableEncodingButtonChanged:` (`:423`), `copyTableName:` (`:748`), `renameTableOfType:from:to:` (`:1738`), `_moveTable:from:to:tempTable:` (`:2879`), `setDatabases` (`:637`) ada tapi tak dideklarasikan di mana pun. `tableEncodingButtonChanged:` ter-wire di XIB.

**Outlet tak terpakai**
- `copyTableToNewDatabaseField`/`copyTableToNewDatabaseMessageField` (`.h:76–77`) tak direferensikan — **UNKNOWN** sisa legacy "duplicate to new database".

**Lain-lain**
- `init` hanya seed `tables` dengan header "TABLES" (`:110`) tapi tidak `tableTypes`, sehingga sebelum `updateTables:` pertama kedua array selisih satu; `updateTables:` membersihkan keduanya (`:223–224`) — jinak tapi rapuh.
- `SPTableTypeTableNewDB` (=5, `SPConstants.h:112`) hanya diteruskan ke `_renameTableOfType:...` dari jalur cross-database copy (`:2831`), ditangani khusus di cabang RENAME TABLE (`:2937, :2943`); bukan tipe list nyata.
- `updateTables:` re-entran (`:372` panggil `subscribeToTablePinningNotifications`, `:339` `refreshPinnedTables` tiap refresh) dan memanggil query di dalam `SPMainQSync` (`:221–229`) — **UNKNOWN** kontrak threading untuk pemanggil non-main.
