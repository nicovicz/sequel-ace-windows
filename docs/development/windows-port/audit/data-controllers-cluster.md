# Audit: Data-controller cluster

Audit method-by-method klaster DataControllers. `Start`/`End` = nomor baris di
file masing-masing. Baseline commit `02c910932`.

File: `SPDatabaseData.m` (594), `SPDatabaseStructure.m` (722), `SPFavoritesController.m` (494),
`SPServerSupport.m` (234).

---

## 1. `SPDatabaseData.m` (594)

Peran: penyedia metadata charset/collation/storage-engine + variabel default server per-koneksi, dengan cache per-encoding lazy + fallback multi-sumber.

### Ivar
Header `.h:44–63`: `characterSetEncoding`, `defaultCollationForCharacterSet`, `defaultCharacterSetCache`, `defaultCollationCache`, `serverDefaultCharacterSetEncoding`, `serverDefaultCollation`, `defaultStorageEngine`, `collations`, `characterSetCollations`, `storageEngines`, `characterSetEncodings`, `cachedCollationsByEncoding`, `connection`, `serverSupport`, `charsetCollationLock`. Property `.h:68,73`. Class extension ivar `.m:62–63`: `_hasShownCharacterSetFallbackWarning`, `_hasShownCollationError`. **Tak ada static**; ada fungsi C non-static `_sortStorageEngineEntry` (deklarasi `.m:56`, definisi `.m:580–583`).

