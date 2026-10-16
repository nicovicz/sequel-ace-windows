# Audit: MCP Swift cluster

Audit method-by-method tiga file klaster MCP. `Start`/`End` = nomor baris di file
masing-masing. Baseline commit `02c910932`.

File: `Source/Controllers/MCP/SPAppController+MCP.swift` (963 baris),
`SPMCPServer.swift` (893), `SPMCPHTTP.swift` (552).

---

## File 1 — `SPAppController+MCP.swift`

- Peran: implementasi `SPMCPDataSource` milik `SPAppController` — resolve tab
  Sequel Ace ke koneksi MySQL live dan lakukan semua kerja database di balik tiap
  tool MCP.

### Tipe
`MCPDesiredState` (private final class singleton cache pref, 44–49);
`MCPResolvedConnection` (private struct connection + metadata, 131–136);
`extension SPAppController: SPMCPDataSource` (138–963).
File-scope: `mcpDefaultPort` (34), `mcpMaxResultRows` (35), `mcpLog` (37), `mcpDBQueue` (40), `mcpDocIDKey` (53).

### `// MARK:`
`140 (Lifecycle), 210 (Connection resolution), 256 (SPMCPDataSource: connections), 285 (favorite metadata), 329 (schema), 512 (queries), 874 (diagnostics), 933 (CSV helpers)`.

### Tabel definisi (urut baris)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 34–53 | | `mcpDefaultPort`/`mcpMaxResultRows`/`mcpLog`/`mcpDBQueue`/`mcpDocIDKey` | Konstanta lintas-fungsi |
| 44–49 | | `MCPDesiredState` (+ `shared`/`known`/`enabled`/`port`) | Cache pref terakhir |
| 55 | 63 | `mcpDocumentID(_:)` | UUID stabil per dokumen (associated object) |
| 65 | 69 | `mcpClampedPort(_:)` | Baca pref port, clamp 1024–65535 else default |
| 72 | 74 | `mcpQuoteIdentifier(_:)` | Gandakan backtick untuk escape identifier |
| 79 | 87 | `mcpEscapeQuoted(_:_:)` | Literal ter-quote (koneksi), fallback manual |
| 91 | 93 | `mcpPathComponents(_:)` | Pecah path ke komponen non-kosong |
| 99 | 117 | `mcpOpenExportFile(base:relativeDirs:filename:)` | Telusuri rantai dir export O_NOFOLLOW, buat dir, buka file |
| 123 | 128 | `mcpDecode(_:)` | Decode NSData UTF-8/base64, else passthrough |
| 131–136 | | `MCPResolvedConnection` (+ `conn`/`id`/`database`/`host`) | Struct koneksi live + metadata |
| 138 | 963 | `extension SPAppController: SPMCPDataSource` | Semua implementasi method data-source |
| 143 | 160 | `setupMCPServer()` | Daftar observer pref, seed state, start server bila enabled |
| 163 | 187 | `mcpDefaultsChanged(_:)` | Reaksi perubahan pref, start/stop server saat berubah |
| 189 | 208 | `startMCPServer(with:)` | Start server pada port ter-clamp, alert saat gagal |
| 214 | 243 | `mcpResolveConnection(_:)` | Resolve conn id atau front tab → koneksi live di main thread |
| 245 | 247 | `mcpNoConnectionError()` | Dict error standar "no matching connection" |
| 250 | 254 | `mcpDBSync(_:)` | Jalankan blok di serial DB queue sinkron |
| 258 | 283 | `mcpListConnections()` | Enumerasi tab terbuka (id/name/host/database/active) |
| 289 | 315 | `mcpFavoriteInfo(for:)` | Resolve nama favorite + group path |
| 318 | 327 | `mcpFavoriteNode(withID:in:)` | Cari leaf node favorite by keychain id |
| 331 | 344 | `mcpListDatabases(onConnection:)` | `SHOW DATABASES` → nama + connection id |
| 346 | 363 | `mcpListTables(inDatabase:connection:)` | `SHOW FULL TABLES` (nama + tipe table/view) |
| 365 | 414 | `mcpDescribeTable(_:inDatabase:connection:)` | Kolom, index, FK satu tabel |
| 416 | 434 | `mcpTableDDL(_:inDatabase:connection:)` | `SHOW CREATE TABLE`, decode kolom DDL NSData |
| 437 | 468 | `mcpListRoutines(ofType:inDatabase:connection:)` | Daftar view/procedure/function/trigger/event via information_schema |
| 470 | 510 | `mcpRoutineDefinition(ofType:name:inDatabase:connection:)` | `SHOW CREATE` routine; trigger dibangun ulang dari information_schema |
| 514 | 595 | `mcpRunQuery(_:params:limit:offset:connection:)` | Bind param, paksa read-only, cap/paginasi, eksekusi |
| 602 | 604 | `mcpBindParams(_:intoSQL:connection:)` | Delegasi binding placeholder ke read-only guard |
| 609 | 614 | `mcpSQLLiteral(for:connection:)` | Render param sebagai NULL, number, atau literal ter-escape |
| 616 | 626 | `mcpKillProcessID(_:connection:)` | Validasi pid positif + `KILL` |
| 633 | 710 | `mcpExecuteResultQuery(_:onConnection:connectionID:maxRows:offset:streamResult:)` | Baca baris terbatas, dedup nama kolom, tandai truncation |
| 712 | 727 | `mcpExplainQuery(_:connection:)` | Tolak EXPLAIN ANALYZE, else jalankan EXPLAIN |
| 729 | 741 | `mcpSampleTable(_:inDatabase:limit:offset:connection:)` | `SELECT *` limit clamp 1–1000 |
| 743 | 758 | `mcpCountRows(inTable:inDatabase:connection:)` | `SELECT COUNT(*)` → Int64 |
| 760 | 872 | `mcpExportResults(_:format:path:connection:)` | Confine path ke folder export, tulis JSON/CSV mode 0600 |
| 876 | 895 | `mcpServerInfo(onConnection:)` | Subset `SHOW VARIABLES` + database + host |
| 897 | 915 | `mcpTableSizes(inDatabase:connection:)` | Estimasi baris + byte data/index per tabel |
| 917 | 931 | `mcpProcessList(onConnection:)` | `SHOW FULL PROCESSLIST` sebagai array dict |
| 942 | 962 | `csvString(fromColumns:rows:)` | Bangun teks CSV (header + field ter-escape) |

