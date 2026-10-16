# Audit: `SADatabaseRenamePlan.swift`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Other/DatabaseActions/SADatabaseRenamePlan.swift`. Baseline commit `02c910932`.

- File: 2596 baris.
- Peran: seluruh fitur "Rename Database" dalam satu file — *plan* yang menolak
  rename bila move per-objek MySQL (create target → `RENAME TABLE` → recreate view
  → `DROP DATABASE`) akan menghilangkan/merusak sesuatu, plus rewriter definisi
  view, antriaan/urutan recreate view, eksekutor yang mengurutkan tiap statement
  lewat satu closure, dan sesi koneksi yang memindah koneksi ke UTF-8 lalu
  merestorenya.
- **Tidak ada `// MARK:`** di seluruh 2596 baris (temuan navigasi).

## 1. Tipe & konstanta

| Tipe | Jenis | Baris |
| --- | --- | --- |
| `SADatabaseRenameObjectKind` | `@objc public enum : Int` | 12–15 |
| `SADatabaseRenamePlan` | `@objc public final class` | 28–353 |
| `SAViewColumnSignature` | `@objc public final class` | 368–490 |
| `SADatabaseRenameViewRewrite` | `@objc public final class` | 492–516 |
| `SADatabaseRenameViewRewriter` | `@objc public final class` | 534–1040 |
| `.Context` / `.Token` | `private enum` | 536–539 / 541–549 |
| `SADatabaseRenameViewQueue` | `@objc public final class` | 1049–1101 |
| `SADatabaseRenameStatementResult` | `@objc public final class` | 1104–1151 |
| `SADatabaseRenameExecutor` | `@objc public final class` | 1157–2327 |
| `.GlobalVisibility` (+`.PartialRevokes`) | `private struct`/`enum` | 1896–1905 |
| `.ViewDefinition` / `.SessionSettings` | `private struct` | 2060–2064 / 2131–2150 |
| `SADatabaseRenameConnectionSession` (+`.OriginalSettings`) | `@objc public final class`/`struct` | 2353–2596 |

Konstanta: `expressionKeywords` (713–715), `nonFunctionKeywords` (719–723),
`selectOptionPredecessors` (727), `sequenceFunctions` (743), `parsingSQLModes` (2198–2201),
`unknownReason` (335–337), `quoteFailureReason` (2109–2111); typealias `Run`/`Quote` (1159–1160),
`Encoding`/`UsesLatin1Transport`/`SetEncoding`/`SetLatin1Transport`/`ConnectionAction`/`Reconnect` (2355–2360).

## 2. Tabel method (urut baris, per tipe)