### `#pragma mark`
`.m:69/70 (Initialisation), 97/98 (Public API), 434/435 (Private API), 585/586 (Other)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 72 | 95 | `init` | Init cache, array, flag one-shot, lock; tanpa query |
| 105 | 127 | `resetAllData` | Bersihkan cache/collation/charset di bawah lock |
| 135 | 156 | `getDatabaseCollations` | Semua collation; info_schema lalu SHOW COLLATION |
| 165 | 218 | `getDatabaseCollationsForEncoding:` | Collation terfilter encoding, fallback alias utf8, cache per-encoding |
| 227 | 246 | `getDefaultCollationForEncoding:` | Cari baris IS_DEFAULT untuk encoding + cache |
| 256 | 265 | `getEncodingFromCollation:` | Map nama collation → charset |
| 272 | 287 | `getDatabaseStorageEngines` | Guard akses tabel engines, list engine didukung tersortir |
| 300 | 334 | `getDatabaseCharacterSetEncodings` | Charset via info_schema → SHOW → fallback |
| 343 | 350 | `getDatabaseDefaultCharacterSetForDatabase:` | Lookup `character_set_database` ter-cache per-DB |
| 359 | 366 | `getDatabaseDefaultCollationForDatabase:` | Lookup `collation_database` ter-cache per-DB |
| 375 | 384 | `getServerDefaultCharacterSet` | Fetch+cache `character_set_server` |
| 393 | 402 | `getServerDefaultCollation` | Fetch+cache `collation_server` |
| 411 | 432 | `getDatabaseDefaultStorageEngine` | Lookup variabel default engine tergantung versi |
| 442–468 | | `_getSingleVariableValue:` (2 varian) + `…assertingDatabase:databaseContextIsRequired:` | `SHOW VARIABLES LIKE`, return satu nilai |
| 474 | 483 | `_getDatabaseDataForQuery:` | Jalankan query, return semua baris dict |
| 485–502 | | `_normalizedCharacterSetEncodingsFromRows:` / `_fallbackCharacterSetEncodings` / `_showCharacterSetFallbackWarning` | Delegasi Swift + warning satu kali |
| 509–583 | | `_normalizedCollationRowsFromRows:` / `_sortedCollationRowsByName:` / `_utf8AliasForEncoding:` / `_getCollationRowsFromShowCollationForEncoding:` / `_sortStorageEngineEntry` | Normalisasi/sortir/mapping |
| 588 | 592 | `dealloc` | Panggil `resetAllData` |

### Temuan / UNKNOWN
- **Dua lapis cache**: `defaultCharacterSetCache`/`defaultCollationCache` = `SADatabaseScopedValueCache` (Swift, alokasi 77–78), dipakai `valueForDatabase:loader:` (346–348, 362–364), dibersihkan `[cache reset]` (116–117). Key = nama DB.
- ⚠️ **`cachedCollationsByEncoding` tak pernah dibersihkan** oleh `resetAllData` (105–127) — padahal `collations`/`characterSetEncodings`/`characterSetCollations` dibersihkan (121–123); diisi 211, dibaca 174. **UNKNOWN** apakah disengaja (data per-encoding dianggap global server).
- ⚠️ **`defaultStorageEngine` tak direset** oleh `resetAllData` (ditulis 428) — berbeda dari `serverDefault*` yang direset (118–119).
- Flag warning one-shot `_hasShownCharacterSetFallbackWarning`/`_hasShownCollationError` (62–63) menjaga alert (319–322, 202–207); direset 124–125.
- Fallback chain: `information_schema.collations` (141) → `SHOW COLLATION` (145); `…WHERE character_set_name` (182) → alias utf8 (188) → `SHOW COLLATION WHERE Charset` (572); `information_schema.character_sets` (309) → `SHOW CHARACTER SET` (313) → hardcoded (318); `SHOW TABLES IN information_schema LIKE 'ENGINES'` guard + `information_schema.engines` (276, 282); `SHOW VARIABLES LIKE` (454).
- Workaround utf8mb3 (#1064): `_utf8AliasForEncoding:` (559–560) + retry alias (184–196).
- **`goto`** di 177/215 (`copy_return`).
- **Thread-safety terbelah**: charset/collation `@synchronized(charsetCollationLock)`; `getDatabaseStorageEngines` + `getDatabaseDefaultStorageEngine` didokumentasikan NOT thread-safe (270, 409).
- Persistensi: **tidak ada** — semua state in-memory dari server live.

---

## 2. `SPDatabaseStructure.m` (722)

Peran: retrieval background + cache tree skema database (database/tabel/field/routine) di atas koneksi klon, ber-key connection ID, dengan lock serialisasi thread.

### Ivar
Header `.h:37–48`: `mySQLConnection`, `structure`, `allKeysofDbStructure`, `structureRetrievalThreads`, `threadManagementLock`, `dataLock`, `connectionCheckLock` (3 terakhir `pthread_mutex_t`). Property weak `delegate` `.h:49`; deklarasi ganda `- (SPDatabaseDocument *)delegate;` `.h:57`. **Tak ada static/#define.**

### `#pragma mark`
`.m:54, 58/59 (Setup and teardown), 118/119 (Information), 135/136 (Structure retrieval from the server), 457/458 (Structure information), 484/485 (SPMySQLConnection delegate methods), 500, 514/515 (Private API)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 64 | 68 | `init` | Raise exception; init langsung dilarang |
| 74 | 101 | `initWithDelegate:` | Init designated; array, mutex, observer close |
| 108 | 116 | `setConnectionToClone:` | Detach thread clone koneksi |
| 121 | 128 | `connection` | Accessor terkunci |
| 130 | 133 | `delegate` | Referensi weak |
| 138 | 144 | `queryDbStructureInBackgroundWithUserInfo:` | Spawn thread query struktur |
| 151 | 446 | `queryDbStructureWithUserInfo:` | Mesin retrieval/rekonsiliasi inti |
| 448 | 455 | `isQueryingDatabaseStructure` | Apakah thread retrieval aktif |
| 463 | 470 | `structure` | Salinan dict terkunci |
| 475 | 482 | `allStructureKeys` | Salinan array key terkunci |
| 490–498 | | `keychainPasswordForConnection:` / `credentialErrorMessageForConnection:` | Forward ke delegate |
| 502 | 512 | `dealloc` | Hapus observer, `_destroy:`, destroy mutex |
| 520 | 530 | `_destroy:` | Saat dokumen tutup, nil delegate + cancel thread |
| 535 | 549 | `_updateGlobalVariablesWithStructure:keys:` | Simpan struktur under connection ID (data lock) |
| 554 | 575 | `_cloneConnectionFromConnection:` | Copy connection, set delegate, pastikan connected |
| 586 | 612 | `_ensureConnectionUnsafe` | Reconnect klon; caller harus pegang lock |
| 621 | 648 | `_addToListAndWaitForFrontCancellingOtherThreads:` | Serialisasi retrieval; opsional cancel lain |
| 653 | 658 | `_removeThreadFromList` | Hapus thread aktif dari list |
| 663 | 678 | `_cancelAllThreadsAndWait` | Cancel + tunggu semua thread keluar |
| 686 | 720 | `_checkConnection` | Cek koneksi ber-lock menghormati cancellation |

### Temuan / UNKNOWN
- **Tiga mutex pthread** dengan disiplin terdokumentasi: init 95–97, destroy 508–510. `_checkConnection` (686–720) menjelaskan rasional `pthread_mutex_trylock` (hindari deadlock vs `_cloneConnectionFromConnection:`).
- **Format key**: `connectionID + SPUniqueSchemaDelimiter + db [+ table [+ field]]` (183, 218, 330, 360, 410–411); seluruh struktur disimpan under `connectionID` (545).
- **Layout array field**: `tmpDetailArray = [type, default, isnull, charset, collation, key, extra, priv, comment, type_display, uniqueCounter]` (364–374); routine `[dtd, dataaccess, det, security_type, definer, uniqueCounter]` (421–426); jenis node di key literal `"  struct_type  "` (377, 430).
- Query: `SHOW FULL COLUMNS FROM %@ FROM %@` per tabel (318); `information_schema.ROUTINES` filtered `ROUTINE_SCHEMA` untuk MySQL ≥5 (386–395).
- Cap/throttle: cap 2000 tabel+view per db (269–274); `usleep(10)` antar tabel (382); `usleep(1000000)` loop tunggu-front (643); `usleep(100000)` (673, 705).
- Reuse/invalidasi: seed dari delegate `getDbStructure`/`allSchemaKeys` (169–174); tambah DB unknown (178–179); prune key DB hilang (194–204); sistem `mysql`/`information_schema`/`performance_schema` **tak pernah** re-query (226); hormati flag `forceUpdate` (230); cache per-db diinvalidasi dengan buang `db_id` + key ber-prefiks (291–293).
- Notifikasi: post `SPDBStructureIsUpdating` (163) + `SPDBStructureWasUpdated` (243, 440); observe `SPDocumentWillCloseNotification` → `_destroy:` (89–92).
- **Dead code**: blok `#if 0` `affectedItem`/`affectedItemType` (276–288).
- **UNKNOWN/smell**: `.h:49` deklarasikan `@property delegate` **dan** `.h:57` deklarasi `- (SPDatabaseDocument *)delegate;` — duplikat getter synthesized yang diimpl manual (130–133).