### Temuan / UNKNOWN
- `mcpResolveConnection` baca state AppKit → hop main `DispatchQueue.main.sync` bila belum di main (241); conn id kosong = "front tab", fallback scan tab terhubung pertama (224–234). Kerja DB serial di `mcpDBQueue.sync` via `mcpDBSync` (250–254). Label queue `com.sequel-ace.mcp.db` (40) **identik** dengan `SPMCPServer.dbQueue` (`SPMCPServer.swift:212`) tapi dua queue berbeda; nested dispatch berarti server queue lalu blok di app queue — serialisasi tetap, duplikasi tampak tak sengaja (**UNKNOWN**).
- **Read-only re-validasi hanya saat ada param**: `if !params.isEmpty { … SPMCPReadOnlyGuard.isReadOnly(bound) }` (526–537). SQL tanpa param bergantung penuh pada cek dispatcher (`SPMCPServer.callTool` 597, 636).
- **Row cap** hanya diterapkan ke SQL comment-stripped yang teksnya sepakat di kedua bacaan backslash (556) dan hanya bila statement diawali `SELECT`/`(` (560) — `WITH` sengaja tak pernah di-cap (544–547). Else SQL asli dieksekusi + paginasi read-side (583–589). Cap = `mcpMaxResultRows = 10000` (35).
- **Confinement export**: resolve symlink base + parent (779, 797–798), tolak path di luar via `inside` (799–803). Write menelusuri tiap komponen dir `O_RDONLY|O_DIRECTORY|O_NOFOLLOW` (99–117), file `O_WRONLY|O_CREAT|O_TRUNC|O_NOFOLLOW` mode 0600 (112), re-`fchmod` 0600 (847). Bookmark security-scoped untuk folder kustom (786–791).
- `mcpKillProcessID` parse pid `Int64` + wajib `> 0` sebelum `KILL \(pid)` (617–622) — nilai ter-interpolasi numerik.
- **UNKNOWN**: `SAMCPToolDefinitions.all()` (dipakai `SPMCPServer.swift:463`) dan `SPMCPFavorite.pathString/idString` di luar scope.

