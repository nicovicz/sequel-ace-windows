# Audit: `SPCustomQuery.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/MainViewControllers/SPCustomQuery.m`. Baseline commit
`02c910932`.

- File: 4079 baris; header `SPCustomQuery.h` 264 baris.
- Peran: controller tab Custom Query — editor SQL, pemisahan/eksekusi query, tabel
  hasil, editing field, menu favorites/history, dan overlay record-view.
- Berisi juga helper privat `SPQueryProgressUpdateDecoupling` (baris 88–99,
  4033–4079) yang memisahkan thread query background dari update progress UI.
- ~90 definisi method konkret + ~19 forward declaration di blok `@interface`
  (baris 97–115). Baris 143 satu-satunya method sebelum pragma pertama.

## 1. Inventaris ivar/property

### typedef/global di .m
| Baris | Item |
| --- | --- |
| 66–69 | `typedef struct QueryProgress { NSUInteger query, total; }` |
| 71 | `typedef void (^QueryProgressHandler)(QueryProgress *)` |
| 1400 | `static NSString * const SPDashStyleCommentMarker = @"-- "` |

### `@interface SPQueryProgressUpdateDecoupling` (88–99)
Baris 90 `QueryProgressHandler updateHandler`; 92 `volatile BOOL dirtyMarker`;
93 `os_unfair_lock qpLock`; 94 `volatile QueryProgress _queryProgress`;
96 `@property(atomic,assign) QueryProgress queryProgress`.

### Class extension `SPCustomQuery ()` (103–121)
103 conformance `<NSMenuItemValidation, NSFontChanging, SATextViewDelegate>`;
117 `NSMutableDictionary<NSNumber*,NSNumber*> *sortCount` (rw);
118 `BOOL recordViewNeedsSelectionRestoreRefresh`; 119 `BOOL suppressRecordViewTaskRefresh`;
104–115 forward declaration method privat/notifikasi.

### `@synthesize` (125–141)
`synthesize` untuk `runAllButton`, `textViewWasChanged`, `bracketHighlighter`,
`sortCount` (125–128); mapping ivar eksplisit untuk `tableDocumentInstance`,
`tablesListInstance`, `textView`, `currentQueryRange`, `sortColumn`, `isDesc`,
`reloadingExistingResult`, `errorTextTitle`, `errorText` (133–141).

### Public `@property` (.h 165–180)
`runAllButton` (165), `textViewWasChanged` (166), `bracketHighlighter` (167);
readonly Swift-exposed: `tableDocumentInstance`, `tablesListInstance`, `textView`,
`currentQueryRange`, `sortColumn`, `isDesc`, `reloadingExistingResult`, `isWorking`,
`errorTextTitle`, `errorText`, `sortCount` (170–180).

### Ivar `.h` 56–163 (grup)
| Baris | Grup |
| --- | --- |
| 57–58 | instance document/tables-list |
| 60–65 | outlet tombol/menu/search query favorites |
| 67–74 | tombol/menu/search query history + popup encoding |
| 76–84 | textView, customQueryView, scroll view, error text, affected rows, value sheet |
| 87 | `runAllButton` (hook layout legacy) |
| 90–93 | outlet/menu item aksi run baru |
| 95–104 | item gear menu (shift, completion, font, autoindent, autopair, autohelp, uppercase, comment) |
| 106–107 | item menu previous/next history |
| 109–111 | tombol info query + split view |
| 112 | `recordViewController` |
| 114–115 | fieldEditor, favoritesManager |
| 117–118 | prefs, mySQLConnection |
| 120–123 | usedQuery, currentQueryRange, currentQueryRanges, currentQueryBeforeCaret |
| 125 | sortColumn |
| 127 | queryStartPosition |
| 129–134 | resultData, resultDataLock, cqColumnDefinition, lastExecutedQuery, editedRow, editedScrollViewRect |
| 136–143 | isWorking, tableRowsSelectable, reloadingExistingResult, queryIsTableSorter, isDesc, isFieldEditable, textViewWasChanged, sortField |
| 145–146 | selectionIndexToRestore, selectionViewportToRestore |
| 148 | fieldIDQueryString |
| 150–151 | numberOfQueries, queryInfoPanePaddingHeight |
| 153–154 | currentHistoryOffsetIndex, historyItemWasJustInserted |
| 156–158 | queryLoadTimer, runAllContinueStopSheetReturnCode, counter load-timer |
| 160–162 | kCellEditorErrorNoMatch / NoMultiTabDb / TooManyMatches |

