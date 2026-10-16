# SQL editor dan eksekusi query

Kontrak source untuk tab Query: pemilihan teks/SQL yang dijalankan, pemisahan
statement, eksekusi streaming & multi-result, EXPLAIN, classifier SQL, dan
fitur editor (highlight, autocompletion, bracket, snippet). Sitasi `file:line`
merujuk checkout `02c910932`.

File: `Source/Controllers/MainViewControllers/SPCustomQuery.m` (4079 baris) + `.h`;
`SPCustomQuery+Explain.swift`, `SPCustomQuerySQLClassifier.swift`;
`Source/Other/Parsing/SPSQLParser.m/.h` (1110 baris), `SPSyntaxParser.h`,
`SPSQLTokenizer.h`, `SPEditorTokens.h`; `Source/Views/TextViews/SPTextView.m` (3884 baris);
`SPTextViewAdditions.m/.h`; `SPBrackets.m/.h`, `SPBracketHighlighter.m/.h`;
`SPNarrowDownCompletion.m/.h`, `SACompletionWindowLayout.swift`; `Console.xib`, `DBView.xib`.

## 1. Alur eksekusi query

### 1.1 Memilih teks SQL

Preferensi `QueryPrimaryControlRunsAll` (`SPConstants.m:273`) menentukan tombol/menu mana yang memetakan ke apa.

- `runPrimaryQueryAction:`/`runSecondaryQueryAction:` (`SPCustomQuery.m:157-176`): keduanya dispatch ke `runAllQueries:` atau `runSelectedQueries:` sesuai `[prefs boolForKey:SPQueryPrimaryControlRunsAll]` (`:159,171`).
- `runAllQueries:` (`192-231`): bangun `SPSQLParser` atas **seluruh isi editor** (`211`), `setDelimiterSupport:YES` (`212`), pecah per `;` via `splitStringByCharacter:';'` (`213`). Bila `containsCarriageReturns`, tiap bagian `normaliseQueryForExecution:` (`216-222`). Reset `queryStartPosition=0` (`225`). Guard: return bila `[tableDocumentInstance isWorking]` (`198`) atau event key-up (`201`).
- `runSelectedQueries:` (`237-281`):
  - Hanya caret (`selectedRange.length == 0`): jalankan **statement di bawah caret** `[[textView string] safeSubstringWithRange:currentQueryRange]` ternormalisasi (`252-258`); beep + `NSLog(@"Could not find a query range suitable to run query")` bila tak ada range (`253-257`).
  - Seleksi non-kosong: pecah **teks terpilih** dengan `SPSQLParser` baru (`262-274`).
- `currentQueryRange` (statement di bawah caret) dihitung tiap perubahan seleksi: `textViewDidChangeSelection:` → `queryRangeAtPosition:lookBehind:` (`3100-3113`).

### 1.2 Pemisahan statement / delimiter (`SPSQLParser`)

`SPSQLParser` adalah subclass `NSMutableString` (`SPSQLParser.h:56`) yang sadar-SQL (mengabaikan komentar/string ter-quote; bracket opsional; dukungan DELIMITER opsional).

- API split: `splitStringByCharacter:` (`SPSQLParser.m:563`), `...skippingBrackets:` (`572`), `...ignoringQuotedStrings:` (`581`), bentuk penuh (`589`); versi range `splitStringIntoRangesByCharacter:` (`631`).
- Scanner inti `firstOccurrenceOfCharacter:afterIndex:skippingBrackets:ignoringQuotedStrings:` (`702-841`):
  - Bila `supportDelimiters && delimiter`, cocokkan string delimiter aktif bukan char (`719-725`).
  - Lewati string ter-quote via `endIndexOfStringQuotedByCharacter:` (`737-747`), lacak `bracketingLevel` untuk `(`/`)` (`750-756`), lewati komentar `--[ws]`, `#`, `/* */` bila `!ignoreCommentStrings` (`759-778`), catat `containsCRs` (`781-783`).
  - Parse statement `DELIMITER xxx` dengan pola regex `^(delimiter[ \t]+(\S+))(?=\s|\Z)` (`787-833`); set `lastMatchIsDelimiter=YES`.