---

## File 2 — `SPMCPServer.swift`

- Peran: HTTP server loopback-only `NWListener` yang mengimplementasi MCP via
  JSON-RPC pada dua transport, plus dispatch tool, resources, completions, prompt.

### Tipe
`SPMCPDataSource` (`@objc public protocol`, 36–88); `SPMCPServer` (`@objc public final class`, 101–220);
`.SSEClient` (private struct, 214–217); lima `private extension` (connection 224–302, SSE 306–374, message 378–440, dispatch 444–544, tool 548–830, HTTP helper 834–893).

### `// MARK:`
`32, 90, 103, 108, 202, 222, 304, 376, 442, 546, 657, 701, 741, 832`.

### Tabel definisi (urut baris; protocol req dirangkum)
Protocol `SPMCPDataSource` (36–88): 16 requirement `mcpListConnections` (38)…`mcpProcessList` (87).

| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 105–106 | | `SPMCPServer.shared` / `init()` | Singleton + init privat |
| 110 | 110 | `dataSource` | Referensi weak ke data source |
| 113 | 117 | `isRunning` | Apakah listener terpasang |
| 120 | 167 | `start(port:completion:)` | Stop listener lama lalu bind listener loopback baru |
| 170 | 172 | `stop()` | Stop convenience tanpa completion |
| 176 | 200 | `stop(completion:)` | Cancel listener + semua SSE client, callback |
| 206–219 | | `maxRequestBytes`/`listener`/`listenerLock`/`serverQueue`/`dbQueue`/`SSEClient`/`sseClients`/`clientsLock` | Konstanta + state |
| 227 | 230 | `accept(connection:)` | Start koneksi + mulai baca request |
| 233 | 265 | `receiveRequest(on:buffer:)` | Akumulasi byte, cap ukuran, dispatch request terparse |
| 268 | 301 | `handle(request:on:)` | Cek loopback + Origin, rutekan ke endpoint |
| 309 | 347 | `handleSSE(request:connection:)` | Buka stream SSE, registrasi client, iklankan message URL |
| 349 | 354 | `listeningPort` | Port ter-bind aktual, fallback pref |
| 357–373 | | `sendSSEEvent`/`sendSSEMessage`/`send(text:on:completion:)` | Pengiriman SSE/teks |
| 382 | 402 | `handleStreamableHTTP(request:connection:)` | Transport POST /mcp; balas JSON, 202 untuk notification |
| 405 | 427 | `handleMessage(request:connection:)` | Legacy POST /message; 202 lalu respons via SSE |
| 430 | 439 | `sendToSSEClient(sessionID:message:)` | Kirim pesan ke client pemilik session id |
| 447 | 496 | `dispatch(jsonRPC:)` | Rutekan method JSON-RPC ke handler |
| 499 | 499 | `supportedProtocolVersions` | Versi MCP diiklankan, terbaru dulu |
| 502 | 526 | `initializeResult(protocolVersion:)` | Bangun hasil initialize + capabilities + instructions |
| 529 | 543 | `jsonRPCSuccess`/`jsonRPCError` | Bungkus envelope JSON-RPC |
| 551 | 655 | `callTool(name:arguments:)` | Dispatch tool bernama dengan cek argumen |
| 659 | 661 | `uriEncode(_:)` | Percent-encode komponen URI resource |
| 664 | 684 | `resourceList()` | Tabel koneksi aktif sebagai resource MCP |
| 687 | 699 | `resourceRead(uri:)` | Parse URI `sequelace://` → schema JSON tabel |
| 703 | 733 | `completion(params:)` | Saran nilai untuk connection/type/database/table |
| 736 | 739 | `completionResult(_:)` | Cap 100 nilai + flag hasMore |
| 744–786 | | `promptDefinitions()` / `promptGet(params:)` | Tiga template prompt + bangun pesan user |
| 790–829 | | `readOnlyRejection`/`dictResult`/`toolResult`/`toolError`/`jsonString`/`defaultExportPath` | Helper hasil tool |
| 837 | 872 | `sendHTTPResponse(connection:status:body:extraHeaders:keepAlive:)` | Tulis respons HTTP teks; cancel kecuali keep-alive |
| 875 | 892 | `sendJSONResponse(connection:jsonData:sessionID:)` | Tulis respons application/json echo session id |

