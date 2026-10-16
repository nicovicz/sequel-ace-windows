# Arsitektur runtime: startup, window/tab, menu, preferensi

Kontrak source untuk kerangka aplikasi macOS: urutan startup, controller
aplikasi, model window/tab, struktur menu, judul/tint window, jalur buka-simpan
dokumen, dan preferensi. Sitasi `file:line` merujuk checkout `02c910932`.

## 1. Urutan startup

| Langkah | Lokasi |
| --- | --- |
| `main()` → `NSApplicationMain(argc, argv)` | `Source/main.m:30-33` |
| Principal class `ReportExceptionApplication`, nib utama `MainMenu` | `Resources/Plists/Info.plist:304-307` |
| `ReportExceptionApplication` subclass `NSApplication`; override `reportException:` (kill pid SSH, forward ke Firebase Crashlytics bila analytics on) dan `sendEvent:` (try/catch) | `Source/Controllers/Other/ReportExceptionApplication.m:15,20-57` |
| `NSApplicationMain` memuat `MainMenu.xib` yang menginstansiasi `SPAppController` (`customClass="SPAppController"`, id 213) | `Source/Interfaces/MainMenu.xib:975-982` |
| `+[SPAppController initialize]` sebelum `init`: muat plist `SPPreferenceDefaults` via `registerDefaults:`, lalu `SPApplyRevisionChanges()` | `Source/Controllers/SPAppController.m:124-134` |
| `-[SPAppController init]`: set ivar, buat `TabManager`, buat `~/tmp` dan `~/.keys`, `switchAppearance`, observe `NSUserDefaultsDidChangeNotification`, `[NSApp setDelegate:self]` | `Source/Controllers/SPAppController.m:94-119` |
| `awakeFromNib`: registrasi handler URL `sequelace://`/`mysql://` (`kAEGetURL`), buat `SPPreferenceController`, `[NSApp setServicesProvider:self]`, registrasi objek AppleScript | `Source/Controllers/SPAppController.m:172-190` |

`applicationDidFinishLaunching:` (`SPAppController.m:195-300`) berurutan:
1. Analytics: `[SAAnalyticsConsentPolicy requestConsentIfNeeded]` + `applyAnalyticsConsent` (`:197-198`; policy di `SAAnalyticsConsentPolicy.swift:27-39,74-117`; gate `shouldConfigureFirebase` `:20-23`).
2. Secure bookmarks `SecureBookmarkManager.sharedInstance`; bila stale → alert "App Sandbox Issue" (Open Files Preferences/Continue) (`:204-231`).
3. Init SQLite: `SQLiteHistoryManager`, `SQLitePinnedTableManager` (`:234-235`).
4. Parse argv `--spfData`/`--dataVersion` → `spfDict` (`:237-248`), fake frame split-view di background (`:249-255`).
5. `checkForNewVersionWithDelay:` dengan `SPDelayBeforeCheckingForNewReleases = 10` (`:59,257`).
6. `addCheckForUpdatesMenuItem` (`:260,306-312`; hanya bila `!isMASVersion`).
7. Registrasi notifikasi distributed `ExternalApplicationWantsToOpenADatabaseConnection` + notifikasi lokal duplicate/prev/next tab/save-to-SPF (`:262-267`).
8. `[SPBundleManager.shared reloadBundles:self]` + `_copyDefaultThemes` (`:269-270`).
9. `[self setupMCPServer]` (`:273`; `Source/Controllers/MCP/SPAppController+MCP.swift:143`).
10. Bila tak ada front document: `tabManager replaceTabServiceWithInitialWindow`, `setState:` dengan `spfDict`, auto-connect opsional (`:276-288`).
11. `installStandaloneConnectionMenuItem` (`:299`; Swift `SPAppController.swift:254`).

**Update**: tanpa Sparkle. Cek GitHub release: `NSBundle.checkForNewVersion(isFromMenuCheck:)`
(`BundleExtension.swift:76`), digerbangi `SAGitHubReleaseCheckPolicy.shouldCheck`
(`GitHub.swift:26`, dipakai `SPAppController.m:322-331`), manager `GitHubReleaseManager.swift`.
**UNKNOWN**: pemanggilan configure Firebase ada di `SAAnalyticsConsentPolicy+Firebase.swift`.

## 2. SPAppController

