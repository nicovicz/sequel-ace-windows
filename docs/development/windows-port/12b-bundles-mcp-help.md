# Bundles, MCP server, dan help viewer

Kontrak source untuk sistem bundle (plugin shell), server MCP lokal, dan
penampil bantuan MySQL. Sitasi `file:line` merujuk checkout `02c910932`.

## 1. Bundles

### 1.1 Format & lokasi file

- Bundle adalah **direktori** berekstensi `.saBundle` (V2 saat ini) atau `.spBundle` (legacy); `SPUserBundleFileExtension=@"spBundle"`, `SPUserBundleFileExtensionV2=@"saBundle"` (`SPConstants.m:79-80`). `.spfs` juga ada (legacy single-file, `:76`).
- Di dalam direktori ada plist bernama **`command.plist`** (`SPConstants.m:466`), dimuat via `PropertyListSerialization` (`SPBundleManagerAdditions.swift:33-43`).
- **Search path** (`SPBundleManager.m:368-371`): `~/Library/Application Support/Sequel Ace/Bundles` (`SPConstants.m:101`) lalu `…/Default Bundles` di `NSBundle.mainBundle.sharedSupportPath` (`:370`).
- **Versi/migrasi**: `SPBundleCurrentVersion=2` (`SPConstants.m:464`); key `bundleVersion` (`:461`). `.spBundle` legacy di-rename ke `.saBundle` + `command.plist` ditulis ulang (`SPBundleManager.m:185-297`). Migrasi string mengganti URL scheme `"sequelpro"`→`"sequelace"` (`SPConstants.m:459-460`). Editor memilih ekstensi per versi: `>1` → V2, else legacy (`SPBundleEditorController.m:1026`).

### 1.2 Key plist bundle

(`SPConstants.m:435-461`): `command`, `scope`, `name`, `category`, `input`, `input_fallback`, `output`, `keyEquivalent`, `internalKeyEquivalent`, `exeUUID`, `tooltip`, `disabled`, `author`, `contact`, `uuid`, `title`, `description`, `trigger`, `withblob`, `isDefaultBundle`, `defaultBundleWasModified`, `bundleVersion`. Internal saat load: `label`, `path`, `keyEquivalent` (`:456-458`).

- **Scope** (`:403-406`): `editor`, `datatable`, `inputfield`, `general`. Editor hanya mengekspos tiga: Input Field / Data Table / General (`SPBundleEditorController.m:50-52`, index `kInputFieldScopeArrayIndex=0`, `kDataTableScopeArrayIndex=1`, `kGeneralScopeArrayIndex=2`, `:45-47`).
- **Input** (`:407-422`): `none`, `selectedtext`, `entirecontent`, `currentword`, `currentquery`, `currentline`, `selectedtablerowsastab|csv|sqlinsert`, `tablerowsastab|csv|sqlinsert`; blob `include`, `exclude`, `imagefilereference`, `filereference`.
- **Output** (`:423-430`): `none`, `insertastext`, `insertassnippet`, `replaceselection`, `replacecontent`, `showastexttooltip`, `showashtmltooltip`, `showashtml`.
- **Trigger** (`:431-434`): `none`, `databasechanged`, `tablechanged`, `tablerowchanged`.
- **Key equivalent** (`SPBundleManager.m:622-656`; dibangun editor `SPBundleEditorController.m:1132-1163`): karakter terakhir = tombol; modifier `^`=Control, `@`=Command, `~`=Option, `$`=Shift (juga meng-uppercase karakter).

### 1.3 Cara bundle dimuat