### `SADatabaseRenamePlan` (28–353)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 30–31 | | `sourceDatabase`/`targetDatabase` | Nama DB sumber/target untuk pesan |
| 34/36/38 | | `tables`/`views`/`unsupportedObjects` | Tabel untuk `RENAME TABLE` / view untuk recreate / objek penghalang |
| 41/43–54 | | `caseInsensitiveNames`, `inspectionFailureReason`, `unsupportedCharacterSet`, `unsupportedDefinitionView`, `objectPrivileges`, `targetPrivileges`, `externalViews`, `createFailureReason`, `failedObject`, `failedObjectReason`, `dropFailureReason`, `viewSignatureMismatch`, `unreadableViewSignatures` | Slot penolakan/keadaan |
| 73 | 96 | `init(sourceDatabase:targetDatabase:lowerCaseTableNames:tableRows:routineRows:eventRows:triggerRows:)` | Pisah baris ke tabel/view + objek penghalang |
| 100 | 105 | `stringRows(_:columns:)` | Pertahankan baris yang N nilai pertamanya string |
| 112 | 114 | `recordInspectionFailure(_:)` | Catat inspeksi sumber gagal |
| 124 | 126 | `recordUnsupportedCharacterSet(_:ofView:)` | Catat view dibuat via charset lain |
| 133 | 135 | `recordUnsupportedDefinition(ofView:)` | Catat definisi view di luar UTF-8 |
| 144 | 146 | `recordObjectPrivileges(_:)` | Catat grant pada objek sumber |
| 158 | 160 | `recordTargetPrivileges(_:)` | Catat grant yang sudah ada di nama target |
| 168 | 170 | `recordExternalViews(_:)` | Catat view eksternal yang membaca sumber |
| 178 | 182 | `recordLibraries(_:)` | Append stored library ke objek penghalang |
| 187 | 189 | `canStart` | True bila inspeksi selesai + tak ada penghalang |
| 195 | 197 | `recordCreateFailure(_:)` | Catat `CREATE DATABASE` target gagal |
| 210 | 217 | `recordMove(of:kind:succeeded:reason:)` | Catat satu move; return apakah lanjut |
| 224 | 226 | `recordDropFailure(_:)` | Catat `DROP DATABASE` sumber gagal |
| 235 | 238 | `recordViewSignatureMismatch(forView:difference:)` | Catat beda signature kolom pertama |
| 245 | 248 | `recordUnreadableViewSignatures(_:)` | Catat signature kolom view tak terbaca |
| 253 | 256 | `mayDropSourceDatabase` | True bila semua berpindah + view masih cocok |
| 259 | 333 | `failureDescription` | Bangun pesan kegagalan berurut untuk user |
| 335 | 337 | `unknownReason` (static var) | Fallback "unknown error" terlokalisasi |
| 341 | 343 | `shortList(_:)` | Lima entri pertama join, elipsis selebihnya |
| 345 | 352 | `description(of:kind:)` | "table 'x'" / "view 'x'" untuk pesan |

### `SAViewColumnSignature` (368–490)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 371–381 | | `name`/`type`/`nullable`/`characterSet`/`collation` | Field signature |
| 391 | 398 | `init(name:type:isNullable:characterSet:collation:)` | Bangun dari satu baris COLUMNS |
| 401 | 410 | `signature` | Rendering satu baris untuk alert |
| 417 | 423 | `matches(_:)` | Banding kelima field |
| 438 | 458 | `signatures(fromRows:)` | Kelompok baris COLUMNS by byte nama view |
| 467 | 489 | `difference(between:after:)` | Kolom pertama ditambah/dihapus/berubah, else nil |

### `SADatabaseRenameViewRewrite` (492–516)
`statement` (495), `failureReason` (497), `referencedObjects` (503); `init(statement:referencedObjects:)` (505–509); `init(failureReason:)` (511–515).

### `SADatabaseRenameViewRewriter` (534–1040)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 536–549 | | `Context`/`Token` enum | Konteks dotted-name; leksem lexer |
| 551–554 | | `sourceDatabase`/`serverLoweredSource`/`quotedTarget`/`caseInsensitiveNames` | State |
| 569 | 574 | `init(sourceDatabase:targetDatabase:caseInsensitiveNames:serverLoweredSource:)` | Init designated |
| 578 | 580 | `init(sourceDatabase:targetDatabase:caseInsensitiveNames:)` | Convenience tanpa server-folded source |
| 589 | 710 | `rewriteCreateStatement(_:forView:)` | Rewrite token-level satu definisi |
| 713–743 | | `expressionKeywords`/`nonFunctionKeywords`/`selectOptionPredecessors`/`sequenceFunctions` | Set keyword |
| 732 | 738 | `isJoin(_:after:)` | Apakah keyword join dua referensi objek |
| 746 | 751 | `takesSequenceArgument(_:)` | Apakah `(` membuka call fungsi sequence |
| 756 | 766 | `opensFunctionCall(after:)` | Apakah `(` mengikuti nama fungsi |
| 772 | 782 | `isSourceDatabase(_:)` | Apakah nama backtick = DB sumber |
| 786 | 788 | `isSource(schemaName:)` | Idem untuk schema name tanpa quote |
| 798 | 829 | `definitionReferencesSource(_:)` | Apakah definisi menyebut objek sumber (fail closed) |
| 835 | 853 | `isNumericLiteral(_:)` | Digit/eksponen/hex/bit, bukan identifier |
| 861 | 863 | `asciiLowercased(_:)` | Case fold ASCII nama byte |
| 866 | 870 | `unquoted(_:)` | Lepas backtick, undo backtick ganda |
| 872 | 874 | `backtickQuoted(_:)` | Escape + backtick-quote nama |
| 885 | 917 | `hasLiteralOutsideUTF8(_:)` | Deteksi introducer gaya `_latin1'…'` |
| 926 | 1028 | `tokens(of:)` | Lexer atas Unicode scalar ke `Token` |
| 1032 | 1039 | `isWordScalar(_:)` | Huruf/digit ASCII/`_`/`$`/word non-ASCII |

