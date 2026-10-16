# Content: memuat baris, pagination, sortir, filter

Kontrak source untuk separuh **load** dari tab Content: SQL SELECT yang dibentuk,
pagination, model sortir, filter (sederhana/lanjutan/kustom), count, streaming,
dan cancel. Sitasi `file:line` merujuk checkout `02c910932`.

File: `Source/Controllers/MainViewControllers/TableContent/SPTableContent.m` (5656 baris)
bagian load/pagination/sort/filter/count/streaming/cancel; `SATableReloadRetryPolicy.swift`;
`SACellFilter*.swift`; `SPContentFilterManager.m/.h`, `SPFilterTableController.m/.h`,
`SPRuleFilterController.m/.h`, `SARuleFilterRootConjunction.swift`;
`SPFilterRuleEditor.swift`, `SPFilterRuleTextField.swift`, `SPRuleFilterDropBox.swift`;
`SPTableFilterParser.m/.h`; xib `ContentFilterManager.xib`, `ContentPaginationView.xib`,
`FilterTableWindow.xib`.

## 1. SELECT untuk load content

Semua dibangun di `-[SPTableContent loadTableValues]` — `SPTableContent.m:950-1233`.

**Statement dasar** (`979-982`):
```objc
queryString = [NSMutableString stringWithFormat:@"SELECT %@%@ FROM %@",
        (activeFilter == SPTableContentFilterSourceTableFilter && filterString && [filterTableController isDistinct]) ? @"DISTINCT " : @"",
        [self fieldListForQuery], [selectedTable backtickQuotedString]];
```
- `%1$@` = `DISTINCT ` hanya bila sumber filter tabel lanjutan, non-kosong, dan `isDistinct` (`SPFilterTableController.m:214-217`).
- `%3$@` = tabel via `backtickQuotedString`.

**Daftar kolom** — `-[SPTableContent fieldListForQuery]` (`3422-3447`):
- Tanpa `dataColumns` → literal `@"*"` (`3424-3426`).
- Selain itu gabung per-kolom dengan `@", "` (`3445`); tiap kolom `[fieldName backtickQuotedString]` (`3442`)
  — kecuali saat `SPLoadBlobsAsNeeded` on dan `columnIsBlobOrText:` → literal `@"NULL"` tak ter-quote (`3436-3440`).

**WHERE** (`984-989`): `[queryString appendFormat:@" WHERE %@", filterString]` bila non-kosong; `isFiltered` di-set (`986-988`).

**ORDER BY** (`991-995`): hanya bila `sortCol && [sortCol integerValue] < dataColumns.count`:
` ORDER BY <name backtick-quoted>` + ` DESC` bila `isDesc` (satu kolom, tanpa `ASC`, tanpa NULLS).

**LIMIT** (`1012-1013`): ` LIMIT %ld,%ld` dengan `(contentPage-1)*[prefs integerForKey:SPLimitResultsValue]`
dan `[prefs integerForKey:SPLimitResultsValue]`; hanya bila `[prefs boolForKey:SPLimitResults]` (`998`).

**Fallback LIMIT** (`1075`): `%@ LIMIT 0,%ld` dari `queryStringBeforeLimit`.

Query dieksekusi di `1026` (`resultStoreFromQueryString:assertingDatabase:`) via store streaming;
disimpan sebagai `usedQuery` (`1025`, `setUsedQuery:` `1754-1757`).

## 2. State machine pagination

State: `contentPage` (`SPTableContent.h:121`), `maxNumRows`, `maxNumRowsIsEstimate`,
`isLimited` (`h:114`), `isFiltered` (`h:113`).