### Temuan / UNKNOWN
- **Transport**: Streamable HTTP `POST /mcp` (MCP 2025-03-26) + legacy HTTP+SSE `GET /sse`/`POST /message` (2024-11-05) — didokumentasikan 97–99, dirutekan 285–300. Bind loopback-only `hostPort("127.0.0.1", …)` (133–135). Pertahanan berlapis tiap request: cek endpoint (270–276) + cek Origin DNS-rebinding (280–283, `SPMCPHTTP.isLoopbackOrigin`). **Catatan**: guard endpoint di dalam `if case let .hostPort(host, _) = connection.endpoint` — endpoint non-`hostPort` melewati cek sepenuhnya (270–276); **UNKNOWN** apakah reachable pada listener TCP.
- **Dispatch**: `dispatch` (447–496) tangani `initialize`, `tools/list`, `tools/call`, `resources/list`, `resources/read`, `completion/complete`, `prompts/list`, `prompts/get`, `ping`, dan **menelan** envelope tanpa id atau prefix `notifications/` dengan mengembalikan dict kosong (455, 490–491) — dict kosong = sinyal "no response" (394, 423). Method tak dikenal → `-32601` (494). `tools/list` dari `SAMCPToolDefinitions.all()` (463) — **UNKNOWN** apakah cocok dengan switch nama tool di `callTool` (560–654). **Tak ada** tool `list_events` meski data source mendukung `type == "event"` (`SPAppController+MCP.swift:453–454`).
- **Read-only guard** ditegakkan di 3 titik: `run_query` (597), `export_results` (636) via `readOnlyRejection` (790–796), `kill_query` saat `SPMCPReadOnly` on (622–624). `explain_query` blok `EXPLAIN ANALYZE` (608–610), guard di-reapply di `mcpExplainQuery` (`SPAppController+MCP.swift:719–721`). Guard = `SPMCPReadOnlyGuard` (`SPMCPHTTP.swift:56–343`).
- **Session**: Streamable HTTP stateless — session id dari `mcp-session-id` atau dibuat baru (389) + di-echo (882) tanpa validasi vs `sseClients`. Legacy `/message` butuh param query `sessionId` (406–409) + lookup di `sseClients` (430–433).
- **Concurrency/lifecycle**: `start` teardown listener lama via `stop { [self] in … }` (125) + handler capture weak yang hidup melewati closure one-shot (145–159). `stop` bersihkan `sseClients` + cancel semua client sebelum cancel listener (182–199). Buffer request cap `maxRequestBytes` 16 MiB dengan 413 (206, 246–249); dispatch di `dbQueue` (391, 419).
- **UNKNOWN**: `SPMCPFavorite` (`SPMCPFavorite.swift:37, 43`).

---

## File 3 — `SPMCPHTTP.swift`

- Peran: parsing HTTP tanpa dependensi app, routing, allow-list Origin loopback,
  guard SQL read-only, sanitasi JSON, escaping CSV, dan paginasi hasil — semua
  unit-testable terisolasi.

