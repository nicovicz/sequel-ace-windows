# Audit: `SPQueryController.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/SubviewControllers/SPQueryController.m`. Baseline commit
`02c910932`.

- File: 1176 baris; header `SPQueryController.h` 149 baris.
- Peran: singleton `NSWindowController` untuk window Query Console — log pesan/
  filter/copy/save, data source tabel, container favorites/history/content-filter
  per dokumen, dan daftar completion SQL.
- Total: 52 definisi method.

## 1. Inventaris ivar/property

### Static/global `.m`
`SPQueryConsoleWindowAutoSaveName = @"QueryConsole"` (44, extern `.h:31`);
`SPTableViewDateColumnID`/`ConnectionColumnID`/`DatabaseColumnID` (45–47, `.h:32-34`);
`SPCompletionTokensFilename = @"CompletionTokens.plist"` (49); key plist keyword/function/snippets (51–53);
`SPMessageTruncateCharacterLength = 256` (55); `sharedQueryController` (71).

### Class extension `.m:59–69`
`<NSMenuItemValidation, NSControlTextEditingDelegate>` (59); deklarasi privat `_updateFilterState`,
`_allowFilterClearOrSave:`, `_messageMatchesCurrentFilters:`, `_getConsoleStringWithTimeStamps:connections:databases:`,
`_addMessageToConsole:connection:isError:database:` (60–64); `@property _SQLiteHistoryManager` (66).
`@synthesize consoleFont, _SQLiteHistoryManager` (75).

### Ivar `.h:38–73`
IBOutlet: `saveLogView`, `consoleTableView`, `consoleSearchField`, `loggingDisabledTextField`,
`progressIndicator`, `includeTimeStampsButton`/`includeConnectionButton`/`includeDatabaseButton`,
`saveConsoleButton`/`clearConsoleButton` (38–47). BOOL: `showSelectStatementsAreDisabled`,
`showHelpStatementsAreDisabled`, `filterIsActive`, `allowConsoleUpdate` (49–52). `consoleFont` (54).
`activeFilterString` (55); `messagesFullSet`/`messagesFilteredSet`/`messagesVisibleSet` (56);
`favoritesContainer`/`historyContainer`/`contentFilterContainer` (59–61); `untitledDocumentCounter` (62);
`numberOfMaxAllowedHistory` (63); `completionKeywordList`/`completionFunctionList`/`functionArgumentSnippets` (65–67);
`prefs` (69); `dateFormatter` (70); `consoleLock` (`pthread_mutex_t`) (72).

## 2. Semua `#pragma mark`

`146/147 (QueryConsoleController), 341/342 (Other), 442/443 (Privat API), 628 (SPQueryControllerInitializer), 731 (SPQueryConsoleDataSource), 802 (SPQueryDocumentsController), 1134/1135 (Completion list controller), 1163`.
`@implementation` 73; `@end` 1176.

## 3. Tabel method (52 definisi, urut baris)

### Singleton/init
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 80 | 91 | `+sharedQueryController` | Kembalikan singleton (lazy) |
| 93 | 138 | `init` | Init state, container, SQLite manager, muat daftar completion |
| 144 | 144 | `copyWithZone:` | Kembalikan self untuk semantik singleton |

### QueryConsoleController (147)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 152 | 172 | `copy:` | Copy baris console terpilih/diklik ke pasteboard |
| 177 | 193 | `copyQueryOnly:` | Copy baris terpilih tanpa prefiks metadata |
| 195 | 233 | `infoStringForRowIndexes:includeTimestamps:includeConnections:includeDatabases:` | Bangun string newline-joined dari baris terlihat |
| 238 | 243 | `clearConsole:` | Kosongkan set pesan full/filtered + reload |
| 248 | 269 | `saveConsoleAs:` | Save panel, tulis konten console ke file |
| 274 | 277 | `toggleShowTimeStamps:` | Show/hide kolom timestamp |
| 282 | 285 | `toggleShowConnections:` | Show/hide kolom connection |
| 290 | 293 | `toggleShowDatabases:` | Show/hide kolom database |
| 298 | 304 | `toggleShowSelectShowStatements:` | Toggle sembunyikan pesan SELECT/SHOW, refresh filter |
| 309 | 315 | `toggleShowHelpStatements:` | Toggle sembunyikan pesan HELP, refresh filter |
| 320 | 323 | `showMessageInConsole:connection:database:` | Tambah pesan non-error ke console |
| 328 | 331 | `showErrorInConsole:connection:database:` | Tambah pesan error ke console |
| 336 | 339 | `consoleMessageCount` | Jumlah set pesan penuh |

