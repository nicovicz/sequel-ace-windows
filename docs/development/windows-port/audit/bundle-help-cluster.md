# Audit: Bundle-support + Help-viewer cluster

Audit method-by-method klaster bundle support + help viewer. `Start`/`End` = nomor
baris di file masing-masing. Baseline commit `02c910932`.

File: `BundleSupport/SPBundleCommandRunner.m` (509), `BundleSupport/SABundleHTMLOutputWindowController.swift` (385),
`HelpViewer/SPHelpViewerClient.m` (300), `HelpViewer/SAHelpViewerModel.swift` (314),
`HelpViewer/SAHelpViewerWindowController.swift` (380). Plus `SAHelpViewerOnlineURLBuilder.swift` (136) sebagai rujukan.

---

## 1. `SPBundleCommandRunner.m` (509)

Runner BASH/script class-method-only: jalankan script Bundle secara async sambil pump run loop untuk ⌘., lalu return STDOUT/STDERR.

### Inventaris
Static `SPBundleRemovedScriptingRuntimeDeprecationURL` (38), `SPBundlePythonRuntimeRemovalURL` (39); kategori `NSObject (SPBundleMethods)` (42–47: getter/setter `lastBundleBlobFilesDirectory` 44–45). Tanpa ivar instance; state hanya di local/`static` (`removedRuntimeNames` 53, `dispatch_once_t` 54).

### `#pragma mark`
**Tidak ada.**

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 51 | 59 | `+removedSystemScriptingRuntimeNames` | Cache lazy set runtime macOS yang dihapus (python/php/ruby) |
| 61 | 79 | `+isCommandAvailableOnPath:withEnvironment:` | Scan entri `PATH` untuk executable command |
| 81 | 120 | `+missingRemovedScriptingRuntimeForInterpreterArguments:environment:` | Deteksi shebang/`env` interpreter = runtime dihapus |
| 122 | 140 | `+missingRemovedScriptingRuntimeFromErrorMessage:` | Parse stderr untuk hint runtime "command not found" |
| 142 | 157 | `+showRemovedScriptingRuntimeUnavailableAlertForRuntime:` | Alert + link release notes Apple |
| 168 | 171 | `+runBashCommand:withEnvironment:atCurrentDirectoryPath:error:` | Wrapper delegasi (caller/context nil) |
| 184 | 507 | `+runBashCommand:…callerInstance:contextInfo:error:` | Runner penuh: tulis script, spawn `/bin/bash`, stream hasil |

### Temuan
- **Shell invocation**: shebang atau fallback `/bin/sh` menulis command ke file temp (206–243); arg dibangun `-c "<scriptHeaderArgs> > <stdoutFilePath> [< inputFile]"` (339, 346). Launch via `NSTask` di `/bin/bash` (257–258, 372).
- **Cancellation**: pump run loop buatan tangan mendengar ⌘. → `terminate` task (392–412); re-aktifkan app di 435.
- **Guard runtime dihapus**: deteksi pre-launch (232) + post-launch stderr (441–451) keduanya memunculkan alert; return `@""` + POSIX code 9.
- **Logging ARG_MAX**: hitung panjang env+arg + warning bila > `ARG_MAX` (332–366) — cited sebagai fix "Couldn't posix_spawn: error 7".
- **Populasi env**: icon/resources/callback URL (263–273); variabel proses/query file `SPBundleShellVariable*` per-dokumen (301–317).
- **UNKNOWN**: `dictionaryValueToString()` (357) + `[NSString stringWithNewUUID]` (194) resolve di luar file.

---

## 2. `SABundleHTMLOutputWindowController.swift` (385)

Window controller WKWebView yang merender output HTML Bundle + rekreasi API JS legacy `window.system` via `SABundleJSBridge`.

### Inventaris
`final class SABundleHTMLOutputWindowController: NSWindowController, NSWindowDelegate` (18). Property 20–36: `windowUUID`, `docUUID`, `suppressExceptionAlerting`, `windowWillCloseHandler`, `model` (`SAWebViewModel`), `bridge` (`SABundleJSBridge`), `titleSubscription`, `restoreFrame`, `origFrame`.