### `SADatabaseRenameViewQueue` (1049–1101)
`pending` (1051), `failedThisPass` (1052), `createdThisPass` (1053), `position` (1054), `stuckView` (1058), `stuckReason` (1059);
`init(views:)` (1063–1065); `next` (1069–1084); `record(_:created:reason:)` (1093–1100).

### `SADatabaseRenameStatementResult` (1104–1151)
`rows` (1107), `error` (1109); `init(rows:)` (1113–1116); `init(error:)` (1120–1123);
`init(rows:resultReturned:errored:errorMessage:)` (1142–1150).

### `SADatabaseRenameExecutor` (1157–2327)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 1159–1191 | | typealias + `run`/`quote`/`changedServer`/`sessionSettingsNotRestored`/`init(run:quote:)` | Konstruksi |
| 1207 | 1561 | `rename(_:to:encoding:collation:)` | Seluruh alur rename; return teks kegagalan |
| 1580 | 1597 | `routineRowsForInspection(schema:caseInsensitiveNames:inspectionError:)` | Routine via `mysql.proc`, else `information_schema` ber-privilege |
| 1612 | 1632 | `libraryRowsForInspection(schema:inspectionError:)` | Objek `CREATE LIBRARY` MySQL 9.2+, diprobe dulu |
| 1655 | 1723 | `externalViewsReadingFromSource(schema:recognisingSourceWith:inspectionError:)` | View di DB lain yang membaca sumber |
| 1756 | 1854 | `privilegeDescriptions(forDatabase:serverLoweredName:caseInsensitiveNames:inspectionError:)` | Grant + partial revoke untuk satu database |
| 1857 | 1860 | `grantDescriptions(_:describe:)` | Deskripsi grant distinct, urutan dijaga |
| 1864 | 1866 | `databaseGrantDescription(pattern:grantee:)` | Teks `` `pattern`.* for 'u'@'h' `` |
| 1870 | 1872 | `objectGrantDescription(object:grantee:)` | Teks `` `table` for 'u'@'h' `` |
| 1876 | 1878 | `restrictionDescription(database:grantee:)` | Teks deskripsi partial-revoke |
| 1882 | 1891 | `likePattern(matchingExactly:)` | Escape `%`, `_`, `!` untuk `LIKE … ESCAPE '!'` |
| 1896–1907 | | `GlobalVisibility` (+ `PartialRevokes`) + `globalVisibility` | Cache privilege global |
| 1909 | 1923 | `globalVisibilityForInspection()` | Baca global SELECT/SHOW VIEW/SHOW_ROUTINE |
| 1925 | 1925 | `partialRevokes` | Cache setting |
| 1931 | 1949 | `partialRevokesForInspection()` | Baca `SHOW VARIABLES LIKE 'partial_revokes'` |
| 1955 | 1965 | `reasonGlobalSelectDoesNotCoverEverything(_:otherwise:)` | Fail closed kecuali global SELECT tak direvoke |
| 1973 | 1975 | `schemaMatch(column:schema:caseInsensitiveNames:)` | Match kolom nama binary `mysql` |
| 2012 | 2016 | `privilegeViewSchemaMatch(column:schema:caseInsensitiveNames:)` | Match view privilege `information_schema` |
| 2018 | 2020 | `inventorySchemaMatch(column:schema:caseInsensitiveNames:)` | Match binary saat server tak fold (TRIGGERS) |
| 2037 | 2056 | `databaseGrantMatch(column:schema:caseInsensitiveNames:literal:columnIgnoresCase:)` | Match pola `LIKE` grant DB vs sumber |
| 2060–2064 | | `ViewDefinition` struct | Statement ter-rewrite, referensi, collation |
| 2073 | 2105 | `orderedByReferences(_:definitions:caseInsensitiveNames:)` | Urutan topologis view by referensi |
| 2109 | 2111 | `quoteFailureReason` (static var) | "connection could not quote" |
| 2115 | 2126 | `text(_:)` | Nilai hasil → String |
| 2131 | 2150 | `SessionSettings` (+ `init?(row:)`) | sql_mode, collation, quoting, connection id |
| 2154 | 2159 | `settings(_:match:)` | Banding read-back nilai sesi yang direstore |
| 2163 | 2175 | `bytes(fromHex:)` | Hex string → bytes |
| 2177 | 2184 | `nibble(_:)` | Satu digit hex |
| 2188 | 2190 | `isUTF8(_:)` | `utf8`/`utf8mb3`/`utf8mb4` |
| 2211 | 2216 | `sqlMode(forViewDefinitions:)` | Buang mode `parsingSQLModes` dari sql_mode sesi |
| 2226 | 2236 | `viewColumnsStatement(schema:)` | Query COLUMNS⋈TABLES untuk kolom view |
| 2250 | 2282 | `compareViewColumns(before:targetLiteral:plan:)` | Banding view yang dibuat ulang vs kolom sumber |
| 2305 | 2326 | `recreate(_:definition:quotedTarget:session:)` | Set collation, jalankan CREATE, buka view untuk verifikasi |