- `-[SPBundleManager reloadBundles:]` (`SPBundleManager.m:343-718`) rebuild `bundleItems`, `bundleCategories`, `bundleTriggers`, `bundleUsedScopes`, `bundleKeyEquivalents`, `installedBundleUUIDs` (`:356-361`). Tiap direktori `*.spBundle`/`*.saBundle` memuat `command.plist` (`:411-412`); file invalid di-beep (`:299-309`) dan **dihapus** (`removeBundle:` `:311-335`, masuk `badBundles`).
- Bundle didaftarkan hanya bila tidak disabled, `name` non-kosong, dan ada `scope` (`:423-427`). UUID wajib (`:431`); default bundle disalin ke Application Support + dibandingkan versinya via `SABundleVersionUpdater` (`:459-579`); default yang dimodifikasi user diduplikasi dengan sufiks `" (user)"` (`:516-544`).
- Trigger didaftarkan `path|scope|[uuid bila output==showashtml]` (`:611-619`). Item diurut by label (`:690-694`). `SPAppDelegate rebuildMenus` dipanggil (`:715`).

### 1.4 Cara bundle dijalankan

- **Dispatch**: `bundleCommandDispatcher:` (`:724-799`) menentukan scope dari first responder, cari key equivalent yang cocok, tanya `SPChooseMenuItemDialog` bila >1 (`:772`). Route ke `executeBundleItemForInputField:`/`ForDataTable:`/`ForApp:` (`:792-798`).
- **General**: `executeBundleItemForApp:` (`:801-952`) tulis input `~/tmp/SP_BUNDLE_INPUT_<uuid>` (`SPConstants.m:467`), bangun env (`:839-844`), jalankan command, petakan exit-code/`action` → output: text/HTML tooltip via `SPTooltip` (`:917-923`) atau window HTML via `SABundleHTMLOutputWindowController` (`:925-944`). Error → alert "BASH Error" (`:946-948`).
- **Input-field/data-table**: `-[SPTextView executeBundleItemForInputField:]` (`SPTextViewAdditions.m:502-641`) dan `-[SPCopyTable executeBundleItemForDataTable:]` (`SPCopyTable.m:1537-…`; env `:1577-1594`).
- **Runner** `+[SPBundleCommandRunner runBashCommand:…]` (`SPBundleCommandRunner.m:184-507`):
  - Command mulai `#!` → interpreter dipecah, body script ditulis ke `~/tmp/SP_BUNDLE_SCRIPT_COMMAND_<uuid>` (`SPConstants.m:469`; `:206-232`). Else body ditulis ke file itu dan dijalankan dengan **`/bin/sh`** (`:234-243`).
  - Task launch **`/bin/bash`** (`:258`); args `-c "<script> > <stdout> [< <input>]"` (`:338-351`); STDOUT → `~/tmp/SP_BUNDLE_OUTPUT_<uuid>` (`SPConstants.m:468`).
  - Interruptible dengan **⌘.** (`:392-412`). Termination status dipakai sebagai action code redirect; status non-zero dalam `SPBundleRedirectActionNone…LastCode` (200-208) = sinyal redirect valid (`:454, 479-499`; enum `SPConstants.h:703-714`).
  - Deteksi runtime scripting yang dihapus Apple (`python`, `python2`, `python2.7`, `php`, `ruby`, `:51-59`) di awal (`:232`) dan dari stderr (`:441`), lalu alert "Removed macOS Runtime" (`:142-157`).

### 1.5 Lingkungan eksekusi