- Loop `splitStringByCharacter:` mengonsumsi match delimiter (`599-602`) dan melewati fragmen `delimiter …` akhir (`619-620, 662-663`).
- `normaliseQueryForExecution:noBackslashEscapes:` (`241-351`): trim whitespace, jalan sadar-quote, konversi CR/CRLF → LF (`323-347`).
- `setNoBackslashEscapes:` memodelkan `NO_BACKSLASH_ESCAPES` (`SPSQLParser.h:112`, `.m:103`). **`SPCustomQuery` tidak pernah men-set `noBackslashEscapes`** — pemisahan selalu mengasumsikan backslash escape (**UNKNOWN** apakah sesi `NO_BACKSLASH_ESCAPES` salah-split; classifier menangani kedua bacaan).

### 1.3 Eksekusi streaming (`performQueriesTask:`)

- Gate `performQueries:` (`689-734`): bila `!SPQueryWarningEnabled` → tanpa warning (`692-695`); bila `queriesContainDestructiveSQL:` NO → tanpa warning (`699-702`); selain itu alert `"Execute SQL?"` / `"Do you really want to proceed with this query?\n\n %@"` (atau `"...these queries?..."` bila count>1) / tombol `"Proceed"` (`714-733`). Pesan diringkas ke `SPMaxQueryLengthForWarning` = 1000 (`SPConstants.m:60`).
- `queriesContainDestructiveSQL:` (`652-683`): default YES; per query trim tanya `[SPCustomQuery isQuerySafeWithoutDestructiveWarning:]`; safe ⇒ NO dan stop (`667-679`).
- `performQueriesWithNoWarning:` (`604-641`): akhiri edit tertunda (`609`), set task text (`"Running query %i of %lu..."`/`"Running query..."`, `611-616`), set `errorTextTitle` = `"Query Status"`, clear `affectedRowsText` (`619-623`), detach `performQueriesTask:` (atau inline bila bukan main thread) (`634-640`).
- `performQueriesTask:` (`736-1138`) — inti:
  - Ambil versi server / flag MariaDB (`747-749`). Post `"SMySQLQueryWillBePerformed"` (`765`). Set `SPCustomQueryQueryMode`, disable query retry (`760,781`), aktifkan cancellation dengan judul `"Stop queries"`(>1)/`"Stop query"` (`787-788`). `SPQueryProgressUpdateDecoupling` (`88-99, 790-796`).
  - **Loop per query** (`799-980`):
    - Trim/skip kosong (`806-809`); simpan untuk `usedQuery`/history (`812`).
    - Baca `SELECT @@lower_case_table_names` lazy hanya saat DROP case-only (`818-827`).
    - Eksekusi `[mySQLConnection resultStoreFromQueryString:query assertingDatabaseContext:databaseName]` (`830`), akumulasi `[resultStore queryExecutionTime]` (`831`).
    - **Multi result set**: hanya hasil query **terakhir** yang diambil/disimpan; hasil antara dibuang via `[resultStore cancelResultLoad]` (`836-872`). `fieldDefinitions` hasil terakhir → `cqColumnDefinition` (`839`), tabel dikonfigurasi untuk copy SQL-INSERT (`862-867`), `updateResultStore:` memuat (`869`).
    - **Affected rows** dari `[mySQLConnection rowsAffectedByLastQuery]` (bila ≠ `~0`) else `[resultStore numberOfRows]` (`875-880`).
    - Error disimpan; single-query mengganti `errors` (`953`); multi-query menambah `"[ERROR in query %ld] %@"` (`904, 948-950`). Error `2006` diberi penjelasan "MySQL server has gone away" (`893-895`).
    - Cancel: bila `[mySQLConnection lastQueryWasCancelled]` → `"Query cancelled."` (`886-887`) dan `break` (`978-979`).
    - Query sukses bisa set `tableListNeedsReload`/`databaseWasChanged` via regex `(?i)^\s*\b(create|alter|drop|rename)\b\s+.` / `(?i)^\s*\b(use|drop\s+database|drop\s+schema)\b\s+.` (`959-964`) dan `SASQLDatabaseContext` (`965-972`).
  - Setelah loop: reload tabel/database bila perlu (`982-994`). `usedQuery = [tempQueries componentsJoinedByString:@";\n"]` (`997`); `lastExecutedQuery` = terakhir (`998`). Set kosong → eksekusi `""` + cancel load (`1001-1005`). Tambah history (`1008-1010`), update UI (`1013-1018`), status (`1021-1067`), post `"SMySQLQueryHasBeenPerformed"`, `SANotificationCenter … "Query Finished"` (`1085-1113`), callback opsional, `endTask` (`1096,1121`).
  - String status dari `statusErrorString` = `"Errors"`/`"No errors"` + bar `"%@; 1 row affected"`, `"%@; %ld rows affected"` + `", first row available after %1$@"` atau `", taking %1$@"`, varian multi-query `"...rows affected in total, by %ld queries taking %@"`, varian cancel `"%@; Cancelled after %@"`/`"%@; Cancelled in query %ld, after %@"` (`1021-1065`).

