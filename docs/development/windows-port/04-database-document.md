# Database document (`SPDatabaseDocument`)

Kontrak source untuk *database document* per koneksi: pemilik koneksi, tables
list + satu subview per tab konten, toolbar, state `.spf`, dan task load
background. Sitasi `file:line` merujuk checkout `02c910932`.

File: `Source/Controllers/MainViewControllers/SPDatabaseDocument.m` (6436 baris),
`SPDatabaseDocument.h` (483 baris), `SPDatabaseDocument+ViewMode.swift` (181 baris).

Peran satu baris: **dokumen koneksi database** — memiliki koneksi, tables list +
satu subview controller per tab konten, toolbar, state sesi `.spf`, dan task
load tabel/tab di background.

## 1. Inisialisasi & window

- `+initialize` (`163-164`): stub kosong.
- `initWithWindowController:` (`166-248`): `instanceId` atomik (`170`), default/flag (`172-195`), image (`197-201`), `taskController` `SATaskController` + delegate (`228-229`), memuat nib **DBView** via `NSNib` ke top-level objects (`240-242`), membuat `databaseStructureRetrieval` (`244`).
- `awakeFromNib` (`250-326`): guard `_mainNibLoaded` (`252`); set toolbar (`258`); history interface (`261`); split-view min (`264-268`); membuat **`SPConnectionController`** (`271`) + delegate (`274`); observer preferensi (`277`); registrasi notifikasi (`281-296`); menu Database-Encoding (`299`); sembunyikan border tab (`302`); sembunyikan activity pane (`305`); `SPCharsetCollationHelper` ×2 (`310-311`); title/represented URL (`314-315`); `taskController centerInParentWindow` (`318`); `initComplete = YES` (`325`).
- `mainToolbar` (`330-338`): bangun `NSToolbar` id `"TableWindowToolbar"`, customizable/autosaving.
- `parentWindowControllerWindow` (`6060-6062`).

## 2. Toolbar

- `selectedToolbarItemIdentifier` (`3215-3218`).
- `toolbar:itemForItemIdentifier:` (`3223-3305`): item DB-selection (view = `chooseDatabaseButton`), history, show/clear console, record-view, **6 item view-mode via `SAViewModeHelper`** (`3276-3283`), user manager.
- `toolbarWillAddItem:`/`toolbarDidRemoveItem:` (`3307-3314`/`3316-3323`).
- `toolbarAllowedItemIdentifiers:` (`3328-3346`), `toolbarDefaultItemIdentifiers:` (`3351-3368`), `toolbarSelectableItemIdentifiers:` (`3373-3385`).
- `validateToolbarItem:` (`3390-3431`): disable semua bila `!_isConnected || _isWorkingLevel` (`3392`); logika console/clear-console; butuh tipe table/view (`3426-3428`).

## 3. Mode view & tab

- `switchToViewMode:` (`5440-5456`): jalur bersama; `SPMainQSync`; abort bila
  `couldCommitCurrentViewActions` gagal (`5444-5447`); pilih tab index + toolbar id,
  update history + pref `SPLastViewMode` (`5449-5452`).
- Wrapper: `viewStructure/viewContent/viewQuery/viewStatus/viewRelations/viewTriggers`
  (`5459-5494`); `viewQuery` fokus `customQueryTextView` (`5470-5473`); `viewStatus`
  load `extendedTableInfoInstance` + fokus create-syntax (`5479-5485`).
- `toggleRecordView:` (`5496-5505`).
- `setStructureRequiresReload:`/`setContentRequiresReload:`/`setStatusRequiresReload:`/`setRelationsRequiresReload:`
  (`5511-5573`): reload-sekarang bila terlihat, selain itu tandai `*Loaded = NO`.
- `tabView:didSelectTabViewItem:` (`5579-5596`): mulai task "Loading %@…", baca
  index di main thread, detach `_loadTabTask:`.
- `currentlySelectedView` (`2379-2414`): main-thread only; memetakan
  `tableTabView.selectedTabViewItem.identifier` → `SPTableViewType`.