- Shell `/bin/bash`; script di `~/tmp/SP_BUNDLE_*` (`SPConstants.m:467-471`).
- Env di `SPBundleCommandRunner.m:260-321` + `[databaseDocument shellVariables]` (`:310-312`). Vars built-in: `SP_ICON_FILE`, `SP_APP_RESOURCES_DIRECTORY`, `SP_BUNDLE_EXIT_*` (200-208), `SP_APP_CALLBACK_URL` (`:263-273`). Vars koneksi (per UUID saat ada doc terhubung): `SP_PROCESS_ID`, `SP_QUERY_FILE`, `SP_QUERY_RESULT_FILE`, `SP_QUERY_RESULT_STATUS_FILE`, `SP_QUERY_RESULT_META_FILE` (`:301-308`).
- Katalog variabel shell lengkap (`SPConstants.m:473-521`): `SP_ALL_DATABASES`, `SP_ALL_FUNCTIONS`, `SP_ALL_PROCEDURES`, `SP_ALL_TABLES`, `SP_ALL_VIEWS`, `SP_APP_RESOURCES_DIRECTORY`, `SP_BUNDLE_BLOB_FILES_DIRECTORY`, `SP_BUNDLE_PATH`, `SP_BUNDLE_SCOPE`, `SP_CURRENT_EDITED_COLUMN_NAME`, `SP_CURRENT_EDITED_TABLE`, `SP_CURRENT_HOST`, `SP_CURRENT_LINE`, `SP_CURRENT_PORT`, `SP_CURRENT_QUERY`, `SP_CURRENT_USER`, `SP_CURRENT_WORD`, `SP_DATABASE_ENCODING`, `SP_DATA_TABLE_SOURCE`, `SP_BUNDLE_EXIT_*`, `SP_ICON_FILE`, `SP_BUNDLE_INPUT`, `SP_BUNDLE_INPUT_TABLE_METADATA`, `SP_BUNDLE_OUTPUT`, `SP_PROCESS_ID`, `SP_QUERY_FILE`, `SP_QUERY_RESULT_FILE`, `SP_QUERY_RESULT_META_FILE`, `SP_QUERY_RESULT_STATUS_FILE`, `SP_RDBMS_TYPE`, `SP_RDBMS_VERSION`, `SP_SELECTED_DATABASE`, `SP_SELECTED_ROW_INDICES`, `SP_SELECTED_TABLE`, `SP_SELECTED_TABLES`, `SP_SELECTED_TEXT`, `SP_SELECTED_TEXT_RANGE`, `SP_USED_QUERY_FOR_TABLE`, `SP_APP_CALLBACK_URL`.

### 1.6 Window output HTML + JS bridge

- `SABundleHTMLOutputWindowController` host WKWebView (`:14-73`); frame `480×270`, autosave `SPBundleHTMLOutputWindow` (`:47-64`).
- API JS `window.system` lama dibuat ulang `SABundleJSBridge` (`:12-93`): panggilan bernilai (`run`, `getShellEnvironmentForName`) lewat `window.prompt()` dengan sentinel `"__sequel_ace_system__"` (`:24, 60-73, 99-113`); fire-and-forget post ke message handler `"system"` (`:28, 64-85, 117-140`). Fungsi: `run`, `getShellEnvironmentForName`, `insertText`, `setText`, `setSelectedTextRange`, `makeHTMLOutputWindowKeyWindow`, `closeHTMLOutputWindow`, `suppressExceptionAlert`; `window.onerror` → `reportError`.
- `bridge.actions.run` menjalankan ulang BASH via `SPBundleCommandRunner` (`SABundleHTMLOutputWindowController.swift:208-243`). Context menu View Source / Save Page As… / Print Page… (`:269-294, 337-384`).

### 1.7 Bundle editor

`SPBundleEditorController : NSWindowController`, nib **`BundleEditor`** (`:104`), xib `BundleEditor.xib` (window `"Bundle Editor"` `:50`). Outlet (`.h:38-75`): `commandTextView`, `commandsOutlineView`, berbagai text field, popup `input/inputFallback/output/scope/trigger/withBlob`, tombol, `keyEquivalentField` (`SRRecorderControl` `:314`), `disabledCheckbox`, `commandScrollView`, `splitView`, sheets. Tree: root `"BUNDLES"` + tiga node scope + opsional kategori (`:141-146, 1640-1751`). `saveBundle:atPath:` menulis `command.plist` (`:997-1083`) — **ROT13** `contact` (`:1050-1052`), set `defaultBundleWasModified` saat edit default bundle (`:1069-1070`).

`SPBundleCommandTextView` (NSTextView subclass) menambah nomor baris, persistensi font, tab stop, helper shift/comment (`SPBundleCommandTextView.h:33-55`).