- **Reset ke halaman 1** saat ganti tabel: `setTableDetails:` (`580-581`).
- **Restore** halaman tersimpan hanya bila limiting on: `701` (`if ([prefs boolForKey:SPLimitResults]) contentPage = pageToRestore;`), diisi `setPageToRestore:` (`3875-3878`).
- **Clamp sebelum query** (`1000-1004`): `contentPage <= 0 → 1`; bila `(contentPage-1)*limit >= maxNumRows` → `contentPage = ceilf(maxNumRows / limit)`.
- **Late-page copy** (`1006-1010`): bila `contentPage > 1`, `queryStringBeforeLimit` = salinan SQL *sebelum* LIMIT.
- **Estimasi rowsToLoad** (`1015-1017`): `rowsToLoad -= (contentPage-1)*limit`, dibatasi limit.
- **Fallback halaman-akhir kosong** (`1071-1081`): bila bukan full-reload wajib, limiting on, `queryStringBeforeLimit` ada, `tableRowsCount == 0`, dan query **tidak** dibatalkan → `contentPage = 1`, `previousTableRowsCount = tableRowsCount`, jalankan ulang `@"%@ LIMIT 0,%ld"`, update store dengan target = limit.
- **isLimited** (`1160-1165`): YES iff limiting on DAN (`contentPage > 1` OR `tableRowsCount == limit`).
- **Tombol next/prev** — `navigatePaginationFromButton:` (`1864-1877`): prev diblokir di `contentPage <= 1`; next diblokir saat `contentPage*limit >= maxNumRows`; lalu delegate ke `filterTable:`.
- **Halaman saat filter/apply** — `filterTable:` (`1651-1664`): bila tombol filter/tabel lanjutan dipakai, limiting off, atau popup page `<= 0` → halaman 1; selain itu clamp; selain itu pakai nilai popup.
- **State tombol/teks** — `updatePaginationState` (`1943-1967`): `maxPage = ceilf(maxNumRows / limit)`; dibekukan ke `contentPage` saat `isFiltered && !isLimited` (`1946-1948`); prev aktif iff `limitResults && contentPage > 1`; next iff `limitResults && contentPage < maxPage`.
- **Visibilitas** — `setPaginationViewVisibility:` (`1897-1937`) (NSPopover `241-245`, atau frame legacy `paginationViewHeight`).
- **Key preferensi**: `SPLimitResults = "LimitResults"` (`SPConstants.m:133`; default `true`
  `PreferenceDefaults.plist:164-165`), `SPLimitResultsValue = "LimitResultsValue"` (`SPConstants.m:134`;
  default `1000` `PreferenceDefaults.plist:166-167`). Nib pagination mengekspos "Limit result to:",
  "Jump to page:", "rows", "Go", "Defer loading of blobs and texts" (`ContentPaginationView.xib:27,39,58,67,78,133`).

## 3. Model filter

**Sumber** — `SPTableContentFilterSource` (`SPTableContent.h:52-56`): `None = -1`,
`RuleFilter = 0`, `TableFilter = 1`, `URLScheme = 2`; di `activeFilter`/`activeFilterToRestore`.

**Selektor** — `-[SPTableContent tableFilterString]` (`1294-1322`):
- `URLScheme` → kembalikan `schemeFilter` apa adanya (`1298-1300`).
- `TableFilter` & window terlihat → `[filterTableController tableFilterString]` (`1303-1305`).
- `RuleFilter` & rule editor tampil → `[ruleFilterController sqlWhereExpressionWithBinary:caseSensitive error:&err]` (`1306-1318`); `caseSensitive` = Shift ditekan saat invoke (`1310`); error → alert "Invalid Filter" + nil (`1314-1316`).
- selain itu nil.

**Sederhana vs lanjutan**
- *Aturan sederhana* = rule editor (`SPRuleFilterEditor`, NSRuleEditor), di `setRuleEditorVisible:` (`1719-1752`);
  baris awal `addStarterFilterExpression` (`1740`) digerbangi `SARuleFilterVisibilityPolicy`
  (`SPFilterRuleEditor.swift`). Per-baris: checkbox + kolom + operator + 1–2 arg (`SPRuleFilterController.m:1848-1864`).
- *Aturan lanjutan* = window filter-table (`FilterTableWindow.xib`), editor WHERE bebas dengan
  "Negate clause", "Select distinct", "Search while typing"; dikompilasi di `SPFilterTableController.m:151-164`:
  `NOT (%@)` bila dinegasi, selain itu teks mentah.
