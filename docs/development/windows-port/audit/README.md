# Audit baris-per-baris file raksasa

Audit method-by-method file terbesar di `Source/`. Setiap dokumen mencantumkan
**setiap** definisi method/func (`- (…)`/`+ (…)`, `func`, dan fungsi/static C
bila ada) tepat sekali dengan rentang baris `Start`/`End` + tujuan satu baris,
plus inventaris ivar/property/tipe, daftar `#pragma mark`/`// MARK:`, dan
temuan/`UNKNOWN`. Baseline commit `02c910932`; sitasi `file:line` relatif ke repo.

Tiga gelombang: **Gelombang 1** = file ObjC/Swift ≥ ~1750 baris; **Gelombang 2** =
file 1000–1750 baris + file Swift ≥ 600 baris; **Gelombang 3** = file 200–999
baris (120 file / ~55.000 baris, dikelompokkan menjadi 18 dokumen klaster
per-subsistem). Total yang dipetakan: ~135.000 baris, ~2.700 definisi.

## Gelombang 1 — file ≥ ~1750 baris

| File | Baris | Method | Dokumen |
| --- | --- | --- | --- |
| `Controllers/MainViewControllers/SPDatabaseDocument.m` | 6436 | 230 | [SPDatabaseDocument.md](./SPDatabaseDocument.md) |
| `Controllers/MainViewControllers/TableContent/SPTableContent.m` | 5656 | ~150 | [SPTableContent.md](./SPTableContent.md) |
| `Controllers/MainViewControllers/ConnectionView/SPConnectionController.m` | 5438 | ~150 | [SPConnectionController.md](./SPConnectionController.md) |
| `Controllers/MainViewControllers/SPCustomQuery.m` | 4079 | 74 (+ helper) | [SPCustomQuery.md](./SPCustomQuery.md) |
| `Controllers/DataExport/SPExportController.m` | 3982 | 136 | [SPExportController.md](./SPExportController.md) |
| `Views/TextViews/SPTextView.m` | 3884 | 74 (+1 fungsi C) | [SPTextView.md](./SPTextView.md) |
| `Controllers/MainViewControllers/TableStructure/SPTableStructure.m` | 3180 | 73 | [SPTableStructure.md](./SPTableStructure.md) |
| `Controllers/SubviewControllers/SPTablesList.m` | 3043 | 92 | [SPTablesList.md](./SPTablesList.md) |
| `Controllers/SubviewControllers/SPRuleFilterController.m` | 2481 | 52 (+8 kelas helper) | [SPRuleFilterController.md](./SPRuleFilterController.md) |
| `Controllers/DataImport/SPFieldMapperController.m` | 2140 | 66 | [SPFieldMapperController.md](./SPFieldMapperController.md) |
| `Controllers/BundleSupport/SPBundleEditorController.m` | 2077 | 66 | [SPBundleEditorController.md](./SPBundleEditorController.md) |
| `Controllers/SubviewControllers/SPUserManager.m` | 2001 | 71 | [SPUserManager.md](./SPUserManager.md) |
| `Views/TableViews/SPCopyTable.m` | 1807 | 29 | [SPCopyTable.md](./SPCopyTable.md) |
| `Controllers/DataImport/SPDataImport.m` | 1760 | 22 | [SPDataImport.md](./SPDataImport.md) |

## Gelombang 2 — file 1000–1750 baris + Swift ≥ 600 baris