### Macro `.h` (34–39)
`SP_SAVE_ALL_FAVORTITE_MENUITEM_TAG` (100001), `SP_SAVE_SELECTION_FAVORTITE_MENUITEM_TAG`
(100000), `SP_FAVORITE_HEADER_MENUITEM_TAG` (200000), `SP_HISTORY_COPY/SAVE/CLEAR_MENUITEM_TAG`
(300000–300002).

## 2. Semua `#pragma mark` (urut baris)

`152 (IBAction methods), 601/602 (Query actions), 1639/1640 (Interface setup), 1700/1701 (Table load actions), 1782/1783 (Accessors), 1846/1847 (Additional methods), 1893/1894 (Retrieving and setting table state), 2058/2059 (Field Editing), 2387/2388 (TableView datasource methods), 2635/2636 (TableView Drag & Drop datasource methods), 2722/2723 (TableView delegate methods), 2898/2899 (TableView notifications), 3045/3046 (TextView delegate methods), 3080/3081 (TextView notifications), 3142/3143 (TextField delegate methods), 3164/3165 (SplitView delegate methods), 3231/3232 (Query favorites manager delegate methods), 3332/3333 (Database content view delegate methods), 3348/3349 (Task interaction), 3390/3391 (Other), 3535 (untitled), 3871/3872 (Private API), 4014, 4031`.
Anchor lain: `@interface SPQueryProgressUpdateDecoupling` (88), `@end` (99);
`@interface SPCustomQuery ()` (103), `@end` (121); `@implementation SPCustomQuery`
(123), `@end` (4029); `@implementation SPQueryProgressUpdateDecoupling` (4033), `@end` (4079).

## 3. Tabel method (urut baris)

### Pre-pragma (123–151)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 143 | 150 | `+columnHeaderAttributedStringForColumnDefinition:showColumnTypes:` | Bangun string header kolom tabel hasil |

### IBAction methods (152)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 157 | 164 | `runPrimaryQueryAction:` | Jalankan semua atau query terpilih sesuai pref |
| 169 | 176 | `runSecondaryQueryAction:` | Jalankan kebalikan aksi primer |
| 181 | 186 | `switchDefaultQueryAction:` | Toggle pref primary-control-runs-all |
| 192 | 231 | `runAllQueries:` | Pecah teks editor dan jalankan semua query berurutan |
| 237 | 281 | `runSelectedQueries:` | Jalankan query caret atau teks terpilih |
| 287 | 300 | `presentQueryFavoriteSaveSheetForQuery:` | Tampilkan sheet simpan favorite SwiftUI |
| 306 | 364 | `chooseQueryFavorite:` | Simpan/sisip favorite atau buka manager |
| 369 | 404 | `chooseQueryHistory:` | Sisip query history terpilih ke editor |
| 409 | 413 | `closeSheet:` | Akhiri + order out sheet modal |
| 419 | 532 | `gearMenuItemSelected:` | Dispatch aksi gear menu (history, shift, toggle) |
| 534 | 564 | `saveQueryHistory:` | Simpan history ke file via save panel |
| 566 | 573 | `copyQueryHistory:` | Copy history ke pasteboard |
| 578 | 591 | `clearQueryHistory:` | Konfirmasi + bersihkan history dokumen/global |
| 596 | 599 | `validModesForFontPanel:` | Batasi mode valid font panel |

### Query actions (601–602)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 604 | 641 | `performQueriesWithNoWarning:withCallback:` | Mulai task + detach thread query tanpa warning |
| 652 | 683 | `queriesContainDestructiveSQL:` | YES bila ada query destruktif |
| 689 | 734 | `performQueries:withCallback:` | Prompt konfirmasi lalu jalankan query |
| 736 | 1138 | `performQueriesTask:` | Thread background inti yang mengeksekusi daftar query |
| 1144 | 1169 | `updateResultStore:` | Muat store hasil streaming ke tabel |
| 1175 | 1295 | `queryRangeAtPosition:lookBehind:` | Hitung range teks query di posisi caret |
| 1301 | 1332 | `queryTextRangeForQuery:startPosition:` | Kembalikan range query ke-N dari posisi |
| 1341 | 1349 | `queryAtPosition:lookBehind:` | Kembalikan string query di posisi (atau sebelumnya) |
| 1351 | 1355 | `selectCurrentQuery` | Pilih range query aktif di editor |
| 1361 | 1397 | `commentOutCurrentQueryTakingSelection:` | Bungkus/lepas query aktif atau seleksi dengan `/* */` |
| 1405 | 1471 | `commentOutCurrentQueryTakingSelectionWithDashes:` | Toggle komentar baris `-- ` per baris |
| 1478 | 1537 | `commentOut` | Toggle komentar baris/blok berdasar seleksi |
| 1543 | 1637 | `updateStatusInterfaceWithDetails:` | Tampilkan teks error/status + pilih baris error |