### Other (342)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 347 | 358 | `controlTextDidChange:` | Update string filter search + refresh state filter |
| 363 | 384 | `observeValueForKeyPath:ofObject:change:context:` | Handler KVO pref logging/gridline/font |
| 389 | 411 | `validateMenuItem:` | Enable/disable item copy/copy-query/clear |
| 413 | 416 | `allowConsoleUpdate` | Getter flag update console |
| 418 | 423 | `setAllowConsoleUpdate:` | Set flag; refresh bila window terlihat |
| 428 | 432 | `updateEntries` | Reload tabel + scroll ke baris terakhir |
| 437 | 440 | `windowFrameAutosaveName` | Nama autosave window console |

### Privat API (443)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 449 | 505 | `_updateFilterState` | Hitung ulang set pesan filtered/visible dari filter aktif |
| 510 | 514 | `_allowFilterClearOrSave:` | Enable/disable tombol save & clear console |
| 520 | 543 | `_messageMatchesCurrentFilters:` | Tes satu pesan terhadap filter teks/SELECT/SHOW/HELP |
| 549 | 583 | `_getConsoleStringWithTimeStamps:connections:databases:` | Bangun teks console penuh untuk save file |
| 588 | 626 | `_addMessageToConsole:connection:isError:database:` | Lock, append pesan, update set filtered/tabel |

### SPQueryControllerInitializer (628)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 633 | 672 | `awakeFromNib` | Wire pref, visibilitas kolom, font, observer saat nib dimuat |
| 677 | 729 | `loadCompletionLists` | Muat plist keyword/function/snippets ke ivar |

### SPQueryConsoleDataSource (731)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 736 | 739 | `numberOfRowsInTableView:` | Jumlah set pesan terlihat |
| 744 | 785 | `tableView:objectValueForTableColumn:row:` | Nilai sel dengan format, truncate, warna error |
| 787 | 800 | `tableView:writeRowsWithIndexes:toPasteboard:` | Tulis baris drag ke pasteboard sebagai string |

### SPQueryDocumentsController (802)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 804 | 888 | `registerDocumentWithFileURL:andContextInfo:` | Registrasi URL dokumen, seed container favorites/history/content-filter |
| 890 | 921 | `removeRegisteredDocumentWithFileURL:` | Hapus entri dokumen kecuali instance lain berbagi URL |
| 923 | 931 | `replaceContentFilterByArray:ofType:forFileURL:` | Ganti satu tipe content-filter dokumen |
| 933 | 941 | `replaceFavoritesByArray:forFileURL:` | Mutable-copy + simpan array favorite dokumen |
| 950 | 953 | `removeFavoriteAtIndex:forFileURL:` | Hapus favorite di index untuk dokumen |
| 955 | 958 | `insertFavorite:atIndex:forFileURL:` | Sisip favorite di index untuk dokumen |
| 960 | 980 | `replaceHistoryByArray:forFileURL:` | Simpan history, notifikasi observer, bersihkan history global bila kosong |
| 982 | 987 | `addFavorite:forFileURL:` | Append favorite ke favorite dokumen |
| 989 | 1031 | `addHistory:forFileURL:` | Merge history ke store dokumen/global via SQLite atau pref |
| 1033 | 1040 | `favoritesForFileURL:` | Kembalikan favorite dokumen atau array kosong |
| 1042 | 1049 | `historyForFileURL:` | Kembalikan history dokumen atau array kosong |
| 1051 | 1071 | `historyMenuItemsForFileURL:` | Bangun item menu terpotong dari history dokumen |
| 1079 | 1089 | `numberOfHistoryItemsForFileURL:` | Jumlah history dokumen |
| 1098 | 1105 | `contentFilterForFileURL:` | Kembalikan content filter dokumen atau dict kosong |
| 1107 | 1132 | `queryFavoritesForFileURL:andTabTrigger:includeGlobals:` | Kembalikan favorite cocok tab-trigger, opsional global |

