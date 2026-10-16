# Query history dan query favorites (penyimpanan)

Kontrak source untuk penyimpanan riwayat query dan query favorites (termasuk
skema SQLite, cakupan per-dokumen vs global, dan format import/export). Sitasi
`file:line` merujuk checkout `02c910932`.

## Peta arsitektur

| Hal | Pemilik in-memory | Store persisten |
| --- | --- | --- |
| Riwayat query (per-dokumen) | `SPQueryController.historyContainer` (`SPQueryController.m:113`) | file `.spf` key `queryHistory` (load saja) / prefs `queryHistory` (legacy) |
| Riwayat query (global/Untitled) | container sama + `SQLiteHistoryManager.queryHist` | `queryHistory.db` (SQLite) atau prefs `queryHistory` |
| Query favorites (dokumen) | `SPQueryController.favoritesContainer` (`:112`) | file `.spf` key `queryFavorites` |
| Query favorites (global) | array flat `SPQueryFavoriteManager.favorites` | `NSUserDefaults` key `queryFavorites` |
| Display format kolom | `SQLiteDisplayFormatManager.sessionOverrides` | `ColumnDisplayOverrides.db` (SQLite) |

Dua backend history yang saling eksklusif, ditukar oleh `_SQLiteHistoryManager.migratedPrefsToDB`
(`SPQueryController.m:818, 972, 1017`).

## 1. Riwayat query

### 1.1 In-memory vs SQLite

- In-memory: `historyContainer` = `NSMutableDictionary` key `fileURL.absoluteString` → `NSMutableArray` `NSString` (`SPQueryController.h:60`, `.m:113`).
- SQLite: singleton `SQLiteHistoryManager.sharedInstance` (`SQLiteHistoryManager.swift:16`), dipegang `SPQueryController._SQLiteHistoryManager` (`SPQueryController.m:66,75,119`). Menyediakan `queryHist: [Int64: String]` (id → teks) (`SQLiteHistoryManager.swift:19`).
- Path DB: `<appSupport>/Data/queryHistory.db` (`SQLiteHistoryManager.swift:38,49`); `SPDataSupportFolder = "Data"` (`SPConstants.m:102`).
- Key pref legacy: `SPQueryHistory = "queryHistory"` (`SPConstants.m:271`). Migrasi digerbangi `SPMigratedQueriesFromPrefs = "SPMigratedQueriesFromPrefs"` (`SPConstants.m:518`; `SQLiteHistoryManager.swift:33,42,54,253,287`).
- Bila folder support tak resolve, SQLite dinonaktifkan + flag direset false (`SQLiteHistoryManager.swift:37-46`). Bila file db tak ada, `migratedPrefsToDB=false` (`:51-55`).

### 1.2 Skema SQLite

Blok skema `setupQueryHistoryDatabase()` (`SQLiteHistoryManager.swift:82-196`). Versi dari `PRAGMA user_version` (`:172-188`).

**v1** (`:92-98`):
```sql
CREATE TABLE QueryHistory (    query        TEXT NOT NULL,    createdTime  REAL NOT NULL)
CREATE UNIQUE INDEX IF NOT EXISTS query_idx ON QueryHistory (query)
```

**v2 — target saat ini** (`:136-152`):
```sql
ALTER TABLE QueryHistory RENAME TO QueryHistory_Old
CREATE TABLE QueryHistory (    id integer PRIMARY KEY AUTOINCREMENT NOT NULL,    query        TEXT NOT NULL,    createdTime  REAL NOT NULL)
INSERT INTO QueryHistory(query, createdTime) SELECT query, createdTime FROM QueryHistory_Old
DROP TABLE QueryHistory_Old
CREATE UNIQUE INDEX IF NOT EXISTS query_idx ON QueryHistory (query)
```
(`id` auto-inc ditambah karena SQLite tak bisa `ALTER TABLE` primary key — komentar `:114-134`.) Tiap langkah dibungkus `beginTransaction`/`commit`, rollback + `failed()` saat error (`:87-165`). `failed()` **assert/crash**: `assert(0 != 0, "Migration failed: ...")` (`:368-370`).

SQL lain: load `SELECT id, query FROM QueryHistory order by id desc LIMIT (?)` (`:207`);
insert `INSERT OR IGNORE INTO QueryHistory (query, createdTime) VALUES (?, ?)` (`:275,311`);
clear `DELETE FROM QueryHistory` (`:331`); size `SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()` (`:233`); vacuum `vacuum` (`:356`).

### 1.3 Maksimum entri