Empat file: `SPAppController.h` (interface/protokol), `.m` (inti ObjC), `.swift`
(extension aksi menu), `+MCP.swift` (server MCP). `.h` mendeklarasikan
`<NSApplicationDelegate, NSOpenSavePanelDelegate, NSFileManagerDelegate, NSWindowDelegate>`
(`SPAppController.h:43`).

**Method ObjC non-menu**: `init` `:94`, `initialize` `:124`, `defaultsChanged:` `:139`,
`switchAppearance` `:153`, `awakeFromNib` `:172`, `applicationDidFinishLaunching:` `:195`,
`applicationShouldTerminateAfterLastWindowClosed:` `:302`, `addCheckForUpdatesMenuItem` `:306`,
`checkForNewVersionFromMenu` `:314`, `checkForNewVersionWithDelay:andIsFromMenuCheck:` `:318`,
`externalApplicationWantsToOpenADatabaseConnection:` `:334`, `switchToPreviousTab:` `:346`,
`switchToNextTab:` `:350`, `duplicateConnectionToTab:` `:354`, `saveConnectionsToSPF:` `:363`,
`validateMenuItem:` `:507`, `openConnectionSheet:` `:552`, `application:openFiles:` `:623`,
`openConnectionFileAtPath:` `:652`, `openSQLFileAtPath:` `:658`, `openSessionBundleAtPath:` `:752`,
`openColorThemeFileAtPath:` `:834`, `handleEvent:withReplyEvent:` `:871`,
`handleMySQLConnectWithURL:` `:887`, `handleEventWithURL:` `:917`,
`doSQLSyntaxHighlightForString:cssLike:` `:1142`, `shellEnvironmentForDocument:` `:1219`,
`registerActivity:` `:1282`, `runningActivities` `:1325`, `doPerformQueryService:userData:error:` `:1390`,
`applicationShouldHandleReopen:` `:1457`, `applicationShouldTerminate:` `:1473`,
`orderedDocuments/Windows` `:1590,1605`, `handleQuit/Open/PrintScriptCommand:` `:1615,1627,1637`,
`newWindowForTab:` `:1650`, `duplicateTab:` `:1657`, `dealloc` `:1678`, `reloadBundles:` `:1687`,
`openBundleEditor:` `:1691`, `bundleCommandDispatcher:` `:1695`, `rebuildMenus` `:1699`,
`_copyDefaultThemes` `:1547`.

**Aksi menu ObjC** (`.m`): Sequel Ace `openAboutPanel:` `:1336`, `openPreferences:` `:1348`,
`visitWebsite:` `:1421`; File `openConnectionSheet:` `:552`, `duplicateTab:` `:1657`;
Bundles `reloadBundles:` `:1687`, `openBundleEditor:` `:1691`, `bundleCommandDispatcher:` `:1695`;
Help `visitHelpWebsite:` `:1429`, `visitFAQWebsite:` `:1437`, `viewKeyboardShortcuts:` `:1445`.

**Aksi menu Swift** (`SPAppController.swift`) — hampir semuanya meneruskan ke
`tabManager.activeWindowController?.databaseDocument…`:
- **File**: `newWindow` `:17`, `newTab` `:21`, `openStandaloneConnectionWindow` `:40`, `export` `:62`,
  `addConnectionToFavorites` `:66`, `saveConnectionSheet` `:70`, `import` `:74`, `importFromClipboard` `:78`, `printDocument` `:82`.
- **Edit**: `performFindPanelAction` `:89`.
- **View**: `viewStructure` `:97`, `viewContent` `:101`, `viewQuery` `:105`, `viewStatus` `:109`,
  `viewRelations` `:113`, `viewTriggers` `:117`, `backForwardInHistory` `:121`, `toggleConsole` `:125`, `toggleNavigator` `:129`.
- **Database**: `showGotoDatabase` `:135`, `addDatabase` `:139`, `removeDatabase` `:143`,
  `copyDatabase` `:147`, `renameDatabase` `:151`, `alterDatabase` `:155`, `refreshTables` `:159`,
  `flushPrivileges` `:163`, `setDatabases` `:167`, `showUserManager` `:171`, `chooseEncoding` `:175`,
  `openDatabaseInNewTab` `:179`, `showServerVariables` `:183`, `showServerProcesses` `:187`, `shutdownServer` `:191`.
- **Table**: `focusOnTableContentFilter` `:197`, `showFilterTable` `:201`, `makeTableListFilterHaveFocus` `:205`,
  `copyCreateTableSyntax` `:209`, `showCreateTableSyntax` `:213`, `checkTable` `:217`, `repairTable` `:221`,
  `analyzeTable` `:225`, `optimizeTable` `:229`, `flushTable` `:233`, `checksumTable` `:237`.