## 2. MCP server

### 2.1 Transport

- `SPMCPServer` singleton HTTP lokal via `Network.framework` `NWListener` (TCP) (`SPMCPServer.swift:101-106, 137-165`). **Loopback saja**: `requiredLocalEndpoint = hostPort(127.0.0.1, port)` (`:131-135`); koneksi masuk diverifikasi `127.0.0.1`/`::1`/`localhost` else `403 Forbidden` (`:268-276`).
- Endpoint (`SPMCPHTTP.route` `:35-50`): **`POST /mcp`** = Streamable HTTP (MCP 2025-03-26), **`GET /sse`** + **`POST /message`** = legacy HTTP+SSE (2024-11-05), **`GET /health`** = `"OK"`; method salah → 405, tak dikenal → 404.
- Port dari `SPMCPServerPort` (default 8765; clamp 1024-65535) (`SPMCPPreferencePane.swift:31-33`). Max request 16 MiB → 413 (`:206, 246-249`). JSON-RPC di `dbQueue` (`:391-401`).

### 2.2 Method protokol

`initialize`, `tools/list`, `tools/call`, `resources/list`, `resources/read`, `completion/complete`, `prompts/list`, `prompts/get`, `ping` (`:457-495`); tak dikenal → error `-32601 "Method not found: <m>"`. Versi didukung `["2025-03-26","2024-11-05"]` (`:499`); serverInfo `name = "sequel-ace-mcp"`.

### 2.3 Tools

Definisi `SAMCPToolDefinitions.swift:21-103`; eksekusi `callTool` (`:551-655`). Semua tool menerima `connection` opsional (default tab aktif).

| Tool | Param wajib | Catatan |
| --- | --- | --- |
| `list_connections` | — | `:561` |
| `list_databases` | — | `:564` |
| `list_tables` | `database` | `:567` |
| `describe_table` | `database`,`table` | `:571` |
| `get_table_ddl` | `database`,`table` | `:577` |
| `list_views`/`list_procedures`/`list_functions`/`list_triggers` | `database` | `:583-587` |
| `get_routine_definition` | `database`,`type`,`name` | `:589` |
| `run_query` | `sql` (opt `params`,`limit`,`offset`) | read-only dicek `:595` |
| `explain_query` | `sql` | tolak ANALYZE `:603` |
| `sample_table` | `database`,`table` (opt `limit` def 10 max 1000, `offset`) | `:613` |
| `kill_query` | `process_id` | diblokir saat read-only `:621` |
| `count_rows` | `database`,`table` | `:628` |
| `export_results` | `sql` (opt `format` json/csv, `path`) | `:634` |
| `server_info` | — | `:642` |
| `table_sizes` | `database` | `:645` |
| `process_list` | — | `:649` |

Annotation per tool (`SAMCPToolDefinitions.swift:107-126`): `readOnlyHint`, `destructiveHint = !readOnly`, `openWorldHint:false`; `run_query`, `kill_query`, `export_results` non-read-only (`:66,85,93`).

### 2.4 Read-only default & guard

- **Default read-only ON**: `SPMCPReadOnly = true` (`PreferenceDefaults.plist:242-243`); `SPMCPServerEnabled=false`, `SPMCPServerPort=8765` (`:238-241`).
- Enforcement: `readOnlyRejection(for:)` menolak apa pun selain satu pernyataan baca non-destruktif saat `SPMCPReadOnly` (`:790-796`); `kill_query` hard-block (`:622-624`); `run_query` validasi ulang SQL terikat (`SPAppController+MCP.swift:526-537`).
- `SPMCPReadOnlyGuard` (`SPMCPHTTP.swift:56-343`): tolak komentar executable `/*!`/`/*M!` (`:167-170`), statement bertumpuk (`;` `:90`), akses file server `OUTFILE`/`DUMPFILE`/`LOAD_FILE` (`:96`), dan wajibkan keyword awal lulus `SPCustomQuerySQLClassifier.isQuerySafeWithoutDestructiveWarning` (`:102`); baca SQL di kedua bacaan backslash-escape (`:70-71, 115-116`). `explainWouldExecute` memblok `EXPLAIN ANALYZE` (`:105-149`).