### `// MARK:`
`38 (Init), 79 (Legacy ObjC API (display content)), 115 (Window delegate), 135 (Bridge wiring (window.system)), 245 (Web view hooks), 335 (Document actions (context menu))`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 40–77 | | `convenience init()` / `init(externalConfiguration:)` / `required init?(coder:)` | Init (traps untuk XIB) |
| 81–113 | | `displayHTMLContent(_:withOptions:)` / `displayURLString(_:withOptions:)` / `applyDisplayOptions(_:)` | API ObjC legacy |
| 117 | 133 | `windowWillClose(_:)` | Restore frame, fire handler, clear konten, unregister |
| 137 | 200 | `configureBridgeActions()` | Sambungkan aksi JS `window.system` ke closure native |
| 202 | 205 | `appShellEnvironment()` | Ambil shell env dari app controller |
| 208 | 243 | `runBashCommand(_:jsUUID:)` | Port legacy `window.system.run`, panggil bundle runner |
| 247 | 299 | `configureModelHooks()` | Wire URL handling, new-window, context menu, keyDown |
| 302 | 333 | `handleKeyDown(_:)` | Implement zoom ⌘±/⌘0 + history ⌘←/→ |
| 337–346 | | `showSourceCode()` | Buka window sibling dengan outer HTML ter-escape |
| 348–374 | | `saveDocument(_:)` | Save-panel sheet tulis source page UTF-8 HTML |
| 376–384 | | `printDocument(_:)` | Print operation modal untuk web view |

### Temuan
- **JS bridge**: closure `bridge.actions.run/.shellEnvironment/.insertText/.setText/.setSelectedTextRange/.makeWindowKey/.closeWindow/.suppressExceptionAlert/.reportJSError` dikonfigurasi 138–199; nama message handler dari `SABundleJSBridge.messageHandlerName` dihapus saat close 126–127.
- **Shell invocation**: `runBashCommand` (208–243) inject env query-file `SPBundleShellVariable*` + panggil `SPBundleCommandRunner.runBashCommand` dengan `callerInstance: NSApp.delegate` (226–232).
- **Kebijakan new-window**: `onCreateWebView` sengaja return nil + re-create controller sibling (259–267); komentar 252–258 jelaskan hindari cross-wiring + hilangnya handle window JS.
- **Tanpa theme hook** — hanya `titleSubscription` pada `model.$pageTitle` (67–71).
- **UNKNOWN**: perilaku `SAWebViewModel` (`loadHTMLString`/`load`/`fetchOuterHTML`/`makeTextLarger`/`Smaller`/`StandardSize`/`goBack`/`goForward`/`reloadInitialHTMLString`/`canGoBack`, dirujuk 85, 93, 307–328, 338, 362).

---

## 3. `SPHelpViewerClient.m` (300)

Client/datasource yang menjalankan query SQL `HELP`, membangun HTML via template engine, dan proxy window help berbasis WKWebView.

### Inventaris
Global `SPHelpViewerSearchTOC = @"contents"` (39), `SPUserClosedHelpViewerNotification` (41); extension `SPHelpViewerClient () <SPHelpViewerDataSource>` (43–49).
Ivar `.h:86–93`: `controller` (`SAHelpViewerWindowController*`), `helpHTMLTemplate`, `mySQLConnection`, `engine` (`MGTemplateEngine*`).

### `#pragma mark`
`86, 287`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 53–84 | | `+initialize` / `-init` | Init (buat controller, load template, setup engine) |
| 88–121 | | `helpViewerClosed:` / `openOnlineHelpForTopic:` | Close + online help |
| 123 | 234 | `HTMLHelpContentsForSearchString:autoHelp:` | Jalankan HELP query + render HTML hasil |
| 236–263 | | `generateHelp:theHelp:` / `+linkToHelpTopic:` / `-setConnection:` | HTML + link |
| 268–298 | | `helpWebViewWindow` / `showHelpFor:addToHistory:calledByAutoHelp:` / `showHelpForCurrentWord:` / `dealloc` | Proxy |