| File | Baris | Method | Dokumen |
| --- | --- | --- | --- |
| `Other/DatabaseActions/SADatabaseRenamePlan.swift` | 2596 | ~150 (7 tipe) | [SADatabaseRenamePlan.md](./SADatabaseRenamePlan.md) |
| `Controllers/DataControllers/SPTableData.m` | 1637 | 39 | [SPTableData.md](./SPTableData.md) |
| `Controllers/SubviewControllers/SPFieldEditorController.m` | 1532 | 43 | [SPFieldEditorController.md](./SPFieldEditorController.md) |
| `Controllers/SubviewControllers/SPNavigatorController.m` | 1309 | 52 | [SPNavigatorController.md](./SPNavigatorController.md) |
| `Other/SSHTunnel/SPSSHTunnel.m` | 1243 | 42 (+1 fungsi C) | [SPSSHTunnel.md](./SPSSHTunnel.md) |
| `Controllers/SubviewControllers/SPQueryController.m` | 1176 | 52 | [SPQueryController.md](./SPQueryController.md) |
| `Model/SPNarrowDownCompletion.m` | 1165 | 21 | [SPNarrowDownCompletion.md](./SPNarrowDownCompletion.md) |
| `Controllers/Other/SPBundleManager.m` | 1119 | 19 | [SPBundleManager.md](./SPBundleManager.md) |
| `Other/Parsing/SPSQLParser.m` | 1110 | 59 | [SPSQLParser.md](./SPSQLParser.md) |
| `Views/SPSplitView.m` | 1105 | 34 | [SPSplitView.md](./SPSplitView.md) |
| `Other/Extensions/StringExtension.swift` | 1093 | ~90 (11 tipe) | [StringExtension.md](./StringExtension.md) |
| `Controllers/DataExport/Exporters/SPSQLExporter.m` | 1028 | 5 | [SPSQLExporter.md](./SPSQLExporter.md) |
| `Controllers/SubviewControllers/SPIndexesController.m` | 1027 | 25 | [SPIndexesController.md](./SPIndexesController.md) |
| `Controllers/Preferences/Panes/SPEditorPreferencePane.m` | 1018 | 36 | [SPEditorPreferencePane.md](./SPEditorPreferencePane.md) |
| `Other/AWS/AWSIAMAuthManager.swift` | 967 | ~71 (6 tipe) | [AWSIAMAuthManager.md](./AWSIAMAuthManager.md) |
| MCP cluster (`SPAppController+MCP`/`SPMCPServer`/`SPMCPHTTP`) | 963+893+552 | ~120 | [MCP-cluster.md](./MCP-cluster.md) |
| `Views/TableViews/SATablesListView.swift` | 900 | 61 | [SATablesListView.md](./SATablesListView.md) |
| Connection cluster (`SAConnectionWindowController`/`SAConnectionFormView`/`SAConnectionService`) | 773+617+617 | ~130 | [connection-cluster.md](./connection-cluster.md) |
| PHP-serialized cluster (`SAPHPSerializedValue`/`SPFieldEditorController+PHPSerialized`) | 762+711 | ~110 | [PHP-serialized-cluster.md](./PHP-serialized-cluster.md) |
| Integration cluster (`GitHubReleaseManager`/`SecureBookmarkManager`/`VaultOIDCHandler`) | 737+664+618 | ~120 | [integration-cluster.md](./integration-cluster.md) |
| Filter-editor cluster (`SPCustomQuerySQLClassifier`/`SPFilterRuleEditor`/`SARecordView`) | 623+607+605 | ~130 | [filter-editor-cluster.md](./filter-editor-cluster.md) |

## Gelombang 3 — file 200–999 baris (klaster per-subsistem)