- *Definisi filter kustom* = `SPContentFilterManager` (`ContentFilterManager.xib`), mengedit entri
  bergaya `ContentFilters.plist` (`MenuLabel`, `Clause`, `NumberOfArguments`, `ConjunctionLabels`,
  `SuppressLeadingFieldPlaceholder`) di bawah `SPContentFilters = "ContentFilters"` (`SPConstants.m:254`).
  Jumlah argumen diturunkan otomatis via regex `${...}` (`SPContentFilterManager.m:188-206`).

**Katalog operator** — `Resources/Plists/ContentFilters.plist` (1192 baris), dikelompokkan per tag tipe:
- `bit` (5-109): `=`, `≠`, `>`, `<`, `≥`, `≤` (semua `= CAST('${}' AS DECIMAL(65,30))`), `IN (${})`,
  `NOT IN __ OR NULL` (`($CURRENT_FIELD NOT IN (${}) OR $CURRENT_FIELD IS NULL)`,
  `SuppressLeadingFieldPlaceholder`), `LIKE`, `BETWEEN` (2 arg), `IS NULL`, `IS NOT NULL`.
- `number` (110-214): sama seperti bit tapi `'${}'` polos.
- `string` (215-421): `=`, `≠`, `LIKE`, `NOT LIKE`, `contains`, `does not contain`, `starts with`,
  `does not start with`, `ends with`, `does not end with`, `matches RegExp`, `does not match RegExp`,
  `IN`, `NOT IN __ OR NULL`, `BETWEEN`, `IS NULL`, `IS NOT NULL`, `is empty` (`LIKE ''`),
  `is not empty`, `length =`/`>`/`<` (`CHAR_LENGTH($CURRENT_FIELD)`),
  `= (Hex String)` (`= $BINARY UNHEX('${}')`).
- `date` (422-500): `=`, `≠`, `is after`, `is before`, `is after or equal to`, `is before or equal to`, `BETWEEN`, `IS NULL`, `IS NOT NULL`.
- `spatial` (501-589): `contains`, `within`, `disjoint`, `equal`, `intersects`, `overlaps`, `touches`, `IS NULL`, `IS NOT NULL` (`MBR*` / `ST_GeomFromText`).
- Prefiks `$BINARY ` (grup string) diganti `BINARY ` (Shift) atau `""` (`SPRuleFilterController.m:1395-1396`, `SPTableFilterParser.m:105`).

**Submenu cell-filter** — `SACellFilterOperator.swift`: `=`, `≠`, `>`, `<`, `≥`, `≤`
(number/bit/integer/float), set date (`=`,`≠`,`is after`,`is before`,`is after or equal to`,
`is before or equal to`), set string (`=`,`≠`,`LIKE`,`NOT LIKE`,`contains`,`does not contain`),
plus `IS NULL`/`IS NOT NULL` untuk setiap grouping. Helper mapping
`ruleFilterGroupForTypeGrouping:` dan `filterDefinitionGroupForTypeGrouping:` (bit→`bit`,
selain `number`); `filterDefinitionGroups = ["number","date","string","spatial","bit"]`.

**Kompilasi ke SQL** — `SPRuleFilterController.m`:
1. Bangun tree terserialisasi (`sqlWhereExpressionWithBinary:` `1777-1802`) dari
   `_serializedFilterIncludingFilterDefinition:withDisabled:` (`1809-1818`).
2. `_flattenSerializedFilter:` (`2102+`).
3. `_writeFilterTree:toString:wrapInParenthesis:binary:error:` (`2138-2216`):
   group → child dijoin `" AND "`/`" OR "` (`2146-2150`), dikurung bila konjungsi campur (`2152-2161`);
   expression → baca definisi `Clause` + `NumberOfArguments` (`2179-2191`), buat `SPTableFilterParser`
   (`2195-2201`), append `[parser filterString]` (`2204, 2212`). Error: definisi hilang (`2173`),
   `NumberOfArguments` hilang (`2180`), clause kosong (`2187`), parser nil (`2207`).