- Pref efektif `SPCustomQueryMaxHistoryItems = "CustomQueryMaxHistoryItems"` (`SPConstants.m:120`); **default 20** (`PreferenceDefaults.plist:100-101`).
- Diterapkan di LIMIT `loadQueryHistory` (`SQLiteHistoryManager.swift:201,207`), `addHistory` (`SPQueryController.m:991,1006,1023`), dan merger (`SAQueryHistoryMerger.swift:48-50`).
- `SPQueryController.numberOfMaxAllowedHistory = 100` (`:109`) **dead** — di-set sekali, tak pernah dibaca.

### 1.4 Cakupan per-dokumen vs global

- Per-dokumen: semua di-key `fileURL.absoluteString` (`SPQueryController.m:998-1008`).
- Global: URL di mana `![fileURL isFileURL]` (dokumen Untitled) (`:1010-1030`).
- `registerDocumentWithFileURL:` untuk Untitled baru seed `historyContainer` dari SQLite `queryHist` (key urut **menurun** via `intSortDesc`) atau prefs `SPQueryHistory` (`:816-841`). Untuk file URL, dari `contextInfo[SPQueryHistory]` (`:865-875`).
- `removeRegisteredDocumentWithFileURL:` buang key bila tak ada dokumen lain memakai URL itu (`:890-921`).
- Persistensi `.spf` history **dimatikan** (keamanan): komentar `// DON'T SAVE QUERY HISTORY IN EXPORTS FOR SECURITY` (`SPDatabaseDocument.m:2722-2723, 2785-2786`); toleransi sisi-baca di `4026-4032`.

### 1.5 Aturan merge/dedup

`SAQueryHistoryMerger` (`SAQueryHistoryMerger.swift:11-52`):
- existing: duplikat non-kosong dibuang, urutan dijaga, duplikat belakangan menang;
- entry baru disisipkan di index 0 satu-satu → beberapa entry baru mendarat terbalik;
- duplikat string kosong **dikecualikan** dari pembuangan (`:33-41`);
- `limit < 0` tak pernah memangkas; `limit == 0` mengosongkan (`:19-20, 48-50`).
- `newestFirst(rowKeyedHistory:)` mengurut `queryHist` per row id tertinggi (`:61-64`).

Normalisasi sebelum merge — `normalizeQueryHistory(arrayToNormalise:)` (`SQLiteHistoryManager.swift:388-452`):
- Bila `SPCustomQuerySaveHistoryIndividually` (`SPConstants.m:121`; **default true**, `PreferenceDefaults.plist:110-111`): query dipertahankan utuh bila `queryMightBeMultiLine` — memuat salah satu `["UNION","JOIN","ANY","SOME","ALL","IN"]` (`:458-471`); selain itu pecah pada `\n` via `separatedIntoLinesByCharset()` (`:407-419`, `StringExtension.swift:114`); tiap potongan lewat `appendToQueryHistory`, menambah `;` akhir bila hilang + dedup via `appendIfNotContains` (`:478-490`).
- Else: pass-through, tambah `;` hanya pada elemen terakhir bila hilang (`:430-439`).

### 1.6 Kapan history dicatat

- Setelah query jalan (non-reload): `performSelectorOnMainThread:@selector(addHistoryEntry:)` (`SPCustomQuery.m:1008-1009`) → `addHistoryEntry:` (`:3418-3421`) → `[SPQueryController addHistory:forFileURL:]`.
- Saat tab/koneksi tutup, teks editor disimpan: `[[SPQueryController sharedQueryController] addHistory:queryString forFileURL:[self fileURL]]` (`SPDatabaseDocument.m:6032-6036`).
- `addHistory:` trim/normalisasi lalu merge + simpan (`SPQueryController.m:989-1031`).

### 1.7 Tampilan

- Dropdown `queryHistoryButton` dibangun `historyItemsHaveBeenUpdated:` — simpan 7 item pertama, append `historyMenuItemsForFileURL:` (`SPCustomQuery.m:3241-3258`).
- Judul item: >64 char → 63 char + `…`; tooltip >256 → 255 + `…` (`SPQueryController.m:1057-1065`).
- Pilih: `chooseQueryHistory:` memetakan index item − 7 (`SPCustomQuery.m:369-404`); replace-vs-append digerakkan `SPQueryHistoryReplacesContent` (`SPConstants.m:272`) + modifier (`:376-381`).
- Filter: `filterQueryHistory:` regex `(?i).*%@.*` atau `localizedCaseInsensitiveContainsString:` (`:3597-3617`).
- `buildHistoryString` join `";\n"` (`:3409-3412`); clear `clearQueryHistory:` (`:578-591`) → `replaceHistoryByArray:@[]`.

## 2. Query favorites

### 2.1 Model data

Tiap favorite adalah `NSDictionary` dengan key `"name"`, `"query"`, opsional `"tabtrigger"`
(`SPQueryFavoriteManager.m:92-96, 211-222`; `SAQueryFavoriteStore.swift:72`). Pseudo-entry
`{"name","query","headerOfFileURL"}` adalah header group, bukan favorite nyata (`:93-95, 115-119`).
Snapshot Swift: `SAQueryFavoriteSelection` (`SAQueryFavoriteStore.swift:23-33`); scope
`SAQueryFavoriteScope { document, global }` (`:12-15`).