### 1.4 Store hasil streaming, rebuild tabel

- `updateResultStore:` (`1140-1169`): clear record view, lock `resultDataLock`, hapus baris lama, `setDataStorage:… updatingExisting:NO` (`1157`), `startDownload` (`1161`), `initQueryLoadTimer` (`1164`), `awaitDataDownloaded` (`1166`).
- `initQueryLoadTimer` (`1707-1716`) jadwalkan `queryLoadUpdate:` tiap 0.02 s; `clearQueryLoadTimer` (`1722-1728`).
- `queryLoadUpdate:` (`1736-1780`): refresh adaptif; interval 1→10→25 (`1770-1777`); autosize kolom di awal dan sekali ≥200 baris (`1761-1765`).
- `updateTableView` (`1900-1966`): rebuild kolom dari `cqColumnDefinition`, set row height, formatter/type grouping per kolom, tooltip header `"%@ – %@%@"` (nama – tipe(panjang)) (`1954`).

## 2. EXPLAIN (`SPCustomQuery+Explain.swift`)

- Trigger: `runExplainQueryAction:` (`+Explain.swift:40-103`), menu **"Explain Current Query"** `⌥⌘E` (`DBView.xib:1659-1664`). Divalidasi hanya bila `!isWorking` dan editor non-kosong (`SPCustomQuery.m:3491-3494`).
- Pemilihan query sama seperti run-selected (`54-89`): caret → `currentQueryRange` ternormalisasi (`58-66`); seleksi → split sadar-delimiter, fragmen komentar diabaikan, **harus tepat satu** statement selain itu `reportUnsupportedExplain()` (`72-87`).
- Kelayakan: `isQueryExplainable(query)` (`91-94`); selain itu `reportUnsupportedExplain()`.
- Invokasi: `performQueries(["EXPLAIN \(query)"], withCallback: nil)` (`96-102`).
- Gagal: `reportUnsupportedExplain()` beep + `errorTextTitle` `"Query Status"` + `errorText` `"EXPLAIN is only supported for a single SELECT or WITH statement."` (`109-116`).

## 3. Classifier SQL (`SPCustomQuerySQLClassifier.swift`)

Dua method `@objc` (`+Explain.swift:23-32`):

- **`isQueryExplainable`** (`Classifier.swift:32-49`): false untuk nil/kosong; false bila strip komentar tak tentu (`39`); buang group `(` awal (`42-44`); true iff teks trim-uppercase diawali `SELECT` atau `WITH` pada batas identifier (`47-48`).
- **`isQuerySafeWithoutDestructiveWarning`** (`51-117`): bila query memuat `\`, dibaca dengan kedua bacaan backslash dan harus aman di keduanya (`64-72`). Wajib: strip komentar diketahui (`96`), lalu leading `SHOW`/`SELECT` ⇒ aman (`107-108`); `EXPLAIN`/`DESCRIBE`/`DESC` ⇒ aman kecuali `EXPLAIN ANALYZE <mutating>` (`110-114`); `WITH` setelah `EXPLAIN ANALYZE` konservatif tidak aman (`338`); selainnya tidak aman (`116`).
- **Penanganan komentar** (`stripSQLComments` `128-274`): ganti `#`, `--[ws]`, `/* */` dengan spasi; pertahankan komentar executable MySQL `/*! … */` dan `/*M! … */` dengan gate versi opsional (`208-263`); flag `hasIndeterminateExecutableComment` (`238-241, 288-303`). Aturan leksikal sengaja mencerminkan `SPMySQLFramework` (`124-127`).
- **`SASQLDatabaseContext`** (`474-623`): menghitung apakah USE/DROP DATABASE mengubah DB aktif (`518-553`), apakah lookup `lower_case_table_names` perlu (`490-516`), regex `USE`/`DROP DATABASE|SCHEMA` (`478-483`), unquote identifier (`569-588`). Dipakai dari `SPCustomQuery.m:818-827,965-972`.