**Parser** — `SPTableFilterParser.m`: `$BINARY `→`BINARY `/`` (`105`), `$CURRENT_FIELD`→field
backtick-quoted (`107`); `%` digandakan saat ada arg (`110`), `${...}`→`%@` (`113-121`), jumlah
placeholder harus sama `numberOfArguments` selain itu `NSBeep` + nil (`124-128`); nama field
diprepend kecuali `suppressLeadingTablePlaceholder` (`131-133`); maks 2 arg (`135-139`);
`escapeFilterArgument:againstClause:` (`164-196`) menggandakan backslash (lebih dalam untuk LIKE),
escape karakter quote yang terdeteksi, mempertahankan `\n\r\t`.

**Konjungsi root** — `SPRuleFilterController.rootIsConjunction` (`m:1259-1310`), diserialisasi
`SARuleFilterRootConjunction.serializedRoot(items:isConjunction:)` (menandai group root
`rootGroup = true`); `SACellFilterMerge.mergedFilter` menambahkan cell filter di bawah konjungsi
yang ada (`extendingMarkedRoot`), menyaring baris starter yang tak tersentuh.

**Terapkan cell filter** — `SACellFilterAction.apply` → `applyCellFilterForColumn:operator:values:isNull:`
(`1837-1855`): bangun rule terserialisasi (`makeSerializedFilterForColumn:` `2049-2058`), merge,
restore, set `activeFilter = RuleFilter`, tampilkan editor, trigger `filterTable:`.

## 4. Model sortir

State: `sortCol` (`NSNumber *`, `h:107`), `isDesc` (`h:111`); field restore
`sortColumnToRestore`, `sortColumnToRestoreIsAsc` (`h:127-129`).

- Entry: `tableView:didClickTableColumn:` (`4676-4692`) → thread baru `sortTableTaskWithColumn:`.
- Toggle tri-state (`1759-1800`): kolom sama → Shift membalik; siklus desc → asc → none;
  kolom baru → `isDesc` per Shift; pindahkan indikator sort.
- Indikator/highlight (`1800-1818`); reload via `loadTableValues` (`1821-1824`).
- ORDER BY diemit di `992-995`. Getter `sortColumnName` (`3725-3730`), `sortColumnIsAscending`
  (`3735-3738`); restore setter `setSortColumnNameToRestore:isAscending:` (`3862-3870`).
- Gagal: alert `"Couldn't sort table. MySQL said: %@"` (`1827`).

## 5. Count

- `maxNumRows`/`maxNumRowsIsEstimate` di-set dari status `Rows` saat ganti tabel
  (`setTableDetails:` `592-606`, `maxNumRowsIsEstimate = YES`).
- `-[SPTableContent updateNumberOfRows]` (`4080-4140`), dipanggil di `1168` setelah tiap load:
  - **Tak terfilter, tak terbatas, tak terputus** → `maxNumRows = tableRowsCount`, estimate NO,
    tulis `Rows` + `RowsCountAccurate="y"` (`4085-4091`).
  - **Selain itu** → `-[SPTableData updateAccurateNumberOfRowsForCurrentTableForcingUpdate:NO]` (`4096`), baca status `Rows` (`4099-4110`), rekonsiliasi (`4114-4139`).
- Query COUNT di `SPTableData.m:1323-1377`: skip bila non-tabel (`1335`), sudah akurat (`1343`),
  atau `SPTableRowCountQueryLevel == Never`/`IfCheap` dan `Data_length >= boundary` (`1354-1360`);
  query (`1364`): `SELECT COUNT(1) FROM %@` — **tanpa WHERE**, mengabaikan filter aktif (count
  seluruh tabel); gagal → return NO tanpa alert (`1365-1367`); sukses → `Rows` + `"y"` (`1370-1371`).
- Fallback terpisah: `SHOW TABLE STATUS` `Rows` `NSNull` → `SELECT COUNT(1) FROM %@`
  (`SPTableData.m:1239`) + alert "Querying row count failed" (`1247-1248`).
- Key: `SPTableRowCountQueryLevel = "TableRowCountQueryLevel"` (`SPConstants.m:130`; default 1
  `PreferenceDefaults.plist:218-219`), `SPTableRowCountCheapSizeBoundary = "TableRowCountCheapLookupSizeBoundary"`
  (`SPConstants.m:131`; default 5242880 `PreferenceDefaults.plist:216-217`).
