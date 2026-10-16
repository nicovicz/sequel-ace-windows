# Daftar objek: tables list, database list, dan navigator

Kontrak source untuk daftar database, daftar tabel/view/procedure/function,
pinned tables, table history, dan navigator. Sitasi `file:line` merujuk checkout
`02c910932`.

File:

| File | Peran |
| --- | --- |
| `Source/Controllers/SubviewControllers/SPTablesList.m/.h` (3043 baris) | Daftar objek + aksi tabel |
| `Source/Controllers/MainViewControllers/SADatabaseListManager.swift` | Popup database + orkestrasi seleksi |
| `Source/Controllers/SubviewControllers/SPNavigatorController.m/.h` | Outline navigator |
| `Source/Views/TableViews/SATablesListView.swift` | Type-ahead tabel |
| `Source/Views/Cells/SATableListCellRenderer.swift` | Render nama+komentar sel |
| `Source/Views/TableViews/SATableHeaderView.swift` | Header tabel |
| `Source/Controllers/Other/SPTableHistoryManager.swift` | Riwayat navigasi (in-memory) |
| `Source/Controllers/Other/SQLitePinnedTableManager.swift` | Pinned tables (SQLite) |
| `Source/Interfaces/DBView.xib`, `Navigator.xib`, `GotoDatabaseDialog.xib` | UI |

Enum/konstanta: `SPTableType` (`SPConstants.h:106-112`): `None=-1, Table=0, View=1,
Proc=2, Func=3, Event=4, TableNewDB=5`. `SPUniqueSchemaDelimiter = @"￸"` U+FFF8
(`SPConstants.m:278`). Key pref `SPDisplayCommentsInTablesList = "DisplayCommentsInTablesList"`
(`SPConstants.m:118`), `SPCopyContentOnTableCopy = "CopyContentOnTableCopy"` (`:292`).

`SPTablesList : NSObject <NSTextFieldDelegate, NSTableViewDelegate>`
(`SPTablesList.h:48`); extension menambah `<NSSplitViewDelegate, NSTableViewDataSource,
NSMenuItemValidation>` (`SPTablesList.m:68-81`). Array backing (`SPTablesList.h:117-125`):
`tables`, `filteredTables`, `pinnedTables`, `tableTypes`, `tableComments`,
`filteredTableTypes`. Init (`SPTablesList.m:91-117`): `filteredTables = tables`,
`filteredTableTypes = tableTypes`; baris pertama diisi `NSLocalizedString(@"TABLES", …)`;
`_SQLitePinnedTableManager = SQLitePinnedTableManager.sharedInstance` (`:113`).

## 1. SPTablesList

### 1.1 Kontrak datasource/delegate

| Method | Lokasi | Perilaku |
| --- | --- | --- |
| `numberOfRowsInTableView:` | `1658` | `[filteredTables count]` |
| `objectValueForTableColumn:…row:` | `1666` | `filteredTables[row]`; `@""` di luar rentang saat import (`1671`) |
| `shouldEditTableColumn:…` | `1679` | `![tableDocumentInstance isWorking]` |
| `setObjectValue:…` | `1687` | entry rename |
| `shouldSelectRow:` | `1880` | blokir saat `isWorking`; izinkan −1; tolak `rowIndex >= count`; baris selectable iff `filteredTableTypes[row] != SPTableTypeNone` (`1905`) |
| `isGroupRow:` | `1911` | baris 0 atau `type == SPTableTypeNone` |
| `willDisplayCell:…` | `1922` | ikon `table-view-small`/`table-small`/`proc-small`/`func-small`; catatan (comment) bila `SPDisplayCommentsInTablesList` (`1940-1967`) |
| `heightOfRow:` | `1979` | 25 untuk baris 0, selain itu dari font |
| `acceptDrop:…` | `1989` | eksekusi SQL drag mentah (`1998`) lalu `updateTables:nil` |
| `validateDrop:…` | `2010` | `NSDragOperationCopy`, drop-above |
| `control:textView:doCommandBySelector:` | `1760` | ↑/↓ memindah seleksi tabel |
| `controlTextDidChange:` / `controlTextDidEndEditing:` | `831` / `847` | validasi nama add/duplicate |
| `tableViewSelectionDidChange:` | `1818` | muat tabel, sinkron navigator |
| `selectionShouldChangeInTableView:` | `1803` | gate (selectability + `couldCommitCurrentViewActions`) |
| `validateMenuItem:` | `2023` | copy/rename/open butuh `selectedRows==1 && [self tableName].length`; remove/truncate butuh `selectedRows>0` |

