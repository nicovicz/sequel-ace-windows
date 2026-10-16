# Inventaris source macOS Sequel Ace (fakta)

Dokumen ini adalah inventaris mekanis seluruh file source aplikasi macOS pada
checkout ini (commit `02c910932`). Angka baris dan daftar file dihasilkan
langsung dari filesystem, bukan perkiraan. Tujuannya menjadi peta kerja 1:1
untuk port Rust di `WindowsPort/`: setiap file macOS perlu punya padanan,
pengganti, atau alasan eksplisit tidak diport.

Cakupan: `Source/` (aplikasi utama). Framework MySQL wire-protocol
(`Frameworks/SPMySQLFramework/`) dan dependensi lain diinventaris terpisah di
dokumen berikutnya. Bahasa: `ObjC` = `.m`, `ObjC header` = `.h`, `Swift` =
`.swift`, `XIB`/`storyboard` = resources UI. `Lapisan` = subdirektori tingkat
pertama di bawah `Source/`.

## Ringkasan

| Metrik | Nilai |
| --- | --- |
| File .m | 155 |
| File .h | 172 |
| File .swift | 175 |
| XIB | 25 |
| storyboard | 1 |
| Total baris .m | 100060 |
| Total baris .h | 14863 |
| Total baris .swift | 41525 |
| Total baris code (m+h+swift) | 156448 |

Baris per lapisan:

| Lapisan | Baris (m+h+swift) |
| --- | --- |
| Controllers | 96096 |
| Interfaces | 0 |
| Model | 5608 |
| Other | 30945 |
| Protocols | 218 |
| ThirdParty | 6337 |
| Views | 17138 |

## Tabel lengkap per file

Diurutkan per lapisan lalu per path. Baris = jumlah baris file.

