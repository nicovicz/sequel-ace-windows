# Audit: History + SQLite store cluster

Audit method-by-method klaster history/SQLite. `Start`/`End` = nomor baris di file
masing-masing. Baseline commit `02c910932`.

File: `SPHistoryController.m` (499), `SQLiteHistoryManager.swift` (491),
`SQLiteDisplayFormatManager.swift` (471), `SQLitePinnedTableManager.swift` (460), `GitHub.swift` (551).

---

## 1. `SPHistoryController.m` (499)

Controller navigasi back/forward per-dokumen; rekam view state ke `SPTableHistoryManager` dan restore sort/page/selection/filter state.

### Ivar
Extension (39–55). `@synthesize modifyingState; navigatingFK` (59–60). Ivar `.h:38–47`: outlet `theDocument`/`historyControl`, `tableContentInstance`, `tablesListInstance`, `tableContentStates` (`NSMutableDictionary` keyed `db.table`, lihat 230, 383), `historyManager`, `modifyingState`, `navigatingFK`, `toolbarItemVisible`. Property `.h:50–51`.

### `#pragma mark`
`62 (Setup and teardown), 94 (Interface interaction), 206 (Adding or updating history entries), 260 (Loading history entries), 361 (Restoring view states), 400 (Private Helper c-Functions)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 67–92 | | `-init` / `-awakeFromNib` / `-dealloc` | Setup + teardown |
| 96–176 | | `countPrevious` / `countForward` / `updateToolbarItem` / `goBackInHistory` / `goForwardInHistory` / `historyControlClicked:` / `setupInterface` / `startDocumentTask:` / `endDocumentTask:` | Interaksi UI |
| 189–204 | | `toolbarWillAddItem:` / `toolbarDidRemoveItem:` | Toolbar |
| 208–258 | | `updateHistoryEntries` / `updateHistoryEntriesWithUpdateToolbarItem:` / `buildEntry` | Tambah/update entry |
| 266–359 | | `loadEntryStart:` / `loadEntryTaskWithEntry:` / `loadEntryFromMenuItem:` | Load entry |
| 366–398 | | `restoreViewStates` / `doRetoreViewState:` | Restore view state |
| 402–497 | | `canCurrentReplaceEntry` / `abortEntryLoad` / `completeLoadEntry` / `menuForEntries` / `menuEntryFor` / `menuItemDescriptionFor` (C statis) | Helper |

### Temuan
- **Semantik history**: entry baru di-push kecuali `canCurrentReplaceEntry` izinkan replace; db sama + (tanpa tabel atau tabel+view sama dengan filter tak berubah/nil) → replace; view identik dengan filter berubah menyalin viewport/selection ke entry **sebelumnya** dan return NO (409–429).
- Key view-state cache: `database.backtickQuotedString + "." + table.backtickQuotedString` (230, 383).
- **UNKNOWN**: struct `SPTableHistoryEntry` (database/table/view/viewPort/contentSortColName/IsAsc/contentPageNumber/selectedRows/activeFilter/filter/filterData/cachedMenuItem) dideklarasi forward `.h:34` — mungkin `SPTableHistoryEntry.swift`. `SPTableHistoryManager` juga di luar file.
- Restore di-skip saat `modifyingState` atau baru navigasi FK (`navigatingFK`, 366–374).

---

## 2. `SQLiteHistoryManager.swift` (491)

Singleton yang mendukung daftar history Custom Query dengan SQLite `queryHistory.db` + migrasi satu kali history plist lama.

### Tipe/schema
`typealias SASchemaBuilder` (13); `final class SQLiteHistoryManager: NSObject` (15) dengan `sharedInstance` (16), `migratedPrefsToDB` (17, dari `SPMigratedQueriesFromPrefs`), `showLogging=false` (18), `queryHist: [Int64: String]` (19), `queue: FMDatabaseQueue` (20), `maxSizeForCrashlyticsLog = 64000` (21), `traceExecution` (22), `sqlitePath` (23, `<appSupport>/Sequel Ace/Data/queryHistory.db` dibangun di 49), `dbSizeHumanReadable`/`dbSize` (24–25), `additionalHistArraySize` (26), `prefs` (27), `Log` (28), `newSchemaVersion: Int32 = 0` (29).

Schema: tabel `QueryHistory(query TEXT NOT NULL, createdTime REAL NOT NULL)` v1 (92–94), index unik `query_idx` on `query` (98); migrasi v2 rename `QueryHistory_Old`, recreate dengan `id integer PRIMARY KEY AUTOINCREMENT NOT NULL`, copy, drop, recreate index (122–152, preview SQL 114–134); versi via `PRAGMA user_version` (172, 185). SQL runtime: `SELECT id, query FROM QueryHistory order by id desc LIMIT (?)` (207); `INSERT OR IGNORE INTO QueryHistory (query, createdTime) VALUES (?, ?)` (275, 311); `DELETE FROM QueryHistory` (331); `vacuum` (356); page-size query (233).

### `#pragma mark`
**Tidak ada.**

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 31 | 78 | `init` | Resolve path, buka queue, setup schema, migrasi/load, ukur ukuran |
| 82 | 196 | `setupQueryHistoryDatabase` | Blok schema: buat v1, migrasi v2, tulis user_version |
| 199–246 | | `loadQueryHistory` / `reloadQueryHistory` / `getDBsize` | Load + ukuran |
| 249 | 289 | `migrateQueriesFromPrefs` | Sisip history prefs lama (urutan terbalik) + tandai migrasi |
| 300–362 | | `updateQueryHistory(newHist:)` / `deleteQueryHistory` / `execSQLiteVacuum` | Update/hapus/vacuum |
| 368–378 | | `failed(error:)` / `logDBError(_:)` | Error |
| 388–490 | | `normalizeQueryHistory(arrayToNormalise:)` / `queryMightBeMultiLine(queryToCheck:)` / `appendToQueryHistory(arrayToAppendTo:queryToAppend:)` | Normalisasi query |