### 2.5 Resources/prompts/completions

- **Resources** (`:664-699`): tabel DB aktif sebagai `sequelace://<connId>/<database>/<table>` (mimeType `application/json`); `resources/read` mengembalikan skema `describe_table`.
- **Prompts** (`:744-786`): `analyze_schema` (`database`), `summarize_table` (`database`,`table`), `optimize_query` (`sql`).
- **Completions** (`:703-739`): `connection`, `type` (`["view","procedure","function","trigger","event"]`), `database`, `table`; cap 100.

### 2.6 Batas auth/izin

- Tanpa token: batas identitas **loopback + Origin** — peer non-loopback → 403 "Forbidden" (`:270-276`); Origin non-loopback (proteksi DNS-rebinding) → 403 "Forbidden origin" (`:280-283`).
- `export_results` membatasi tulis ke folder export terkonfigurasi, resolve symlink + telusuri path dengan `O_NOFOLLOW` tiap komponen, mode file `0600` (`SPAppController+MCP.swift:770-871`).
- CSV export men-quote sel yang dimulai `= + - @ \t \r` (`SPMCPHTTP.swift:458-495`). Baris hasil dibatasi 10000 (`SPAppController+MCP.swift:35`).

## 3. Help viewer

### 3.1 Pencarian bantuan MySQL (SQL)

- `SPHelpViewerClient` (`SPHelpViewerClient.m:123-234`) query langsung:
  - `HELP '<searchString>'` (`:134`, tick-quoted; `"%"` literal di-escape `"\\%"` `:131`).
  - Nol baris → retry `HELP '<searchString>%'` (`:145`).
  - `"contents"` (magic TOC, `SPHelpViewerSearchTOC=@"contents"` `:39`) meminta daftar isi (`:209-217`).
- Kolom terpakai `name`, `description`, `example` (`:158-205`). Cross-link `[HELP …]` → anchor internal (`:193-197`); URL `http(s)|ftp|file …html` → link (maks 200) (`:176-190`).
- Error: gagal query render `ERROR <id> (<sqlstate>): <message>` (`:135-140`); nol baris render blok "No results found." (`:148-152`).

### 3.2 URL bantuan online

- `SAHelpViewerOnlineURLBuilder`: MariaDB (deteksi `"mariadb"` pada versi) → `https://mariadb.com/kb/en/<slug>/`, empty → `https://mariadb.com/kb/en/server/`, else → `https://mariadb.com/docs?q=<topic>` (`:29-31, 94-116`). MySQL search → `https://dev.mysql.com/doc/search/?d=<docID>&p=1&q=<encoded>` (`:89`), `&=+?#` dihapus dulu (`:81-88`). `docID` per versi (`:13-73`): 5.6→11, 5.7→12, 8.0→201, 8.4→371, ≥9.0→515; default 8.0.

### 3.3 Penggunaan WKWebView

- `SAHelpViewerWindowController` membangun `WKWebViewConfiguration`, inject `WKUserScript` seleksi + handler `"saHelpViewerSelection"` (`:32, 51-56, 329-336`). Render HTML via `webViewModel.loadHTMLString(helpString, baseURL: SAHelpViewerModel.internalHelpBaseURL)` (`:140`).
- Link internal resolve ke skema `sequelace-help://help/` (`SAHelpViewerModel.swift:89-92`); topik diklik → lookup help; link `http/https/ftp` → browser; lainnya ditolak (`:202-222, 239-257`).
- Riwayat term = stack cap 20 mengganti riwayat WebView (`:77, 129-184`). Perubahan appearance memanggil hook JS `window.onThemeChange('dark'|'light')` (`:238-240`).
- Template HTML `SPMySQLHelpTemplate.html` (`SPConstants.m:92`), diproses `MGTemplateEngine`; variabel `bodyClass`/`title`/`body` (`SPHelpViewerClient.m:67-80, 236-252`).