### `SADatabaseRenameConnectionSession` (2353–2596)
Typealias (2355–2360); callback `run`/`quote`/`connectionEncoding`/`connectionUsesLatin1Transport`/`setConnectionEncoding`/`setConnectionLatin1Transport`/`storeEncodingForRestoration`/`restoreStoredEncoding`/`reconnect` (2362–2370);
`executorSettingsRestored` (2373), `changedServer` (2377), `warningDescription` (2383), `connectionUsable` (2388);
`OriginalSettings` (2391–2395); `init(run:quote:encoding:…)` (2415–2433); `rename(_:to:encoding:collation:)` (2445–2484);
`runExecutor(_:to:encoding:collation:)` (2486–2492); `restore(_:restoresEncoding:)` (2502–2515); `finish(_:restored:)` (2520–2530);
`reapply(_:)` (2538–2550); `isRestored(_:exactCollation:)` (2557–2572); `isSameCharacterSet(_:_:)` (2576–2578);
`collation(_:belongsTo:)` (2583–2590); `normalizedCharacterSet(_:)` (2592–2595).

## 3. Temuan menonjol / UNKNOWN

**Model plan: tolak, jangan pindah sebagian** (11–27). Jenis objek penghalang dibangun di initializer → string terlokalisasi: trigger (83–86), procedure (87–89), function (90–92), event (93–95); library via `recordLibraries` (178–182). `canStart` (187–189) gate tunggal (semua slot penolakan kosong); `failureDescription` (259–333) ladder prioritas ketat. `mayDropSourceDatabase` (253–256) re-check `canStart` + state create/move/compare — drop satu-satunya langkah ireversibel. `recordMove` (210–217) return false untuk **setiap** move setelah satu gagal → caller berhenti di kegagalan pertama (by design).

**Step & SQL** (semua lewat closure `run`): hanya dua jenis move `.table`/`.view` (12–15; 1460, 1499/1513/1540). Statement: `CREATE DATABASE` (1327–1333), `SHOW CREATE VIEW` (1409), `RENAME TABLE src.t TO tgt.t` (1459), `USE` (1496/1477), replay `CREATE` ter-rewrite (2316), `SELECT 1 FROM tgt.view LIMIT 0` (2321), `DROP VIEW` undo (2322), `DROP DATABASE` sumber (1556), `SET` sql_mode/quote_show_create/collation_connection (1402, 2313, 2510). Urutan: order `information_schema` untuk tabel (1457), dependency order untuk view (`orderedByReferences` 2073–2105).