### Temuan
- **SQL HELP**: `HELP <tickQuotedString>` di 134; wildcard `%` fallback `HELP 'term%'` di 145; magic string TOC di 209–216.
- **URL**: delegasi ke `SAHelpViewerOnlineURLBuilder.onlineHelpURLForTopic:serverVersionString:mysqlMajorVersion:…` (110–114), pakai version string ter-cache karena `-isMariaDB` baca raw handle (komentar 109).
- **HTML/link**: regex deteksi http/ftp/file URL (177–190, cap 200 loop), konversi cross-link `[HELP x]` + `See/In/and [x]` via `linkToHelpTopic:` (193–197); template var `bodyClass/title/body` (247–251).
- **Theme hook**: kelas dark dari match `effectiveAppearance` (239–245) → `addBodyClass = @"dark"`.
- **UNKNOWN**: nama resource `SPHTMLHelpTemplate` + selector `tickQuotedString` resolve di luar file.

---

## 4. `SAHelpViewerModel.swift` (314)

State + logika keputusan murni untuk help viewer (bebas AppKit/WebKit, dibagi dengan target unit test).

### Tipe & konstanta
`enum SAHelpTarget: Int, CaseIterable` (14–49: mysql/page/web); `enum SAHelpNavigationDecision: Equatable` (54–64: lookupTerm/openExternally/allow/deny); `final class SAHelpViewerModel: ObservableObject` (71–314) dengan `historyCapacity = 20` (77), `internalHelpScheme = "sequelace-help"` (89), `internalHelpBaseURL` (92), `externallyOpenableSchemes = ["http","https","ftp"]` (97); `enum NavigationKind` (190–197); `enum KeyCommand` (289–296).

### `// MARK:`
`99 (Search field state), 129 (Term history), 186 (Navigation policy (pure)), 224 (Window title), 234 (Theme), 242 (Context menu (pure)), 285 (Keyboard shortcuts (pure))`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 113–127 | | `focusSearchField()` / `clearSelection()` | Fokus + reset seleksi |
| 153–184 | | `visit(_:)` / `goBack()` / `goForward()` | Riwayat term |
| 202 | 222 | `policy(for:kind:)` | Matriks routing navigasi murni |
| 228–240 | | `windowTitle(pageTitle:)` / `themeChangeScript(isDarkAppearance:)` | Judul + JS hook tema |
| 280 | 283 | `offersSelectionSearchItems(selectedText:menuItemIdentifiers:)` | Tampilkan item "Search in…" |
| 300 | 313 | `keyCommand(characters:hasCommand:hasShift:)` | Map key event → `KeyCommand` |

### Temuan
- **Kebijakan navigasi**: link internal `sequelace-help://` → `lookupTerm` (208–215); hanya http/https/ftp buka eksternal (217–221); `.other` (load `loadHTMLString`) diizinkan (204).
- **Theme hook**: `themeChangeScript` (238–240) hasilkan string JS; tanpa import AppKit.
- **Context menu**: string literal `WKMenuItemIdentifier…` di-prune (251–267); heuristik link-vs-selection (271–283) menggantikan `WebElementLinkIsLive` legacy.
- **Keyboard map**: `+`/`=` larger, `-` smaller, ⌘F focus, ⇧⌘G/⌘G find, ⇧⌘M/P/W target select (303–311).
- Self-contained by design (komentar 66–70).

---

## 5. `SAHelpViewerWindowController.swift` (380)

Controller panel help berbasis WKWebView: kebijakan navigasi, kustomisasi context menu, tema, dan bridge data-source ObjC.

### Inventaris
`final class SAHelpViewerWindowController: NSWindowController, NSWindowDelegate` (22) dengan `dataSource` (24), `model` (26), `webViewModel` (27), `titleSubscription` (28), `appearanceObservation` (29); `selectionMessageHandlerName = "saHelpViewerSelection"` (32); `SAHelpViewerSelectionHandler` (341–352); `SAHelpViewerPanel` (356–366); extension `NavigationKind` (368–380).

