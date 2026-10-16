# Audit: Server-tools subview cluster

Audit method-by-method klaster subview server-tools. `Start`/`End` = nomor baris
di file masing-masing. Baseline commit `02c910932`.

File: `SPProcessListController.m` (802), `SPServerVariablesController.m` (401),
`SPGotoDatabaseController.m` (454), `SPCharsetCollationHelper.m` (289).

---

## 1. `SPProcessListController.m` (802)

Controller sheet modal daftar proses server (`DatabaseProcessList` nib) — refresh, filter, auto-refresh, copy, save, kill query/connection.

### Ivar
`connection` (ivar+property+synthesize `.h:35,57`, `.m:71`); `showFullProcessList`/`processListThreadRunning` (BOOL `.h:37`); `autoRefreshTimer` (`.h:39`); `prefs` (`.h:41`); `processes`/`processesFiltered` (`.h:42`); 12 IBOutlet `.h:44–54`.

### Konstanta lokal
`SPKillProcessQueryMode` (42), `SPKillProcessConnectionMode` (43), `SPTableViewIDColumnIdentifier` (44), `SPTableViewProgressColumnIdentifier` (45), `SPKillModeKey` (47), `SPKillIdKey` (48). ⚠️ Empat konstanta kill (42, 43, 47, 48) **tak dirujuk di mana pun** (grep) — legacy/unused; hanya dua kolom identifier dipakai (114, 115, 298, 680).

### `#pragma mark`
`73/74 (Serialization), 83/84 (Initialisation), 149/150 (IB action methods), 346/347 (Other methods), 414/415 (Text field delegate methods), 432/433 (Window delegate methods), 449/450 (Private API), 728 (SPProcessListControllerDataSource — protokol tak ada), 730 (Tableview delegate methods), 792`.

Class extension `.m:52–67` deklarasi 13 prototipe — bukan definisi.

### Method (39 definisi)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 76 | 81 | `+_serializedProcessRow:includeProgress:` | Serialisasi satu baris proses via helper Swift |
| 86 | 103 | `init` | Muat nib, init prefs/array/flag full-list |
| 105 | 137 | `awakeFromNib` | Judul, show/hide kolom, font, lebar, observer |
| 142 | 147 | `windowDidLoad` | Sinkron UI interval auto-refresh |
| 155 | 181 | `copy:` | Copy baris proses terpilih |
| 186 | 190 | `closeSheet:` | Akhiri sheet dengan tag sender |
| 195 | 205 | `showWindow:` | Mulai timer auto-refresh bila pref on |
| 210 | 232 | `refreshProcessList:` | Guard busy, disable UI, spawn thread |
| 237 | 263 | `saveServerProcesses:` | Simpan list terfilter ke file |
| 268 | 277 | `killProcessQuery:` | Konfirmasi lalu kill query terpilih |
| 282 | 291 | `killProcessConnection:` | Konfirmasi lalu kill connection |
| 296 | 299 | `toggleShowProcessID:` | Toggle visibilitas kolom Id |
| 304 | 309 | `toggeleShowFullProcessList:` | Flip flag full-list + refresh |
| 314 | 322 | `toggleProcessListAutoRefresh:` | Start/stop timer auto-refresh |
| 327 | 330 | `setAutoRefreshInterval:` | Set interval dari tag sender |
| 335 | 344 | `setCustomAutoRefreshInterval:` | Tampilkan sheet interval kustom |
| 352 | 360 | `displayProcessListWindow` | Alias list terfilter, refresh, show window |
| 365 | 382 | `validateMenuItem:` | Enable item copy/kill/interval |
| 387 | 390 | `windowFrameAutosaveName` | `@"ProcessList"` |
| 395 | 412 | `observeValueForKeyPath:…` | Live-update gridlines/font dari pref |
| 420 | 430 | `controlTextDidChange:` | Terapkan filter / validasi interval kustom |
| 438 | 447 | `windowWillClose:` | Lepas array terfilter, kill timer |
| 455 | 478 | `_processListRefreshed` | Re-filter, re-sort, re-enable UI (main thread) |
| 483–506 | | `_startAutoRefreshTimer` / `_killAutoRefreshTimer` / `_fireAutoRefresh:` | Timer |
| 511–551 | | `_updateSelectedAutoRefreshIntervalInterface` / `_startAutoRefreshTimerWithInterval:` | Interval UI |
| 556 | 602 | `_getDatabaseProcessListInBackground:` | Fetch background, normalisasi Id/Time, push baris |
| 607–646 | | `_killProcessQueryWithId:` / `_killProcessConnectionWithId:` | Jalankan KILL (varian TiDB), alert saat error |
| 651 | 705 | `_updateServerProcessesFilterForFilterString:` | Filter baris by banyak kolom, update count/tombol |
| 710–726 | | `_addPreferenceObservers` / `_removePreferenceObservers` | Observer font + gridlines |
| 735–790 | | `numberOfRowsInTableView:` / `objectValueForTableColumn:row:` / `sortDescriptorsDidChange:` / `tableViewColumnDidResize:` | Datasource + sort + persist lebar |
| 794 | 800 | `dealloc` | Bersihkan flag thread, hapus observer |