### Tipe
`SPMCPHTTP` (enum namespace, 12–51, dengan `Route` 25–32);
`SPMCPReadOnlyGuard` (enum namespace, boundary keamanan, 56–343);
`SPMCPJSON` (enum, 348–384); `HTTPRequest` (struct, 386–455);
`SAMCPCSV` (enum, 458–495); `SAMCPResultPage` (enum, 500–551).

### `// MARK:`
**Tidak ada** di seluruh file ini (dan tak ada di kelima file MCP Swift).

### Tabel definisi (urut baris)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 15 | 22 | `SPMCPHTTP.isLoopbackOrigin(_:)` | True bila Origin host 127.0.0.1/localhost/::1 |
| 25–32 | | `SPMCPHTTP.Route` | Enum endpoint |
| 35 | 50 | `SPMCPHTTP.route(method:path:)` | Mapping murni method+path → route |
| 59 | 72 | `SPMCPReadOnlyGuard.isReadOnly(_:)` | Tolak executable comment, lalu wajib kedua bacaan backslash lolos |
| 74 | 103 | `SPMCPReadOnlyGuard.isReadOnly(_:backslashEscapes:)` | Strip comment, tolak statement bertumpuk + akses file |
| 111 | 117 | `SPMCPReadOnlyGuard.explainWouldExecute(_:)` | True bila EXPLAIN akan mengeksekusi target |
| 136 | 149 | `SPMCPReadOnlyGuard.explainWouldExecute(_:backslashEscapes:)` | Scan token untuk kata ANALYZE tak ter-quote |
| 158 | 161 | `SPMCPReadOnlyGuard.isUnquotedIdentifierScalar(_:)` | Apakah scalar melanjutkan identifier unquoted |
| 167 | 170 | `SPMCPReadOnlyGuard.hasExecutableComment(_:)` | Deteksi comment MySQL `/*!`/MariaDB `/*M!` |
| 185 | 239 | `SPMCPReadOnlyGuard.stripCommentsQuoteAware(_:backslashEscapes:)` | Buang comment SQL, pertahankan string/identifier |
| 246 | 252 | `SPMCPReadOnlyGuard.sqlForResultLimiting(_:)` | SQL comment-stripped hanya bila kedua bacaan sepakat |
| 269 | 286 | `SPMCPReadOnlyGuard.bindPlaceholders(in:params:literal:)` | Substitusi `?` tak ter-quote dengan literal, cek jumlah |
| 296 | 342 | `SPMCPReadOnlyGuard.placeholderOffsets(in:backslashEscapes:)` | Offset scalar `?` di luar quote/comment |
| 352 | 359 | `SPMCPJSON.string(from:)` | Sanitasi + pretty-print nilai ke teks JSON |
| 364 | 383 | `SPMCPJSON.sanitize(_:)` | Konversi rekursif nilai ke tipe JSON-safe |
| 386–455 | | `HTTPRequest` (+ `method`/`path`/`headers`/`body`, `init?(data:)` 394–440, `queryString` 442, `queryParam(_:)` 445–454) | Tipe nilai request HTTP/1.1 ter-parse |
| 471 | 484 | `SAMCPCSV.escapedField(_:)` | Guard formula-injection + quoting gaya RFC |
| 492 | 494 | `SAMCPCSV.isPlainNumber(_:)` | Apakah field angka desimal ASCII polos |
| 501–509 | | `SAMCPResultPage.rowLimit`/`trailingLimitPattern`/`hasTrailingLimit` | Clamp + deteksi LIMIT |
| 514 | 533 | `SAMCPResultPage.sqlPageWithinTrailingLimit(_:requested:offset:cap:)` | Rewrite LIMIT jadi page tool + lookahead |
| 537 | 551 | `SAMCPResultPage.consumeRows(maxRows:offset:nextRow:appendRow:)` | Skip offset, kumpulkan page, konsumsi satu baris lookahead |