**Rewriter view (518–1040, konsern terbesar)**: state machine token-level (bukan regex); `Token` pisahkan name/word/literal/symbol/whitespace/block-comment (541–549); comment disalin verbatim + diabaikan untuk konteks (601–618). Konteks objek-vs-kolom di-switch `FROM`/`…JOIN` (632–639), `expressionKeywords` (713–715), bookkeeping parenthesis (594, 644–661). Hanya tiga rewiring: nama `VIEW` clause (676–685), bagian database referensi objek dua-bagian (686–694), bagian database referensi kolom tiga-bagian (698–701). Alias dibiarkan (686–697 dengan `expectingAlias`). Tanpa `VIEW` clause → tolak (706–708). `referencedObjects` sengaja over-collect (498–503, 690–692). Sequence MariaDB special-case (740–751, 650–669).

**Edge case**: trigger ditolak (cross-DB RENAME terblokir) — inventaris trigger pakai perbandingan binary (2018–2020); routine/event tak pernah dipindah → ditolak (1563–1597); stored library MySQL 9.2+ diprobe + ditolak (1599–1632); events `SHOW EVENTS` + fallback `mysql.event` (1222–1230); view eksternal ditolak (162–170, 296–301) via `VIEW_TABLE_USAGE` atau scan `VIEWS` (1678–1693/1696–1722, over-inclusive); privilege ditolak (137–160) dengan `LIKE`+`ESCAPE` dan equality saat `partial_revokes` on (2022–2056), partial revoke via `JSON_SEARCH` di `mysql.user.User_attributes` (1747–1755, 1796–1819); charset hazard (1292–1309, 1415–1423, 876–917); drift kolom view (2238–2282); `SQL SECURITY DEFINER` view dibuka sekali, yang gagal di-drop + dilaporkan (2300–2304, 2320–2324); integritas koneksi `CONNECTION_ID()`+`DATABASE()` dibaca ulang (1517–1542), restore sesi dua kali + diverifikasi (1348–1380); hasil tanpa object **bukan** sukses (1125–1150); identitas nama by byte `Array(name.utf8)` (56–61, 425–432, 2066–2072, 2258–2262); `lower_case_table_names` folded vs binary (1967–1975, 1244–1265, 1527–1534).

**UNKNOWN / smell**
- **Foreign key: tidak ada penanganan** (grep `FOREIGN`/`foreign`/`REFERENTIAL` = 0). Tak check/rewrite FK lintas DB; mengandalkan error server + `recordMove` berhenti (210–217).
- **Rollback: tidak ada** — saat gagal, tabel yang sudah pindah tak di-drop (199–201, 1460–1463).
- **source == target: UNKNOWN** — tak ada perbandingan di layer ini; tampaknya abort hanya karena `CREATE DATABASE` gagal (1450–1454).
- **Atribusi kegagalan pakai nama view arbitrer:** `plan.views[0]` (1499) dan `plan.views[last]` (1540) dicatat sebagai "failed object".
- **Penempatan doc rusak di 1977–2011** (doc TRIGGERS di atas `privilegeViewSchemaMatch`, bukan `inventorySchemaMatch`, + baris `- Returns:` terduplikasi 1997/2011); doc 355 salah tempat.
- **Tak ada `// MARK:`** di 2596 baris; dua tipe >400 baris (`rename` 1207–1561 = 355 baris; `rewriteCreateStatement` 589–710).
- **Inkonsistensi akses:** `signatures(fromRows:)` (438) dan kedua init `SADatabaseRenameViewRewrite` (505, 511) non-`@objc`; `isSource(schemaName:)` (786) & `definitionReferencesSource` (798) internal (kemungkinan untuk test).
- **Performa:** fallback `externalViewsReadingFromSource` membaca + hex-decode **semua** definisi view (1700–1721) — paling mahal; tanpa shortcut `performance_schema`.
