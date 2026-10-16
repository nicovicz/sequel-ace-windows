# Audit: Main-controller Swift cluster

Audit method-by-method klaster main-controller Swift. `Start`/`End` = nomor baris di
file masing-masing. Baseline commit `02c910932`.

File: `MainViewControllers/SADatabaseListManager.swift` (490),
`MainViewControllers/SAQueryFavoriteSaveWindowController.swift` (254),
`MainViewControllers/SATaskController.swift` (395), `SPAppController.swift` (289).

---

## 1. `SADatabaseListManager.swift` (490)

Memiliki konfigurasi popup "choose database" + orkestrasi seleksi database background-thread, di belakang protokol `SADatabaseSelectionDelegate`.

### Tipe & konstanta
`@objc protocol SADatabaseSelectionDelegate: AnyObject` (31–111); `@objc final class SADatabasePartition: NSObject` (114–123); `SATableListEntry` (132–143); `SATableListResultParser` (145–203); `SADatabaseListManager` (205–490).
`schemaPathDelimiter = "\u{FFF8}"` (213, cocok `SPUniqueSchemaDelimiter`); `systemDatabaseNames: Set = {mysql, information_schema, performance_schema, sys}` (244–249).

### `// MARK:`
`33 (State the manager reads and writes), 44 (Predicates), 52 (Bridges to project ObjC objects), 92 (Tables-list UI (main-thread)), 107 (Post-completion)` — semua di dalam body protokol.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 36–110 | | ~19 requirement protokol (`currentSelectedDatabase`, `chooseDatabaseButton`, `selectMySQLDatabase(_:)`, `rebuildDatabasesPopup()`, `selectTablesListItem(named:)`, `processDatabaseChangedBundleTriggers()`, dst) | Mirror state + aksi ke dokumen ObjC |
| 115–122 | | `systemDatabases` / `userDatabases` / `init(systemDatabases:userDatabases:)` | Partition |
| 133–142 | | `name` / `comment` / `isView` / init | Entry baris tabel |
| 150–202 | | `parse(rows:fieldNames:displayTableComments:)` / `index(of:in:)` / `isShowTablesName(_:)` / `string(in:at:)` | Normalisasi baris |
| 213 | 213 | `schemaPathDelimiter` | Separator path skema navigator |
| 224–230 | | `navigatorSchemaPath(connectionID:selectedDatabaseTitle:)` | Bangun path skema navigator |
| 244–264 | | `systemDatabaseNames` / `partition(databases:)` | Split sistem/user jaga urutan |
| 290 | 334 | `configurePopup(_:databases:currentDatabase:addDatabaseSelector:refreshDatabasesSelector:)` | Bangun ulang popup + pilih current |
| 355 | 479 | `performSelection(database:item:delegate:)` | Orkestrasi seleksi database+tabel background |
| 484–489 | | `mainSync(_:)` (privat) | Main-thread run inline atau `dispatch_sync` |

### Temuan
- **Algoritma popup** (`configurePopup`, 290–334): header tetap = "Choose Database..." → separator → "Add Database..." → "Refresh Databases" → separator (297–310); header action `target == nil` agar lewat responder chain (302–308); lalu DB sistem, separator hanya bila ada (317–322), lalu DB user (323–325); akhirnya pilih `currentDatabase` by title bila non-kosong else item 0 (327–331). Nama kosong di-skip (317, 323). Return partition untuk di-cache caller (333).
- **Kontrak retry seleksi** (`performSelection`, 355–479): skip kerja koneksi bila `database == currentSelectedDatabase` (364); stale index/select gagal → rebuild popup + retry sekali (370–382); gagal → end task, alert bila connected, snap popup balik ke DB live (384–412); sukses → update title/index popup (414–433), bersihkan current table, reset data, deteksi encoding, reattach tables list, refresh title (435–449); akhir restore fokus tabel/filter (452–470) + history state (472–475).
- `mainSync` (484–489) reimplementasi eksplisit `SPMainQSync` — **UNKNOWN**: disebut `SPMainQSync` di komentar tapi diimplementasi di sini sebagai `Thread.isMainThread` check + `DispatchQueue.main.sync`.
- Delimiter `"\u{FFF8}"` **di-inline**, bukan di-import (207–213) — sinkronisasi lintas file manual.