### Temuan
- **Migrasi**: baca array string `SPQueryHistory` (asc by creation), reverse agar dropdown tampil terbaru dulu (259–265); **tidak** menghapus key prefs by design (283–284); set `SPMigratedQueriesFromPrefs = true` (287). Key hilang → tetap unmigrated (250–254).
- ⚠️ **Potensi bug**: `startingSchemaVersion` di-assign dua kali, baris kedua menimpa dengan `rs.long` (175–176); `newSchemaVersion` property instance dimutasi di dalam closure schema (104, 111, 157) — state mutable bersama lintas dua jalur schema.
- Queue di-force-unwrap (44, 59) → crash bila `FMDatabaseQueue(path:)` nil; `setupQueryHistoryDatabase` bisa jalan dengan `sqlitePath == ""` di jalur error (43–46).
- `migratedPrefsToDB` dideklarasi tanpa initializer tapi di-assign di `init` sebelum `super.init()` (17, 33, 41); `showLogging` tampak tak dipakai. **UNKNOWN** consumer `queryHist`/`updateQueryHistory` (di `SPCustomQuery`).

---

## 3. `SQLiteDisplayFormatManager.swift` (471)

Memersist override display-format per-kolom di `ColumnDisplayOverrides.db`, dengan fallback in-session + pelaporan masalah.

### Tipe/schema
`typealias SchemaBuilder` (19); `final class SQLiteDisplayFormatManager: NSObject` (18) dengan `sqliteTableName = "ColumnDisplayOverrides"` (21), `dbFileName = "ColumnDisplayOverrides.db"` (24), `log` (25), `queue: FMDatabaseQueue?` (28), `stateLock: NSLock` (30), `sessionOverrides: [SADisplayFormatTableKey: [String: String]]` (35), `databasePath: String?` (37), `problems: SASQLiteStoreProblemReporter` (43).
`struct SADisplayFormatTableKey: Hashable` (312–322, host/db/table sebagai `Data` byte-exact); `enum SASQLiteStoreProblemKind` (332–341); `final class SASQLiteStoreProblem: NSObject` (345–418); `final class SASQLiteStoreProblemReporter` (423–470); `fileprivate extension FMDatabase` (299–307).

Schema: tabel `ColumnDisplayOverrides(id INTEGER PK AUTOINCREMENT NOT NULL, hostName, databaseName, tableName, columnName, format TEXT NOT NULL, UNIQUE(host,database,table,column))` v1 (209–220); index unik `host_db_table_idx` (221–223); versi via `PRAGMA user_version` (266, 292). SQL runtime: SELECT by host/db/table/column ORDER BY id DESC (82–87); SELECT by host/db/table ORDER BY id DESC (119–124); `INSERT OR REPLACE INTO … VALUES (?,?,?,?,?)` (161–163).