**Dua enum mode view paralel (harus sinkron):**
- `SPViewMode` C (`SPConstants.h:40-47`): `SPStructureViewMode=1, SPContentViewMode=2,
  SPRelationsViewMode=3, SPTableInfoViewMode=4, SPQueryEditorViewMode=5, SPTriggersViewMode=6`.
- `SAViewMode` Swift (`+ViewMode.swift:15-21`): `structure=0, content=1, query=2,
  status=3, relations=4, triggers=5`, dengan `tabIndex == rawValue` (`24`),
  `toolbarIdentifier` (`35-44`), `preferencesValue` (`47-56`), `fromPreferences` (`59-69`),
  label/tooltip/gambar (`72-120`), `actionSelectorName` (`123-132`),
  `makeToolbarItem(target:)` (`135-144`), bridge ObjC `SAViewModeHelper` (`150-181`).

⚠️ **Diskrepansi**: `SPViewModeToMainToolbarMap` (`SPConstants.m:315-324`) memetakan
index 3→`SPMainToolbarCustomQuery` dan 5→`SPMainToolbarTableRelations`, **tertukar**
relatif `SPViewMode` (3=Relations, 5=Query) dan `SAViewMode.preferencesValue`. Hanya
dibaca di jalur abort `switchToViewMode:` (`5445`). Maksud **UNKNOWN** — kemungkinan bug laten.

## 4. Subview controller yang dimiliki

| Controller | Ivar (`.h`) | Peran |
| --- | --- | --- |
| `SPTablesList` | `tablesListInstance` 73 | tables list |
| `SPTableStructure` | `tableSourceInstance` 74 | tab Structure |
| `SPTableContent` | `tableContentInstance` 75 | tab Content |
| `SPTableRelations` | `tableRelationsInstance` 76 | tab Relations |
| `SPTableTriggers` | `tableTriggersInstance` 77 | tab Triggers |
| `SPCustomQuery` | `customQueryInstance` 78 | tab Query |
| `SPDataImport` | `tableDumpInstance` 79 | import |
| `SPTableData` | `tableDataInstance` 80 | cache status |
| `SPExtendedTableInfo` | `extendedTableInfoInstance` 81 | tab Table Info |
| `SPDatabaseData` | `databaseDataInstance` 82 | metadata server DB |
| history/export | `spHistoryControllerInstance` 83, `exportControllerInstance` 84 | IB outlets idle |
| `SPHelpViewerClient` | `helpViewerClientInstance` 85 | help |
| `SPConnectionController` | `connectionController` 157 | dibuat di `awakeFromNib` (271) |
| `SPProcessListController` | `processListController` 158 | lazy (913-921) |
| `SPServerVariablesController` | `serverVariablesController` 159 | lazy (900-908) |
| `SPUserManager` | `userManagerInstance` 91 | lazy (2065-2072) |
| `SATaskController` | `taskController` 188 (`.m:228`) | UI progress task |
| `SPDatabaseStructure` | `databaseStructureRetrieval` 220 | builder skema background (`.m:244`) |
| `SPGotoDatabaseController` | `gotoDatabaseController` 221 | lazy (1024-1026) |
| `SAHTMLPrintRenderer` | `printRenderer` 192 | lazy (6131) |

Semua subview tab **kecuali** connection controller, `taskController`,
`databaseStructureRetrieval`, `printRenderer` adalah IBOutlet dari nib **DBView**
(`NSNib initWithNibNamed:@"DBView"`, `.m:241`) dengan `self` sebagai owner.
Sisanya dibuat di kode.

## 5. Model task & threading

- **Counter working-level** `_isWorkingLevel` (`.h:180`) dengan hop main-thread di
  `startTaskWithDescription:` (`1130-1133`) dan `endTask` (`1201`). Setiap `start`
  menaikkan (`1146`); level pertama men-set `databaseListIsSelectable = NO`, post
  `SPDocumentTaskStartNotification`, validasi toolbar (`1155-1160`). `endTask`
  menurunkan, assert ≥0 (`1204-1205`), disable cancellation, di nol panggil
  `taskController endTaskDisplay`, re-enable selection, post `SPDocumentTaskEndNotification` (`1211-1223`).