---

## 2. `SAQueryFavoriteSaveWindowController.swift` (254)

Pengganti SwiftUI untuk sheet save/replace Query Favorite lama, dibungkus `NSWindowController` yang host panel.

### Tipe & konstanta
`@MainActor private final class SAQueryFavoriteSaveViewModel: ObservableObject` (12–48); `private struct SAQueryFavoriteSaveView: View` (50–141); `@MainActor @objc final class SAQueryFavoriteSaveWindowController: NSWindowController` (144–254).
`private let preferences = UserDefaults.standard` (148).

### `// MARK:`
**Tidak ada.**

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 13–29 | | `name` / `saveGlobally` / `selectedFavoriteID` / `selections` / `errorMessage` / `isShowingError` (@Published) + init | View model |
| 31–47 | | `selectedFavorite` / `canSave` / `refreshSelections(_:)` / `showError(_:)` | Turunan |
| 51–129 | | `model` / `onSave` / `onCancel` / `isNameFocused` / `isReplacing` / `body` | View |
| 131–140 | | `selectionTitle(_:)` | String "[Scope] Name" untuk picker |
| 145–148 | | `query` / `fileURL` / `model` / `preferences` (private let) | Field |
| 151–187 | | `init(query:fileURL:defaultSaveGlobally:)` / `init?(coder:)` | Build model, NSPanel, hosting view |
| 189 | 233 | `save()` | Persist mutasi, post notifikasi, finish/error |
| 235–242 | | `finish(returnCode:)` | Akhiri sheet atau tutup window |
| 244–253 | | `documentFavorites(for:)` / `globalFavorites()` (static) | Load favorites |

### Temuan
- **Sheet save** (`save`, 189–233): baca favorites document/global fresh (190–191), panggil `SAQueryFavoriteStore.save(...)` dengan name/scope/replacement (194–201); `.document` → `SPQueryController.shared().replaceFavorites(by:forFileURL:)` (204–208), `.global` → `preferences.set(..., forKey: SPQueryFavorites)` (209–211); post `.SPQueryFavoritesHaveBeenUpdated` (213–216) lalu finish `.OK` (217). `SPQueryFavorites` = `"queryFavorites"` (`SPConstants.m:269`).
- **Penanganan stale-replacement** (218–232): saat `SAQueryFavoriteSaveError.selectedFavoriteChanged` → refresh selections + pesan "changed or removed in another window"; error lain → pesan missing-name.
- Panel fixed-size `NSPanel` 360×280, `.titled`, `isReleasedWhenClosed = false` (164–171); konten via `NSHostingView` (175–181).
- **UNKNOWN**: bentuk/encoding array favorites — API mengetikkannya `[Any]` (244–249, 251–253), jadi graf persist opaque.

---

## 3. `SATaskController.swift` (395)

Memiliki UI progress task dokumen — window progress borderless, spinner/determinate indicator, label deskripsi/durasi, tombol cancel, fade-in + timer query-time.

### Tipe & konstanta
`@objc protocol SATaskControllerDelegate: AnyObject` (29–41); `@objc final class SATaskController: NSObject` (43–395). **Tak ada konstanta** (hanya lokal: `progressIndicatorShadow` 106, dict atribut 389–392).