| Klaster | File (baris) | Dokumen |
| --- | --- | --- |
| Preferensi | `SPNetworkPreferencePane.m` (776), `SPFilePreferencePane.m` (601), `SPMCPPreferencePane.swift` (377), `SPPreferenceController.m` (352), `SPGeneralPreferencePane.m` (258) | [preferences-cluster.md](./preferences-cluster.md) |
| DataControllers | `SPDatabaseStructure.m` (722), `SPDatabaseData.m` (594), `SPFavoritesController.m` (494), `SPServerSupport.m` (234) | [data-controllers-cluster.md](./data-controllers-cluster.md) |
| Tab metadata tabel | `SPTableRelations.m` (761), `SPExtendedTableInfo.m` (738), `SPTableTriggers.m` (694), `SPTableInfo.m` (525), `SAForeignKeyReferenceRuleSupport.swift` (218) | [table-metadata-tabs-cluster.md](./table-metadata-tabs-cluster.md) |
| Server tools | `SPProcessListController.m` (802), `SPGotoDatabaseController.m` (454), `SPServerVariablesController.m` (401), `SPCharsetCollationHelper.m` (289) | [server-tools-cluster.md](./server-tools-cluster.md) |
| AWS | `AWSSTSClient.swift` (577), `SAAWSLoginRefresh.swift` (525), `AWSCredentials.swift` (479), `AWSDirectoryBookmarkManager.swift` (391), `AWSSSOClient.swift` (380), `RDSIAMAuthentication.swift` (366), `AWSLoginCredentialsProvider.swift` (307) | [aws-cluster.md](./aws-cluster.md) |
| CategoryAdditions | `SPTextViewAdditions.m` (834), `SPStringAdditions.m` (683), `SPDataAdditions.m` (633), `SPDataStorage.m` (590), `SPFileManagerAdditions.m` (222) | [category-additions-cluster.md](./category-additions-cluster.md) |
| Model | `SPTooltip.m` (529), `SAConnectionInfo.swift` (410), `SPTreeNode.m` (336), `SATooltipDismissal.swift` (328), `SAConnectionInfo+Favorite.swift` (264), `SAConnectionInfo+ConnectionString.swift` (250), `ConnectionStringParser.swift` (206) | [model-cluster.md](./model-cluster.md) |
| Utility/Keychain/Data/Debug | `SPConstants.m` (535), `OSLog.swift` (521), `SPFileHandle.m` (484), `SPFunctions.m` (453), `NSAlertExtension.swift` (375), `SAKeychain.swift` (286), `SADragPasteboard.swift` (262), `SPLogger.m` (261), `SAPrintUtility.swift` (220) | [utility-keychain-cluster.md](./utility-keychain-cluster.md) |
| Exporter | `SPCSVExporter.m` (406), `SPXMLExporter.m` (342), `SPDotExporter.m` (240), `SPExportFile.m` (238) | [exporters-cluster.md](./exporters-cluster.md) |
| View/Cell/Control | `SPColorSelectorView.m` (459), `SPActivityTextFieldCell.m` (353), `SPTableView.m` (340), `SPComboPopupButton.m` (304), `SPTextAndLinkCell.m` (283), `ImageAndTextCell.m` (205), `SPWindow.m` (201) | [views-cell-controls-cluster.md](./views-cell-controls-cluster.md) |
| Content-helper Swift | `SPRuleFilterDropBox.swift` (398), `SACellFilterMenuBuilder.swift` (333), `SATableReloadRetryPolicy.swift` (247), `SAFieldEditorCommitPolicy.swift` (239), `SACSVImportStreamDecoder.swift` (211), `SACellFilterOperator.swift` (206), `SACellFilterAction.swift` (201) | [content-helper-swift-cluster.md](./content-helper-swift-cluster.md) |
| TextView | `SPBundleCommandTextView.m` (809), `SPEditSheetTextView.m` (324) | [textview-cluster.md](./textview-cluster.md) |
| Bundle + Help viewer | `SPBundleCommandRunner.m` (509), `SABundleHTMLOutputWindowController.swift` (385), `SAHelpViewerWindowController.swift` (380), `SAHelpViewerModel.swift` (314), `SPHelpViewerClient.m` (300) | [bundle-help-cluster.md](./bundle-help-cluster.md) |
| History/SQLite | `GitHub.swift` (551), `SPHistoryController.m` (499), `SQLiteHistoryManager.swift` (491), `SQLiteDisplayFormatManager.swift` (471), `SQLitePinnedTableManager.swift` (460) | [history-sqlite-cluster.md](./history-sqlite-cluster.md) |
| Connection-form Swift | `SAConnectionFormModel.swift` (540), `SAFavoritesListDataSource.swift` (449), `SAConnectionDetailsValidator.swift` (320), `SAFavoriteDuplicateMatcher.swift` (214) | [connection-form-cluster.md](./connection-form-cluster.md) |
| Main-controller Swift | `SADatabaseListManager.swift` (490), `SATaskController.swift` (395), `SPAppController.swift` (289), `SAQueryFavoriteSaveWindowController.swift` (254) | [main-controllers-swift-cluster.md](./main-controllers-swift-cluster.md) |
| Vault + SSH tunnel | `VaultAuthManager.swift` (482), `VaultClient.swift` (320), `SASSHTunnelSocketServer.swift` (301), `SASSHTunnelPeerValidator.swift` (257), `SAVaultRoleListController.swift` (247), `SASSHTunnelAuthMessage.swift` (207) | [vault-ssh-cluster.md](./vault-ssh-cluster.md) |
| Parsing + ThirdParty | `SPCSVParser.m` (649), `MGTemplateStandardMarkers.m` (631), `MGTemplateEngine.m` (619), `NoodleLineNumberView.m` (566), `YRKSpinningProgressIndicator.m` (375), `SPJSONFormatter.m` (368) | [parsing-thirdparty-cluster.md](./parsing-thirdparty-cluster.md) |