### Interface setup (1639–1640)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1646 | 1665 | `updateQueryInteractionInterface` | Update judul tombol/menu run per pref |
| 1671 | 1698 | `updateContextualRunInterface` | Update judul run-current/previous/selection |

### Table load actions (1700–1701)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1707 | 1716 | `initQueryLoadTimer` | Mulai timer update tabel 0.02s |
| 1722 | 1728 | `clearQueryLoadTimer` | Invalidate timer load tabel |
| 1736 | 1780 | `queryLoadUpdate:` | Redraw/auto-size tabel hasil bertahap + throttle |

### Accessors (1782–1783)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1789 | 1792 | `currentResult` | Kembalikan hasil aktif tanpa NULL, terpotong |
| 1799 | 1802 | `currentResultRowCount` | Kembalikan jumlah baris hasil |
| 1812 | 1844 | `currentDataResultWithNULLs:truncateDataFields:` | Export hasil sebagai array header+baris |

### Additional methods (1846–1847)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1852 | 1870 | `setConnection:` | Simpan koneksi, reset state, wire menu |
| 1875 | 1879 | `doPerformQueryService:` | Muat teks query lalu jalankan semua |
| 1881 | 1891 | `doPerformLoadQueryService:` | Sisip teks query + syntax highlight |

### Retrieving and setting table state (1893–1894)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1900 | 1966 | `updateTableView` | Bangun ulang kolom tabel hasil dari definisi |
| 1968 | 1980 | `_updateColumnHeadersForCurrentPreference` | Refresh cell header saat pref berubah |
| 1985 | 1988 | `resultSelectedRowIndexes` | Getter baris terpilih tabel |
| 1993 | 1996 | `resultViewport` | Getter visible rect tabel |
| 2001 | 2006 | `setResultSelectedRowIndexesToRestore:` | Simpan seleksi untuk restore saat reload |
| 2011 | 2014 | `setResultViewportToRestore:` | Simpan viewport untuk restore |
| 2019 | 2023 | `storeCurrentResultViewForRestoration` | Simpan seleksi + viewport bersama |
| 2028 | 2032 | `clearResultViewDetailsToRestore` | Bersihkan seleksi/viewport tersimpan |
| 2038 | 2056 | `autosizeColumns` | Auto-size kolom, lewati lebar tersimpan |

### Field Editing (2058–2059)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2068 | 2139 | `fieldEditStatusForRow:andColumn:` | Uji apakah baris sel bisa diidentifikasi bijektif |
| 2145 | 2218 | `argumentForRow:ofTable:andDatabase:includeBlobs:` | Bangun klausa WHERE identifikasi baris |
| 2220 | 2385 | `saveCellValue:forTableColumn:row:` | Issue UPDATE untuk sel teredit, dengan warning |

### TableView datasource methods (2387–2388)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2390 | 2393 | `isWorking` | Kembalikan flag query berjalan |
| 2398 | 2401 | `numberOfRowsInTableView:` | Jumlah baris tabel hasil |
| 2406 | 2439 | `tableView:willDisplayCell:forTableColumn:row:` | Grey sel NULL/loading |
| 2444 | 2454 | `tableView:objectValueForTableColumn:row:` | Nilai tampil sel |
| 2456 | 2469 | `tableView:setObjectValue:forTableColumn:row:` | Picu simpan atau delegasi ke field editor |
| 2474 | 2619 | `tableView:didClickTableColumn:` | Sortir: tulis ulang ORDER BY + jalankan ulang query |
| 2621 | 2633 | `tableSortCallback` | Callback bersihkan state sortir setelah query sort |

### TableView Drag & Drop datasource methods (2635–2636)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2644 | 2653 | `tableView:pasteboardWriterForRow:` | Sediakan item pasteboard per-baris untuk drag |
| 2659 | 2669 | `tableView:draggingSession:willBeginAtPoint:forRowIndexes:` | Lampirkan teks seleksi tab-delimited ke drag |