### `// MARK:`
`324 (Store problems)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 50 | 63 | `init(databasePath:)` | Buka/buat store, lapor masalah pertama bila unusable |
| 66–68 | | `isPersistent` (var) | True saat store usable ada |
| 72–145 | | `displayOverrideFor(...)` / `allDisplayOverridesFor(...)` | Baca override |
| 151 | 174 | `replaceOverrideFor(...colName:format:)` | Set format di session + store (INSERT OR REPLACE) |
| 180–295 | | `openStore(at:problem:)` / `setupDatabase(in:)` / `loadCurrentSchemaVersion(_:)` / `verifyTable(_:)` / `finalizeSchemaVersion(_:_:_:)` (static) | Store/schema |
| 300–306 | | `FMDatabase.executeQuery(_:)` / `executeUpdate(_:)` | Wrapper throwing |
| 317–321 | | `SADisplayFormatTableKey.init` | Encode host/db/table → UTF-8 Data |
| 360–417 | | `SASQLiteStoreProblem.init` / `message(consequence:)` / `revealURL` (var) | Pesan masalah |
| 430–469 | | `firstProblem` / `report(_:)` / `onProblem(_:)` | Pelaporan |

### Temuan
- Store hanya kenyamanan: store unusable → format ditahan in-memory, hilang launch berikutnya; tak pernah terminasi app (10–17, 27–35). Baca gagal hanya di-log; tulis gagal dilaporkan (38–43, 137–139, 168–171).
- **Migrasi/idempotensi**: `CREATE TABLE IF NOT EXISTS` sengaja diadopsi dari file v0 peninggalan launch crash (202–208); tabel asing gagal `verifyTable` dan dibiarkan (228–235).
- `INSERT OR REPLACE` mengandalkan unique constraint (162, 218); peta session ditulis sebelum store (153–155), jadi baca selalu mencerminkan pilihan user.
- **UNKNOWN**: `SASQLiteSharedStoreLocation.swift` (`sharedInstance` app-target, penempatan folder) dirujuk komentar (22–23, 48–49) tapi di luar file ini.

---

## 4. `SQLitePinnedTableManager.swift` (460)

Memersist pinned table per host/koneksi+database di `pinnedTables.db`, dengan migrasi legacy host→connectionID.

### Tipe/schema
`final class SQLitePinnedTableManager: NSObject` (21) dengan `log` (23), `dbFileName = "pinnedTables.db"` (26), `traceSQLiteExecutionsKey = "SPTraceSQLiteExecutions"` (31), `migratedPinnedTablesKey = "SPMigratedPinnedTablesToConnectionIDs"` (34), `queue: FMDatabaseQueue?` (37), `storeWasLoaded: Bool` (41), `prefs`/`traceExecution` (42–43), `databasePath: String?` (45), `problems` (48), `stateLock: NSLock` (51), `migratedLegacyPinnedTableTokens: Set<String>` (52), `pinnedTablesDatabaseDictionary: [String:[String:[String]]]` (53), `sessionOnlyMigratedTokens: Set<String> = []` (60), `recordGeneration = 0` (63), `sqliteConstraint = 19` (68), `SAMigrationRecord` (328–331).

Schema: tabel `PinnedTables(id INTEGER PK AUTOINCREMENT NOT NULL, hostName, databaseName, pinnedTableName, UNIQUE(host,database,pinnedTable))` v1 (155–160); index `host_db_idx` (164); versi via `PRAGMA user_version` (185, 195). SQL runtime: `SELECT hostName, databaseName, pinnedTableName FROM PinnedTables order by id desc` (224); `INSERT INTO PinnedTables (…) VALUES (?,?,?)` (408); `DELETE FROM PinnedTables where …` (376); `SELECT 1 FROM PinnedTables WHERE … LIMIT 1` (438).

### `#pragma mark`
**Tidak ada.**

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 80 | 111 | `init(databasePath:prefs:)` | Buka store, load pin, pertahankan store hanya bila baca sukses |
| 114–208 | | `isPersistent` (var) / `openStore(at:traceExecution:problem:)` / `setupPinnedTablesDatabase(...)` | Store + schema |
| 217 | 250 | `loadPinnedTablesHistory(from:traceExecution:)` | Baca semua pin (terbaru dulu) |
| 254–266 | | `getPinnedTables(hostName:databaseName:)` / `pinTable(...)` | Baca/pin |
| 280 | 325 | `migratePinnedTablesFromLegacyHost(_:toConnectionIdentifier:databaseName:)` | Pindah pin legacy host → connection key sekali per tuple |
| 344–385 | | `persistMigrationRecord(_:)` / `unpinTable(...)` | Migrasi + unpin |
| 395–459 | | `pinLocked(...)` / `storeHoldsPin(...)` / `addToPinnedTablesDatabaseDictionary(...)` / `logDBError(_:)` | Privat |