- **UI progress di `SATaskController`** (`.m:228-229`); dokumen meneruskan
  `setTaskDescription:` (`1168`), `setTaskPercentage:` (`1178`),
  `setTaskProgressToIndeterminateAfterDelay:` (`1190`), `enableTaskCancellationWithTitle:…`
  (`1238`), `disableTaskCancellation` (`1252`), `setTaskIndicatorShouldAnimate:` (`1278`).
- **Cancellation**: `taskControllerDidRequestCancellation` (`1288-1298`) memilih kill
  query di thread structure-retrieval (`setLastQueryWasCancelled:YES` +
  `killQueryOnThreadID:`) selain itu `[mySQLConnection cancelCurrentQuery]`.
- **Background pakai raw `NSThread detachNewThreadWithName:SPCtxt(...)`** (bukan GCD):
  `_selectDatabaseAndItem:` (`670-673`), `_copyDatabaseWithDetails:` (`4944-4947`),
  `_loadTabTask:` (`5588-5591`), `_loadTableTask` (`5671-5674`), `restoreSession` (`543`),
  print `NSThread *printThread` (`6102-6103`). Semua punya cabang "sudah di background
  thread → jalankan inline" (`544, 676, 4950, 5594, 5677, 6110`).
- **Kembali ke main thread**: `SPMainQSync(...)` blok (`997, 4112, 5442, 5470, 5479,
  5838, 6013, 6161`); `[self onMainThread]` untuk fire-and-run (`488, 575, 1132, 1201,
  1233, 1247, 3173, 4969, 4972, 5520`); `postNotificationOnMainThreadWithName:`
  (`968, 982, 4975, 5073, 5144, 5645, 5790`); `performSelectorOnMainThread:` (`6001, 6282, 6319`).
- **Locking**: `@synchronized(self)` sekitar update history di `_loadTableTask`
  (`5847-5849`); `@autoreleasepool` membungkus tiap task panjang.
- **Perintah eksternal panjang**: `parentTabShouldClose` (`3468-3475`) kill PID
  aktivitas BASH jalan dengan `kill -9 -<pid>`.

## 6. Siklus hidup koneksi

**Connect**: `connect` (`2124-2130`) → `[connectionController initiateConnection:self]`;
guard `if (mySQLVersion) return;` (`2128`). Auto-connect dipicu dari `setState:fromFile:`
saat `auto_connect` (`3878-3880`) dan `setStateFromConnectionFile:` (`4060-4063 → 4066`).
`connectionControllerInitiatingConnection:` set judul window "Connecting…" (`4145`).

**Sukses — `setConnection:` (`409-578`), wajib main thread (`407`):**
- early-return bila `userTriggeredDisconnect` (`411-413`); `_isConnected = YES` (`415`); bangun `SPServerSupport` (`420-422`); registrasi dokumen ke `SPQueryController` (`425-426`).
- warning `skip-show-database` opsional (`448-463`); `setDatabases` (`466`).
- sebarkan koneksi ke `databaseStructureRetrieval` (`471`), `databaseDataInstance` (`473`), `tablesListInstance` (`480`), dan tiap subview tabel (`493-507`).
- encoding: terapkan pref `SPDefaultEncoding` atau autodetect (`483-489`).
- post notifikasi `SANotificationCenter` "Connected" (`511-512`).
- restore konten query editor dari `spfSession["queries"]` (`515-526`); `queryEditorInitString` opsional (`529-533`).
- bila `spfSession != nil`: restore divider (`538`) + `restoreSession` di background (`541-544`); selain itu ambil view default/terakhir dari pref (`546-568`).
- `detectDatabaseEncoding` (`570`); fokus ke filter tables-list kecuali tab Query (`574-576`).