### TableView delegate methods (2722–2723)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2731 | 2800 | `tableView:toolTipForCell:rect:tableColumn:row:mouseLocation:` | Tooltip sel / thumbnail image |
| 2805 | 2888 | `tableView:shouldEditTableColumn:row:` | Buka sheet field-editor untuk editing |
| 2893 | 2896 | `tableView:shouldSelectRow:` | Cegah seleksi saat loading |

### TableView notifications (2898–2899)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2904 | 2948 | `tableViewSelectionDidChange:` | Update record view, jalankan bundle command |
| 2953 | 2957 | `tableViewColumnDidMove:` | Refresh record view setelah kolom dipindah |
| 2962 | 3004 | `tableViewColumnDidResize:` | Persist lebar kolom ke pref |
| 3009 | 3043 | `tableView:sizeToFitWidthOfColumn:` | Auto-size kolom double-click (10.13+) |

### TextView delegate methods (3045–3046)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3052 | 3078 | `textView:doCommandBySelector:` | Tangkap Enter untuk jalankan query / tutup sheet |

### TextView notifications (3080–3081)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3083 | 3089 | `textView:willChangeSelectionFromCharacterRange:toCharacterRange:` | Validasi sesi snippet saat seleksi berubah |
| 3091 | 3094 | `textView:shouldChangeTextInRange:replacementString:` | Matikan bracket highlight sebelum edit |
| 3100 | 3140 | `textViewDidChangeSelection:` | Hitung ulang query range, tombol, brace highlight |

### TextField delegate methods (3142–3143)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3148 | 3157 | `controlTextDidChange:` | Filter favorites/history saat mengetik search |
| 3159 | 3162 | `undoManagerForTextView:` | Kembalikan undo manager dokumen |

### SplitView delegate methods (3164–3165)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3170 | 3174 | `splitView:canCollapseSubview:` | Hanya pane info yang bisa collapse |
| 3179 | 3182 | `splitView:effectiveRect:forDrawnRect:ofDividerAtIndex:` | Nonaktifkan resize divider pane info |
| 3187 | 3190 | `setMySQLversion:` | Teruskan versi MySQL ke editor |
| 3192 | 3206 | `showCompletionList:` | Tampilkan list completion database/tabel/field |
| 3211 | 3215 | `showAutoHelpForCurrentWord:` | Buka help untuk kata di caret |
| 3221 | 3229 | `helpWindowClosedByUser:` | Nonaktifkan autohelp saat help window tutup |

### Query favorites manager delegate methods (3231–3232)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3241 | 3258 | `historyItemsHaveBeenUpdated:` | Bangun ulang popup menu history |
| 3263 | 3330 | `queryFavoritesHaveBeenUpdated:` | Bangun ulang menu favorites dokumen + global |

### Database content view delegate methods (3332–3333)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3335 | 3338 | `usedQuery` | Getter string query terakhir |
| 3343 | 3346 | `dataColumnDefinitions` | Kembalikan definisi kolom hasil |

### Task interaction (3348–3349)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3354 | 3367 | `startDocumentTaskForTab:` | Disable kontrol interaktif selama task |
| 3372 | 3388 | `endDocumentTaskForTab:` | Re-enable kontrol setelah task |

### Other (3390–3391)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3396 | 3399 | `numberOfQueries` | Kembalikan jumlah query ter-cache |
| 3404 | 3407 | `currentQueryRange` | Kembalikan range teks query aktif |
| 3409 | 3412 | `buildHistoryString` | Gabung entri history untuk simpan/copy |
| 3418 | 3421 | `addHistoryEntry:` | Tambah query ke history file/global |
| 3426 | 3451 | `observeValueForKeyPath:ofObject:change:context:` | Reaksi perubahan pref font/grid/tipe kolom |
| 3456 | 3497 | `validateMenuItem:` | Enable/judul item menu favorites & history |
| 3499 | 3523 | `processFieldEditorResult:contextInfo:` | Simpan data field-editor + restore fokus |
| 3525 | 3528 | `setFieldEditorSelectedRange:` | Teruskan seleksi ke copy table |
| 3530 | 3533 | `fieldEditorSelectedRange` | Kembalikan seleksi field-editor copy table |