### Temuan
- Store unusable → pin tetap jalan in-memory, tak dipersist; app tak pernah terminasi (16–19).
- **Owner-rule**: store yang terbuka tapi tak bisa dibaca akan dibuang (`loadPinnedTablesHistory` sudah menutupnya) agar write tak menambah baris di samping yang belum ter-load (101–105); baris tanpa host/db/table name membatalkan seluruh baca (228–238).
- **Migrasi legacy**: diklaim + dipindah dalam satu langkah terkunci agar dokumen konkuren bermigrasi sekali; dijaga `storeWasLoaded` + `everyPinStored` (306–319). Bila tak tersimpan penuh, token masuk `sessionOnlyMigratedTokens` (tak dipersist) agar launch berikutnya retry (54–60, 313–316).
- `persistMigrationRecord` menulis di luar lock sengaja (observer defaults menunggu main thread yang mungkin pegang lock); writer yang menemukan generasi lebih baru loop menulisnya juga (334–358).
- **Constraint handling**: kode constraint di-mask `code & 0xFF == 19` + dikonfirmasi `storeHoldsPin`, agar constraint non-unique bukan salah dibaca sebagai duplikat idempoten (410–421).
- **UNKNOWN**: `PinnedTableMigrationPlanner` + `SASQLiteSharedStoreLocation.swift` dirujuk (285, 301, 74–75) tapi di luar file.

---

## 5. `GitHub.swift` (551)

Kebijakan logika murni untuk cek update GitHub in-app: decode/urutan release, phased rollout, settling, pagination, presentasi error.

### Tipe/konstanta
`enum SAAppStoreReceiptPolicy` (12–23); `final class SAGitHubReleaseCheckPolicy` (26–35); `enum SAGitHubReleaseAppVariant` (37–40); `struct SAGitHubReleaseTagIdentity` (42–76, parse `channel/version-build`); `struct SAGitHubRelease: Decodable, Comparable` (83–267) dengan `canonicalBetaArtifactNamingMinimumBuild = 20_110` (84), `settlingInterval = 15*60` (85); `enum SAGitHubReleaseRolloutPolicy` (271–362) dengan `installationSeedPreferenceKey = "SAGitHubReleaseRolloutSeed"` (272), `dailyPercentages = [1,2,5,10,20,50,100]`/`cohortBucketCount = 10_000`/`day = 86400` (274–276); `enum SAGitHubReleaseSettlingPolicy` (364–392); `struct SAGitHubReleaseCheckTracker` (394–421); `enum SAGitHubReleasePagination` (423–467, `pageSize=100`/`maximumPageCount=10`); `struct SAGitHubReleaseAsset: Decodable` (469–510); `enum SAGitHubReleaseErrorPolicy` (513–551, `ephemeralURLErrorCodes` 535–548, `ephemeralHTTPStatusCodes = [403,408,425,429]` 550).

### `#pragma mark`
**Tidak ada.**

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 13–34 | | `isAppStoreInstall(...)` / `shouldCheck(...)` | Gate cek |
| 47–110 | | `SAGitHubReleaseTagIdentity.init?(_:)` / `isASCIIDigits(_:)` / `<` / `==` | Identitas + ordering |
| 127–267 | | `init(from:)` / `decodeList(from:)` / `matchesInstalledBuild(...)` / `compatibleAppZip(for:)` / `inferredAppVariant` / `onlyAsset(in:)` / canonical/settling/tagIdentity/phasedRolloutStartedAt/latestReleaseChange | Model release |
| 278–361 | | `rolloutPercentage(at:startedAt:)` / `shouldOffer(...)` / `newestOfferedRelease(...)` / `installationSeed(in:)` / `cohortBucket(...)` | Rollout |
| 365–420 | | `retryDelay(...)` / `retryOperation(...)` / `SAGitHubReleaseCheckTracker.*` | Settling + tracker |
| 427–509 | | `nextPageURL(...)` / `SAGitHubReleaseAsset.*` | Pagination + asset |
| 514 | 533 | `shouldPresentError(...)` | Presentasi error |

### Temuan
- **Kebijakan seleksi**: terbaru = `max()` by published date lalu build per-channel; release lama yang sudah full-rollout sengaja **tidak** ditawarkan selama terbaru masih phased (314–333). `draft`/`prerelease` hilang decode → `true` agar respons parsial tak tampak publik (133–135).
- **Artifact**: butuh tepat satu ZIP cocok (`onlyAsset`, 204–206); canonical beta naming untuk channel beta build ≥ 20110 (208–215, 222–244); release lebih lama fallback heuristik alpha/beta (177–190).
- **Rollout**: mirror persentase phased 7 hari Apple (269, 274–285); seleksi cohort pakai hash FNV-1a seed+tag karena `Hasher` Swift per-process acak (349–360); cek manual bypass gate (295–296).
- **Pagination**: hard-limit 10 halaman × 100 (424–425), validasi link `https`/`api.github.com` (454–457); stop setelah build terpasang ditemukan (437–439).
- **UNKNOWN**: layer networking yang fetch/decode release + memanggil kebijakan ini tak ada di file — bagaimana `receiptURL`, HTTP status, dan `perform` dipasok tak diverifikasi. `build` di `SAGitHubReleaseTagIdentity` menolak build leading-zero (55) — rasional tak terdokumentasi.