- **Help**: `showMySQLHelp` `:243`. Plus `installStandaloneConnectionMenuItem` `:254`, `dialogOKCancel` `:280`.

Validasi menu `validateMenuItem:` (`SPAppController.m:507-529`): `newWindow:`/`openConnectionSheet:`/
`openStandaloneConnectionWindow:` → YES; `newTab:` → tidak ada sheet terpasang; `duplicateTab:` →
`getConnection != nil`; selain itu delegasikan ke dokumen.

## 3. Model window/tab

**SPWindowController** (`Source/Controllers/Window/SPWindowController.swift`, `NSWindowController`):
- `databaseDocument = SPDatabaseDocument(windowController: self)` — dokumen DB adalah property lazy
  1:1 milik controller (`:36`). `uniqueID: UUID` `:38`; `frameAutosaveName = "MainWindow"` `:41`.
- `awakeFromNib` `:43`: tambah `.fullScreenPrimary`, `setupFramePersistence` `:101`, `setupAppearance` `:112`, `observeTitlebarTintPreference` `:162`.
- `setupAppearance` menyisipkan `contentBackground` NSBox opaque, lalu `databaseDocument.databaseView()`,
  set `window.tab.accessoryView = tabAccessoryView` (`:112-127`).
- `updateWindow(title:tabTitle:)` (`:179-186`); `updateWindowAccessory(color:isSSL:)` (`:194-198`).
- `NSWindowDelegate`: `windowShouldClose` digerbangi `databaseDocument.parentTabShouldClose()` +
  `setSpfSessionDocData:nil` (`:202-211`); `windowWillClose` simpan frame + post `SPDocumentWillClose` (`:213-221`).

**TabManager** (`Source/Controllers/Window/TabManager.swift`, `@objc` NSObject, dimiliki `SPAppController` `:78,104`):
- `managedWindows: [ManagedWindow]` (`:50`); `ManagedWindow` menyimpan controller, window, dan
  `NotificationToken` closing subscription (`:37-46`).
- `mainWindow` `:55`, `activeWindowController` `:67`, `windowControllers` `:71`, `windows: Set<NSWindow>` `:75`.
- `newWindowForTab()` `:100` (tab via `addTabbedWindow`, `:146`); `newWindowForWindow()` `:111`
  (window terpisah via `addChildWindow`, `:158`); `replaceTabServiceWithInitialWindow()` `:118`.
- `windowControllerWithDocument(processID:)` `:125` (ObjC `windowControllerWithDocumentWithProcessID:`).
- `createNewWindowController` `:133` → `SPWindowController(windowNibName: "MainWindow")`.
- Tab switching `switchToPreviousTab`/`switchToNextTab` `:91,95`.

**SPWindow** (`Source/Views/SPWindow.m`): `isReleasedWhenClosed` YES `:50`; `sendEvent:` menambah
menu konteks ⌥⎋ dan tab-switching ⇧⌘[ / ⇧⌘] / ⌥⌘← / ⌥⌘→ via `SPWindowSelectNextTabNotification`/
`SPWindowSelectPreviousTabNotification` `:61-131`; `undoManager` per-dokumen `:141-147`.

**Tab accessory**: `SPWindowTabAccessory` — garis warna 5pt + lock SSL opsional
(`update(color:isSSL:)` `:95-105`), teks judul (`setTitle` `:107`), tooltip lock
"Connection Secured via SSL" `:88`.

**Cara dokumen menempel**: `TabManager` membuat `SPWindowController`; controller itu membuat
`SPDatabaseDocument(windowController:)` secara lazy (`SPWindowController.swift:36`). `SPAppController`
lalu mengarahkannya via `tabManager.newWindowForWindow`/`newWindowForTab` dan
`windowController.databaseDocument setState:` / `setStateFromConnectionFile:`
(`.m:340-341,358-359,653-654,795-817,913-914`). `MainWindow.xib` File's Owner `SPWindowController`
dengan outlet `window` dan `SPWindow` (frameAutosaveName `MainWindow`) `MainWindow.xib:9-26`.

**Session restore**: `openSessionBundleAtPath:` baca `info.plist`, iterasikan `windows`→`tabs`
(terbalik), buat window/tab + `setStateFromConnectionFile:` (`SPAppController.m:752-832`).
Save pasangannya `saveConnectionsToSPF:` `:363-502`.