## 4. Fitur editor (`SPTextView`)

### 4.1 Syntax highlighting (lexer flex, kelas kata CSS-mode)

- Lexer `SPSQLTokenizer.l`; konstanta token `SPEditorTokens.h:35-44` (`SPT_DOUBLE_QUOTED_TEXT=1`, `SPT_SINGLE_QUOTED_TEXT=2`, `SPT_COMMENT=3`, `SPT_BACKTICK_QUOTED_TEXT=4`, `SPT_RESERVED_WORD=5`, `SPT_WHITESPACE=6`, `SPT_NUMERIC=7`, `SPT_VARIABLE=8`, `SPT_WORD=9`, `SPT_OTHER=10`); interface `SPSyntaxParser.h:32-36`.
- `doSyntaxHighlightingWithForce:` (`SPTextView.m:2866-3114`): skip bila highlighting off; skip bila `strlength > SP_MAX_TEXT_SIZE_FOR_SYNTAX_HIGHLIGHTING` (2.000.000, `77-78,2887`); teks `> SP_TEXT_SIZE_TRIGGER_FOR_PARTLY_PARSING` (10.000, `.h:31`) hanya highlight ±`SP_SYNTAX_HILITE_BIAS` (1500) di sekitar viewport (`2899-2989`), else seluruh buffer (`2993`).
- Token→warna (`3019-3050`): quote→`quoteColor`, reserved→`keywordColor`, numeric→`numericColor`, backtick→`backtickColor`, comment→`commentColor`, variable→`variableColor`, else `otherTextColor`.
- Atribut: `NSForegroundColorAttributeName`; `kLEXToken` (`3088-3093`); `kSQLkeyword` (auto-uppercase, `3097-3098`); `kQuote` (`3099-3101`).
- Auto-uppercase keyword (`3060-3083`) satu token kemudian bila `autouppercaseKeywordsEnabled && textBufferSizeIncreased && !delBackwardsWasPressed`.
- Trigger: `textStorage:didProcessEditing:` → `doSyntaxHighlightingWithForce:NO` (`3492-3553`); KVO pref warna/font → `doSyntaxHighlightingWithForceWrapper:` (`260-334, 336-356`).

### 4.2 Autocompletion