---

## 3. `SPFavoritesController.m` (494)

Peran: singleton pengelola tree favorites koneksi in-memory, dipersist sebagai XML plist di Application Support.

### Ivar
Header `.h:44–50`: `favoritesTree`, `favoritesData`, `writeLock`, `favoritesLock` (2 terakhir `pthread_mutex_t`). Property readonly `.h:55,60` (synthesize 55–56). Static `sharedFavoritesController` `.m:40`. Class extension 42–51.

### `#pragma mark`
`.m:58/59 (Initialisation), 87, 105/106 (Favorites data handling), 150/151 (Favorites interaction), 239/240 (Private API), 485`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 61 | 68 | `allocWithZone:` | Redirect alokasi ke singleton |
| 70 | 85 | `init` | Nil tree/data, init mutex, load + bangun tree |
| 94 | 103 | `sharedFavoritesController` | Buat+return singleton (lazy) |
| 114 | 124 | `saveFavorites` | Detach thread simpan representasi tree |
| 129 | 132 | `saveFavoritesSynchronously` | Simpan di thread pemanggil |
| 140 | 148 | `reloadFavoritesWithSave:` | Opsional save, lalu reload + rebuild tree |
| 161 | 173 | `addGroupNodeWithName:asChildOfNode:` | Buat node group, add, save, post notifikasi |
| 183 | 191 | `addFavoriteNodeWithData:asChildOfNode:` | Tambah node favorite, save, post notifikasi |
| 196 | 221 | `_addFavoriteNodeWithData:asChildOfNode:` | Bangun node favorite/group rekursif dari data |
| 229 | 237 | `removeFavoriteNode:` | Hapus node dari tree, save, post notifikasi |
| 246 | 294 | `_loadFavorites` | Load/buat Favorites.plist di Application Support |
| 299 | 341 | `_constructFavoritesTree` | Bangun tree SPTreeNode dari root favoritesData |
| 351 | 380 | `_constructBranchForNodeData:` | Bangun branch rekursif dari data group/favorite |
| 387 | 467 | `_saveFavoritesData:` | Serialisasi + tulis plist atomik dengan backup/restore |
| 475 | 483 | `_addNode:asChildOfNode:` | Append node ke parent atau group default |
| 487 | 492 | `dealloc` | Destroy kedua mutex |