### 2.2 Lokasi/key penyimpanan

- Dokumen: `SPQueryController.favoritesContainer[fileURL.absoluteString]` (`:112, 811-814, 853-863`); dipersist ke `.spf` key `SPQueryFavorites = "queryFavorites"` (`SPConstants.m:269`; tulis `SPDatabaseDocument.m:2721, 2784, 4028-4029`; baca `:3660`).
- Global: `NSUserDefaults` key `queryFavorites` (`SPQueryFavoriteManager.m:104-105, 465`; `SAQueryFavoriteSaveWindowController.swift:210, 252`).
- API: `replaceFavoritesByArray:forFileURL:`, `addFavorite:forFileURL:`, `removeFavoriteAtIndex:forFileURL:`, `insertFavorite:atIndex:forFileURL:`, `favoritesForFileURL:`, `queryFavoritesForFileURL:andTabTrigger:includeGlobals:` (`SPQueryController.h:126-142`).

### 2.3 Cakupan global vs dokumen

`SPQueryFavoriteManager` meratakan kedua scope ke satu array `favorites` dengan header:
`"Global"` (`:92-96`) → global dari prefs (`:104-107`) → header dokumen (komponen terakhir fileURL, `:115-119`) → favorite dokumen dari `SPQueryController` (`:121-124`).

### 2.4 Add/edit/duplicate/delete

- Add `addQueryFavorite:` buat `{"name":"New Favorite","query":""}`; index sisip tergantung seleksi (`:201-254`).
- Duplicate `duplicateQueryFavorite:` hanya bila tepat 1 baris; nama jadi `"<name> Copy"` (`:259-265, 212`).
- Delete `removeQueryFavorite:` alert konfirmasi, hapus index terpilih terbalik (`:270-292`).
- Delete-all `removeAllQueryFavorites:` alert tapi handler utama `nil` → **no-op** (`:297-299`).
- Edit nama inline via `setObjectValue:...` (`:508-520`); tab trigger disanitasi buang regex `(?i)[^[:L:]0-9]+` (`:615-621`).
- Commit saat tutup: dokumen → `replaceFavoritesByArray:...`, global → `[prefs setObject:... SPQueryFavorites]`, post `SPQueryFavoritesHaveBeenUpdatedNotification` (`:438-471`).
- Drag-reorder via pasteboard `SPFavoritesPasteboardDragType = "SPFavoritesPasteboard"` (`SPConstants.m:70`).

### 2.5 Import/export

Export `.spf` (`:327-342`, `:833-871`): `{ version:1, format:"query favorites", encrypted:NO, queryFavorites:[…] }`
(`SPFVersionKey`/`SPFFormatKey` `SPConstants.m:394-395`, `SPFQueryFavoritesContentType = "query favorites"` `SPConstants.m:85`), XML plist atomik.
Save satu query ke `.sql`: `saveFavoriteToFile:` (`:304-325, 822-832`).
Import `importFavoritesByAdding:` (`:344-358`, `:761-817`). `importFavoritesByReplacing:` **kosong** (`:360-362`); item menunya `hidden="YES"` (`QueryFavoriteManager.xib:172-176`).

### 2.6 Placeholder & tab trigger

`insertPlaceholder:` tabel lookup (`:371-387`): `@100 "default_value"`, `@101 "$(shell_command)"`,
`@102 "¦a¦b¦"`, `@103 "¦¦a¦b¦¦"`, `@104 "¦"`, `@105 "$SP_SELECTED_TABLE"`,
`@106 "$SP_SELECTED_TABLES"`, `@107 "$SP_SELECTED_DATABASE"`, `@108/@109 "¦$SP_ASLIST_ALL_FIELDS¦"`,
`@110/@111 "¦$SP_ASLIST_ALL_TABLES¦"`, `@112/@113 "¦$SP_ASLIST_ALL_DATABASES¦"`.
Tag tak dikenal → `NSException` `Inserted placeholder (%lld) not found` (`:389`). Regex region snippet
`(?s)(?<!\\\\)\\$\\{(1?\\d):(.{0}|[^\\{\\}]*?[^\\\\])\\}` (`:398`).

Tab-trigger: di `SPTextView` keyDown, saat Tab dengan `tabTrigger` dan file URL, fetch via
`queryFavoritesForFileURL:andTabTrigger:includeGlobals:YES` dan sisipkan `query` dari match pertama
(`SPTextView.m:2401-2408`). Pencocokan key `"tabtrigger"`: dokumen dulu, lalu satu match global
(`break`) (`SPQueryController.m:1107-1132`).