- Teks count `updateCountText` (`1330-1383`): `"%@ row(s) in table"`, `"Rows %@ - %@ of %@%@ from table"`,
  `"%@ row(s) of %@%@ match filter"`, `"Rows %@ - %@ from filtered matches"`.

## 6. Streaming & cancel

**Streaming** — `-[SPTableContent updateResultStore:approximateRowCount:]` (`1239-1285`):
`tableLoadTargetRowCount = targetRowCount` (`1243`); swap store `[tableValues setDataStorage:theResultStore updatingExisting:!![tableValues count]]` (`1251`);
`[theResultStore startDownload]` (`1255`); dengan `SPLoadBlobsAsNeeded`, kolom blob/text ditandai
unloaded (`setColumnAsUnloaded:`) agar dimuat lambat (`1260-1266`); timer progress
`initTableLoadTimer` (`1269`, `1389-1397`, 0.02 s) → `tableLoadUpdate:` (`1417-1468`)
(throttle 1→10→25 tick, `setTaskPercentage:`, autosize di baris pertama dan >200);
`[tableValues awaitDataDownloaded]` (`1271`); count akhir/autosize/`noteNumberOfRowsChanged`
di main queue (`1273-1284`).

**Flag cancel**
- Tombol Stop per load: `enableTaskCancellationWithTitle:NSLocalizedString(@"Stop",…)` (`1021`);
  disarm di `1048` dan `1089`.
- Abort column-mismatch → `[mySQLConnection cancelCurrentQuery]` + `[resultStore cancelResultLoad]` (`1049-1050`).
- `isInterruptedLoad = YES` saat `[mySQLConnection lastQueryWasCancelled] || [mySQLConnection queryErrored]`
  (`1083-1086`); menggerakkan teks count partial (`1340-1344`) dan memblokir "delete all rows" (`2182`).
- Error (`1191-1223`): rule-filter/none → alert `"The table data couldn't be loaded (presumably due to used filter clause)…"` atau polos; advanced table → `setFilterError:message:sqlstate:`; clear saat sukses (`1222`).
- Budget retry `SATableReloadRetryPolicy`/`SATableReloadCoordinator` (default limit 3); wired di
  `loadTable:` (`377-388`), `_startPendingFullReloadIfIdle` (`465-470`), `reloadTableTask` (`1527-1565`),
  mismatch check (`1039-1064`).

## 7. String yang terlihat pengguna (separuh load)

`SPTableContent.m`: "Loading table data..." (`970`), "Stop" (`1021`), "Reloading data..." (`1498`),
"The column list for '%@' did not match…" + "Table contents not loaded" (`1515,1520`),
"Filtering table..." (`1667`), "Loading %@..." (`1669`), "Loading page %lu..." (`1671`),
"Invalid Filter" (`1315`), "The table data couldn't be loaded presumably due to used filter clause. \n\nMySQL said: %@" (`1195`),
"The table data couldn't be loaded.\n\nMySQL said: %@" (`1198`), "Couldn't sort table. MySQL said: %@" (`1827`).

`SPRuleFilterController.m`: "Fatal error while retrieving content filter. No filter definition found." (`2174`),
"Error while retrieving filter clause. No “NumberOfArguments” key found." (`2181`),
"Content Filter clause is empty." (`2188`), "No valid SQL expression could be generated…" (`2208`).

`SPFilterTableController.m`: "WHERE clause not valid" / "Filter" (`222`).

`SPContentFilterManager.m`: "[multiple selection]", "[no selection]", "[name required]",
"Number of arguments: %lu", "Maximum number of arguments is 2!", "Remove selected content filters?".

`SPFilterRuleEditor.swift`/`SPRuleFilterDropBox.swift`: "Add Filter", "Add AND/OR Group",
"Drop a value here, or click to add a filter", "Copy WHERE Clause".

`SACellFilterMenuBuilder.swift`: "Copy %@ Values", "Copy %@ Values as SQL".

Key pref: `LoadBlobsAsNeeded` (`SPConstants.m:129`), `RuleFilterEditorLastVisibilityChoice` (`:217`),
`ContentFilters` (`:254`), `RuleFilterShowDropZone` (default `true` `PreferenceDefaults.plist:206-207`).