### Temuan / UNKNOWN
- **Singleton**: `allocWithZone:` return `[self sharedFavoritesController]` (61–68) — `return nil;` di 67 unreachable setelah return `@synchronized`; `sharedFavoritesController` pakai double-check `@synchronized(self)` + `[[super allocWithZone:NULL] init]` (94–103).
- **Format persistensi**: XML plist root dict `@{SPFavoritesRootKey: data}` (403–408), atomik (284, 447). File = Application Support `SPDataSupportFolder` + `SPFavoritesDataFile` (255/263, 417/425). Konstanta key di luar file — **UNKNOWN** nilai persisnya.
- **Protokol backup/restore save**: plist lama → backup UUID (426–431); gagal move → coba delete (433–444); gagal write → restore backup (449–460); sukses → hapus backup (462–464). Data diserialisasi **sebelum** menyentuh file (399–408).
- **Dua mutex beda peran**: `favoritesLock` (load/build/saveFavorites enqueue: 116, 248, 301); `writeLock` (serialisasi write disk: 390). Keduanya destroy di dealloc (489–490).
- Bentuk tree: root `SPGroupNode` + group "Favorites" (nama di-uppercase, 314–320); child dari `SPFavoriteChildrenKey` (325). Group vs favorite dari keberadaan `SPFavoriteChildrenKey` (201–207, 356–377). Root non-dict → log error, `favoritesTree` tak berubah (312–338).
- Notifikasi: `SPConnectionFavoritesChangedNotification` dipost setelah add-group/add-favorite/remove (170, 188, 236).
- `saveFavorites` mengirim snapshot `[[[favoritesTree childNodes] objectAtIndex:0] dictionaryRepresentation]` dihitung di thread pemanggil sambil pegang `favoritesLock` (121), lalu ditulis off-thread.
- **UNKNOWN**: `_addNode:asChildOfNode:` mengasumsikan `favoritesTree` punya ≥1 child saat `parent` nil (481) — tanpa guard nil selain `parent`.

---

## 4. `SPServerSupport.m` (234)

Peran: helper perbandingan versi murni — diberi nomor versi MySQL, mengekspos flag `supports*`; tak memiliki koneksi.

### Ivar
Header `.h:49–64`: `isMySQL5`, `isMySQL8`, `supportsFulltextOnInnoDB`, `supportsFractionalSeconds`, `serverMajorVersion`, `serverMinorVersion`, `serverReleaseVersion`. Property `.h:69,74,79` (synthesize 52–54) + readonly `.h:84,89,94,99` (synthesize 49–55). Class extension 35–45. **Tak ada static.**

### `#pragma mark`
`.m:57/58 (Initialisation), 84/85 (Public API), 163/164 (Private API), 219/220 (Other)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 70 | 82 | `initWithMajorVersion:minor:release:` | Simpan versi lalu panggil `evaluate` |
| 99 | 110 | `evaluate` | Hitung semua flag dari nomor versi |
| 125 | 133 | `isEqualToOrGreaterThanMajorVersion:minor:release:` | Banding versi server vs versi pasokan |
| 140 | 161 | `description` | String debug enumerasi ivar `supports*` via runtime |
| 170 | 177 | `_invalidate` | Reset semua flag ke NO |
| 194 | 217 | `_compareServerMajorVersion:minor:release:withServerMajorVersion:minor:release:` | Urutkan dua triple versi |
| 222 | 232 | `dealloc` | Zero integer versi + invalidate |

### Temuan / UNKNOWN
- **Threshold capability**: `isMySQL5 = (major == 5)` (104); `isMySQL8 = (major >= 8)` (105); `supportsFractionalSeconds` + `supportsFulltextOnInnoDB` = `>= 5.6.4` (108–109).
- `evaluate` idempoten: `_invalidate` dulu (102) lalu recompute.
- Pengurutan versi: major → minor → release sebagai integer (201–216). Doc 182–183: release `XX` > `X` (5.0.18 > 5.0.8) — integer memenuhi ini; string release zero-padded tak ditangani — **UNKNOWN/edge**.
- **`description` refleksi**: `class_copyIvarList` (145), filter prefix `"supports"` (151), baca via `object_getIvar` (152), free list (158). ⚠️ **`object_getIvar` diterapkan ke BOOL (primitif)** — API pointer objek, jadi pemakaian teknis salah meski mungkin tampak jalan; **bug laten**, tak diverifikasi runtime.
- Persistensi: **tidak ada**.