| Baris | File | Bahasa | Lapisan |
| --- | --- | --- | --- |
| 25 | Source/Controllers/BundleSupport/SABundleHTMLOutputView.swift | Swift | Controllers |
| 385 | Source/Controllers/BundleSupport/SABundleHTMLOutputWindowController.swift | Swift | Controllers |
| 180 | Source/Controllers/BundleSupport/SABundleJSBridge.swift | Swift | Controllers |
| 45 | Source/Controllers/BundleSupport/SABundleRunner.h | ObjC header | Controllers |
| 73 | Source/Controllers/BundleSupport/SABundleRunner.m | ObjC | Controllers |
| 36 | Source/Controllers/BundleSupport/SPBundleCommandRunner.h | ObjC header | Controllers |
| 509 | Source/Controllers/BundleSupport/SPBundleCommandRunner.m | ObjC | Controllers |
| 146 | Source/Controllers/BundleSupport/SPBundleEditorController.h | ObjC header | Controllers |
| 2077 | Source/Controllers/BundleSupport/SPBundleEditorController.m | ObjC | Controllers |
| 91 | Source/Controllers/DataControllers/SPDatabaseData.h | ObjC header | Controllers |
| 594 | Source/Controllers/DataControllers/SPDatabaseData.m | ObjC | Controllers |
| 68 | Source/Controllers/DataControllers/SPDatabaseStructure.h | ObjC header | Controllers |
| 722 | Source/Controllers/DataControllers/SPDatabaseStructure.m | ObjC | Controllers |
| 73 | Source/Controllers/DataControllers/SPFavoritesController.h | ObjC header | Controllers |
| 494 | Source/Controllers/DataControllers/SPFavoritesController.m | ObjC | Controllers |
| 106 | Source/Controllers/DataControllers/SPServerSupport.h | ObjC header | Controllers |
| 234 | Source/Controllers/DataControllers/SPServerSupport.m | ObjC | Controllers |
| 88 | Source/Controllers/DataControllers/SPTableData.h | ObjC header | Controllers |
| 1637 | Source/Controllers/DataControllers/SPTableData.m | ObjC | Controllers |
| 111 | Source/Controllers/DataExport/Exporters/SPCSVExporter.h | ObjC header | Controllers |
| 406 | Source/Controllers/DataExport/Exporters/SPCSVExporter.m | ObjC | Controllers |
| 70 | Source/Controllers/DataExport/Exporters/SPCSVExporterProtocol.h | ObjC header | Controllers |
| 100 | Source/Controllers/DataExport/Exporters/SPDotExporter.h | ObjC header | Controllers |
| 240 | Source/Controllers/DataExport/Exporters/SPDotExporter.m | ObjC | Controllers |
| 78 | Source/Controllers/DataExport/Exporters/SPDotExporterProtocol.h | ObjC header | Controllers |
| 152 | Source/Controllers/DataExport/Exporters/SPExporter.h | ObjC header | Controllers |
| 135 | Source/Controllers/DataExport/Exporters/SPExporter.m | ObjC | Controllers |
| 50 | Source/Controllers/DataExport/Exporters/SPHTMLExporter.h | ObjC header | Controllers |
| 56 | Source/Controllers/DataExport/Exporters/SPHTMLExporter.m | ObjC | Controllers |
| 63 | Source/Controllers/DataExport/Exporters/SPHTMLExporterProtocol.h | ObjC header | Controllers |
| 50 | Source/Controllers/DataExport/Exporters/SPPDFExporter.h | ObjC header | Controllers |
| 56 | Source/Controllers/DataExport/Exporters/SPPDFExporter.m | ObjC | Controllers |
| 63 | Source/Controllers/DataExport/Exporters/SPPDFExporterProtocol.h | ObjC header | Controllers |
| 146 | Source/Controllers/DataExport/Exporters/SPSQLExporter.h | ObjC header | Controllers |
| 1028 | Source/Controllers/DataExport/Exporters/SPSQLExporter.m | ObjC | Controllers |
| 77 | Source/Controllers/DataExport/Exporters/SPSQLExporterProtocol.h | ObjC header | Controllers |
| 92 | Source/Controllers/DataExport/Exporters/SPXMLExporter.h | ObjC header | Controllers |
| 342 | Source/Controllers/DataExport/Exporters/SPXMLExporter.m | ObjC | Controllers |
| 70 | Source/Controllers/DataExport/Exporters/SPXMLExporterProtocol.h | ObjC header | Controllers |
| 101 | Source/Controllers/DataExport/Model/SPExportFile.h | ObjC header | Controllers |
| 238 | Source/Controllers/DataExport/Model/SPExportFile.m | ObjC | Controllers |
| 50 | Source/Controllers/DataExport/SAExportOutputEncoding.swift | Swift | Controllers |
| 357 | Source/Controllers/DataExport/SPExportController.h | ObjC header | Controllers |
| 3982 | Source/Controllers/DataExport/SPExportController.m | ObjC | Controllers |
| 40 | Source/Controllers/DataExport/SPExportFileNameTokenObject.h | ObjC header | Controllers |
| 74 | Source/Controllers/DataExport/SPExportFileNameTokenObject.m | ObjC | Controllers |
| 42 | Source/Controllers/DataExport/SPExportUtilities.h | ObjC header | Controllers |
| 50 | Source/Controllers/DataExport/SPExportUtilities.m | ObjC | Controllers |
| 211 | Source/Controllers/DataImport/SACSVImportStreamDecoder.swift | Swift | Controllers |
| 85 | Source/Controllers/DataImport/SAImportProgressReporter.swift | Swift | Controllers |
| 151 | Source/Controllers/DataImport/SPDataImport.h | ObjC header | Controllers |
| 1760 | Source/Controllers/DataImport/SPDataImport.m | ObjC | Controllers |
| 213 | Source/Controllers/DataImport/SPFieldMapperController.h | ObjC header | Controllers |
| 2140 | Source/Controllers/DataImport/SPFieldMapperController.m | ObjC | Controllers |
| 127 | Source/Controllers/MCP/SAMCPToolDefinitions.swift | Swift | Controllers |
| 963 | Source/Controllers/MCP/SPAppController+MCP.swift | Swift | Controllers |
| 53 | Source/Controllers/MCP/SPMCPFavorite.swift | Swift | Controllers |
| 552 | Source/Controllers/MCP/SPMCPHTTP.swift | Swift | Controllers |
| 893 | Source/Controllers/MCP/SPMCPServer.swift | Swift | Controllers |
| 47 | Source/Controllers/MainViewControllers/ConnectionView/ImportExport/SPFavoritesExportProtocol.h | ObjC header | Controllers |
| 53 | Source/Controllers/MainViewControllers/ConnectionView/ImportExport/SPFavoritesExporter.h | ObjC header | Controllers |
| 113 | Source/Controllers/MainViewControllers/ConnectionView/ImportExport/SPFavoritesExporter.m | ObjC | Controllers |
| 54 | Source/Controllers/MainViewControllers/ConnectionView/ImportExport/SPFavoritesImportProtocol.h | ObjC header | Controllers |
| 47 | Source/Controllers/MainViewControllers/ConnectionView/ImportExport/SPFavoritesImporter.h | ObjC header | Controllers |
| 134 | Source/Controllers/MainViewControllers/ConnectionView/ImportExport/SPFavoritesImporter.m | ObjC | Controllers |
| 320 | Source/Controllers/MainViewControllers/ConnectionView/SAConnectionDetailsValidator.swift | Swift | Controllers |
| 101 | Source/Controllers/MainViewControllers/ConnectionView/SAConnectionFormHelpers.swift | Swift | Controllers |
| 540 | Source/Controllers/MainViewControllers/ConnectionView/SAConnectionFormModel.swift | Swift | Controllers |
| 617 | Source/Controllers/MainViewControllers/ConnectionView/SAConnectionFormView.swift | Swift | Controllers |
| 47 | Source/Controllers/MainViewControllers/ConnectionView/SAConnectionViewCoordinator.swift | Swift | Controllers |
| 773 | Source/Controllers/MainViewControllers/ConnectionView/SAConnectionWindowController.swift | Swift | Controllers |
| 61 | Source/Controllers/MainViewControllers/ConnectionView/SAFavoriteDeletionPrompt.swift | Swift | Controllers |
| 214 | Source/Controllers/MainViewControllers/ConnectionView/SAFavoriteDuplicateMatcher.swift | Swift | Controllers |
| 98 | Source/Controllers/MainViewControllers/ConnectionView/SAFavoriteItem+Tree.swift | Swift | Controllers |
| 136 | Source/Controllers/MainViewControllers/ConnectionView/SAFavoriteItem.swift | Swift | Controllers |
| 56 | Source/Controllers/MainViewControllers/ConnectionView/SAFavoriteSearchMatcher.swift | Swift | Controllers |
| 67 | Source/Controllers/MainViewControllers/ConnectionView/SAFavoriteSearchTreeWalker.swift | Swift | Controllers |
| 102 | Source/Controllers/MainViewControllers/ConnectionView/SAFavoritesList.swift | Swift | Controllers |
| 449 | Source/Controllers/MainViewControllers/ConnectionView/SAFavoritesListDataSource.swift | Swift | Controllers |
| 35 | Source/Controllers/MainViewControllers/ConnectionView/SAFavoritesListDelegate.swift | Swift | Controllers |
| 196 | Source/Controllers/MainViewControllers/ConnectionView/SAFavoritesListView.swift | Swift | Controllers |
| 411 | Source/Controllers/MainViewControllers/ConnectionView/SPConnectionController.h | ObjC header | Controllers |
| 5438 | Source/Controllers/MainViewControllers/ConnectionView/SPConnectionController.m | ObjC | Controllers |
| 56 | Source/Controllers/MainViewControllers/ConnectionView/SPConnectionControllerDelegateProtocol.h | ObjC header | Controllers |
| 158 | Source/Controllers/MainViewControllers/ConnectionView/SPDuplicateImportViewController.swift | Swift | Controllers |
| 490 | Source/Controllers/MainViewControllers/SADatabaseListManager.swift | Swift | Controllers |
| 254 | Source/Controllers/MainViewControllers/SAQueryFavoriteSaveWindowController.swift | Swift | Controllers |
| 138 | Source/Controllers/MainViewControllers/SAQueryFavoriteStore.swift | Swift | Controllers |
| 395 | Source/Controllers/MainViewControllers/SATaskController.swift | Swift | Controllers |
| 42 | Source/Controllers/MainViewControllers/SPBracketHighlighter.h | ObjC header | Controllers |
| 114 | Source/Controllers/MainViewControllers/SPBracketHighlighter.m | ObjC | Controllers |
| 40 | Source/Controllers/MainViewControllers/SPBrackets.h | ObjC header | Controllers |
| 157 | Source/Controllers/MainViewControllers/SPBrackets.m | ObjC | Controllers |
| 117 | Source/Controllers/MainViewControllers/SPCustomQuery+Explain.swift | Swift | Controllers |
| 264 | Source/Controllers/MainViewControllers/SPCustomQuery.h | ObjC header | Controllers |
| 4079 | Source/Controllers/MainViewControllers/SPCustomQuery.m | ObjC | Controllers |
| 623 | Source/Controllers/MainViewControllers/SPCustomQuerySQLClassifier.swift | Swift | Controllers |
| 181 | Source/Controllers/MainViewControllers/SPDatabaseDocument+ViewMode.swift | Swift | Controllers |
| 483 | Source/Controllers/MainViewControllers/SPDatabaseDocument.h | ObjC header | Controllers |
| 6436 | Source/Controllers/MainViewControllers/SPDatabaseDocument.m | ObjC | Controllers |
| 86 | Source/Controllers/MainViewControllers/SPDatabaseDocument.swift | Swift | Controllers |
| 91 | Source/Controllers/MainViewControllers/SPExtendedTableInfo.h | ObjC header | Controllers |
| 738 | Source/Controllers/MainViewControllers/SPExtendedTableInfo.m | ObjC | Controllers |
| 201 | Source/Controllers/MainViewControllers/TableContent/SACellFilterAction.swift | Swift | Controllers |
| 75 | Source/Controllers/MainViewControllers/TableContent/SACellFilterColumnIdentifier.swift | Swift | Controllers |
| 333 | Source/Controllers/MainViewControllers/TableContent/SACellFilterMenuBuilder.swift | Swift | Controllers |
| 122 | Source/Controllers/MainViewControllers/TableContent/SACellFilterMerge.swift | Swift | Controllers |
| 206 | Source/Controllers/MainViewControllers/TableContent/SACellFilterOperator.swift | Swift | Controllers |
| 247 | Source/Controllers/MainViewControllers/TableContent/SATableReloadRetryPolicy.swift | Swift | Controllers |
| 278 | Source/Controllers/MainViewControllers/TableContent/SPTableContent.h | ObjC header | Controllers |
| 5656 | Source/Controllers/MainViewControllers/TableContent/SPTableContent.m | ObjC | Controllers |
| 218 | Source/Controllers/MainViewControllers/TableRelations/SAForeignKeyReferenceRuleSupport.swift | Swift | Controllers |
| 96 | Source/Controllers/MainViewControllers/TableRelations/SPTableRelations.h | ObjC header | Controllers |
| 761 | Source/Controllers/MainViewControllers/TableRelations/SPTableRelations.m | ObjC | Controllers |
| 53 | Source/Controllers/MainViewControllers/TableStructure/SAAutoIncrementRuleSupport.swift | Swift | Controllers |
| 47 | Source/Controllers/MainViewControllers/TableStructure/SPTableFieldValidation.h | ObjC header | Controllers |
| 134 | Source/Controllers/MainViewControllers/TableStructure/SPTableFieldValidation.m | ObjC | Controllers |
| 159 | Source/Controllers/MainViewControllers/TableStructure/SPTableStructure.h | ObjC header | Controllers |
| 3180 | Source/Controllers/MainViewControllers/TableStructure/SPTableStructure.m | ObjC | Controllers |
| 122 | Source/Controllers/MainViewControllers/TableStructure/TableSortHelper.swift | Swift | Controllers |
| 89 | Source/Controllers/MainViewControllers/TableTriggers/SPTableTriggers.h | ObjC header | Controllers |
| 694 | Source/Controllers/MainViewControllers/TableTriggers/SPTableTriggers.m | ObjC | Controllers |
| 551 | Source/Controllers/Other/GitHub.swift | Swift | Controllers |
| 737 | Source/Controllers/Other/GitHubReleaseManager.swift | Swift | Controllers |
| 25 | Source/Controllers/Other/ProgressViewController.swift | Swift | Controllers |
| 112 | Source/Controllers/Other/ProgressWindowController.storyboard | storyboard | Controllers |
| 49 | Source/Controllers/Other/ProgressWindowController.swift | Swift | Controllers |
| 59 | Source/Controllers/Other/ReportExceptionApplication.m | ObjC | Controllers |
| 109 | Source/Controllers/Other/SASQLiteSharedStoreLocation.swift | Swift | Controllers |
| 51 | Source/Controllers/Other/SPBundleManager.h | ObjC header | Controllers |
| 1119 | Source/Controllers/Other/SPBundleManager.m | ObjC | Controllers |
| 45 | Source/Controllers/Other/SPBundleManagerAdditions.swift | Swift | Controllers |
| 33 | Source/Controllers/Other/SPDocumentController.h | ObjC header | Controllers |
| 66 | Source/Controllers/Other/SPDocumentController.m | ObjC | Controllers |
| 67 | Source/Controllers/Other/SPHistoryController.h | ObjC header | Controllers |
| 499 | Source/Controllers/Other/SPHistoryController.m | ObjC | Controllers |
| 161 | Source/Controllers/Other/SPTableHistoryManager.swift | Swift | Controllers |
| 471 | Source/Controllers/Other/SQLiteDisplayFormatManager.swift | Swift | Controllers |
| 491 | Source/Controllers/Other/SQLiteHistoryManager.swift | Swift | Controllers |
| 460 | Source/Controllers/Other/SQLitePinnedTableManager.swift | Swift | Controllers |
| 65 | Source/Controllers/Other/SecureBookmark.swift | Swift | Controllers |
| 81 | Source/Controllers/Other/SecureBookmarkData.swift | Swift | Controllers |
| 664 | Source/Controllers/Other/SecureBookmarkManager.swift | Swift | Controllers |
| 84 | Source/Controllers/Preferences/Panes/SPEditorPreferencePane.h | ObjC header | Controllers |
| 1018 | Source/Controllers/Preferences/Panes/SPEditorPreferencePane.m | ObjC | Controllers |
| 56 | Source/Controllers/Preferences/Panes/SPFilePreferencePane.h | ObjC header | Controllers |
| 601 | Source/Controllers/Preferences/Panes/SPFilePreferencePane.m | ObjC | Controllers |
| 54 | Source/Controllers/Preferences/Panes/SPGeneralPreferencePane.h | ObjC header | Controllers |
| 258 | Source/Controllers/Preferences/Panes/SPGeneralPreferencePane.m | ObjC | Controllers |
| 377 | Source/Controllers/Preferences/Panes/SPMCPPreferencePane.swift | Swift | Controllers |
| 71 | Source/Controllers/Preferences/Panes/SPNetworkPreferencePane.h | ObjC header | Controllers |
| 776 | Source/Controllers/Preferences/Panes/SPNetworkPreferencePane.m | ObjC | Controllers |
| 44 | Source/Controllers/Preferences/Panes/SPNotificationsPreferencePane.h | ObjC header | Controllers |
| 90 | Source/Controllers/Preferences/Panes/SPNotificationsPreferencePane.m | ObjC | Controllers |
| 48 | Source/Controllers/Preferences/Panes/SPPreferencePane.h | ObjC header | Controllers |
| 73 | Source/Controllers/Preferences/Panes/SPPreferencePane.m | ObjC | Controllers |
| 42 | Source/Controllers/Preferences/Panes/SPTablesPreferencePane.h | ObjC header | Controllers |
| 72 | Source/Controllers/Preferences/Panes/SPTablesPreferencePane.m | ObjC | Controllers |
| 97 | Source/Controllers/Preferences/SPPreferenceController.h | ObjC header | Controllers |
| 352 | Source/Controllers/Preferences/SPPreferenceController.m | ObjC | Controllers |
| 80 | Source/Controllers/Preferences/SPPreferencePaneProtocol.h | ObjC header | Controllers |
| 44 | Source/Controllers/Preferences/SPPreferencesUpgrade.h | ObjC header | Controllers |
| 120 | Source/Controllers/Preferences/SPPreferencesUpgrade.m | ObjC | Controllers |
| 114 | Source/Controllers/SPAppController.h | ObjC header | Controllers |
| 1806 | Source/Controllers/SPAppController.m | ObjC | Controllers |
| 289 | Source/Controllers/SPAppController.swift | Swift | Controllers |
| 314 | Source/Controllers/SubviewControllers/HelpViewer/SAHelpViewerModel.swift | Swift | Controllers |
| 136 | Source/Controllers/SubviewControllers/HelpViewer/SAHelpViewerOnlineURLBuilder.swift | Swift | Controllers |
| 183 | Source/Controllers/SubviewControllers/HelpViewer/SAHelpViewerView.swift | Swift | Controllers |
| 380 | Source/Controllers/SubviewControllers/HelpViewer/SAHelpViewerWindowController.swift | Swift | Controllers |
| 103 | Source/Controllers/SubviewControllers/HelpViewer/SPHelpViewerClient.h | ObjC header | Controllers |
| 300 | Source/Controllers/SubviewControllers/HelpViewer/SPHelpViewerClient.m | ObjC | Controllers |
| 111 | Source/Controllers/SubviewControllers/SAAboutView.swift | Swift | Controllers |
| 24 | Source/Controllers/SubviewControllers/SAAboutWindowController.swift | Swift | Controllers |
| 65 | Source/Controllers/SubviewControllers/SAQueryHistoryMerger.swift | Swift | Controllers |
| 302 | Source/Controllers/SubviewControllers/SARuleFilterRootConjunction.swift | Swift | Controllers |
| 19 | Source/Controllers/SubviewControllers/SAUserManagerPrivilegeNormalizer.swift | Swift | Controllers |
| 23 | Source/Controllers/SubviewControllers/SPAutosizingTextView.swift | Swift | Controllers |
| 139 | Source/Controllers/SubviewControllers/SPCharsetCollationHelper.h | ObjC header | Controllers |
| 289 | Source/Controllers/SubviewControllers/SPCharsetCollationHelper.m | ObjC | Controllers |
| 82 | Source/Controllers/SubviewControllers/SPContentFilterManager.h | ObjC header | Controllers |
| 886 | Source/Controllers/SubviewControllers/SPContentFilterManager.m | ObjC | Controllers |
| 711 | Source/Controllers/SubviewControllers/SPFieldEditorController+PHPSerialized.swift | Swift | Controllers |
| 263 | Source/Controllers/SubviewControllers/SPFieldEditorController.h | ObjC header | Controllers |
| 1532 | Source/Controllers/SubviewControllers/SPFieldEditorController.m | ObjC | Controllers |
| 129 | Source/Controllers/SubviewControllers/SPFilterTableController.h | ObjC header | Controllers |
| 716 | Source/Controllers/SubviewControllers/SPFilterTableController.m | ObjC | Controllers |
| 91 | Source/Controllers/SubviewControllers/SPGotoDatabaseController.h | ObjC header | Controllers |
| 454 | Source/Controllers/SubviewControllers/SPGotoDatabaseController.m | ObjC | Controllers |
| 112 | Source/Controllers/SubviewControllers/SPIndexesController.h | ObjC header | Controllers |
| 1027 | Source/Controllers/SubviewControllers/SPIndexesController.m | ObjC | Controllers |
| 98 | Source/Controllers/SubviewControllers/SPNavigatorController.h | ObjC header | Controllers |
| 1309 | Source/Controllers/SubviewControllers/SPNavigatorController.m | ObjC | Controllers |
| 73 | Source/Controllers/SubviewControllers/SPProcessListController.h | ObjC header | Controllers |
| 802 | Source/Controllers/SubviewControllers/SPProcessListController.m | ObjC | Controllers |
| 149 | Source/Controllers/SubviewControllers/SPQueryController.h | ObjC header | Controllers |
| 1176 | Source/Controllers/SubviewControllers/SPQueryController.m | ObjC | Controllers |
| 81 | Source/Controllers/SubviewControllers/SPQueryFavoriteManager.h | ObjC header | Controllers |
| 885 | Source/Controllers/SubviewControllers/SPQueryFavoriteManager.m | ObjC | Controllers |
| 278 | Source/Controllers/SubviewControllers/SPRuleFilterController.h | ObjC header | Controllers |
| 2481 | Source/Controllers/SubviewControllers/SPRuleFilterController.m | ObjC | Controllers |
| 57 | Source/Controllers/SubviewControllers/SPServerVariablesController.h | ObjC header | Controllers |
| 401 | Source/Controllers/SubviewControllers/SPServerVariablesController.m | ObjC | Controllers |
| 56 | Source/Controllers/SubviewControllers/SPTableInfo.h | ObjC header | Controllers |
| 525 | Source/Controllers/SubviewControllers/SPTableInfo.m | ObjC | Controllers |
| 207 | Source/Controllers/SubviewControllers/SPTablesList.h | ObjC header | Controllers |
| 3043 | Source/Controllers/SubviewControllers/SPTablesList.m | ObjC | Controllers |
| 279 | Source/Controllers/SubviewControllers/SPUserManager.h | ObjC header | Controllers |
| 2001 | Source/Controllers/SubviewControllers/SPUserManager.m | ObjC | Controllers |
| 58 | Source/Controllers/Window/NotificationToken.swift | Swift | Controllers |
| 121 | Source/Controllers/Window/SAWindowTitleBuilder.swift | Swift | Controllers |
| 60 | Source/Controllers/Window/SAWindowTitlebarTint.swift | Swift | Controllers |
| 241 | Source/Controllers/Window/SPWindowController.swift | Swift | Controllers |
| 183 | Source/Controllers/Window/TabManager.swift | Swift | Controllers |
| 844 | Source/Interfaces/BundleEditor.xib | XIB | Interfaces |
| 3766 | Source/Interfaces/ConnectionView.xib | XIB | Interfaces |
| 339 | Source/Interfaces/Console.xib | XIB | Interfaces |
| 415 | Source/Interfaces/ContentFilterManager.xib | XIB | Interfaces |
| 145 | Source/Interfaces/ContentPaginationView.xib | XIB | Interfaces |
| 4968 | Source/Interfaces/DBView.xib | XIB | Interfaces |
| 944 | Source/Interfaces/DataMigrationDialog.xib | XIB | Interfaces |
| 446 | Source/Interfaces/DatabaseProcessList.xib | XIB | Interfaces |
| 176 | Source/Interfaces/DatabaseServerVariables.xib | XIB | Interfaces |
| 49 | Source/Interfaces/EncodingPopupView.xib | XIB | Interfaces |
| 1135 | Source/Interfaces/ExportDialog.xib | XIB | Interfaces |
| 1568 | Source/Interfaces/FieldEditorSheet.xib | XIB | Interfaces |
| 336 | Source/Interfaces/FilterTableWindow.xib | XIB | Interfaces |
| 134 | Source/Interfaces/GotoDatabaseDialog.xib | XIB | Interfaces |
| 455 | Source/Interfaces/ImportAccessory.xib | XIB | Interfaces |
| 328 | Source/Interfaces/IndexesView.xib | XIB | Interfaces |
| 1045 | Source/Interfaces/MainMenu.xib | XIB | Interfaces |
| 28 | Source/Interfaces/MainWindow.xib | XIB | Interfaces |
| 228 | Source/Interfaces/Navigator.xib | XIB | Interfaces |
| 2750 | Source/Interfaces/Preferences.xib | XIB | Interfaces |
| 102 | Source/Interfaces/ProgressIndicatorLayer.xib | XIB | Interfaces |
| 509 | Source/Interfaces/QueryFavoriteManager.xib | XIB | Interfaces |
| 188 | Source/Interfaces/SSHQuestionDialog.xib | XIB | Interfaces |
| 117 | Source/Interfaces/SaveSPFAccessory.xib | XIB | Interfaces |
| 1686 | Source/Interfaces/UserManagerView.xib | XIB | Interfaces |
| 206 | Source/Model/ConnectionStringParser.swift | Swift | Model |
| 41 | Source/Model/CoreData/SPPrivilegesMO.h | ObjC header | Model |
| 49 | Source/Model/CoreData/SPPrivilegesMO.m | ObjC | Model |
| 47 | Source/Model/CoreData/SPUserMO.h | ObjC header | Model |
| 115 | Source/Model/CoreData/SPUserMO.m | ObjC | Model |
| 55 | Source/Model/SACompletionWindowLayout.swift | Swift | Model |
| 250 | Source/Model/SAConnectionInfo+ConnectionString.swift | Swift | Model |
| 264 | Source/Model/SAConnectionInfo+Favorite.swift | Swift | Model |
| 410 | Source/Model/SAConnectionInfo.swift | Swift | Model |
| 762 | Source/Model/SAPHPSerializedValue.swift | Swift | Model |
| 328 | Source/Model/SATooltipDismissal.swift | Swift | Model |
| 48 | Source/Model/SPChooseMenuItemDialog.h | ObjC header | Model |
| 172 | Source/Model/SPChooseMenuItemDialog.m | ObjC | Model |
| 57 | Source/Model/SPConsoleMessage.h | ObjC header | Model |
| 64 | Source/Model/SPConsoleMessage.m | ObjC | Model |
| 110 | Source/Model/SPNarrowDownCompletion.h | ObjC header | Model |
| 1165 | Source/Model/SPNarrowDownCompletion.m | ObjC | Model |
| 33 | Source/Model/SPSingleton.h | ObjC header | Model |
| 44 | Source/Model/SPSingleton.m | ObjC | Model |
| 57 | Source/Model/SPTooltip.h | ObjC header | Model |
| 529 | Source/Model/SPTooltip.m | ObjC | Model |
| 63 | Source/Model/TreeNodes/SPFavoriteNode.h | ObjC header | Model |
| 111 | Source/Model/TreeNodes/SPFavoriteNode.m | ObjC | Model |
| 63 | Source/Model/TreeNodes/SPGroupNode.h | ObjC header | Model |
| 128 | Source/Model/TreeNodes/SPGroupNode.m | ObjC | Model |
| 37 | Source/Model/TreeNodes/SPNamedNode.h | ObjC header | Model |
| 64 | Source/Model/TreeNodes/SPTreeNode.h | ObjC header | Model |
| 336 | Source/Model/TreeNodes/SPTreeNode.m | ObjC | Model |
| 479 | Source/Other/AWS/AWSCredentials.swift | Swift | Other |
| 391 | Source/Other/AWS/AWSDirectoryBookmarkManager.swift | Swift | Other |
| 967 | Source/Other/AWS/AWSIAMAuthManager.swift | Swift | Other |
| 307 | Source/Other/AWS/AWSLoginCredentialsProvider.swift | Swift | Other |
| 156 | Source/Other/AWS/AWSMFATokenDialog.swift | Swift | Other |
| 380 | Source/Other/AWS/AWSSSOClient.swift | Swift | Other |
| 577 | Source/Other/AWS/AWSSTSClient.swift | Swift | Other |
| 366 | Source/Other/AWS/RDSIAMAuthentication.swift | Swift | Other |
| 77 | Source/Other/AWS/SAAWSDirectoryWriteAccessPrompt.swift | Swift | Other |
| 525 | Source/Other/AWS/SAAWSLoginRefresh.swift | Swift | Other |
| 115 | Source/Other/AWS/SAAWSSignInCommand.swift | Swift | Other |
| 61 | Source/Other/CategoryAdditions/HyperlinkTextField.swift | Swift | Other |
| 98 | Source/Other/CategoryAdditions/SPArrayAdditions.h | ObjC header | Other |
| 159 | Source/Other/CategoryAdditions/SPArrayAdditions.m | ObjC | Other |
| 56 | Source/Other/CategoryAdditions/SPCategoryAdditions.h | ObjC header | Other |
| 37 | Source/Other/CategoryAdditions/SPColorAdditions.h | ObjC header | Other |
| 96 | Source/Other/CategoryAdditions/SPColorAdditions.m | ObjC | Other |
| 59 | Source/Other/CategoryAdditions/SPDataAdditions.h | ObjC header | Other |
| 633 | Source/Other/CategoryAdditions/SPDataAdditions.m | ObjC | Other |
| 36 | Source/Other/CategoryAdditions/SPDataBase64EncodingAdditions.h | ObjC header | Other |
| 126 | Source/Other/CategoryAdditions/SPDataBase64EncodingAdditions.m | ObjC | Other |
| 124 | Source/Other/CategoryAdditions/SPDataStorage.h | ObjC header | Other |
| 590 | Source/Other/CategoryAdditions/SPDataStorage.m | ObjC | Other |
| 35 | Source/Other/CategoryAdditions/SPDateAdditions.h | ObjC header | Other |
| 55 | Source/Other/CategoryAdditions/SPDateAdditions.m | ObjC | Other |
| 37 | Source/Other/CategoryAdditions/SPFileManagerAdditions.h | ObjC header | Other |
| 222 | Source/Other/CategoryAdditions/SPFileManagerAdditions.m | ObjC | Other |
| 55 | Source/Other/CategoryAdditions/SPMainThreadTrampoline.h | ObjC header | Other |
| 162 | Source/Other/CategoryAdditions/SPMainThreadTrampoline.m | ObjC | Other |
| 59 | Source/Other/CategoryAdditions/SPMutableArrayAdditions.h | ObjC header | Other |
| 124 | Source/Other/CategoryAdditions/SPMutableArrayAdditions.m | ObjC | Other |
| 29 | Source/Other/CategoryAdditions/SPNSMutableDictionaryAdditions.h | ObjC header | Other |
| 52 | Source/Other/CategoryAdditions/SPNSMutableDictionaryAdditions.m | ObjC | Other |
| 42 | Source/Other/CategoryAdditions/SPNotLoaded.h | ObjC header | Other |
| 91 | Source/Other/CategoryAdditions/SPNotLoaded.m | ObjC | Other |
| 54 | Source/Other/CategoryAdditions/SPObjectAdditions.h | ObjC header | Other |
| 101 | Source/Other/CategoryAdditions/SPObjectAdditions.m | ObjC | Other |
| 19 | Source/Other/CategoryAdditions/SPPointerArrayAdditions.h | ObjC header | Other |
| 22 | Source/Other/CategoryAdditions/SPPointerArrayAdditions.m | ObjC | Other |
| 44 | Source/Other/CategoryAdditions/SPScreenAdditions.h | ObjC header | Other |
| 54 | Source/Other/CategoryAdditions/SPScreenAdditions.m | ObjC | Other |
| 132 | Source/Other/CategoryAdditions/SPStringAdditions.h | ObjC header | Other |
| 683 | Source/Other/CategoryAdditions/SPStringAdditions.m | ObjC | Other |
| 21 | Source/Other/CategoryAdditions/SPTaskAdditions.h | ObjC header | Other |
| 43 | Source/Other/CategoryAdditions/SPTaskAdditions.m | ObjC | Other |
| 57 | Source/Other/CategoryAdditions/SPTextViewAdditions.h | ObjC header | Other |
| 834 | Source/Other/CategoryAdditions/SPTextViewAdditions.m | ObjC | Other |
| 59 | Source/Other/CategoryAdditions/SPThreadAdditions.h | ObjC header | Other |
| 107 | Source/Other/CategoryAdditions/SPThreadAdditions.m | ObjC | Other |
| 17 | Source/Other/CategoryAdditions/SPURLAdditions.h | ObjC header | Other |
| 30 | Source/Other/CategoryAdditions/SPURLAdditions.m | ObjC | Other |
| 35 | Source/Other/CategoryAdditions/SPWindowAdditions.h | ObjC header | Other |
| 87 | Source/Other/CategoryAdditions/SPWindowAdditions.m | ObjC | Other |
| 812 | Source/Other/Data/SPConstants.h | ObjC header | Other |
| 535 | Source/Other/Data/SPConstants.m | ObjC | Other |
| 2596 | Source/Other/DatabaseActions/SADatabaseRenamePlan.swift | Swift | Other |
| 42 | Source/Other/DatabaseActions/SPCreateDatabaseInfo.h | ObjC header | Other |
| 40 | Source/Other/DatabaseActions/SPCreateDatabaseInfo.m | ObjC | Other |
| 74 | Source/Other/DatabaseActions/SPDatabaseAction.h | ObjC header | Other |
| 74 | Source/Other/DatabaseActions/SPDatabaseAction.m | ObjC | Other |
| 48 | Source/Other/DatabaseActions/SPDatabaseCopy.h | ObjC header | Other |
| 68 | Source/Other/DatabaseActions/SPDatabaseCopy.m | ObjC | Other |
| 79 | Source/Other/DatabaseActions/SPDatabaseRename.h | ObjC header | Other |
| 129 | Source/Other/DatabaseActions/SPDatabaseRename.m | ObjC | Other |
| 84 | Source/Other/DatabaseActions/SPTableCopy.h | ObjC header | Other |
| 165 | Source/Other/DatabaseActions/SPTableCopy.m | ObjC | Other |
| 37 | Source/Other/DatabaseActions/SPViewCopy.h | ObjC header | Other |
| 86 | Source/Other/DatabaseActions/SPViewCopy.m | ObjC | Other |
| 61 | Source/Other/DebuggingSupport/SPFavoriteColorSupport.h | ObjC header | Other |
| 131 | Source/Other/DebuggingSupport/SPFavoriteColorSupport.m | ObjC | Other |
| 84 | Source/Other/DebuggingSupport/SPLogger.h | ObjC header | Other |
| 261 | Source/Other/DebuggingSupport/SPLogger.m | ObjC | Other |
| 123 | Source/Other/Extensions/BundleExtension.swift | Swift | Other |
| 33 | Source/Other/Extensions/ByteCountFormatterExtension.swift | Swift | Other |
| 72 | Source/Other/Extensions/CollectionExtension.swift | Swift | Other |
| 21 | Source/Other/Extensions/DateComponentsFormatterExtension.swift | Swift | Other |
| 43 | Source/Other/Extensions/DateExtension.swift | Swift | Other |
| 136 | Source/Other/Extensions/DateFormatterExtension.swift | Swift | Other |
| 45 | Source/Other/Extensions/DispatchQueueExtension.swift | Swift | Other |
| 22 | Source/Other/Extensions/FileManagerExtension.swift | Swift | Other |
| 375 | Source/Other/Extensions/NSAlertExtension.swift | Swift | Other |
| 44 | Source/Other/Extensions/NSDictionaryExtension.swift | Swift | Other |
| 30 | Source/Other/Extensions/NSImageExtensions.swift | Swift | Other |
| 19 | Source/Other/Extensions/NSViewExtension.swift | Swift | Other |
| 24 | Source/Other/Extensions/NumberFormatterExtension.swift | Swift | Other |
| 32 | Source/Other/Extensions/PopupButtonExtensions.swift | Swift | Other |
| 86 | Source/Other/Extensions/SAArchiving.swift | Swift | Other |
| 1093 | Source/Other/Extensions/StringExtension.swift | Swift | Other |
| 38 | Source/Other/Extensions/StringRegexExtension.swift | Swift | Other |
| 52 | Source/Other/Extensions/UserDefaultsExtension.swift | Swift | Other |
| 105 | Source/Other/FileCompression/SPFileHandle.h | ObjC header | Other |
| 484 | Source/Other/FileCompression/SPFileHandle.m | ObjC | Other |
| 32 | Source/Other/Formatters/SABaseFormatter.swift | Swift | Other |
| 199 | Source/Other/Formatters/SAUuidFormatter.swift | Swift | Other |
| 286 | Source/Other/Keychain/SAKeychain.swift | Swift | Other |
| 53 | Source/Other/Keychain/SAKeychainAccess.swift | Swift | Other |
| 62 | Source/Other/Keychain/SAKeychainDisabled.swift | Swift | Other |
| 88 | Source/Other/Keychain/SAKeychainNaming.swift | Swift | Other |
| 98 | Source/Other/Keychain/SAKeychainProviding.swift | Swift | Other |
| 37 | Source/Other/Parsing/SAJSONValueDetector.swift | Swift | Other |
| 124 | Source/Other/Parsing/SPCSVParser.h | ObjC header | Other |
| 649 | Source/Other/Parsing/SPCSVParser.m | ObjC | Other |
| 44 | Source/Other/Parsing/SPEditorTokens.h | ObjC header | Other |
| 124 | Source/Other/Parsing/SPJSONFormatter.h | ObjC header | Other |
| 368 | Source/Other/Parsing/SPJSONFormatter.m | ObjC | Other |
| 42 | Source/Other/Parsing/SPParserUtils.h | ObjC header | Other |
| 311 | Source/Other/Parsing/SPSQLParser.h | ObjC header | Other |
| 1110 | Source/Other/Parsing/SPSQLParser.m | ObjC | Other |
| 36 | Source/Other/Parsing/SPSQLTokenizer.h | ObjC header | Other |
| 36 | Source/Other/Parsing/SPSyntaxParser.h | ObjC header | Other |
| 61 | Source/Other/Parsing/SPTableFilterParser.h | ObjC header | Other |
| 182 | Source/Other/Parsing/SPTableFilterParser.m | ObjC | Other |
| 156 | Source/Other/SSHTunnel/SASSHTunnelAskpass.swift | Swift | Other |
| 153 | Source/Other/SSHTunnel/SASSHTunnelAssistantSocketMain.swift | Swift | Other |
| 207 | Source/Other/SSHTunnel/SASSHTunnelAuthMessage.swift | Swift | Other |
| 131 | Source/Other/SSHTunnel/SASSHTunnelAuthService.swift | Swift | Other |
| 74 | Source/Other/SSHTunnel/SASSHTunnelAuthSource.swift | Swift | Other |
| 179 | Source/Other/SSHTunnel/SASSHTunnelFailure.swift | Swift | Other |
| 257 | Source/Other/SSHTunnel/SASSHTunnelPeerValidator.swift | Swift | Other |
| 134 | Source/Other/SSHTunnel/SASSHTunnelPromptCoordinator.swift | Swift | Other |
| 108 | Source/Other/SSHTunnel/SASSHTunnelSocketClient.swift | Swift | Other |
| 135 | Source/Other/SSHTunnel/SASSHTunnelSocketIO.swift | Swift | Other |
| 301 | Source/Other/SSHTunnel/SASSHTunnelSocketServer.swift | Swift | Other |
| 91 | Source/Other/SSHTunnel/SASSHTunnelTransport.swift | Swift | Other |
| 127 | Source/Other/SSHTunnel/SPSSHTunnel.h | ObjC header | Other |
| 1243 | Source/Other/SSHTunnel/SPSSHTunnel.m | ObjC | Other |
| 170 | Source/Other/SSHTunnel/SequelAceTunnelAssistant.m | ObjC | Other |
| 521 | Source/Other/Utility/OSLog.swift | Swift | Other |
| 77 | Source/Other/Utility/SAAnalyticsConsentPolicy+Firebase.swift | Swift | Other |
| 118 | Source/Other/Utility/SAAnalyticsConsentPolicy.swift | Swift | Other |
| 56 | Source/Other/Utility/SAAppearancePreference.swift | Swift | Other |
| 50 | Source/Other/Utility/SAAsyncResultBox.swift | Swift | Other |
| 43 | Source/Other/Utility/SABundleVersionUpdater.swift | Swift | Other |
| 73 | Source/Other/Utility/SAConnectionAttemptSequence.swift | Swift | Other |
| 51 | Source/Other/Utility/SAConnectionLostAlert.swift | Swift | Other |
| 163 | Source/Other/Utility/SAConnectionLostSheetCopy.swift | Swift | Other |
| 617 | Source/Other/Utility/SAConnectionService.swift | Swift | Other |
| 39 | Source/Other/Utility/SADatabaseScopedValueCache.swift | Swift | Other |
| 262 | Source/Other/Utility/SADragPasteboard.swift | Swift | Other |
| 48 | Source/Other/Utility/SAImageRenderer.swift | Swift | Other |
| 56 | Source/Other/Utility/SAKeyboardShortcut.swift | Swift | Other |
| 98 | Source/Other/Utility/SALocalNetworkPermissionChecker.swift | Swift | Other |
| 111 | Source/Other/Utility/SAMarkedTextInputState.swift | Swift | Other |
| 56 | Source/Other/Utility/SANotificationCenter.swift | Swift | Other |
| 220 | Source/Other/Utility/SAPrintUtility.swift | Swift | Other |
| 124 | Source/Other/Utility/SATypeAheadMatcher.swift | Swift | Other |
| 29 | Source/Other/Utility/SATypeAheadSelectionLoad.swift | Swift | Other |
| 72 | Source/Other/Utility/SPCompatibility.h | ObjC header | Other |
| 131 | Source/Other/Utility/SPFunctions.h | ObjC header | Other |
| 453 | Source/Other/Utility/SPFunctions.m | ObjC | Other |
| 47 | Source/Other/Utility/SPIdMenu.h | ObjC header | Other |
| 72 | Source/Other/Utility/SPIdMenu.m | ObjC | Other |
| 33 | Source/Other/Utility/SPPanelOptions.h | ObjC header | Other |
| 59 | Source/Other/Utility/SPPanelOptions.m | ObjC | Other |
| 62 | Source/Other/Vault/SAVaultCredentialsPath.swift | Swift | Other |
| 69 | Source/Other/Vault/SAVaultRoleFilter.swift | Swift | Other |
| 247 | Source/Other/Vault/SAVaultRoleListController.swift | Swift | Other |
| 482 | Source/Other/Vault/VaultAuthManager.swift | Swift | Other |
| 320 | Source/Other/Vault/VaultClient.swift | Swift | Other |
| 618 | Source/Other/Vault/VaultOIDCHandler.swift | Swift | Other |
| 38 | Source/Protocols/SAConnectionDelegate.swift | Swift | Protocols |
| 41 | Source/Protocols/SADatabaseDocumentProviding.swift | Swift | Protocols |
| 83 | Source/Protocols/SAFavoritesProviding.swift | Swift | Protocols |
| 56 | Source/Protocols/SATaskManaging.swift | Swift | Protocols |
| 73 | Source/Sequel-Ace-Bridging-Header.h | ObjC header | Sequel-Ace-Bridging-Header.h |
| 122 | Source/ThirdParty/Localization/xibLocalizationPostprocessor.m | ObjC | ThirdParty |
| 34 | Source/ThirdParty/MGTemplateEngine/ICUTemplateMatcher.h | ObjC header | ThirdParty |
| 163 | Source/ThirdParty/MGTemplateEngine/ICUTemplateMatcher.m | ObjC | ThirdParty |
| 81 | Source/ThirdParty/MGTemplateEngine/MGTemplateEngine.h | ObjC header | ThirdParty |
| 619 | Source/ThirdParty/MGTemplateEngine/MGTemplateEngine.m | ObjC | ThirdParty |
| 12 | Source/ThirdParty/MGTemplateEngine/MGTemplateFilter.h | ObjC header | ThirdParty |
| 40 | Source/ThirdParty/MGTemplateEngine/MGTemplateMarker.h | ObjC header | ThirdParty |
| 11 | Source/ThirdParty/MGTemplateEngine/MGTemplateStandardFilters.h | ObjC header | ThirdParty |
| 108 | Source/ThirdParty/MGTemplateEngine/MGTemplateStandardFilters.m | ObjC | ThirdParty |
| 16 | Source/ThirdParty/MGTemplateEngine/MGTemplateStandardMarkers.h | ObjC header | ThirdParty |
| 631 | Source/ThirdParty/MGTemplateEngine/MGTemplateStandardMarkers.m | ObjC | ThirdParty |
| 11 | Source/ThirdParty/MGTemplateEngine/NSArray_DeepMutableCopy.h | ObjC header | ThirdParty |
| 37 | Source/ThirdParty/MGTemplateEngine/NSArray_DeepMutableCopy.m | ObjC | ThirdParty |
| 11 | Source/ThirdParty/MGTemplateEngine/NSDictionary_DeepMutableCopy.h | ObjC header | ThirdParty |
| 38 | Source/ThirdParty/MGTemplateEngine/NSDictionary_DeepMutableCopy.m | ObjC | ThirdParty |
| 53 | Source/ThirdParty/NSMutableArray-MultipleSort.h | ObjC header | ThirdParty |
| 70 | Source/ThirdParty/NSMutableArray-MultipleSort.m | ObjC | ThirdParty |
| 15 | Source/ThirdParty/NSNotificationCenterThreadingAdditions.h | ObjC header | ThirdParty |
| 47 | Source/ThirdParty/NSNotificationCenterThreadingAdditions.m | ObjC | ThirdParty |
| 221 | Source/ThirdParty/RegexKitLite/RegexKitLite.h | ObjC header | ThirdParty |
| 2681 | Source/ThirdParty/RegexKitLite/RegexKitLite.m | ObjC | ThirdParty |
| 68 | Source/ThirdParty/SPReachability.h | ObjC header | ThirdParty |
| 151 | Source/ThirdParty/SPReachability.m | ObjC | ThirdParty |
| 76 | Source/ThirdParty/Views/NoodleLineNumberView.h | ObjC header | ThirdParty |
| 566 | Source/ThirdParty/Views/NoodleLineNumberView.m | ObjC | ThirdParty |
| 80 | Source/ThirdParty/Views/YRKSpinningProgressIndicator.h | ObjC header | ThirdParty |
| 375 | Source/ThirdParty/Views/YRKSpinningProgressIndicator.m | ObjC | ThirdParty |
| 46 | Source/Views/AccessoryViews/SPEncodingPopupAccessory.h | ObjC header | Views |
| 153 | Source/Views/AccessoryViews/SPEncodingPopupAccessory.m | ObjC | Views |
| 124 | Source/Views/AccessoryViews/SPWindowTabAccessory.swift | Swift | Views |
| 54 | Source/Views/Cells/ImageAndTextCell.h | ObjC header | Views |
| 205 | Source/Views/Cells/ImageAndTextCell.m | ObjC | Views |
| 88 | Source/Views/Cells/SATableListCellRenderer.swift | Swift | Views |
| 55 | Source/Views/Cells/SPActivityTextFieldCell.h | ObjC header | Views |
| 353 | Source/Views/Cells/SPActivityTextFieldCell.m | ObjC | Views |
| 33 | Source/Views/Cells/SPColorWellCell.h | ObjC header | Views |
| 68 | Source/Views/Cells/SPColorWellCell.m | ObjC | Views |
| 72 | Source/Views/Cells/SPComboBoxCell.h | ObjC header | Views |
| 149 | Source/Views/Cells/SPComboBoxCell.m | ObjC | Views |
| 37 | Source/Views/Cells/SPFavoriteTextFieldCell.h | ObjC header | Views |
| 54 | Source/Views/Cells/SPFavoriteTextFieldCell.m | ObjC | Views |
| 53 | Source/Views/Cells/SPPillAttachmentCell.h | ObjC header | Views |
| 109 | Source/Views/Cells/SPPillAttachmentCell.m | ObjC | Views |
| 40 | Source/Views/Cells/SPTableTextFieldCell.h | ObjC header | Views |
| 96 | Source/Views/Cells/SPTableTextFieldCell.m | ObjC | Views |
| 51 | Source/Views/Cells/SPTextAndLinkCell.h | ObjC header | Views |
| 283 | Source/Views/Cells/SPTextAndLinkCell.m | ObjC | Views |
| 79 | Source/Views/Controls/SPColorSelectorView.h | ObjC header | Views |
| 459 | Source/Views/Controls/SPColorSelectorView.m | ObjC | Views |
| 45 | Source/Views/Controls/SPComboPopupButton.h | ObjC header | Views |
| 304 | Source/Views/Controls/SPComboPopupButton.m | ObjC | Views |
| 607 | Source/Views/Controls/SPFilterRuleEditor.swift | Swift | Views |
| 97 | Source/Views/Controls/SPFilterRuleTextField.swift | Swift | Views |
| 34 | Source/Views/Controls/SPFontPreviewTextField.h | ObjC header | Views |
| 69 | Source/Views/Controls/SPFontPreviewTextField.m | ObjC | Views |
| 48 | Source/Views/Controls/SPPopUpButtonCell.h | ObjC header | Views |
| 51 | Source/Views/Controls/SPPopUpButtonCell.m | ObjC | Views |
| 398 | Source/Views/Controls/SPRuleFilterDropBox.swift | Swift | Views |
| 39 | Source/Views/Controls/SPTableFooterPopUpButtonCell.h | ObjC header | Views |
| 55 | Source/Views/Controls/SPTableFooterPopUpButtonCell.m | ObjC | Views |
| 47 | Source/Views/OutlineViews/SPFavoritesOutlineView.h | ObjC header | Views |
| 176 | Source/Views/OutlineViews/SPFavoritesOutlineView.m | ObjC | Views |
| 35 | Source/Views/OutlineViews/SPNavigatorOutlineView.h | ObjC header | Views |
| 65 | Source/Views/OutlineViews/SPNavigatorOutlineView.m | ObjC | Views |
| 33 | Source/Views/OutlineViews/SPOutlineView.h | ObjC header | Views |
| 100 | Source/Views/OutlineViews/SPOutlineView.m | ObjC | Views |
| 605 | Source/Views/SARecordView.swift | Swift | Views |
| 380 | Source/Views/SAWebView.swift | Swift | Views |
| 40 | Source/Views/SPDataCellFormatter.h | ObjC header | Views |
| 258 | Source/Views/SPDataCellFormatter.m | ObjC | Views |
| 47 | Source/Views/SPFillView.h | ObjC header | Views |
| 54 | Source/Views/SPFillView.m | ObjC | Views |
| 63 | Source/Views/SPGeometryDataView.h | ObjC header | Views |
| 305 | Source/Views/SPGeometryDataView.m | ObjC | Views |
| 46 | Source/Views/SPImageView.h | ObjC header | Views |
| 103 | Source/Views/SPImageView.m | ObjC | Views |
| 71 | Source/Views/SPSplitView.h | ObjC header | Views |
| 1105 | Source/Views/SPSplitView.m | ObjC | Views |
| 40 | Source/Views/SPWindow.h | ObjC header | Views |
| 201 | Source/Views/SPWindow.m | ObjC | Views |
| 239 | Source/Views/TableViews/SAFieldEditorCommitPolicy.swift | Swift | Views |
| 27 | Source/Views/TableViews/SATableHeaderView.swift | Swift | Views |
| 900 | Source/Views/TableViews/SATablesListView.swift | Swift | Views |
| 250 | Source/Views/TableViews/SPCopyTable.h | ObjC header | Views |
| 1807 | Source/Views/TableViews/SPCopyTable.m | ObjC | Views |
| 35 | Source/Views/TableViews/SPDatabaseContentViewDelegate.h | ObjC header | Views |
| 41 | Source/Views/TableViews/SPTableView.h | ObjC header | Views |
| 340 | Source/Views/TableViews/SPTableView.m | ObjC | Views |
| 30 | Source/Views/TextViews/SATextViewDelegate.swift | Swift | Views |
| 55 | Source/Views/TextViews/SPBundleCommandTextView.h | ObjC header | Views |
| 809 | Source/Views/TextViews/SPBundleCommandTextView.m | ObjC | Views |
| 38 | Source/Views/TextViews/SPEditSheetTextView.h | ObjC header | Views |
| 324 | Source/Views/TextViews/SPEditSheetTextView.m | ObjC | Views |
| 177 | Source/Views/TextViews/SPTextView.h | ObjC header | Views |
| 3884 | Source/Views/TextViews/SPTextView.m | ObjC | Views |
| 33 | Source/main.m | ObjC | main.m |