### Temuan / UNKNOWN
- **Parsing HTTP**: `HTTPRequest.init?(data:)` sengaja permisif + streaming-friendly: return nil (bukan throw) sampai terminator `\r\n\r\n` + body penuh tiba (395–397, 434–439), menggerakkan loop re-arm `SPMCPServer.receiveRequest` (251–262). Body dibingkai hanya `content-length` (431); `Transfer-Encoding: chunked` → `body = nil` — **UNKNOWN** apakah chunked sengaja tak didukung. `path` dipecah pada `?` tapi tak percent-decoded (410), query param di-decoded (450). `queryString` `private let` dideklarasikan **setelah** initializer yang menugaskannya (442).
- **Routing**: `SPMCPHTTP.route` murni (35–50) → `.streamableHTTP`/`.sse`/`.message`/`.health`/`.methodNotAllowed`/`.notFound`. `methodNotAllowed` menghasilkan 405 + header `Allow: POST` (`SPMCPServer.swift:294–297`).
- **Read-only guard (boundary keamanan)**: executable comment ditolak **sebelum** strip (62, 163–170); statement dinilai di **kedua** bacaan backslash, lolos hanya bila keduanya lulus (64–71, 113–116); strip comment quote-aware agar `#`/`--` dalam literal tak menyembunyikan `OUTFILE`/`LOAD_FILE`/`;` (75–80, 172–184), bekerja pada Unicode scalar (178–180), comment diganti satu spasi agar token tak menyatu (207–213); penolakan: core kosong (86), sisa `;` (90), `OUTFILE`/`DUMPFILE`/`LOAD_FILE` (96), plus `SPCustomQuerySQLClassifier.isQuerySafeWithoutDestructiveWarning` (102). `explainWouldExecute` menganggap ANALYZE unquoted apa pun mengeksekusi (136–149); dot dipertahankan sebagai identifier scalar (151–161). Binding menolak query yang posisi `?` berbeda antar bacaan backslash (272–274) + cek jumlah tepat (275–276).
- **UNKNOWN/out-of-file**: `SASQLCommentSyntax.endsLineComment`/`isCommentWhitespace` (`SADatabaseAssertion.swift:19–45`); `SPCustomQuerySQLClassifier.isQuerySafeWithoutDestructiveWarning` (`:51, 82`) dan `sqlTokens(from:backslashEscapes:)` (`:358`).
- **JSON/CSV hygiene**: `SPMCPJSON.sanitize` ada karena `JSONSerialization` raise exception ObjC tak tertangkap pada nilai MySQL non-JSON (345–347): Data→UTF-8/base64, Date→ISO-8601, lainnya description (374–382). `SAMCPCSV.escapedField` prefix sel `= + - @ \t \r` dengan tanda kutip tunggal untuk defuse formula injection spreadsheet (459–477); quoting pakai opsi `.literal` (478–482).
- **Paginasi**: `SAMCPResultPage` untuk hasil yang harus dikonsumsi tanpa rewrite SQL (498–499). `sqlPageWithinTrailingLimit` return nil ketimbang mengubah semantik diam-diam saat integer overflow `Int` atau LIMIT bukan klausa trailing sederhana (511–527); ambil tepat satu baris lookahead untuk deteksi truncation (530). `consumeRows` generik (535–551).

### Ringkasan lintas-file
| Konsern | Situs utama |
| --- | --- |
| Transport (NWListener, bind loopback, routing) | `SPMCPServer.swift:120–167, 268–301, 309–347, 382–427` |
| JSON-RPC dispatch | `SPMCPServer.swift:447–496, 529–543` |
| Read-only guard | `SPMCPHTTP.swift:56–343`; ditegakkan `SPMCPServer.swift:597, 622, 636, 790–796`; re-cek `SPAppController+MCP.swift:534, 719` |
| Confinement export path | `SPAppController+MCP.swift:760–872, 99–117` |
| Data-source protocol | `SPMCPServer.swift:36–88`; implementasi `SPAppController+MCP.swift:138–963` |