## Ringkasan bug / hazard yang ditemukan dari source

Ini **temuan statis dari kode**, bukan laporan runtime. Masing-masing punya
sitasi di dokumen terkait.

| File | Temuan | Sitasi |
| --- | --- | --- |
| `SPDatabaseDocument.m` | Key restore halaman tidak cocok: baca `@"pageNumber"` padahal tulis `@"contentPageNumber"` → restore tak efektif | `.m:4095` vs `:3705` |
| `SPDatabaseDocument.m` | `compareDatabase:` dead/eksperimental (membandingkan baris dengan dirinya sendiri) | `.m:752–815` |
| `SPDatabaseDocument.m` | Logika hasil maintenance tabel terduplikasi 6×; dispatch bundle-trigger terduplikasi | `.m:1606–1934`, `5284`, `5853–5895` |
| `SPDatabaseDocument.m` | `#pragma`/switch `connectionID` tak konsisten (case SSH tanpa `break`) | `.m:2258–2264` |
| `SPTableContent.m` | Pembangun WHERE terduplikasi (`argumentForRow:` dua varian) | `.m:1976–2049` vs `3292–3402` |
| `SPTableContent.m` | `visibleColumns` dideklarasikan tak dipakai; `NSLog` dealloc tertinggal | `.h:101`, `.m:5609` |
| `SPCustomQuery.m` | Blok reorder favorites terkomentari merujuk ivar tak terdefinisi; TODO (#2607) duplikasi kode dengan SPTableContent | `.m:2671–2720`, `3885` |
| `SPCustomQuery.m` | Selector `runExplainQueryAction:` ditangani tapi didefinisikan di file lain | `.m:3491` |
| `SPExportController.m` | Stub HTML/PDF; Excel tak ada; banyak `@throw @"exportType not implemented!"` | `.m:3920–3946`, `3333–3360` |
| `SPExportController.m` | `writeCSVHeaderToExportFile:` menulis nama tabel dari `[[tables firstObject] firstObject]` (bukan tabel aktif) | `.m:1932` |
| `SPTextView.m` | `visibleRange` hanya di-assign di satu cabang; cabang lain hanya log "returning" tanpa return | `.m:2903–2930` |
| `SPTextView.m` | Method privat ber-prefiks `_` berisiko bentrok AppKit (`_setTextSelectionColor:`) | `.m:3836` |
| `SPTableStructure.m` | `((firstChar == '"') && (lastChar = '"'))` pakai assignment (`=`), bukan `==` | `.m:1219–1220` |
| `SPTableStructure.m` | Offset `currentlyEditingRow - 1` bisa −1 untuk field pertama | `.m:989, 995, 1003` |
| `SPTablesList.m` | `setDatabases` legacy (tanpa argumen) vs item menu `setDatabases:` (dengan colon) → popup copy-sheet kemungkinan basi | `.m:637` vs `:646` |
| `SPRuleFilterController.m` | `_arrangedCategoryIndexForScopeIndex:andCategory:` `return 0;` tak-bersyarat → hanya scope pertama discan | `.m:2071` |
| `SPFieldMapperController.m` | Precedence bug `integerValue == (isUPDATE) ? A : B` (`==` mengikat lebih kuat dari `?:`) | `.m:1891` |
| `SPFieldMapperController.m` | `insertNULLValue:` tak memperbarui `fieldMappingGlobalValuesSQLMarked` paralel | `.m:1117` |
| `SPBundleEditorController.m` | `saveBundle:atPath:` return YES walau `writeToURL:` gagal | `.m:1077–1082` |
| `SPBundleEditorController.m` | Early-return bug di `_arrangedCategoryIndexForScopeIndex:andCategory:` | `.m:2071` |
| `SPUserManager.m` | `contextDidChange:` tak pernah diregistrasi (dead); `_clearData` tanpa pemanggil | `.m:1164`, `1076` |
| `SPUserManager.m` | Dekode bit `mysql.global_priv` lewat substring `"access"`, bukan parse JSON | `.m:208–215` |
| `SPUserManager.m` | Selector notifikasi nol-argumen `_initializeSchemaPrivs` | `.m:138` vs `577` |
| `SPDataImport.m` | Return `buildFieldMappingArrayWithData:…` tak konsisten (YES saat data kosong) | `.m:1221` vs `1228/1263` |
| `SPDataImport.m` | Substitusi `$N` terduplikasi 3× | `.m:1352, 1404, 1482` |
| `SPCopyTable.m` | Guard enum pada `isBlob` adalah dead code | `.m:1464` |
| `SPCopyTable.m` | CSV tanpa escaping RFC-4180 untuk koma/quote internal | `.m:472` |
| `SPSplitView.m` | Potensi heap overwrite: `resizeProportions[1]` ditulis saat veto resize padahal array dialokasikan `subviewCount` | `.m:852–855` vs `840` |
| `SPSplitView.m` | Off-by-one: terima `subviewIndex == count`, lalu `objectAtIndex:` | `.m:131` vs `205/466` |
| `SPSplitView.m` | Mismatch komentar vs kode: dok ber-"weak" tapi ivar `__unsafe_unretained` | `.m:62` vs `1088` |
| `SPSQLParser.m` | `case ')'` tanpa `break;` → jatuh ke `case '-'` (deteksi comment bisa salah) | `.m:755–759` |
| `SPSQLParser.m` | `readBytesAsString` fallback `.isoLatin1` membuat jalur error decode seolah unreachable | `.m:513` vs `520` |
| `SPTableData.m` | Sentinel `count == 0` membuat hasil kosong memicu fetch ulang | `.m:185, 253, 337, 364` |
| `SPTableData.m` | Cabang nil-syntax tak mencatat negative-cache (hanya `NSLog`) | `.m:1067–1070` |
| `SPQueryController.m` | `numberOfMaxAllowedHistory` di-assign tapi tak dibaca (dead) | `.h:63`, `.m:109` |
| `SPQueryController.m` | Tipe elemen `historyContainer` campur (array vs bridged Swift) | `.m:963` vs `936–939` |
| `SPBundleManager.m` | Bundle "load failure" log pakai `error` enumeration luar, bukan `loadErr` | `.m:414` |
| `SPNavigatorController.m` | `reloadData` off main thread di `reloadAfterFiltering` (potensi pelanggaran AppKit) | `.m:701–702` |
| `SPNavigatorController.m` | `connectionName` di-assign dari `connectionID` → cek mismatch membandingkan dengan dirinya sendiri | `.m:425–428` |
| `SPSSHTunnel.m` | Ivar `requestedResponse` ditulis oleh callback pertanyaan **dan** password (shared-state smell) | `.m:1056` vs `1146` |
| `SPSSHTunnel.m` | `prepareFilePathForSshCommand:` dinamai "escape" tapi hanya percent-decode (misnomer) | `.m:1191–1196` |
| `SPIndexesController.m` | Saat error hapus FK, tak return → eksekusi lanjut ke DROP index | `.m:850–857` |
| `AWSIAMAuthManager.swift` | `clearCachedCredentials` publik tapi tanpa call site di file; cache key hanya nama profile | `.swift:119, 341` |
| `GitHubReleaseManager.swift` | Tak ada verifikasi integritas (checksum/signature) app yang diunduh | `.swift:429–712` |
| `GitHubReleaseManager.swift` | `manager?.isReachable` dibaca tapi tanpa `startListening()` (kemungkinan dead) | `.swift:312, 706` |
| `SecureBookmarkManager.swift` | `revokeBookmark` tak panggil `stopAccessingSecurityScopedResource()` (leak scope, ada FIXME) | `.swift:597–601` |
| `SecureBookmarkManager.swift` | KVO force-unwrap `change.newValue!` bisa crash bila key dihapus eksternal | `.swift:51–54` |
| `VaultOIDCHandler.swift` | Perbandingan `state` CSRF sengaja dimatikan (didelegasikan ke Vault) | `.swift:317–330` |
| `VaultOIDCHandler.swift` | `Equatable` menganggap semua `vaultError` sama → kasar untuk test | `.swift:614` |
| `SADatabaseRenamePlan.swift` | Tak ada penanganan FK lintas database (grep 0); tak ada rollback tabel yang sudah pindah | `.swift:todo` |
| `SADatabaseRenamePlan.swift` | Atribusi kegagalan pakai nama view arbitrer `plan.views[0]`/`[last]` | `.swift:1499, 1540` |
| `SADatabaseRenamePlan.swift` | Doc comment salah tempat di 1977–2011 (TRIGGERS di atas fungsi privilege) | `.swift:1977–2011` |
| `SPConnectionController+service` | Identitas attempt dilacak dua kali (UInt + UInt64) — keduanya harus current | `SAConnectionWindowController.swift:245`, `SAConnectionService.swift:166` |
| `SPEditorPreferencePane.m` | `SPSaveColorScheme` dead; import `tmTheme` diterima tapi parser asumsi plist sendiri | `.m:41`, `168` |
| `SPSQLExporter.m` | Delegate `…WillBeginWritingData:` dikirim dua kali | `.m:452, 470` |
| `StringExtension.swift` | 1093 baris/11 tipe tanpa satu pun `// MARK:` | `.swift` |
| `SADatabaseRenamePlan.swift` | 2596 baris tanpa `// MARK:`; dua tipe >400 baris | `.swift` |
| `MCP cluster` | `mcpResolveConnection` bila endpoint non-`hostPort` melewati cek loopback | `SPMCPServer.swift:270–276` |
| `MCP cluster` | Read-only re-validasi hanya saat ada param (SQL tanpa param mengandalkan dispatcher) | `SPAppController+MCP.swift:526–537` |
| `SPDataStorage.m` | Baris teredit lewati cek bounds kolom → `CFArrayGetValueAtIndex` tak dijaga (UB) | `.m:156, 188, 222` |
| `SPDataStorage.m` | `removeRowsInRange:` loop unsigned underflow (`--i >= location`) saat location 0 | `.m:433–437` (kini tak terjangkau) |
| `SPDataStorage.m` | `replaceObjectInRow:column:withObject:` mutasi setelah keluar lock; tanpa cek bounds kolom | `.m:394–397` |
| `SPTextViewAdditions.m` | `selectEnclosingBrackets:` pakai `co`/`cc` tak terinisialisasi saat tak ada closer | `.m:115, 154–170` |
| `SPStringAdditions.m` | `tickQuotedString` tak escape backslash/NUL/newline (quoting literal MySQL tak aman) | `.m:244–247` |
| `SPStringAdditions.m` | `levenshteinDistanceWithWord:` trim dibuang (komentar klaim trimmed); return CGFloat untuk NSInteger | `.m:449–450, 455` |
| `SPStringAdditions.m` | `nonConsecutivelySearchString:` `NSMakeRange(my.location - prev.length, …)` bisa negatif → NSRangeException | `.m:608–609` |
| `SPFileManagerAdditions.m` | `*errorOut` di-deref setelah guard NULL (crash DEBUG-only) | `.m:89, 128` |
| `SPFileManagerAdditions.m` | Doc klaim baca 5 KB, implementasi 500 byte; cek BOM sebelum verifikasi panjang | `.m:163` vs `179`, `182–197` |
| `SPNetworkPreferencePane.m` | `dealloc` panggil `removeObserver:forKeyPath:` (API KVO) untuk registrasi `NSNotificationCenter` → observer tak terlepas | `.m:82` vs `73` |
| `SPFilePreferencePane.m` | `dealloc` hanya log; tak melepas observer `SPBookmarksChangedNotification` | `.m:180–183` vs `63` |
| `SPPreferenceController.m` | `shortcutItem` nil laten: `SPPreferenceToolbarShortcuts` dirujuk tanpa item/pane | `.m:298, 317, 331, 345` |
| `SPDatabaseData.m` | `cachedCollationsByEncoding`/`defaultStorageEngine` tak direset `resetAllData` | `.m:105–127` |
| `SPServerSupport.m` | `description` memakai `object_getIvar` pada ivar BOOL (primitif) — pemakaian API salah | `.m:145–158` |
| `SPProcessListController.m` | `_killAutoRefreshTimer` invalidate tapi tak pernah nil timer | `.m:491–498` |
| `SPProcessListController.m` | `toggleShowProcessID:` `setHidden:([sender state])` — semantik terbalik | `.m:298` |
| `SPProcessListController.m` | 4 konstanta kill (`SPKill*Key`/`Mode`)/protokol `…DataSource` tak terpakai/tak ada | `.m:42, 43, 47, 48, 728` |
| `SPExtendedTableInfo.m` | `loadTable:` pakai panjang `tableCommentsTextView` sebagai range `tableCreateSyntaxTextView` | `.m:425` vs `268` |
| `SPTableTriggers.m` | Jalur edit `DROP`+`CREATE` bisa kehilangan data bila re-create gagal (diakui in-code) | `.m:265–277` |
| `SPDotExporter.m` | Nama tabel/kolom diinterpolasi tanpa escaping ke DOT/HTML | `.m:144–155` |
| `SPCSVExporter.m` | Cek separator akhir pakai `[csvRow count]` bukan `csvCellCount` | `.m:368` |
| `SPCSVExporter.m` | Deklarasi `@class SPXMLExporter` di `SPDotExporter.h` (mislabel copy-paste) | `.h:37` |
| `SPTooltip.m` | `SPTooltipPreferencesIdentifier` tak dirujuk; `+setDisplayOptions:` body dikomentari (dead) | `.h:40`, `.m:316` |
| `SPFileHandle.m` | Ivar `bufferPosition`/`endOfFile` diinisialisasi tapi tak pernah dipakai | `.m:90–91` |
| `SPLogger.m` | `_outputTimeString` dideklarasikan tanpa caller in-file | `.m:249–254` |
| `SPComboPopupButton.m` | `_drawDotBevelStyleWithGradient:` / `rectForColorViewAtIndex:` switch dead (tak dipanggil/satu body) | `.m:305–339`, `164–176` |
| `SPTableView.m` | `setFont:` tak panggil `[super setFont:]`; `menuForEvent:` banding nama kelas delegate | `.m:295–304, 126–136` |
| `SPWindow.m` | Option+Escape re-use event key asli untuk `menuForEvent:`, bukan event sintetis | `.m:90` |
| `SPBundleCommandTextView.m` | `commentOut` append setelah setSelection (komentar klaim replace); literal drag-mask `4` | `.m:303, 629` |
| `SPEditSheetTextView.m` | `performDragOperation:` + `insertFileContentOfFile:` duplikat verbatim kelas Bundle | `.m:164–218, 248–307` |
| `SPTextViewAdditions.m` | `executeBundleItemForInputField:` deteksi tipe via string nama kelas (`SPTextView`/`SPCopyTable`/suffix `Content`) | `.m:549, 590, 598, 602` |
| `SQLiteHistoryManager.swift` | `startingSchemaVersion` di-assign 2× dan `newSchemaVersion` state instance di closure | `.swift:175–176, 104/111/157` |
| `VaultAuthManager.swift` | Coalescing in-flight tanpa deadline global → waiter poll selamanya bila leader macet | `.swift:208` |
| `SASSHTunnelPeerValidator.swift` | Fail-open: tanpa team identifier → terima semua peer (hanya 1 log) | `.swift:136–154` |
| `SASSHTunnelAuthMessage.swift` | Kegagalan encode JSON ditelan (`try? … ?? Data()` → newline kosong) | `.swift:129` |
| `MGTemplateStandardMarkers.m` | `load` pakai `NSClassFromString` + instansiasi kelas apa pun | `.m:511–527` |
| `SPCSVParser.m` | Ivar `NSScanner *csvScanner` tak dipakai; skip baris kosong via rekursi (risiko kedalaman) | `.h:78`, `.m:290–302` |
| `NoodleLineNumberView.m` | `numberClass`/`maxWidthOfGlyph1`/`RangeOfLineIMP` dead; nomor baris dimatikan >3 jt char | `.m:90, 513/556, 50, 488–492` |
| `YRKSpinningProgressIndicator.m` | `_fadeOutAnimationTimer`/`_isFadingOut`/`poolFlushCounter` vestigial | `.h:46–47`, `.m:180–188` |

## Catatan metodologi

- Line number `Start`/`End` = batas method; `End` = baris sebelum anchor
  berikutnya (termasuk komentar yang mendahului method berikutnya).
- Setiap definisi muncul tepat sekali; deklarasi/prototipe di class-extension
  tidak dihitung sebagai definisi.
- Ini audit **source statis**. Tidak ada klaim perilaku runtime; semua item di
  tabel temuan adalah pembacaan kode, bukan hasil uji.