### 1.2 Query SQL (dikutip persis; semua via `queryString:…`)

Di `updateTables:` (`196`), pilih jenis list dari pref `SPDisplayCommentsInTablesList` (`245-250`):
- `SHOW TABLE STATUS` — `:247` (`assertingDatabase:databaseName`)
- `SHOW FULL TABLES` — `:249`
- Metadata routine — `:277`: `SELECT * FROM information_schema.routines WHERE routine_schema = %@ ORDER BY routine_name` (`%@` = `[databaseName tickQuotedString]`)

Delete (`_removeTable:` `2386`):
- `SET FOREIGN_KEY_CHECKS = 0` (`2397`, hanya bila force)
- `DROP %@ %@` (`2419`; `%@`#1 = `VIEW`/`TABLE`/`PROCEDURE`/`FUNCTION` dari `2406-2417`; `%@`#2 = `backtickQuotedString` nama)
- `SET FOREIGN_KEY_CHECKS = 1` (`2481`)

Truncate (`_truncateTable` `2495`): `TRUNCATE TABLE %@` (`2501`).

Add (`_addTableWithDetails:` `2559`): `CREATE TABLE %@ (id INT(11) UNSIGNED NOT NULL%@) %@ %@ %@` (`2594`)
(`%@`#2 = `@""` untuk CSV selain `@" PRIMARY KEY AUTO_INCREMENT"`; lalu
`DEFAULT CHARACTER SET %@` `2582`, `DEFAULT COLLATE %@` `2587`, `ENGINE = %@` `2591`).

Duplicate/cross-DB (`_copyTable` `2666`):
- `SHOW CREATE %@ %@` (`2715`)
- `CREATE VIEW %@ %@` (`2737`); `CREATE TABLE %@ %@` (`2762`)
- `SHOW CREATE PROCEDURE %@` (`2774`); `SHOW CREATE FUNCTION %@` (`2776`)
- Recreate proc/func dari `objectAtIndex:2` SHOW CREATE, nama diganti regex `` (?<=%@ )(`[^`]+?`) `` (`2792`)
- `INSERT INTO %@ SELECT * FROM %@` (`2808-2812`, hanya bila switch copy-content on)
- cross-DB move: `ALTER TABLE %@.%@ RENAME %@.%@` (`2885`)

Rename (`_renameTableOfType:` `2906`): view/tabel `RENAME TABLE %@ TO %@` (`2939`);
proc/func `SHOW CREATE %@ %@` (`2971`) + recreate (`2985`) + `DROP %@ %@` (`2990`).
Rename beda-kapitalisasi saja menjalankan dua rename via nama sementara `%@-%d` (`2912-2934`).

Drag-import (`acceptDrop:` `1989`): string SQL navigator dieksekusi verbatim (`1998`).

### 1.3 Routine/view

- Normalisasi hasil oleh `SATableListResultParser.parse(rows:fieldNames:displayTableComments:)`
  (`SADatabaseListManager.swift:145-203`): nama dari `"Name"` atau `Tables_in_*`; tipe dari
  `"Table_type"`; komentar dari `"Comment"`; `isView = comment == "VIEW" || tableType == "VIEW"`
  (`:176`). Placeholder nama hilang `"..."` (`:171`).
- Baris ditambahkan (`261-270`): view → `SPTableTypeView` + `tableListContainsViews=YES` (`265-266`);
  selain itu `SPTableTypeTable`.
- Blok routine hanya bila tak ada error dan `numberOfFields > 3` (`284`): header `PROCS & FUNCS`
  (`287`), nama dari kolom index 3, tipe `PROCEDURE`→`SPTableTypeProc` selain itu
  `SPTableTypeFunc` dari kolom index 4 (`290-299`).
- Header section disisipkan di index 0: `TABLES & VIEWS` bila `tableListContainsViews`,
  selain itu `TABLES` (`310-315`); baris tipe `SPTableTypeNone` di index 0 (`317`).

### 1.4 Filter/pencarian (`updateFilter:` `2236`)

- Teks dari `listFilterField`. Tiap baris: section pinned selalu disimpan (`2253-2273`);
  selain itu match regex `isMatchedByRegex:` (`2276`) fallback `rangeOfString:…NSCaseInsensitiveSearch` (`2277`).
- Header diemit: `TABLES & VIEWS`/`TABLES` (`2284-2288`) dan `PROCS & FUNCS` (`2293`).
- Kosong → `NO MATCHES` (`2305`); terpilih-tapi-terfilter → `CURRENT SELECTION` + baris (`2310-2315`).
- Filter dibersihkan memulihkan alias `filteredTables = tables` (`2321-2322`).
- Filter box auto-muncul via `showFilter` saat `[tables count] > 20` (`343-351`); placeholder `Filter` (`357`).

### 1.5 Urutan

- Tidak ada sort umum `tables`; urutan server dipertahankan. Sort eksplisit: pinned
  diurut **descending** `localizedCompare:` lalu disisipkan di index 1 (`2134-2145`);
  item baru/copy disisipkan `localizedCompare:` ascending (`2616-2628`, `2850-2861`).

### 1.6 Restore seleksi & refresh

- `updateTables:` menyimpan `previousSelectedTable` (`207`), rebuild, re-select by
  `indexOfObject:` (`325-332`); selain itu clear (`333-337`).
- `refreshPinnedTables` (`2082`) = `removePinnedTablesSection` (`2105`) + `initPinnedTables` (`2121`) + reload.
- Rename via `setObjectValue:…` (`1687`): trim, tolak kosong/duplikat (`isTableNameValid:…ignoringSelectedTable:` `1704`),
  konfirmasi opsional bila `SPQueryWarningEnabled` (`1711`), lalu `renameTableOfType:…` (`1738`).

### 1.7 Aksi context-menu/gear (dipasang di `setSelectionState:` `872`)

| Aksi | Method | SQL/efek |
| --- | --- | --- |
| Delete | `removeTable:` `499` → `_removeTable:` `2386` | `DROP <VIEW\|TABLE\|PROCEDURE\|FUNCTION> \`name\`` + opsional `SET FOREIGN_KEY_CHECKS` 0/1 |
| Truncate | `truncateTable:` `688` → `_truncateTable` `2495` | `TRUNCATE TABLE \`name\`` |
| Rename | `renameTable:` `671`, `setObjectValue:` `1687` → `renameTableOfType:` `1738` | `RENAME TABLE` atau `SHOW CREATE`+recreate+`DROP` (proc/func) |
| Duplicate | `copyTable:` `585` → `_copyTable` `2666` | `SHOW CREATE`, `CREATE VIEW/TABLE`, opsional `INSERT INTO … SELECT`, cross-DB `ALTER TABLE…RENAME` |
| Copy Table Name | `copyTableName:` `748` | tanpa SQL; copy `"db.table"` (`755-761`) |
| Pin/Unpin | `togglePinTable:` `711` | panggil pinned manager + post notifikasi (`743`) |
| Open in New Tab | `openTableInNewTab:` `768` | `SPDocumentDuplicateTabNotification` |
| Open in New Window | `openTableInNewWindow:` `785` | `[SPAppDelegate newWindow:]` + `-setState:` |

Judul menu per tipe: `Copy Create Table/View/Procedure/Function Syntax`,
`Show Create …`, `Delete Table(s)/…/Items`, `Rename Table/View/Procedure/Function…`,
`Duplicate …`, `Pin/Unpin …` (`914-1259`).

## 2. `SADatabaseListManager`

- Deklarasi `SADatabaseSelectionDelegate` (`:31-111`) — kontrak host untuk setiap mutasi UI.
- `schemaPathDelimiter = "\u{FFF8}"` (`:213`); `navigatorSchemaPath(connectionID:selectedDatabaseTitle:)`
  → `<connectionID>` atau `<connectionID>∟<db>` (`:224-230`).
- `systemDatabaseNames = ["mysql","information_schema","performance_schema","sys"]` (`:244-249`);
  `partition(databases:)` mempertahankan urutan input di tiap bucket (`:253-264`).
- `configurePopup(...)` (`:290-334`): clear; placeholder `Choose Database...` di index 0 (`:299`),
  separator, `Add Database...` (`:303`), `Refresh Databases` (`:306`), separator; DB sistem lalu
  separator lalu DB user (`:317-325`); pilih `currentDatabase` by title selain itu index 0 (`:327-331`).
  Pemanggil menyuplai list dari `[mySQLConnection databases]` (`SPDatabaseDocument.m:607`).
- `performSelection(database:item:delegate:)` (`:355-479`), orkestrasi `-_selectDatabaseAndItem:`:
  1. guard history-state (`:358-362`).
  2. bila `database != currentSelectedDatabase`: `indexOfItem(withTitle:)` + `selectMySQLDatabase:` (`:364-368`).
  3. **Stale retry** — bila `(targetIndex == NSNotFound || !didSelect) && isConnected`:
     `rebuildDatabasesPopup()`, query index lagi, retry `selectMySQLDatabase:` sekali (`:372-382`).
  4. Saat gagal: `endLoadingTask()`, `presentUnableToSelectDatabaseAlert(name:)` bila terhubung,
     snap popup ke DB aktif (`:384-412`).
  5. Saat sukses: tambah item bila hilang (`:414-425`), select, set `currentSelectedDatabase`,
     clear `currentSelectedTable`, `resetDatabaseData()`, `detectDatabaseEncoding()`,
     `reattachTablesListConnection()`, `refreshWindowTitle()` (`:427-450`).
  6. Restore fokus: `selectTablesListItem(named:)` atau fokus filter/list (`:452-470`);
     `endLoadingTask()`, `processDatabaseChangedBundleTriggers()` (`:477-478`).
- **UNKNOWN**: `SPTablesList` juga punya `setDatabases` (`:637`) + outlet `chooseDatabaseButton`
  (`SPTablesList.h:75`) dengan menu lebih sederhana (tanpa `Add Database...`) — tampak jalur
  legacy/mungkin dead (DBView.xib punya dua outlet `chooseDatabaseButton` `:13` dan `:4382`).

## 3. Pinned tables + table history

### 3.1 SQLitePinnedTableManager (SQLite)

- File `pinnedTables.db` (`:26`) di folder application-support via
  `SASQLiteSharedStoreLocation.path(forFileName:)` (`SASQLiteSharedStoreLocation.swift:70-73`).
- Bentuk in-memory `[host/connection][database][table]` (`:53`). Satu `NSLock` menjaga state + write (`:51`).
- Skema (`:155-160`):
  `CREATE TABLE IF NOT EXISTS PinnedTables (id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, hostName TEXT NOT NULL, databaseName TEXT NOT NULL, pinnedTableName TEXT NOT NULL, CONSTRAINT host_db_table UNIQUE (hostName, databaseName, pinnedTableName))`
- Index: `CREATE INDEX IF NOT EXISTS host_db_idx ON PinnedTables (hostName, databaseName)` (`:164`).
- SQL lain: `SELECT id, hostName, databaseName, pinnedTableName FROM PinnedTables LIMIT 0` (`:167`);
  `PRAGMA user_version` (`:185`); `PRAGMA user_version = <n>` (`:195`);
  `SELECT hostName, databaseName, pinnedTableName FROM PinnedTables order by id desc` (`:224`);
  `DELETE FROM PinnedTables where hostName=? and databaseName=? and pinnedTableName=?` (`:376`);
  `INSERT INTO PinnedTables (hostName, databaseName, pinnedTableName) VALUES (?, ?, ?)` (`:408`);
  `SELECT 1 FROM PinnedTables WHERE hostName=? AND databaseName=? AND pinnedTableName=? LIMIT 1` (`:438`).
- API: `getPinnedTables(hostName:databaseName:)` (`:254`), `pinTable(...)` (`:262`),
  `unpinTable(...)` (`:363`), `migratePinnedTablesFromLegacyHost(...)` (`:280`).
- Konsumen di SPTablesList: `initPinnedTables` (`:2121`) migrate (`:2126`) lalu
  `getPinnedTablesWithHostName:databaseName:` (`:2130`); `_pinnedTablesConnectionIdentifier`
  (`:2090`) memilih `connectionID`, selain host (`:2097-2098`).

### 3.2 SPTableHistoryManager (**in-memory, bukan SQLite**)

- Dua stack `prevStack`/`nextStack` (`:36-37`), maksimum 50 (`:79-81`). API: `push` (`:39`),
  `replaceTopWithEntry` (`:44`), `navigate(to:)` (`:50`), `backEntries`/`forwardEntries` (`:74-75`).
- Entry `SPTableHistoryEntry` (`:130-160`): `database`, `table`, `view`, `viewPort`,
  `contentSortColName`, `contentSortColIsAsc`, `contentPageNumber`, `activeFilter`,
  `selectedRows`, `filter`, `filterData`, `cachedMenuItem`.
- Tidak ada lokasi persistensi — **UNKNOWN** bila ada store.

## 4. Navigator

- Tree **dibangun oleh `SPDatabaseStructure`**, bukan navigator. `SPNavigatorController`
  mengonsumsi dictionary bersarang siap pakai.
- Key struktur (`SPDatabaseStructure.m`): `connectionID ∟ db` (`:183`), `db_id ∟ table`
  (`:330`), `table_id ∟ field` (`:360`), dijoin `SPUniqueSchemaDelimiter` (U+FFF8). Child
  tabel/view dari `SPTablesList` (`:251-266`); field via `SHOW FULL COLUMNS FROM %@ FROM %@`
  (`:318`); routine via `SELECT SPECIFIC_NAME, ROUTINE_TYPE, … FROM
  `information_schema`.`ROUTINES` WHERE `ROUTINE_SCHEMA` = %@` (`:395`). Tiap dict tabel/view
  dapat `type` (`:256/:263`); tiap dict tabel dapat `"  struct_type  "` = `SPTableType`
  (`:377`, `:430`). Disimpan di `[self structure]` per connectionID (`:545`); `allStructureKeys` (`:475`).
- Ingestion `updateEntriesForConnection:` (`SPNavigatorController.m:406`): prune DB terhapus
  (`:438-445`), salin `[[doc databaseStructureRetrieval] structure]` ke `schemaData` (`:446-453`),
  atau marker `…&DEL&no data loaded yet` bila tak ada (`:455`).
- Outline datasource: `child:ofItem:` (`:777`) sort key via `compareStrings` cache >50-key (`:791-825`);
  `objectValueForTableColumn:byItem:` (`:865`) set ikon: connection (`:883`),
  table/view/proc/func dari `struct_type` (`:899-916`), database (`:918`), field (`:932`);
  `isItemExpandable:` menyembunyikan proc/func (`:835-843`).
- Filter `filterTree:` (`:596`): pola lowercase, `.`→delimiter (`:599`); panjang min 2 (`:630`);
  bangun `schemaDataFiltered` dari `allSchemaKeys` predikat `SELF CONTAINS[c] %@` (`:639-640`),
  `NSBeep()` saat miss (`:676-677`); reload di background (`:681`).
- Path seleksi: `selectPath:` (`:210`); `schemaPathExistsForConnection:andDatabase:` (`:482`).
  Tables-list push path `connectionID∟db∟table` saat `syncMode` (`SPTablesList.m:1861-1873`).

## 5. Swift views

- `SATablesListView.swift`: ganti type-select AppKit dengan fuzzy type-ahead; `NSTextInputClient`
  untuk komposisi IME (`:20`); baris kandidat dari datasource + `SPTablesList.tableType(atRow:)`
  (`searchableRows()` `:866-890`); commit tertunda timer 300 ms (`:22,369-384`); commit lewat
  gate delegate (`:521-522`); badge feedback `showSearchFeedback`/`hideSearchFeedback` (`:737-772`).
- `SATableListCellRenderer.swift`: hitung frame kolom nama/komentar (`:19-38`), gambar nama +
  komentar (`:41-87`).
- `SATableHeaderView.swift`: `validate(menu:forTableColumn:)` (`:9-11`), `menu(for:)` (`:15-26`).

## 6. String yang terlihat pengguna (terpilih)

**SPTablesList**: header `TABLES`/`TABLES & VIEWS`/`PROCS & FUNCS`/`PINNED`/`NO MATCHES`/`CURRENT SELECTION`
(`110,311,287,2107,2305,2311`); filter `Filter` (`357`); `Delete %@ '%@'?` (`525`);
`Force delete (disables integrity checks)` (`572`); `Truncate table '%@'?` (`698`);
`The name '%@' is already used.` (`1706`); `Duplicate %@ '%@' to:` (`624`);
`Table must have a name.` (`2673`); `Couldn't get create syntax.…` (`2724`);
`Cannot duplicate a table with triggers to a different database.` (`2745`);
`'%@' is of an unknown type.` (`2997`).

**SADatabaseListManager**: `Choose Database...` (`:299`), `Add Database...` (`:303`), `Refresh Databases` (`:306`).

**Navigator**: `Filtering failed. Please try again.` (`SPNavigatorController.m:688`); `ssh: %@` (`:952`).

**Pinned store** (`SASQLiteSharedStoreLocation.swift`): `Pinned tables are not being saved` (`:53`) +
`Tables you pin still stay at the top until you quit Sequel Ace, but they are not kept.` (`:45`);
`Column display formats are not being saved` (`:32`) + `Display formats you choose still work until you quit Sequel Ace, but they are not kept.` (`:25`).

## 7. UNKNOWN

- `SPTablesList -setDatabases` vs `SADatabaseListManager.configurePopup` — logika popup
  duplikat; mana yang memiliki popup toolbar live tidak dikonfirmasi.
- Handler item menu Table utama (Check/Repair/…, Show/Copy Create Syntax) tidak di
  `SPTablesList.m` (hanya judul/hidden) — resolusi di tempat lain.
- Lokasi persistensi `SPTableHistoryManager` — tak ditemukan; murni in-memory.