## 4. Struktur MainMenu.xib

File Owner `NSApplication`; outlet delegate → `SPAppController` (`:970-972`). Menu top-level:

- **Sequel Ace** (`:22`): About → `openAboutPanel:` (`:25-28`); Preferences… (⌘,) → `openPreferences:` (`:34-36`); Visit Website… → `visitWebsite:` (`:42-44`); Services (`:50`); Hide/Hide Others/Show All (`:56-69`); Quit (⌘Q) → `terminate:` (`:75-77`).
- **File** (tag 1, `:83`): New Connection Window (⌘N, tag 1000) → `newWindow:` (`:86-88`); New Connection Tab (⌘T, tag 1102) → `newTab:` (`:91-93`); Duplicate Connection Tab (⇧⌘T) → `duplicateTab:` (`:96-98`); Add To Favorites (⇧⌘A) → `addConnectionToFavorites:` (`:102-104`); Open… (⌘O) → `openConnectionSheet:` (`:108-110`); Open Recent + Clear Menu → `clearRecentDocuments:` (`:113-119`); Save Connection/Save Session/Save Query (⌘S) → `saveConnectionSheet:` (`:128-142`); Import… → `import:` (`:148-150`); Import from Clipboard… → `importFromClipboard:` (`:153-156`); Export… → `export:` (`:159-161`); Close (⌘W) → `performClose:` (`:165-167`); Print... (⌘P) → `printDocument:` (`:173-175`).
- **Edit** (tag 2, `:181`): Undo/Redo/Cut (`:186,191,197`); Copy varian → `copy:` (`:200-220`); Paste `:225`; Select ▸ Word/Line/Active Query/Enclosing Brackets/All (`selectCurrentWord:` `:235`, `selectCurrentLine:` `:241`, `selectCurrentQuery:` `:247`, `selectEnclosingBrackets:` `:252`, `selectAll:` `:257`); Insert NULL value → `insertNULLvalue:` `:267`; Convert ▸ (`doSelectionUpperCase:` `:286`, `doSelectionLowerCase:` `:292`, `doSelectionTitleCase:` `:298`, `doTranspose:` `:305`, `doRemoveDiacritics:` `:312`, normalisasi `:322-341`); Move ▸ (`:357,363`); Find ▸ (`performFindPanelAction:` `:377`, `performTextFinderAction:` `:382,387`).
- **View** (tag 3, `:506`): Table Structure (⌘1) → `viewStructure:` (`:509-511`); Table Content (⌘2) → `viewContent:` (`:514-516`); Table Relations (⌘3) → `viewRelations:` (`:519-521`); Table Triggers (⌘4) → `viewTriggers:` (`:524-526`); Table Info (⌘5) → `viewStatus:` (`:529-531`); Custom Query (⌘6) → `viewQuery:` (`:534-536`); Back/Forward In History → `backForwardInHistory:` (`:540-549`); Show Console → `toggleConsole:` (`:553-555`); Clear Console → `clearConsole:` (`:558-560`); Show Navigator → `toggleNavigator:` (`:564-567`); Display Binary Data as Hex `:571`; Hide/Show Toolbar → `toggleToolbarShown:` (`:578-581`); Customize Toolbar `:584-587`; Enter Full Screen `:591-594`.
- **Database** (tag 4, `:600`): Go to Database… → `showGotoDatabase:` (`:604-606`); Add/Delete/Duplicate/Rename/Alter → `addDatabase:`/`removeDatabase:`/`copyDatabase:`/`renameDatabase:`/`alterDatabase:` (`:610-637`); Refresh Tables → `refreshTables:` (`:641-644`); Flush Privileges → `flushPrivileges:` (`:648-651`); Refresh Databases → `setDatabases:` (`:654-656`); User Accounts → `showUserManager:` (`:662-664`); "View Using Encoding" submenu (semua `chooseEncoding:`, tag 10-190) (`:670-785`); Open Database in New Tab → `openDatabaseInNewTab:` (`:792-795`); Show Server Variables → `showServerVariables:`; Show Server Processes → `showServerProcesses:`; Shutdown Server… → `shutdownServer:` (`:799-814`).
- **Table** (tag 5, `:820`): Filter Content → `focusOnTableContentFilter:` (`:823-826`); Advanced Filter Content → `showFilterTable:` (`:829-831`); Filter Tables → `makeTableListFilterHaveFocus:` (`:834-837`); Copy Create Table Syntax → `copyCreateTableSyntax:` (`:841-844`); Show Create Table Syntax → `showCreateTableSyntax:` (`:847-850`); Check/Repair/Analyze/Optimize/Flush/Checksum Table (`:856-884`).
- **Bundles** (tag 6 `SPMainMenuBundles`, `:890`; `SPConstants.h:171`): isi dibangun runtime oleh `rebuildMenus` (`:1699-1804`) — Bundle Editor → `openBundleEditor:` `:1710`, Reload Bundles → `reloadBundles:` `:1713`, submenu scope Input Field/Data Table/General `:1723-1728`, item → `bundleCommandDispatcher:` `:1782`.
- **Window** (tag 7, `:894`): Minimize/Zoom/Bring All to Front (`:897-911`).
- **Help** (tag 8, `:917`): Sequel Ace Help (hidden) → `showHelp:` (`:920-922`); Online Help → `visitHelpWebsite:`; Online FAQ → `visitFAQWebsite:`; SSH Tunnel Debugging Info (hidden) → `showConnectionDebugMessages:` (`:938-941`); Keyboard Shortcuts → `viewKeyboardShortcuts:`; MySQL Help → `showMySQLHelp:`; MySQL Help for Word → `showMySQLHelpForCurrentWord:` (`:952-960`).
- **Dock Menu** (id 956, `:983-1015`): New Connection Window → `newWindow:`, New Connection Tab → `newTab:`, Open… → `openConnectionSheet:`, Open Recent → Clear Menu `clearRecentDocuments:`.