### Temuan / UNKNOWN
- **SQL fetch**: `SHOW FULL PROCESSLIST` bila `showFullProcessList`, else `[connection listProcesses]` (564) — hanya bila `[connection isConnected]` (562).
- **SQL kill**: `KILL TIDB QUERY %lld`/`KILL QUERY %lld` (611, 614); `KILL TIDB CONNECTION %lld`/`KILL CONNECTION %lld` (633, 636). Cabang TiDB dari `[connection serverVersionString]` mengandung `"TiDB"` (610, 632). Error via `queryErrored`/`lastErrorMessage` (618–619, 640–641).
- **Konteks eksekusi kill**: dipanggil dari handler utama NSAlert (275, 289) **sinkron** — **UNKNOWN** apakah KILL bisa memblok main thread; tak ada dispatch background.
- **Copy**: bangun output `_serializedProcessRow:` join newline tanpa tab untuk baris terpilih (161–179, 168); serialisasi didelegasikan ke Swift `SPProcessListRowSerializer` (`StringExtension.swift:240–284`).
- **Save**: header `# MySQL server processes for <host>` lalu satu baris serialized per baris via `writeToURL:atomically:encoding:…error:NULL` (249, 253, 259) — error diabaikan senyap.
- **Filter**: substring case-insensitive pada Id/User/Host/db/Command/Time/State/Info + Progress (bila kolom tampil) (682–690); filter kosong alias `processesFiltered = processes` (666); label `"Showing %lu of %lu processes"` (698).
- **Truncation**: nilai string tepat 100 char + full list off → append `…` (753–755).
- **Normalisasi baris**: `Id` (64-bit) + `Time` (32-bit signed) dikonversi NSString → NSNumber (576–589) agar sort benar.
- Gating: `refreshProcessList:` abort bila dokumen `isWorking` atau thread background jalan (214, 217) — juga berlaku untuk refresh pasca-kill (623, 645).
- ⚠️ **Timer lifecycle**: `_killAutoRefreshTimer` invalidate tapi **tak pernah** nil `autoRefreshTimer` (491–498).
- ⚠️ **Toggle mencurigakan**: `toggleShowProcessID:` panggil `setHidden:([sender state])` langsung (298) — semantik inversi UNKNOWN.
- Definisi `_getDatabaseProcessListInBackground:` punya `;` nyasar di signature (556).
- **Protokol hantu**: `#pragma mark - SPProcessListControllerDataSource` (728) menamai protokol yang tak ada di codebase.

---

## 2. `SPServerVariablesController.m` (401)

Controller sheet variabel server (`DatabaseServerVariables` nib) — fetch `SHOW VARIABLES`, filter, copy name/value, save `.cnf`.

### Ivar
`connection` (`.h:35,46`, `.m:51`); `prefs` (`.h:37`); `variables`/`variablesFiltered` (`.h:38`); 4 IBOutlet `.h:40–43`. **Tak ada konstanta lokal.**

### `#pragma mark`
`53/54 (Initialisation), 85/86 (IBAction methods), 156/157 (Other methods), 232/233 (Tableview delegate methods), 251/252 (Text field delegate methods), 266/267 (Private API), 393`.

Class extension `.m:42–47` deklarasi 3 prototipe.

### Method (19 definisi)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 56–83 | | `init` / `awakeFromNib` | Muat nib, gridlines, row height, font, observer |
| 91–124 | | `copy:` / `copyServerVariableName:` / `copyServerVariableValue:` / `closeSheet:` | Copy + tutup |
| 129 | 154 | `saveServerVariables:` | Simpan variabel terfilter ke `.cnf` |
| 162 | 180 | `displayServerVariablesSheetAttachedToWindow:` | Alias array, fetch, filter, present sheet |
| 185–230 | | `validateMenuItem:` / `observeValueForKeyPath:…` | Menu + pref |
| 238–264 | | `numberOfRowsInTableView:` / `objectValueForTableColumn:row:` / `controlTextDidChange:` | Datasource + filter |
| 272 | 281 | `_getDatabaseServerVariables` | `SHOW VARIABLES`, reset array |
| 287 | 331 | `_updateServerVariablesFilterForFilterString:` | Filter by nama/nilai, update count/tombol |
| 336 | 371 | `_copyServerVariablesToPasteboardIncludingName:andValue:` | Bangun/copy string nama/nilai |
| 376–399 | | `_addPreferenceObservers` / `_removePreferenceObservers` / `dealloc` | Teardown |