### `// MARK:`
`34 (Init), 108 (MySQL help (ObjC API)), 143 (Actions), 203 (Keyboard shortcuts), 237 (Navigation policy), 259 (Context menu), 303 (Theme), 316 (Window delegate), 325 (Selection bridge)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 36–106 | | `init()` / `required init?(coder:)` / `deinit` | Setup panel/config/view/KVO |
| 111–141 | | `showHelp(for:addToHistory:calledByAutoHelp:)` | Ambil HTML dari data source + load |
| 146–201 | | `submitSearch()` / `navigate(_:)` / `findInPage(...)` / `showHelpForWebViewSelection` / `searchInDocForWebViewSelection` | Aksi |
| 206 | 235 | `handleKeyEquivalent(_:)` | Map key command → aksi |
| 239 | 257 | `decidePolicy(for:)` | Terapkan kebijakan model → action policy WebKit |
| 264 | 301 | `customizeContextMenu(_:)` | Prune + sisip entri "Search in…" |
| 305–323 | | `themeChanged()` / `windowShouldClose(_:)` | Tema + close |
| 329–351 | | `makeSelectionUserScript()` / `SAHelpViewerSelectionHandler.*` | Bridge seleksi |
| 356–378 | | `SAHelpViewerPanel.keyEquivalentHandler` / `performKeyEquivalent` / `NavigationKind.init(_:)` | Panel + mapping |

### Temuan
- **Kebijakan navigasi**: `decidePolicy` (239–257): `.lookupTerm`→cancel, `.openExternally`→`NSWorkspace.open`, `.allow`, `.deny`→cancel.
- **JS bridge/seleksi**: user script `selectionchange` disuntik, post ke `saHelpViewerSelection` (329–335); handler update `model.selectedText` (349–351); dihapus di `deinit` (104–105).
- **Routing HELP sql**: `showHelp` map term kosong → `SPHelpViewerSearchTOC` kecuali autoHelp (113–119), lalu minta `htmlHelpContents(forSearchString:autoHelp:)` (121).
- **Theme hook**: `appearanceObservation` KVO (88–94) → `themeChanged` → `evaluateJavaScript(SAHelpViewerModel.themeChangeScript(...))` (305–313).
- **UNKNOWN**: `SAWebViewModel` (`loadHTMLString:baseURL:`, `find`, `evaluateJavaScript`, `webView`) di luar file.

---

## 6. Rujukan — `SAHelpViewerOnlineURLBuilder.swift` (136)

`@objc final class SAHelpViewerOnlineURLBuilder: NSObject` (11) · `private enum MySQLDocumentID` (13–19: doc-id 11/12/201/371/515) · `onlineHelpURL(forTopic:…)` (22–41) · `isMariaDBServerVersion` (43–49) · `mysqlDocumentID` (51–73) · `mysqlOnlineHelpURL` (75–92) · `mariaDBOnlineHelpURL` (94–116) · `serverVersionIsGreaterThanOrEqualTo` (118–135).
**URL building**: MySQL `…/doc/search/?d=<id>&p=1&q=<topic>` (89) setelah strip `&=+?#` dari query (82); MariaDB slug `https://mariadb.com/kb/en/<slug>/` (102) dengan query fallback (112) + default `/server/` (115).

## Ringkasan lintas-file
- **Shell invocation** terpusat di `SPBundleCommandRunner.m` (257–412), dijangkau dari `SABundleHTMLOutputWindowController.swift:226` (`window.system.run`).
- **JS bridge (`window.system`)** = `SABundleJSBridge` di `SABundleHTMLOutputWindowController.swift:138–199`; help viewer pakai bridge seleksi terpisah (`SAHelpViewerWindowController.swift:329–351`).
- **HELP SQL** hanya di `SPHelpViewerClient.m:134–145`.
- **Theme hook** dihasilkan `SAHelpViewerModel.themeChangeScript` (238–240), dikonsumsi `SAHelpViewerWindowController.themeChanged` (305–313); window Bundle tanpa theme hook.
- **UNKNOWN (luar scope)**: `SAWebViewModel`, `SABundleJSBridge`, `SAPrintUtility`, `SPBundleManager`, `SPBundleShellVariable*`.