- **Sumber word-list** — `suggestionsForSQLCompletionWith:...` (`366-677`): semua kata di text storage (prefix-match, hanya bila < 10.000 char) (`379-394`); keyword `SPQueryController keywordList` (`398-400`) dan `functionList` (`403-405`); struktur DB dari `SPNavigatorController dbStructureForConnection:` (`421`) — database, tabel (`table-small-square`), view, proc, func, field + tipe; DB/tabel aktif/`%used-in-query` diprioritaskan (`439-551`); fallback `SPTablesList` (`614-649`). List eksplisit `"$SP_ASLIST_ALL_TABLES"`, `"$SP_ASLIST_ALL_DATABASES"`, `"$SP_ASLIST_ALL_FIELDS"` (`1579-1731`), menu tag 8000/8001/8002 (`SPCustomQuery.m:3192-3206`; `DBView.xib:1477-1494`: `"All Databases Names" ⌥⌘1`, `"All Table and View Names" ⌥⌘2`, `"All Field Names From Current Table" ⌥⌘3`). Kind invalid → `NSLog(@"“%@” is not a valid completion list")` + beep (`1701-1704`).
- **Trigger**: auto `doAutoCompletion` setelah `CustomQueryAutoCompleteDelay` (`3487-3489`); `doAutoCompletion` (`679-721`) menekan pada set `"'`;,()[]{}=+/<> \t\n\r"`, pilih mode spell-checker bila di dalam quote (`kQuote`), else fuzzy per `CustomQueryAutoCompleteFuzzy`. Manual: ESC (`2332-2347`), F5 mode spell-checker (`2348-2352`), menu "Completion List" (`SPCustomQuery.m:484-489`, ⌃ = fuzzy).
- **Window** `SPNarrowDownCompletion : NSWindow` (`SPNarrowDownCompletion.h:36`); layout `SACompletionWindowLayout.swift:16-54`; event loop `watchUserEvents` (`SPNarrowDownCompletion.m:804-940`): Return/Enter/→/Tab → `completeAndInsertSnippet`; ESC tutup; backspace/ketik re-filter.

### 4.3 Bracket matching

- `SPBrackets` (`SPBrackets.h:33-38`): pasangan `{}`, `()`, `[]` (`SPBrackets.m:42-55`); `findMatchingBracketAtPosition:inString:` (`67-79`) → `nextMatchingClosingBracket:` (`103-128`)/`nextMatchingOpeningBracket:` (`130-155`), sadar komentar via `isInComment:` (`81-101`).
- `SPBracketHighlighter`: `highlightColor = systemTealColor`, `enabled=YES`; `bracketHighlight:inRange:` (`56-84`); `highlightOff` (`90-97`).
- Wiring: saat seleksi berubah, `currentQueryRange.length < 1000` ⇒ `bracketHighlight:caretPosition-1 inRange:currentQueryRange` (`SPCustomQuery.m:3134-3136`); dibersihkan via `highlightOff` (`3091-3094`).

### 4.4 Snippet / placeholder

- Entry: TAB dengan tab trigger terdefinisi menyisipkan favorite via `insertAsSnippet:atRange:` (`SPTextView.m:2356-2408`).
- `insertAsSnippet:atRange:` (`1897-2185`): grammar placeholder `${n:…}` (regex `(?s)(?<!\\)\$\{(1?\d):…\}` `1920`) dan mirror `${n}` (`1921`); `${x:…}`→`${x:\`…\`}` (`1933`); mendukung 0…18 (`1952-1956`) → pesan `"Only snippets in the range of 0…18 allowed."` (`1953`); makro `$SP_SELECTED_TABLES`, `$SP_SELECTED_TABLE`, `$SP_SELECTED_DATABASE` (fallback `<table>`, `<database>`, `1965-2008`) dan `$(bash command)` via `SPBundleCommandRunner runBashCommand:` (`2014-2025`).
- Sesi snippet/mirror: `SnippetControlInfo`/`MirrorControlInfo` (`SPTextView.h:42-52`); `selectCurrentSnippet` (`1806`), `processMirroredSnippets` (`1736-1801`), `endSnippetSession` (`1567-1574`), `checkForCaretInsideSnippet` (`2186-2251`); TAB/⇧TAB navigasi (`2361-2399`).
- Rendering: snippet di-highlight di `drawViewBackgroundInRect:` (`3186-3208`; aktif orange, lain amber) via `roundedBezierPathAroundRange:` (`3214-3277`). Highlight query aktif pakai `queryHiliteColor` bila `shouldHiliteQuery` (`3171-3184`).

### 4.5 Perilaku editor lain