### 2.7 Sheet simpan Swift (`SAQueryFavoriteSaveWindowController`)

`SAQueryFavoriteStore.save` (murni, testable) membuat `["name": name, "query": query]` atau, saat
replace, hanya memutasi `"query"` pada dict terpilih (`SAQueryFavoriteStore.swift:61-86`);
revalidasi seleksi terhadap list terbaru, melempar `.selectedFavoriteChanged` saat ambigu (`:112-137`).
Controller menulis scope dokumen via query controller atau global via prefs, lalu post
`.SPQueryFavoritesHaveBeenUpdated` (`SAQueryFavoriteSaveWindowController.swift:203-216`).

## 3. Display format (`SQLiteDisplayFormatManager`)

- File `ColumnDisplayOverrides.db` (`:24`); tabel `ColumnDisplayOverrides` (`:21`); di `<appSupport>/Data/` via `SASQLiteSharedStoreLocation.path` (`:24-27, 70-78`); `sharedInstance` app-target-only (`:24-37`).
- Versi dari `PRAGMA user_version`, target 1 (`:264-295`). DDL (`:209-223`):
```sql
CREATE TABLE IF NOT EXISTS ColumnDisplayOverrides (
  id            INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  hostName      TEXT NOT NULL,
  databaseName  TEXT NOT NULL,
  tableName     TEXT NOT NULL,
  columnName    TEXT NOT NULL,
  format        TEXT NOT NULL,
  CONSTRAINT host_db_table UNIQUE (hostName, databaseName, tableName, columnName)
)
CREATE UNIQUE INDEX IF NOT EXISTS host_db_table_idx ON ColumnDisplayOverrides (hostName, databaseName, tableName, columnName)
```
- Baca: satu kolom `SELECT ... WHERE hostName=? and databaseName=? and tableName=? and columnName=? ORDER BY id DESC` (`:82-87`); seluruh tabel `... WHERE hostName=? and databaseName=? and tableName=? ORDER BY id DESC` (`:119-124`).
- Tulis: `INSERT OR REPLACE INTO ColumnDisplayOverrides (hostName, databaseName, tableName, columnName, format) VALUES (?, ?, ?, ?, ?)` (`:161-163`).
- Perilaku: override sesi menjawab tiap baca apa pun kesehatan store (`:29-35,72-76,111-115`); nilai tersimpan default di bawah nilai sesi (`:143`); format kosong = "tanpa override" (`:147-150`). `isPersistent` false ⇒ in-memory (`:65-68`).
- Konsumen: `SPTableContent.m:5421` (tulis), `5456` (semua), `5489` (satu). Saat ini hanya format `"UUID"` (`:5443, 5473-5477`), hanya untuk kolom `binary` panjang 16 (`:5503`).

## 4. String yang terlihat pengguna (dikutip persis)

**History/console**: `"Query logging is currently disabled"` (`SPQueryController.m:367, 651`);
`"ConsoleLog"` (`:260`); `"Untitled %ld"` (`:808`); `"Clear History?"`/`"Clear"` (`SPCustomQuery.m:587`);
`"Are you sure you want to clear the global history list? This action cannot be undone."` (`:582`);
`"Are you sure you want to clear the history list for “%@”? This action cannot be undone."` (`:584`).

**Favorites**: `"[multiple selection]"`/`"[no selection]"` (`SPQueryFavoriteManager.m:47-48`);
`"Global"`/`"%@ Copy"`/`"New Favorite"` (`:93,212,219`); `"Remove selected query favorites?"` (`:277`);
`"Remove all query favorites?"` (`:298`); `"Error while reading data file"`/`"File couldn't be read. (%@)"`/`"No query favorites found."` (`:781,812`);
`"Error while converting query favorite data"` (`:863`); help URL `"https://sequel-ace.com/favorites.html"` (`:475`).

**Store SQLite**: `"Sequel Ace could not find or create its data folder in Application Support."` (`SQLiteDisplayFormatManager.swift:383`);
`"Column display formats are not being saved"`/`"Pinned tables are not being saved"` (`SASQLiteSharedStoreLocation.swift:32,53`);
`"Display formats you choose still work until you quit Sequel Ace, but they are not kept."` (`:25`).

## 5. UNKNOWN / celah

- `.SPQueryFavoritesHaveBeenUpdated`, `SPQueryController.shared()`, `replaceFavorites(by:forFileURL:)` — **tak terdefinisi di workspace**.
- `numberOfMaxAllowedHistory = 100` — di-set tapi tak dibaca.
- `removeAllQueryFavorites:` no-op (handler alert `nil`).
- `importFavoritesByReplacing:` kosong; item menunya hidden.
- Persistensi `.spf` history sengaja dilewati; history tidak round-trip lewat file sesi.