### Untitled `#pragma mark -` (3535)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3537 | 3578 | `init` | Init ivar, storage, string error terlokalisasi |
| 3583 | 3592 | `filterQueryFavorites:` | Filter menu favorites by search field |
| 3597 | 3617 | `filterQueryHistory:` | Filter menu history by search field |
| 3624 | 3703 | `control:textShouldBeginEditing:` | Rutekan sel blob ke sheet field editor |
| 3709 | 3749 | `control:textView:doCommandBySelector:` | Tangani arrow/escape di search/editor tabel |
| 3754 | 3864 | `awakeFromNib` | Bangun record view, observer, menu, KVO |
| 3866 | 3869 | `toggleRecordView` | Toggle overlay record-view |

### Private API (3871–3872)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3883 | 3922 | `_resultDataItemAtRow:columnIndex:preserveNULLs:asPreview:` | Ambil nilai sel terformat/preview (locking) |
| 3924 | 3936 | `_recordViewSelectedRow` | Validasi satu baris terpilih untuk record view |
| 3938 | 3942 | `_recordViewColumnForFieldID:` | Map field ID ke kolom tabel |
| 3944 | 3962 | `_recordViewStringForValue:tableColumn:` | Format nilai untuk display record-view |
| 3964 | 4000 | `_updateRecordView` | Refresh field/state seleksi record-view |
| 4003 | 4012 | `documentWillClose:` | Stop load timer saat dokumen tutup |

### `#pragma mark -` (4014)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4016 | 4027 | `dealloc` | Hapus observer + cancel timer/request |

### `@implementation SPQueryProgressUpdateDecoupling` (4033)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4035 | 4044 | `initWithBlock:` | Salin handler progress, init unfair lock |
| 4046 | 4058 | `setQueryProgress:` | Tandai dirty + jadwalkan update progress main-thread |
| 4060 | 4066 | `queryProgress` | Getter thread-safe progress saat ini |
| 4068 | 4077 | `_updateProgress` | Invokasi handler ter-coalesce di main thread |

## 4. Temuan menonjol / UNKNOWN

**Dead code yang dikomentari**
- **2671–2720** — seluruh blok `validateDrop:`/`acceptDrop:` implementasi reorder `queryFavoritesView` dikomentari; merujuk ivar tak terdefinisi `queryFavorites`/`queryFavoritesView`.
- **757** — `// BOOL queriesSeparatedByDelimiter = NO;` (digerbangi komentar 996).
- **996** — `// if(!queriesSeparatedByDelimiter) // TODO: How to combine queries delimited by DELIMITER?`.

**Marker TODO/FIXME**
- 958 — regex adalah kompromi, TODO perbaikan.
- 1058 — TODO: `mysql_query()` return dini, pengukuran waktu query tak berguna (#264).
- 2736 — TODO perbaikan tooltip di atas WKWebView.
- 3172 — TODO: split view editor/hasil tak boleh collapsible.
- 3225 — TODO: penanganan help-window-closed tak seharusnya di konteks dokumen.
- 3699 — TODO: `textShouldBeginEditing:` tak dipanggil sampai mengetik.
- 3885 — TODO (#2607): duplikasi kode dengan `SPTableContent.m tableView:objectValueForTableColumn:…`.

**UNKNOWN / referensi lintas-file**
- **3491** — `validateMenuItem:` menangani `@selector(runExplainQueryAction:)`, selector **tak didefinisikan di file ini** (ada di `SPCustomQuery+Explain.swift`).
- **64, 293, 3772–3824, 4044, 4081** — bergantung pada tipe Swift `sequel-ace-Swift.h` (`SAQueryFavoriteSaveWindowController`, `SARecordViewController`, `SADragPasteboard`, `SACellFilterColumnIdentifier`, `SARecordViewColumnMapping`, `SARecordViewEditSupport`, `SABaseFormatter`, `SASQLDatabaseContext`, `SANotificationCenter`) — perilaku **UNKNOWN** dari file ini.
- **2911–2946** — blok bundle-command trigger bergantung pada cek nama kelas string (`SPCopyTable`, `SABundleHTMLOutputWindowController`) — coupling rapuh.

**Catatan struktural**
- **101–102** — class-extension ditambah untuk conformance formal `NSMenuItemValidation`/`NSFontChanging`/`SATextViewDelegate`.
- **97–115** — method forward-declared (deklarasi, bukan definisi); grup 104–115 menduplikasi private API yang diimplementasi di 3883–4012 dan 3221–3330.
- **287, 596** — `presentQueryFavoriteSaveSheetForQuery:` dan `validModesForFontPanel:` berada di bawah pragma "IBAction methods" padahal bukan IBAction.
- **1400** — `static const SPDashStyleCommentMarker` dideklarasikan di tengah `@implementation`, bukan di atas file.