- **Auto-pairing** `keyDown:` (`2440-2574`): pasangan `(` `"` `` ` `` `'` `{`; wrap seleksi via `wrapSelectionWithPrefix:suffix:` (`2547`); toggle `CustomQueryAutoPairCharacters`.
- **Auto-indent**: `doCommandBySelector:` (`2681-2723`); ←Delete hapus indent lunak (`2726-2734`); toggle `CustomQueryAutoIndent`.
- **Soft indent/shift**: `shiftSelectionRight`/`Left` (`1369,1444`); menu `"Shift Right"` ⌘] / `"Shift Left"` ⌘[ (`DBView.xib:1443-1452`).
- **Komentar**: `commentOut` (`1478-1537`) toggle `-- ` per baris; `commentOutCurrentQueryTakingSelection:` (`1361-1397`, `/* */`) dan `...WithDashes:` (`1405-1471`, `-- `, marker `SPDashStyleCommentMarker = "-- "` `1400`); pref `UseDashStyleForBlockComment`. Menu `"Comment Line"` ⌘/ dan `"Comment Current Query"` ⌥⌘/ (`DBView.xib:1453-1463`).
- **Lain**: ⌘+/⌘=/⌘- ukuran teks, ⌘0 reset (`2420-2438`); autohelp timer (`2296-2302, 2271-2287`); gear menu (`419-532`): history prev/next (⌃↑/⌃↓), Shift Right/Left, Comment, Completion List, Editor Font…, Indent New Lines, Auto-pair, Auto-uppercase, Auto-completion, Update Help while typing (`DBView.xib:1417-1538`).

## 5. String yang terlihat pengguna (terpilih, dikutip persis)

**Eksekusi/status**: `"Running query %i of %lu..."`/`"Running query..."` (`612,615`); `"Query Status"` (`619`); `"Stop queries"`/`"Stop query"` (`787`); `"Query cancelled."` (`887`); `"Errors"`/`"No errors"` (`1022`); `"%@; Cancelled in query %ld, after %@"` (`1025`); `"%@; Cancelled after %@"` (`1030`); `"%@; %ld rows affected"` (`1052`); `", first row available after %1$@"` (`1060`); `"[ERROR in query %ld] %@\n"` (`904,948`); `"Execution stopped!\n"` (`941`); `"(This usually indicates that the connection has been closed by the server after inactivity, …)"` (`894`); `"MySQL Error"`/`"Run All"`/`"Continue"`/`"Stop"` (`921-927`); `"There were no errors."` (`1628`); `"Execute SQL?"`/`"Do you really want to proceed with this query?\n\n %@"`/`"Proceed"` (`714-726`).

**EXPLAIN**: `"EXPLAIN is only supported for a single SELECT or WITH statement."` (`+Explain.swift:112`).

**Menu (DBView.xib)**: `"Run Current"`, `"Run Current Query"` (⌘R), `"Run All Queries"` (⌥⌘R), `"Switch Default"`, `"Explain Current Query"` (⌥⌘E) (`1640-1664`); `"Show Completion List"`, `"All Databases Names"`, `"All Table and View Names"`, `"All Field Names From Current Table"` (`1473-1494`); `"Shift Right"`/`"Shift Left"` (`1443-1452`); `"Comment Line"`/`"Comment Current Query"` (`1453-1463`).

**Editor/highlight/snippet**: `"Applying syntax highlighting..."` (`350`); `"Only snippets in the range of 0…18 allowed."` (`1953`); `"MySQL Help for Selection"` (`3289`); `"“%@” is not a valid completion list"` (`1702`).

**Console.xib**: judul `"Console"` (`:26`); placeholder `"Filter"` (`:40`); header `"Time"`, `"Connection"`, `"Database"`, `"Query"` (`:68,80,92,104`); gear `"Show Time Stamps"`, `"Show Connections"`, `"Show Databases"`, `"Show SELECT/SHOW Statements"`, `"Show HELP Statements"` (`:161-206`); `"Clear Console"` tooltip (`:140`); `"Save as..."` (`:230`); context menu `"Copy"`, `"Copy Raw Query"` (`:278,284`).

## 6. UNKNOWN

- `SPCustomQuery.m:957` komentar "queriesSeparatedByDelimiter" + TODO — apakah batch multi-`DELIMITER` dimaksudkan jalan sebagai satu, belum jelas.
- Apakah split menghormati sesi `NO_BACKSLASH_ESCAPES`: `SPSQLParser` mendukung tapi `SPCustomQuery` selalu default; hanya classifier menangani kedua bacaan.
- Grammar `SPSQLTokenizer.l` (aturan regex) tidak dibaca.