## 5. Judul & tint titlebar

**SAWindowTitleBuilder** (`Source/Controllers/Window/SAWindowTitleBuilder.swift`, Swift murni, tanpa AppKit):
- `SAWindowTitleResult(windowTitle:tabTitle:)` `:18`; `SAWindowConnectionState{connecting,disconnected,connected}` `:31`.
- `buildTitle(connectionState:filePath:isUntitled:bundleName:connectionName:database:table:mySQLVersion:showServerVersionInTitle:)` `:52`:
  `.connecting` → keduanya "Connecting…" (`:66-69`); `.disconnected` → `pathPrefix + bundleName` (`:71-73`);
  `.connected` → window `pathPrefix + "(MySQL <version>) "? + connectionName + "/db"? + "/table"?`, tab sama tanpa path/version (`:75-92`).
- `displayName(isConnected:filePath:isUntitled:bundleName:)` `:100`. `pathPrefix` = `"<filename> — "` hanya bila file URL nyata non-Untitled (`:117-120`).

**Tint titlebar** (`SAWindowTitlebarTint.swift`): struct `backgroundColor`, `titlebarAppearsTransparent`,
`separatorStyle`. `init(favoriteColor:isEnabled:)` — bila enabled dan ada warna → `backgroundColor=favoriteColor`,
transparan, separator `.none`; selain `.windowBackgroundColor`/false/`.automatic` (`:49-59`).
Diterapkan di `SPWindowController.applyTitlebarTint(_:)` (`:136-157`), digerbangi
`SPDisplayConnectionColorInTitlebar` (`:143`), digambar ulang pada `UserDefaults.didChangeNotification` (`:162-173`).

## 6. Jalur buka/simpan dokumen, recents, URL scheme

- **Open panel**: `openConnectionSheet:` membangun `NSOpenPanel` (tipe `.spf`, `.sql`, `.spfs`) + accessory encoding, lalu `application:openFiles:` (`SPAppController.m:552-617`).
- **Router**: `application:openFiles:` dispatch per ekstensi: `.sql`→`openSQLFileAtPath:`, `.spf`→`openConnectionFileAtPath:`, `.spfs`→`openSessionBundleAtPath:`, theme→`openColorThemeFileAtPath:`, bundle→`SPBundleManager.openUserBundleAtPath:`, selain itu `NSBeep` + log (`:623-650`).
- **`.spf`**: `openConnectionFileAtPath:` — new window + `setStateFromConnectionFile:` + recent (`:652-656`).
- **`.sql`**: `openSQLFileAtPath:` — warning ukuran (>1 MB), deteksi encoding, muat ke query editor atau window baru (`:658-750`).
- **`.spfs`**: `openSessionBundleAtPath:` — parse `info.plist`, rebuild window/tab (`:752-832`).
- **Recents**: semua via `[[NSDocumentController sharedDocumentController] noteNewRecentDocumentURL:…]` (`:501,655,746,831`).
- **URL scheme**: handler didaftarkan di `awakeFromNib` (`:177-180`). `handleEvent:withReplyEvent:` menerima `sequelace://` dan `mysql://` (`:871-885`). `mysql://` → `handleMySQLConnectWithURL:` parse connection string, buka window + `setState:fromFile:NO` (`:887-915`). `sequelace://PROCESS_ID@command/params` → `handleEventWithURL:` (`:917-1137`) mendukung command `chooseItemFromList` `:949`, `LaunchFavorite` `:970`, `SyntaxHighlighting` `:1005`, `passToDoc` `:1072`, dirutekan ke `SPDatabaseDocument` yang cocok berdasarkan process id (`:1049-1069`).