### Temuan / UNKNOWN
- **SQL**: satu `SHOW VARIABLES`, hasil dipaksa string via `setReturnDataAsStrings:YES` (275, 277); array dibersihkan + diisi semua baris (279–280). **Tanpa** penanganan error/`queryErrored` (beda dari process list).
- **Filter**: substring case-insensitive pada `Variable_name` + `Value` (315–316); filter kosong alias + re-enable save (301–311); count `"%lu of %lu"` (325).
- **Save**: `NSSavePanel` `.cnf` via `UTType typeWithFilenameExtension:@"cnf"` (133); output `# MySQL server variables for <host>` lalu `name = value` per variabel (143, 147), `error:NULL`.
- **Copy**: helper emit `name = value`/name-only/value-only; butuh minimal satu dari nama/nilai (339); operasi pada `selectedRowIndexes` dijaga `safeObjectAtIndex:` (348–350).
- Titik gap lokalisasi: judul tombol save hardcoded `@"Save As..."`/`@"Save View As..."` (305, 330), beda dari versi terlokalisasi di SPProcessListController (669, 704).
- `closeSheet:` melepas `variablesFiltered` hanya bila alokasi berbeda (121–123).

---

## 3. `SPGotoDatabaseController.m` (454)

Dialog modal "Go To Database" (`GotoDatabaseDialog` nib) — fuzzy/word search atas nama database, highlight match, opsional entri nama kustom.

### Ivar
`.h:41–44` outlet `searchField`/`okButton`/`cancelButton`/`databaseListView`; `.h:46–52` `unfilteredList`, `filteredList`, `isFiltered`, `allowCustomNames` (+property `.h:61`, synthesize `.m:84`), `highlightAttrs`; kelas helper privat `SPGotoFilteredItem` (`.m:59–78`) dengan `string`/`matches`/`isCustomItem`.

### `#pragma mark`
`57, 80, 111/112 (IBAction), 191/192 (Public), 233/234 (Private), 321/322 (NSTableViewDataSource), 344/345 (NSTableViewDelegate), 390/391 (NSControlTextEditingDelegate), 436/437 (NSUserInterfaceValidations), 449`.

Fungsi C lokal: `static BOOL StringQualifiesForWordSearch(NSString *s)` (deklarasi 55, definisi 451–454). Class extension `.m:33–53` deklarasi 5 prototipe.

### Method (17 definisi)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 77 | 77 | `+item` (SPGotoFilteredItem) | Factory instance item terfilter |
| 86 | 102 | `init` | Muat nib, init list/flag/highlight attrs |
| 104 | 109 | `windowDidLoad` | Wire double-click list ke OK |
| 114–126 | | `okClicked:` / `cancelClicked:` | Hentikan modal YES/NO + hide |
| 128 | 169 | `searchChanged:` | Bangun ulang list terfilter, tambah item kustom, pilih baris |
| 171 | 189 | `toggleWordSearch:` | Bungkus/lepas quote teks search, re-search |
| 194 | 208 | `selectedDatabase` | Nama database terpilih atau nil |
| 210 | 215 | `setDatabaseList:` | Ganti list database |
| 217 | 231 | `runModal` | Reset field, fokus, jalankan app-modal loop |
| 236 | 314 | `_buildFilterList:didFindExactMatch:` | Bangun list match + highlight, deteksi exact |
| 316 | 319 | `qualifiesForWordSearch` | Apakah field saat ini ter-quote |
| 324–342 | | `numberOfRowsInTableView:` / `objectValueForTableColumn:row:` | Datasource |
| 347 | 388 | `tableView:willDisplayCell:forTableColumn:row:` | Terapkan styling highlight/underline |
| 393 | 434 | `control:textView:doCommandBySelector:` | Map ESC/arrow/Return ke aksi dialog |
| 439 | 445 | `validateUserInterfaceItem:` | Sinkron state item menu word-search |

### Temuan / UNKNOWN
- **Tanpa SQL/akses DB** — hanya filter in-memory list yang disuplai via `setDatabaseList:` (210–215).
- **Dua mode search** di `_buildFilterList:`: word search ter-quote (`rangeOfString:` dengan `NSCaseInsensitiveSearch|NSDiacriticInsensitiveSearch|NSWidthInsensitiveSearch`, 238, 249) vs fuzzy per-karakter via `nonConsecutivelySearchString:matchingRanges:` (272).
- Deteksi **exact-match** set `*exactMatch` bila satu match menutupi seluruh string (254–257, 277–284); menekan entri nama kustom saat ada exact match (146).
- **Custom name**: bila `allowCustomNames` + tak ada exact match, string search mentah (di-dequote) ditambahkan sebagai baris `isCustomItem` (146–157), dirender dengan `⁈` + underline (377–380).
- Sort: hasil terfilter diurut by jumlah match group terendah lalu alfabetis (295–312); dilewati di mode word-search (297).
- **Highlight attrs** (bg kuning, underline oranye, underline tebal) dibangun di `init` (92–96), diterapkan 383; item kustom dapat single underline (379).
- **Keyboard** di `control:textView:doCommandBySelector:`: ESC → cancel (396–399), Down/Up → pindah seleksi (407–425), Return → OK (428–431); bedakan key event nyata via `NSEventTypeKeyDown` (403–404).
- `StringQualifiesForWordSearch`: true hanya untuk panjang > 1 dibungkus `"`/`'` cocok (451–454).
- `selectedDatabase` bisa mengembalikan nama di luar list saat custom names aktif (didokumentasikan `.h:77–79`).
- **UNKNOWN**: `_buildFilterList:` tak membersihkan `filteredList` (tanggung jawab caller, `.m:42`; `searchChanged:` bersihkan di 130).