### `// MARK:`
`47 (Outlets (ProgressIndicatorLayer.xib, File's Owner = self)), 55 (Progress UI state), 83 (Cancellation state), 89 (Setup), 113 (Task lifecycle (driven by SPDatabaseDocument)), 171 (Description & progress), 253 (Cancellation), 313 (Query-execution timer), 329 (Window placement & teardown), 379 (Helpers)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 32–40 | | `taskParentWindow()` / `taskControllerDidRequestCancellation()` | Requirement protokol |
| 45–87 | | outlet + state (`taskProgressLayer`, `taskProgressIndicator`, `taskDescriptionText`, `taskDurationTime`, `taskCancelButton`, `taskProgressWindow` lazy, flag, timer) | Outlet + state |
| 91–111 | | `init()` (@objc override) | Load nib, konfigurasi warna/shadow indicator |
| 118–146 | | `beginTask(isFirstLevel:)` | Reset indicator; first level start timer |
| 150–169 | | `endTaskDisplay()` | Stop timer, hide window, reset indeterminate |
| 175–241 | | `setTaskDescription(_:)` / `setTaskPercentage(_:)` / `setTaskProgressToIndeterminate(afterDelay:)` / `makeProgressIndeterminate()` | Deskripsi + progress |
| 246–251 | | `setTaskIndicatorShouldAnimate(_:)` | Start/stop spinner di main |
| 264–311 | | `enableTaskCancellation(...)` / `disableTaskCancellation()` / `cancelTask(_:)` | Cancel |
| 317–327 | | `resetQueryTimer()` / `showQueryExecutionTime()` | Timer query |
| 332–377 | | `centerInParentWindow()` / `shutDown()` / `fadeInTaskProgressWindow(_:)` | Window |
| 383–394 | | `attributedString(_:)` | Builder attributed string bold+shadow |

### Temuan
- **Progress UI** (`beginTask`, 118–146): saat `isFirstLevel` hide tombol cancel (130) + jadwalkan timer fade 30 fps + timer query 1 s (135–144). `endTaskDisplay` (150–169) invalidate kedua timer, render waktu final, reset ke indeterminate.
- **Switch determinate** (`setTaskPercentage`, 183–212): panggilan persentase pertama lompat indicator ke `0.5` + stop animasi (192–194); off-main hop ke main (187–189, 203–209); update hanya saat silang ±`taskProgressValueDisplayInterval` (200–211).
- **Catatan bug-fix indeterminate** (`makeProgressIndeterminate`, 228–241): komentar 230–233 jelaskan aslinya cancel terhadap indicator bukan `self`, jadi tak pernah cancel; fix panggil `NSObject.cancelPreviousPerformRequests(withTarget: self, selector:, object:)` (234–236).
- **Cancellation** (`cancelTask`, 288–311): guard `taskCanBeCancelled` (289), disable tombol (291), perform callback bila respond (298–302), special-case `SAFieldRemovalTask` lewat `requestQueryCancellation` sebelum notifikasi delegate (304–310).
- **Penempatan window** (`fadeInTaskProgressWindow`, 352–377): tahan window hidden <0.5 s (357), tambah sebagai child window sekali (359–361), center sebelum reveal pertama (366), clamp alpha 1.0 + invalidate timer saat fade penuh (370–376).
- **UNKNOWN**: perilaku `YRKSpinningProgressIndicator` + `DateComponentsFormatter.hourMinSecFormatter` (325) di luar file; wiring `@IBOutlet`↔xib tak dapat diverifikasi dari source.

---

## 4. `SPAppController.swift` (289)

Trampoline menu-action yang forward ke `tabManager.activeWindowController?.databaseDocument`, + action window koneksi standalone, installer item menu File dinamis, dan helper modal OK/Cancel. Body kelas `SPAppController` sendiri **tidak** di file ini — hanya extension.

### Tipe & konstanta
`extension SPAppController` menu action (13–246); `private struct StandaloneConnectionWindow` (25–32); extension installer item menu (250–277); extension dialog helper (279–289).
`private static var standaloneConnectionWindows: [StandaloneConnectionWindow] = []` (35).

### `// MARK:`
`11 (Menu actions), 15 (File menu actions), 86 (Edit menu actions), 95 (View menu actions), 133 (Database menu actions), 195 (Table menu actions), 241 (Help menu actions), 248 (Standalone Connection Window Menu Item)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 17–23 | | `newWindow(_:)` / `newTab(_:)` | Tab manager |
| 28–35 | | `controller` / `closingSubscription` / `standaloneConnectionWindows` | Retain window standalone |
| 40 | 60 | `openStandaloneConnectionWindow(_:)` | Buka window koneksi decoupled; retain hingga close |
| 62–84 | | `export` / `addConnectionToFavorites` / `saveConnectionSheet` / `import` / `importFromClipboard` / `printDocument` | File menu action |
| 89–131 | | `performFindPanelAction` / `viewStructure` / `viewContent` / `viewQuery` / `viewStatus` / `viewRelations` / `viewTriggers` / `backForwardInHistory` / `toggleConsole` / `toggleNavigator` | Edit/View |
| 135–193 | | `showGotoDatabase` / `addDatabase` / `removeDatabase` / `copyDatabase` / `renameDatabase` / `alterDatabase` / `refreshTables` / `flushPrivileges` / `setDatabases` / `showUserManager` / `chooseEncoding` / `openDatabaseInNewTab` / `showServerVariables` / `showServerProcesses` / `shutdownServer` | Database menu |
| 197–239 | | `focusOnTableContentFilter` / `showFilterTable` / `makeTableListFilterHaveFocus` / `copyCreateTableSyntax` / `showCreateTableSyntax` / `checkTable` / `repairTable` / `analyzeTable` / `optimizeTable` / `flushTable` / `checksumTable` | Table menu |
| 243–245 | | `showMySQLHelp(_:)` | Help menu |
| 254 | 276 | `installStandaloneConnectionMenuItem()` | Sisip item menu File "New Connection Window" |
| 280 | 288 | `dialogOKCancel(question:text:)` | Alert warning modal return OK/Cancel |

### Temuan
- **Menu action** (13–246): tiap `@IBAction` one-line forward ke `tabManager.activeWindowController?.databaseDocument.<method>`; optional chain no-op senyap saat tak ada dokumen aktif. `performFindPanelAction` (89–93) satu-satunya yang punya logika — rutekan Cmd+F ke content filter hanya saat `currentlySelectedView() == .content`.
- **Window koneksi standalone** (`openStandaloneConnectionWindow`, 40–60): buat `SAConnectionWindowController`, show, retain di `standaloneConnectionWindows` + observer `NSWindow.willCloseNotification` (49–55) yang hapus entry saat close; komentar 44–48 catat ini melepas token agar deinit unregister + hindari isu concurrency Swift 6.
- **Installer item menu** (`installStandaloneConnectionMenuItem`, 254–276): guard idempoten pada selector existing (260–262), sisip di `min(2, count)` setelah "New Tab" (265), set `Cmd+Shift+N` dengan `target = nil` (270–273).
- **UNKNOWN**: deklarasi kelas utama `SPAppController`, `tabManager`, `SAConnectionWindowController`, `NotificationToken` di luar file. `dialogOKCancel` (280–288) tak ada caller in-file.

## Ringkasan
- Tiga file `MainViewControllers` sengaja menghindari dependensi bridging-header ObjC (nama DB sistem, delimiter skema, result parser di-inline) agar bisa dikompilasi ke target Unit Tests (`SADatabaseListManager.swift:207–243`).
- `SPAppController.swift` murni layer aksi/dispatch; kerja nyata didelegasikan ke `databaseDocument`.
- Coupling lintas-file: `configurePopup` mengandalkan responder chain mencapai `SPDatabaseDocument.addDatabase:`/`setDatabases:` (277–282); `SAQueryFavoriteSaveWindowController` mengandalkan `SAQueryFavoriteStore` + key `SPQueryFavorites` tetap sinkron (194–216).