## 7. Preferensi

**SPPreferenceController** (`NSWindowController`, `SPPreferenceController.h:48`): `init` →
`initWithWindowNibName:@"Preferences"` + `SPMCPPreferencePane` programatik (`SPPreferenceController.m:60-69`).
`windowDidLoad` (`:74-95`) membangun array `preferencePanes`. Toolbar (`setupToolbar` `:166-243`).

Pane (urutan `SPPreferenceController.m:85-93`; semua view dari `Preferences.xib`):

| Pane class | Identifier toolbar | Nama tampil | Sitasi nama |
| --- | --- | --- | --- |
| `SPGeneralPreferencePane` | `SPPreferenceToolbarGeneral` | "General" | `SPGeneralPreferencePane.m:239` |
| `SPTablesPreferencePane` | `SPPreferenceToolbarTables` | "Tables" | `SPTablesPreferencePane.m:55` |
| `SPNotificationsPreferencePane` | `SPPreferenceToolbarNotifications` | "Alerts & Logs" | `SPNotificationsPreferencePane.m:77` |
| `SPEditorPreferencePane` | `SPPreferenceToolbarEditor` | "Query Editor" | `SPEditorPreferencePane.m:715` |
| `SPNetworkPreferencePane` | `SPPreferenceToolbarNetwork` | "Network" | `SPNetworkPreferencePane.m:110` |
| `SPFilePreferencePane` | `SPPreferenceToolbarFile` | "Files" | `SPFilePreferencePane.m:205` |
| `SPMCPPreferencePane` (Swift) | `SPPreferenceToolbarMCP` | "MCP Server" | `SPMCPPreferencePane.swift:204,208` |

Identifier ada di `SPConstants.m:327-335`. `SPPreferenceToolbarFavorites`/`Shortcuts` ada sebagai
konstanta tapi tak ada pane/toolbar item yang di-wire. **UNKNOWN**: nama nib pane — pane
diinstansiasi dari `Preferences.xib`; hanya controller yang punya nib bernama ("Preferences" `:62`).

**SPPreferencesUpgrade** (`SPPreferencesUpgrade.m`): `SPApplyRevisionChanges()` `:43` membandingkan
`CFBundleVersion` dengan `SPLastUsedVersion`; bila berubah tulis versi baru + `doBundleUpdate = YES`
(`:59-65`); catatan migrasi `<2061` (`:74`), reset font query-editor Monaco `<20026` (`:77-85`),
catatan kolom default `<20027` (`:88-90`); `showPostMigrationReleaseNotes:` menampilkan setelah 0.1 s
(`:94,100-118`). Dipanggil dari `+initialize` (`SPAppController.m:133`).

## 8. String yang terlihat pengguna (terpilih)

**SPAppController.m**: "App Sandbox Issue" `:218`; "Check for Updates..." `:309`;
"Do you really want to load a SQL file with %@ of data into the Query Editor?" `:677`;
"Connection data file couldn't be read. (%@)" `:767`; "The color theme '%@' already exists." `:855`;
"Invalid query parameters given" `:902`; "sequelace URL scheme command not supported." `:1078`;
"Are you sure you want to quit the app?" `:1476`; "Couldn't copy default themes to Application Support Theme folder!\nError: %@" `:1580`.

**SAAnalyticsConsentPolicy.swift**: "Help Improve Sequel Ace" `:80`; "Share Analytics" `:91`; "Don't Share" `:97`.

**SPWindow.m**: "Hide Toolbar" `:187`; "Show Toolbar" `:189`. **SPWindowTabAccessory.swift**:
"Connection Secured via SSL" `:88`. **SAWindowTitleBuilder.swift**: "Connecting…" `:67`.