---

## 4. `SPCharsetCollationHelper.m` (289)

Helper non-view yang menjadi proxy antara pasangan popup charset + collation dan sumber data `SPDatabaseData`/`SPServerSupport`.

### Ivar
`.h:43–44` `charsetButton`/`collationButton`; `.h:46–54` property `databaseData`/`serverSupport`/`promoteUTF8`/`defaultCharset`/`defaultCollation`/`selectedCharset`/`selectedCollation`; `.h:53–54` `defaultCharsetFormatString`/`defaultCollationFormatString`; `.h:55` `_oldCharset`; `.h:57` `_enabled`. Synthesize `.m:49–58`. **Tak ada konstanta lokal.** Class extension `.m:38–45` deklarasi 4 prototipe.

### `#pragma mark`
**Tidak ada.**

### Method (7 definisi)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 60 | 80 | `initWithCharsetButton:CollationButton:` | Assert arg, set default, wire target tombol |
| 82 | 108 | `charsetButtonClicked:` | Update selected charset, reset + refresh collations |
| 110 | 124 | `collationButtonClicked:` | Update selected collation dari popup |
| 126 | 192 | `refreshCharsets` | Isi popup charset dari encoding server |
| 194 | 272 | `refreshCollations` | Isi popup collation untuk charset terpilih |
| 274–276 | | `enabled` | Getter `_enabled` |
| 278 | 287 | `setEnabled:` | Assert deps, init old charset, refresh charsets |

### Temuan / UNKNOWN
- **Tanpa SQL langsung** — data dari `[databaseData getDatabaseCharacterSetEncodings]` (131) + `[databaseData getDatabaseCollationsForEncoding:charsetId]` (221). `serverSupport` di-assert tapi **tak dibaca** di file ini (282).
- **Semantik seleksi**: index 0 = item "Default" → `selectedCharset`/`selectedCollation` nil (87–89, 115–117); item lain bawa charset id via `representedObject` (93, 150, 177) atau collation via title (121).
- **Deteksi perubahan**: ganti charset reset `selectedCollation` + refresh hanya saat charset benar-benar berubah vs `_oldCharset` (98–106); `_oldCharset` init di `setEnabled:` (283).
- **Promosi UTF-8**: dengan `promoteUTF8`, charset ber-prefiks `utf8` disisip di atas (157–168).
- **Injeksi item Default**: `defaultCharset` disisip index 0 diformat `defaultCharsetFormatString`, separator hanya bila ada item lain (175–181).
- **Default collation**: tiga kasus — inherited default (hanya saat charset nil/inherited) memakai `defaultCollationFormatString`, selain `IS_DEFAULT == "Yes"` jadi `Default (%@)` (234–241, 255–261); dummy "Default" ditambah dulu lalu dihapus (208, 238, 258).
- **`setEnabled:YES`** melakukan `NSAssert` non-nil `databaseData`/`serverSupport` (281–282) — trap di debug bila unset.
- **UNKNOWN**: `defaultCollationFormatString` dipakai di 236 tapi bisa menghasilkan `Default ((null))` bila `defaultCollation` nil — hanya dijaga oleh cabang `charsetIsInherited` (234), bukan nil check.

## Ringkasan lintas-file
- `SPProcessListController` + `SPServerVariablesController` adalah controller sheet hampir kembar (sama-sama `connection` property, `prefs`, pola filter/alias/observer, `-copy:`, `-closeSheet:`); bedanya process list menambah auto-refresh + kill + persist lebar kolom, variabel menambah ekspor `.cnf`.
- **Serialisasi di Swift**: satu-satunya formatter baris proses = `SPProcessListRowSerializer` (`StringExtension.swift:240–302`), dijangkau dari `+_serializedProcessRow:includeProgress:` (76–81).
- Tanpa SQL filter/injeksi di scope ini selain KILL/SHOW; `SPGotoDatabaseController` + `SPCharsetCollationHelper` tanpa SQL.