### Completion list controller (1135)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1140 | 1143 | `functionList` | Daftar function completion atau array kosong |
| 1148 | 1151 | `keywordList` | Daftar keyword completion atau array kosong |
| 1158 | 1161 | `argumentSnippetForFunction:` | Snippet argumen untuk nama function (uppercase) |

### Tail (1163)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1165 | 1174 | `dealloc` | Hapus observer KVO, tutup queue SQLite, destroy mutex |

## 4. Temuan menonjol / UNKNOWN

**Container history (backend ganda: prefs vs SQLite)**
- `historyContainer` = `NSMutableDictionary` ber-key string URL file; di-seed di `registerDocumentWithFileURL:` (817–841, 865–875). Registrasi fileURL nil membuat URL sintetik "Untitled N" (`untitledDocumentCounter` 807–809).
- Dua backend beralih pada `_SQLiteHistoryManager.migratedPrefsToDB`: jalur SQLite (818–830) baca `queryHist` (dict by row id), sort key menurun via `intSortDesc` (`SPFunctions.m:441`); jalur pref (831–840) baca `SPQueryHistory`.
- `addHistory:forFileURL:` (989–1031) tulis history per-dokumen (998–1008) **dan** untuk URL non-file (Untitled) history global (1012–1030). Dedup/limit didelegasikan ke Swift `SAQueryHistoryMerger` (1004–1006, 1021–1023). Komentar 1000–1002: mereplikasi dedup `NSPopUpButton` lama.
- `replaceHistoryByArray:forFileURL:` (960–980) membersihkan history global untuk URL non-file + array kosong, bercabang lagi pada `migratedPrefsToDB` (972–978).
- **UNKNOWN:** tipe elemen `historyContainer` campur — normalnya `NSMutableArray`, tapi `replaceHistoryByArray:` bisa menyimpan `historyArray` bridged Swift langsung (963), sedangkan `replaceFavoritesByArray:` eksplisit `mutableCopy` (936–939).
- **UNKNOWN:** `numberOfMaxAllowedHistory` (`.h:63`) di-assign `100` di `init` (109) tapi tak pernah dibaca di file ini — limit nyata dari pref `SPCustomQueryMaxHistoryItems` (991). Dead ivar.

**Container favorites**
- `favoritesContainer` (`.h:59`) dict ber-key URL file. `registerDocumentWithFileURL:` init array kosong untuk untitled (811–814) dan seed dari `contextInfo[SPQueryFavorites]` untuk file URL (853–863).
- `replaceFavoritesByArray:forFileURL:` (933–941) `mutableCopy` input — eksplisit untuk bertahan dari bridging Swift/NSArray karena API add/remove memutasi in-place (komentar 936–938).
- `queryFavoritesForFileURL:andTabTrigger:includeGlobals:` (1107–1132) filter favorite by key `@"tabtrigger"`, opsional scan pref global `SPQueryFavorites` (1120–1128), menambah **paling banyak satu** match global (`break` 1126).

**Threading/lock console**
- `pthread_mutex_t consoleLock` (`.h:72`), init 121, destroy 1173. `_addMessageToConsole:…` (588–626) lock 606, unlock 625; append ke `messagesFullSet` + kondisional `messagesFilteredSet` (608–616), lalu dispatch main-thread `updateEntries` (619–623).
- `copy:`/`infoStringForRowIndexes:` didokumentasikan **NOT THREAD-SAFE** (`.h:113`).

**Lain**
- `messagesVisibleSet` adalah alias bergaya weak yang menunjuk ke `messagesFullSet` (106, 463) atau `messagesFilteredSet` (491) — tak pernah dimiliki independen.
- `saveConsoleAs:` pakai `UTType typeWithFilenameExtension:SPFileExtensionSQL` (252).
- `SPLog` `"message is empty, returning"` (593) dan `"fileURLStr = %@"` (995).
- `dealloc` (1165–1174) tak lazim untuk window controller tapi menutup queue SQLite bersama + mutex.