**Lost connection** `connectionLost:` (`5970-6006`): kembalikan
`SPMySQLConnectionLostDecision`. Hanya bila window ada, terlihat, dan
`appIsTerminating == NO` (`5979`): deminiaturize (`5983-5985`), raise (`5986`),
bangun `SAConnectionLostSheetCopy` (AWS-aware, `5989-5991`), `SAConnectionLostAlert
runModalForWindow:` (`5993`) — **Reconnect** → `SPMySQLConnectionLostReconnect`,
selain itu **Disconnect** → `SPMySQLConnectionLostDisconnect` + `closeAndDisconnect`.

**Disconnect/close** `closeConnection` (`2132-2146`): bersihkan delegate, disconnect
`mySQLConnection` + structure connection, `_isConnected = NO`, post "Disconnected".
`closeAndDisconnect` (`6023-6058`): sembunyikan window + close setelah 1 s, simpan
teks query ke history (`6032-6036`), cancel autocompletion/auto-help (`6039-6048`),
`closeConnection` atau `[connectionController cancelConnection:]` (`6050-6054`).
`documentWillClose:` (`6396-6421`): assert main thread, `closeConnection`, lepas
observer + notifikasi, `[taskController shutDown]`, tutup process list, bersihkan
delegate connection-controller (#2924), `[printRenderer invalidate]`.

**Keychain**: `keychainPasswordForConnection:` (`5929`),
`credentialErrorMessageForConnection:` (`5937`, AWS-IAM saja),
`keychainPasswordForSSHConnection:` (`5949`).

## 7. Persistensi sesi `.spf`/`.spfs`

Ekstensi: `SPFileExtensionDefault = "spf"` (`SPConstants.m:75`),
`SPBundleFileExtension = "spfs"` (76), `SPFileExtensionSQL = "sql"` (77). Format tag:
`SPFFormatKey = "format"` → `SPFConnectionContentType = "connection"`;
`SPFVersionKey = "version"` (`SPConstants.m:394-395, 86`).

**Key top-level `.spf`** (`saveDocumentWithFilePath:…` `2752-2815`): `version` (2756),
`format` (2757), `rdbms_type` (2758), `rdbms_version` (2759), `auto_connect` (2762),
`queryFavorites`/`ContentFilters` (2784, 2787), `encrypted` (2792), `data` (2808 atau
terenkripsi 2814). **Query history sengaja TIDAK disimpan** ("DON'T SAVE QUERY HISTORY
IN EXPORTS FOR SECURITY", `2785-2786, 2722-2723`).

**Dict `data`**: `stateDetails` dari `stateIncludingDetails:` minus yang multi-keyed
(`2788-2789`), plus `connection` (`3583-3655`) dan `session` (`3666-3727`).

**Sub-key `connection`** (`3584-3655`): `rdbms_type`, `type` (nama enum), `aws_profile`,
`aws_region`, `socket`, `ssh_host`, `ssh_user`, `ssh_keyLocationEnabled`,
`ssh_keyLocation`, `ssh_port`, `sshRemoteSocketPath`, `vault_host`, `vault_port`,
`vault_oidc_mount`, `vault_credentials_path`, `kcid`, `name`, `host`, `user`, `port`,
`database`, `password` (hanya dengan `includePasswords` dan non-Vault), `ssh_password`,
`useSSL`, `allowDataLocalInfile`, `enableClearTextPlugin`, `requestServerPublicKey`,
`sslKeyFileLocationEnabled`/`sslKeyFileLocation`,
`sslCertificateFileLocationEnabled`/`sslCertificateFileLocation`,
`sslCACertFileLocationEnabled`/`sslCACertFileLocation`, `SPFavoriteColorIndexKey`.

**Sub-key `session`** (`3666-3727`): `table` (3671), `view` (salah satu `SP_VIEW_*`,
3673-3696), `connectionEncoding` (3698), `isToolbarVisible` (3700),
`windowVerticalDividerPosition` (3701), `contentSortCol` (3703),
`contentSortColIsAsc` (3704), `contentPageNumber` (3705), `contentViewport` (3706),
`contentFilterV2` (3708), `contentSelection` (archived, 3710-3713 + 2799-2805),
`queries` (3718-3723; dikompres via `compress` bila >50 000 char, key `enablecompression`).

**State save-panel in-memory** (`spfDocData`, `219-221`): `encrypted` (2657),
`e_string` (2659), `auto_connect` (2661), `save_password` (2662), `include_session`
(2663), `save_editor_content` (2664-2667).

**Jalur buka — `setStateFromConnectionFile:` (`3889-4069`)**: baca plist, validasi
`format == "connection"` (3913), butuh `data` (3921), tangani `encrypted` → sheet
password (`inputTextWindow`, modal 3937-3954), decode dict polos (3971) atau keyed-archived
(3975-4010), butuh `connection` (4013), angkat `queryFavorites`/`queryHistory`/`ContentFilters`
(4028-4036), seed `spfDocData` (4039-4063), set file URL, lalu `setState:` (4066) yang
auto-connect. `setState:fromFile:` (`3742-3883`) menerapkan tiap key koneksi.
`restoreSession` (`4074-4135`) me-restore table/view/sort/page/viewport/filter/selection/encoding/toolbar.

**Simpan preferensi saja** (`onlyPreferences = YES`, `2675-2749`): baca ulang file,
butuh `format == "connection"` (2715), tulis ulang hanya `queryFavorites` + `ContentFilters`
(2721-2724).

## 8. DDL database & seleksi

- `setDatabases:` (`598-621`): panggil `SADatabaseListManager configurePopup:` (609-613),
  persist `allSystemDatabases`/`allDatabases` (619-620).
- `chooseDatabase:` (`627-647`), `selectDatabase:item:` (`652-678`),
  `_selectDatabaseAndItem:` (`5157-5164`, trampoline ke `SADatabaseListManager
  performSelectionWithDatabase:item:delegate:`).
- `addDatabase:` (`683-721`), `alterDatabase` (`726-750`),
  `copyDatabase` (`820-848`, alert "Only Partially Supported" bila objek non-tabel),
  `renameDatabase` (`853-868`), `removeDatabase:` (`873-888`), `refreshTables` (`893-895`).
- `_addDatabase` (`5043-5077`): paksa utf8mb4 (`5053-5055`), `SPDatabaseAction
  createDatabase:withEncoding:collation:`.
- `_alterDatabase` (`5082-5107`): bangun `ALTER DATABASE … DEFAULT CHARACTER SET/COLLATE`.
- `_removeDatabase` (`5114-5147`): `DROP DATABASE`, error sheet setelah 0.3 s.
- `compareDatabase:` (`752-815`): **dead code** eksperimental.
- `showGotoDatabase` (`1023-1042`): lazy `SPGotoDatabaseController`.

## 9. Operasi tabel

- `showCreateTableSyntax:` (`1487-1592`): bangun `SHOW CREATE TABLE/VIEW/PROCEDURE/FUNCTION`
  untuk semua baris terpilih, tampilkan sheet.
- `checkTable` (`1606-1659`) `CHECK TABLE`; `analyzeTable` (`1664-1717`) `ANALYZE TABLE`;
  `optimizeTable` (`1722-1776`) `OPTIMIZE TABLE`; `repairTable` (`1781-1834`) `REPAIR TABLE`;
  `flushTable` (`1839-1893`) `FLUSH TABLE`; `checksumTable` (`1898-1934`) `CHECKSUM TABLE`.
- `saveCreateSyntax:` (`1939-1961`) menulis `-- Create syntax for 'table'`.
- `exportSelectedTablesAs:`/`exportData` (`2012-2024`) proxy `SPExportController`.
- `doPerformQueryService:`/`doPerformQueryLoadQueryService:` (`2192-2103`).

## 10. Entry SQL eksternal (URL scheme)

`handleSchemeCommand:` (`4176-4662`): tunggu sampai 5 s sementara working/disconnected,
lalu dispatch: `SelectDocumentView` (4206), `SelectTable` (4230), `SelectTables` (4240),
`SelectDatabase` (4247), `SetSelectedTextRange` (4269), `InsertText` (4284), `SetText`
(4296), `SelectTableRows` (4309), `ReloadContentTable` (4317), `ReloadTablesList` (4322),
`ReloadContentTableWithWHEREClause` (4327), `RunQueryInQueryEditor` (4342),
`CreateSyntaxForTables` (4357), `ExecuteQuery` (4491 — `streamingQueryString:assertingDatabase:` 4524).
Tak didukung → alert "Remote Error" (4661).

## 11. Printing

`printDocument` (`6070-6090`, warning limit baris), `startPrintDocumentOperation`
(`6095-6112`, `printThread` detached), `loadPrintWebViewWithHTMLString:` (`6129-6150`,
`SAHTMLPrintRenderer`), `generateHTMLForPrinting` (`6155-6284`),
`generateTableInfoHTMLForPrinting` (`6289-6321`), `columnNames` (`6328-6361`),
`connectionInformation` (`6366-6394`).

## 12. String yang terlihat pengguna (terpilih)

**Koneksi/server**: `"The skip-show-database variable of the database server is set to ON. …"` (`454-460`); `"Do you really want to shutdown the server?"` (`924`); `"Shutdown failed!"` / `"MySQL said:\n%@"` (`927`); `"No connection available"` (`5964`); `"Remote Error"` / `"URL scheme command “%@” unsupported"` (`4661`).

**DDL database**: `"Delete database '%@'?"` / `"Are you sure you want to delete the database '%@'? This operation cannot be undone."` (`883-885`); `"Only Partially Supported"` / `"Duplicating the database '%@' is only partially supported …"` (`828-833`); `"Database must have a name."` (`4931, 4995, 5048`); `"Unable to copy database"` (`4981`); `"Unable to rename database"` (`5020`); `"Couldn't create database.\nMySQL said: %@"` (`5065`); `"Couldn't alter database.\nMySQL said: %@"` (`5101`); `"Couldn't delete the database.\nMySQL said: %@"` (`5123`); `"Unable to select database %@.\nPlease check you have the necessary privileges to view the database, and that the database still exists."` (`5254-5255`).

**Maintenance tabel**: judul/judul pesan pola `"Unable to <check|analyze|optimize|repair|flush> table"` + `"An error occurred while trying to <verb> the %@.\n\nMySQL said:%@"` (`1619, 1677, 1736, 1794, 1852`) — termasuk **salah tulis `"optimze"/"optimzing"`** di `1736-1738`; sukses `"…successfully <verb>ed…"`; checksum `"Unable to perform the checksum"` (`1911-1912`).

**File/sesi**: `"Error while reading connection data file"` (`2703-2708`); `"Unknown file format"` / `"The chosen file “%@” contains ‘%@’ data."` (`3914-3915`); `"Connection file is encrypted"` / `"Please enter the password for ‘%@’:"` (`3932-3933`); `"Wrong data format or password."` (`4006`).

**Printing**: `"Continue to print?"` / `"Are you sure you want to print the current content view of the table '%@'?\n\nIt currently contains %@ rows, which may take a significant amount of time to print."` (`6081-6082`).

## 13. Pengamatan/UNKNOWN

- `SPDatabaseDocument.swift` juga ada di direktori sama — di luar scope; **UNKNOWN** apakah stub atau port penuh.
- `compareDatabase:` (`752-815`) dead code.
- `generateHTMLForPrinting` membaca `spfSession["pageNumber"]` (`4095`) padahal key tersimpan
  `contentPageNumber` (`3705`) — mismatch key, kemungkinan bug restore.
- Banyak ivar bukan property (mis. `processListController`, `serverVariablesController`,
  `userManagerInstance`, `databaseStructureRetrieval`) — retensi bergantung nib/graph.
- `SPViewModeToMainToolbarMap` menukar query/relations — kemungkinan bug laten.