## 4. String yang terlihat pengguna (terpilih, dikutip persis)

**Bundle manager/runner**: `"Bundles Installation Error"`/`"Couldn't create Application Support Bundle folder!\nError: %@"` (`SPBundleManager.m:375`); `"Default Bundles Update"`/`"The following default Bundles were updated:\n%@\nYour modifications were stored as “(user)”."` (`:709`); `"Error while installing Bundle"`/`"The Bundle ‘%@’ has no UUID which is necessary to identify installed Bundles."` (`:1031`); `"Removed macOS Runtime"`/`"This Bundle attempted to run “%@”, but that runtime is not available on this macOS version.…"` (`SPBundleCommandRunner.m:144-150`); `"Couldn't launch task.\nException reason: %@\n ENV length: %lu"` (`:375-376`); `"Bundle Error"`/`"Error for “%@”:\n%@"` (`SPBundleManager.m:856`); `"BASH Error"` (`:948`).

**Bundle editor**: `"BUNDLES"` (`:142`); `"Input Field"`/`"Data Table"`/`"General"` (`:50-52`); `"Remove selected Bundle?"`/`"Are you sure you want to move the selected Bundle to the Trash and remove them respectively?"`/`"Remove"` (`:705-707`); `"Error while saving the Bundle."` (`:838`); tooltip `"Installed Bundles"` (`:1288`).

**MCP**: `"Method not found: <m>"` (`:494`); `"Request too large"` (`:247`); `"Forbidden"` (`:273`); `"Forbidden origin"` (`:281`); `"Method Not Allowed. POST to /mcp …"` (`:296`); `"No active database connection. Please connect to a database in Sequel Ace first."` (`:553`); `"Unknown tool: <name>"` (`:653`); `"Read-only mode is enabled. Only single, non-destructive read statements (SELECT, SHOW, DESCRIBE, EXPLAIN) are allowed. Turn off read-only mode in Sequel Ace Preferences > MCP Server to run write queries."` (`:793`); `"EXPLAIN ANALYZE is not allowed; it would execute the statement."` (`:609`).

**MCP preference pane**: `"Enable MCP Server (localhost only)"` (`:88`); `"Read-only mode (reject queries that modify data)"` (`:102`); `"Default: 8765. Any unprivileged port (1024–65535) may be used."` (`:127`); `"MCP server is running on http://127.0.0.1:%ld/mcp"` (`:329-337`).

**Help viewer**: `"MySQL Help Query Failed"` (`SPHelpViewerClient.m:138`); `"No results found."` (`:150`); `"Multiple Results for “%@”"` (`:211`); `"Table of Contents"` (`:215`); `"Error reading “%@.html”!<br>%@"` (`:78`); window title `"MySQL Help"` (`SAHelpViewerModel.swift:229`); placeholder `"Search MySQL Help"` (`:33`).

## 5. UNKNOWN

- Bundle writing: `SPBundleManager.m:239` memakai `SPBundleFileName` untuk `command.plist` migrasi, tapi `SPBundleEditorController.m:1015-1026` berkomentar "we should NOT get any V1 bundles" — kasus ekstensi pre-V2 tidak konsisten.
- Produser konkret `lastBundleBlobFilesDirectory`/`SP_BUNDLE_BLOB_FILES_DIRECTORY` di luar file yang diperiksa.
- MCP tanpa token autentikasi — keamanan hanya loopback+Origin; apakah build sandbox menegakkan tambahan, **UNKNOWN**.
- Tabel server di balik `HELP` (mis. `mysql.help_topic`) tak dirujuk namanya; **UNKNOWN**.
