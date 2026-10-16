# Audit: `SPTableContent.m`

Audit method-by-method. `Start`/`End` adalah nomor baris di
`Source/Controllers/MainViewControllers/TableContent/SPTableContent.m`.
Baseline commit `02c910932`.

- File: 5656 baris; header `SPTableContent.h` 278 baris.
- Peran: controller tab "Content" — load/format/edit baris, filter, sortir,
  pagination, render/lebar kolom, overlay record-view, dan
  `ContentPaginationViewController` internal.

Struktur berkas:

| Baris | Elemen |
| --- | --- |
| 1–66 | Header lisensi + `#import` |
| 68–72 | Static `TableContentKVOContext` |
| 74–101 | `@interface ContentPaginationViewController : NSViewController` (workaround privat) |
| 103–149 | `@interface SPTableContent () <SATableHeaderViewDelegate, NSMenuItemValidation, SPComboBoxCellDelegate>` |
| 151–5612 | `@implementation SPTableContent` |
| 5614–5656 | `@implementation ContentPaginationViewController` |

## 1. Inventaris ivar/property

### `SPTableContent.h` (62–170)
| Baris | Isi |
| --- | --- |
| 63–90 | IBOutlet: `tableDocumentInstance`, `tablesListInstance`, `tableSourceInstance`, `tableInfoInstance`, `spHistoryControllerInstance`, `tableContentView`, tombol add/duplicate/remove/reload, `multipleLineEditingButton`, `countText`, kontrol pagination, `contentViewPane`, `paginationViewController`, `paginationPopover`, `paginationView` |
| 88–91 | `ruleFilterController`, `filterTableController`, `columnFilterSearchField`, `mySQLConnection` |
| 93–95 | `_mainNibLoaded`, `isWorking`, `tableValuesLock` (mutex pthread) |
| 97–104 | `selectedTable`, `usedQuery`, `tableValues`, `dataColumns`, `visibleColumns`, `columnFilterTerms`, `keys`, `oldRow` |
| 105–119 | count/flag: `tableRowsCount`, `previousTableRowsCount`, `sortCol`, `isEditingRow`, `isEditingNewRow`, `isSavingRow`, `isDesc`, `setLimit`, `isFiltered`, `isLimited`, `isInterruptedLoad`, `maxNumRowsIsEstimate`, `prefs`, `currentlyEditingRow`, `maxNumRows` |
| 121–132 | `contentPage`, `activeFilter`, `activeFilterToRestore`, `schemeFilter`, `sortColumnToRestoreIsAsc`, `tableRowsSelectable`, `sortColumnToRestore`, `pageToRestore`, `selectionToRestore`, `selectionViewportToRestore` |
| 134–143 | `paginationViewHeight`, state timer (`tableLoadTimer`, `tableLoadInterfaceUpdateInterval`, `tableLoadTimerTicksSinceLastUpdate`, `tableLoadLastRowCount`, `tableLoadTargetRowCount`), `cqColumnDefinition`, `isFirstChangeInView` |
| 145–153 | string error (`kCellEditorErrorNoMatch/NoMultiTabDb/TooManyMatches`), warna (`textForegroundColor`, `nullHighlightColor`, `displayOverrideHighlightColor`), `fieldEditor` |
| 155–169 | view layout (`contentAreaContainer`, `filterRuleEditorContainer`, `tableContentContainer`), `recordViewController`, `showFilterRuleEditor`, `ruleEditorVisibilityHasBeenApplied`, `filtersToRestore`, `@package tableDataInstance` + `toggleRuleFilterButton` |
| 172 | `@property (readonly, strong) SPTablesList *tablesListInstance;` (synthesize .m:153) |

### Property class-extension `.m` (105–110)
| Baris | Anggota |
| --- | --- |
| 107 | `deferRecordViewRefreshUntilTableLoadCompletes` (BOOL) |
| 108 | `suppressRecordViewTaskRefresh` (BOOL) |
| 109 | `comboBoxSelectionTracker` (`SAComboBoxSelectionTracker`) |
| 110 | `reloadCoordinator` (`SATableReloadCoordinator`) |

### `ContentPaginationViewController` (79–101)
`action` (SEL, 81), `page`, `maxPage` (83–84), IBOutlet `paginationGoButton`, `paginationPageField`, `paginationPageStepper` (86–88); property `target`, `action`, `page`, `maxPage` (94–99).

## 2. Semua `#pragma mark` (urut baris)

`155, 368, 369 (Table loading methods and information), 1470, 1471 (Table interface actions), 1857, 1858 (Pagination), 1969, 1970 (Edit methods), 2567, 2568 (Data accessors), 2721, 3709, 3710 (Filter Table), 3719, 3720 (Retrieving and setting table state), 3955, 3956 (Table drawing and editing), 4165, 4166 (Task interaction), 4241, 4242 (KVO methods), 4302, 4303 (Other methods), 4342, 4343 (Combo box delegate methods), 4360, 4361 (TableView datasource methods), 4553 (SPTableContentDataSource_Private_API), 4649 (SPTableContentFilter), 4669, 4670 (TableView delegate methods), 5244, 5245 (Control delegate methods), 5384, 5385 (Database content view delegate methods), 5400, 5401 (Methods for Column Display Format), 5480, 5481 (SPTableHeaderViewDelegate Methods), 5510, 5511 (UserDefault Helper Functions), 5523, 5524 (Column Filter Methods), 5585, 5614`.

## 3. Tabel method (urut baris)

### Lifecycle (155–367)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 157 | 220 | `init` | Init store, flag, timer, warna, string error, mutex |
| 222 | 361 | `awakeFromNib` | Wiring outlet, record-view, popover pagination, observer |
| 363 | 366 | `toggleRecordView` | Toggle overlay record-view |

### Table loading methods and information (369)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 377 | 388 | `loadTable:` | Wrapper membungkus `_loadTableContents:` dengan reload coordinator |
| 395 | 455 | `_loadTableContents:` | Ambil definisi tabel, set detail, reload nilai, restore viewport |
| 465 | 470 | `_startPendingFullReloadIfIdle` | Mulai reload tertunda saat coordinator idle |
| 475 | 521 | `_setViewBlankState` | Reset kolom/data/tombol ke state "tanpa tabel" |
| 528 | 716 | `setTableDetails:` | Bangun ulang kolom/filter/tombol untuk tabel baru dipilih |
| 724 | 726 | `_buildTableColumns:withFont:` | Wrapper delegasi dengan `filterTerms:nil` |
| 742 | 827 | `_buildTableColumns:withFont:filterTerms:` | Bangun `NSTableColumn`, lebar, header, indikator sort, filter |
| 829 | 846 | `buildTooltip()` *(static)* | Susun tooltip header dari definisi kolom |
| 848 | 915 | `configureDataCell()` *(static)* | Buat cell enum/FK/text dengan formatter, limit, alignment |
| 917 | 920 | `selectedTable` | Kembalikan string nama tabel aktif |
| 927 | 942 | `clearTableValues` | Reassign `tableValues`, bersihkan record view (mutex) |
| 950 | 1233 | `loadTableValues` | Bangun/jalankan SELECT dengan filter/sort/limit; proses hasil + restore seleksi |
| 1239 | 1285 | `updateResultStore:approximateRowCount:` | Stream hasil ke storage, update progress, autosize |
| 1294 | 1322 | `tableFilterString` | Kembalikan klausa WHERE untuk sumber filter aktif |
| 1330 | 1383 | `updateCountText` | Format teks jumlah baris/seleksi dengan kualifikasi filter/limit |
| 1389 | 1397 | `initTableLoadTimer` | Buat timer UI-update 0.02s untuk loading |
| 1403 | 1409 | `clearTableLoadTimer` | Invalidate timer load |
| 1417 | 1468 | `tableLoadUpdate:` | Tick timer: update progress, redraw, throttle frekuensi |

### Table interface actions (1471)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1478 | 1484 | `reloadTable:` | Reset budget reload lalu mulai reload |
| 1493 | 1505 | `_startTableReload` | Reservasi state popup, detach/panggil `reloadTableTask` |
| 1513 | 1522 | `_reportUnresolvedColumnMismatchForTable:` | Warning kolom tak pernah cocok hasil |
| 1527 | 1565 | `reloadTableTask` | Simpan baris, loop load selama reload diminta, akhiri task |
| 1570 | 1577 | `_tableDataReloadDidFinish` | Lepas reservasi popup, lanjutkan edit tertunda |
| 1582 | 1595 | `_resumeDeferredComboBoxEdit` | Commit edit popup yang tertunda saat loading |
| 1601 | 1684 | `filterTable:` | Tentukan sumber filter, halaman, task string; detach `filterTableTask` |
| 1686 | 1701 | `filterTableTask` | Restore history, bersihkan nilai, reload dengan filter baru |
| 1703 | 1712 | `toggleRuleEditorVisible:` | Flip visibilitas rule editor; clear rule filter bila disembunyikan |
| 1714 | 1717 | `setRuleEditorVisible:animate:` | Overload delegasi dengan `tableChanged:NO` |
| 1719 | 1752 | `setRuleEditorVisible:animate:tableChanged:` | Terapkan policy visibilitas, seed starter rule, ukur editor |
| 1754 | 1757 | `setUsedQuery:` | Simpan salinan query terakhir |
| 1759 | 1835 | `sortTableTaskWithColumn:` | Toggle sort tri-state, update indikator, reload nilai |
| 1837 | 1855 | `applyCellFilterForColumn:operator:values:isNull:` | Merge filter terserialisasi, tampilkan editor, terapkan |

### Pagination (1858)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1864 | 1877 | `navigatePaginationFromButton:` | Cek bound prev/next lalu re-filter |
| 1883 | 1886 | `togglePagination:` | Show/hide popover pagination |
| 1888 | 1892 | `popoverDidClose:` | Sembunyikan view pagination saat popover tutup |
| 1897 | 1937 | `setPaginationViewVisibility:` | Show/hide popover atau animasi tinggi view legacy |
| 1943 | 1967 | `updatePaginationState` | Enable/atur tombol halaman + min/max field |

### Edit methods (1970)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1976 | 2049 | `argumentForRow:ofTable:andDatabase:includeBlobs:` | Bangun WHERE dari PK atau semua kolom tabel tertentu |
| 2054 | 2096 | `addRow:` | Tambah baris kosong ber-default + masuk mode edit |
| 2101 | 2171 | `duplicateRow:` | Salin baris terpilih (auto-increment di-NULL), sisip, edit |
| 2176 | 2565 | `removeRow:` | Konfirmasi + hapus baris terpilih (tiga strategi) |

### Data accessors (2568)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2574 | 2610 | `currentResult` | Kembalikan nama field + baris sebagai nilai tampil |
| 2618 | 2719 | `currentDataResultWithNULLs:hideBLOBs:` | Kembalikan baris dengan penanganan NULL/blob/geometry/image |

### Additional methods (2721)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2726 | 2731 | `setConnection:` | Simpan koneksi, disable vertical drag motion |
| 2738 | 2754 | `clickLinkArrow:` | Validasi state/simpan baris, detach `clickLinkArrowTask:` |
| 2756 | 2892 | `clickLinkArrowTask:` | Resolve target FK, encode binary/UUID, navigasi + filter |
| 2900 | 3006 | `_saveRowToTableWithQuery:` | Eksekusi tulis baris; tangani no-rows/sukses/error |
| 3013 | 3124 | `deriveQueryString` | Bangun SQL INSERT/UPDATE dari sel berubah |
| 3136 | 3217 | `saveRowToTable` | Banding baris lama/baru, warning, delegasi save-with-query |
| 3225 | 3249 | `saveRowOnDeselect` | Akhiri editing, simpan bila baris sedang diedit |
| 3256 | 3274 | `cancelRowEditing` | Revert/hapus baris teredit, reset state edit |
| 3281 | 3284 | `argumentForRow:` | Overload convenience (limit disertakan) |
| 3292 | 3402 | `argumentForRow:excludingLimits:` | Bangun WHERE dari PK/semua kolom, escape per tipe |
| 3408 | 3417 | `tableContainsBlobOrTextColumns` | YES bila ada kolom blob/text |
| 3422 | 3447 | `fieldListForQuery` | Daftar field dipisah koma (placeholder NULL untuk blob tersembunyi) |
| 3456 | 3531 | `fieldEditStatusForRow:andColumn:` | Jumlah match + WHERE pembuktian editability sel |
| 3537 | 3541 | `showErrorSheetWith:` | Warning satu tombol dari array title/message |
| 3543 | 3593 | `processFieldEditorResult:contextInfo:` | Rutekan hasil sheet field editor ke baris/view + refresh |
| 3595 | 3707 | `saveViewCellValue:forTableColumn:row:` | Update sel view via UPDATE + reload |

### Filter Table (3710)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3715 | 3717 | `showFilterTable` | Tampilkan window filter lanjutan |

### Retrieving and setting table state (3720)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3725 | 3730 | `sortColumnName` | Nama kolom sortir aktif atau nil |
| 3735 | 3738 | `sortColumnIsAscending` | Kembalikan `!isDesc` |
| 3749 | 3822 | `selectionDetailsAllowingIndexSelection:` | Dictionary seleksi ber-key PK (atau index) |
| 3827 | 3830 | `pageNumber` | Kembalikan halaman aktif |
| 3837 | 3840 | `viewport` | Kembalikan visible rect tabel |
| 3845 | 3847 | `tablesListWidth` | Lebar pane splitter kiri |
| 3854 | 3857 | `filterSettings` | Kembalikan rule-filter terserialisasi |
| 3862 | 3870 | `setSortColumnNameToRestore:isAscending:` | Simpan kolom/urutan sort untuk load berikut |
| 3875 | 3878 | `setPageToRestore:` | Simpan halaman untuk load berikut |
| 3883 | 3888 | `setSelectionToRestore:` | Simpan dictionary seleksi untuk load berikut |
| 3893 | 3896 | `setViewportToRestore:` | Simpan viewport rect untuk load berikut |
| 3901 | 3905 | `setFiltersToRestore:` | Simpan filter untuk load berikut |
| 3912 | 3920 | `storeCurrentDetailsForRestoration` | Snapshot sort/page/seleksi/viewport/filter |
| 3925 | 3933 | `clearDetailsToRestore` | Reset semua state restore ke default |
| 3935 | 3938 | `filterTableData` | Kembalikan blob data filter tabel |
| 3940 | 3943 | `setFilterTableData:` | Set blob data filter tabel |
| 3945 | 3948 | `activeFilter` | Kembalikan sumber filter aktif |
| 3950 | 3953 | `setActiveFilterToRestore:` | Simpan sumber filter untuk load berikut |

### Table drawing and editing (3956)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3958 | 4051 | `updateFilterRuleEditorSize:animate:` | Tata letak rule editor/drop box/container (marshal off-main) |
| 4053 | 4063 | `filterRuleEditorPreferredSizeChanged:` | Tata ulang saat preferred size editor berubah |
| 4065 | 4070 | `contentViewSizeChanged:` | Tata ulang setelah container resize |
| 4080 | 4140 | `updateNumberOfRows` | Set `maxNumRows` dari count murah atau path akurat/estimasi |
| 4146 | 4163 | `autosizeColumns` | Ukur kolom ke konten, hormati lebar tersimpan |

### Task interaction (4166)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4168 | 4171 | `isWorking` | Kembalikan `isWorking` |
| 4176 | 4194 | `startDocumentTaskForTab:` | Disable kontrol editing selama task |
| 4199 | 4227 | `endDocumentTaskForTab:` | Enable kembali kontrol, refresh record view |

### KVO methods (4242)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4230 | 4239 | `documentWillClose:` | Stop timer/bersihkan state combo saat dokumen tutup |
| 4247 | 4300 | `observeValueForKeyPath:ofObject:change:context:` | Reaksi perubahan pref gridline/font/hex/tipe kolom/dropzone |

### Other methods (4303)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4308 | 4330 | `validateMenuItem:` | Enable/judul item menu add/duplicate/remove |
| 4332 | 4335 | `setFieldEditorSelectedRange:` | Teruskan range terpilih ke `tableContentView` |
| 4337 | 4340 | `fieldEditorSelectedRange` | Kembalikan range field-editor `tableContentView` |

### Combo box delegate methods (4343)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4345 | 4348 | `comboBoxCell:willPopUpWindow:` | Lacak nilai saat popup dibuka |
| 4350 | 4353 | `comboBoxCellSelectionDidChange:` | Catat item dipilih |
| 4355 | 4358 | `comboBoxCellDidDismissPopUp:` | Catat nilai dismiss di tracker |

### TableView datasource methods (4361)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4363 | 4370 | `numberOfRowsInTableView:` | Kembalikan `tableRowsCount` |
| 4372 | 4440 | `tableView:objectValueForTableColumn:row:` | Nilai tampil (NULL/geometry/hex/short-string) |
| 4442 | 4531 | `tableView:setObjectValue:forTableColumn:row:` | Terapkan nilai teredit, policy commit popup, catat edit |
| 4533 | 4551 | `cellValueIsDisplayedAsHexForColumn:` | YES untuk binary/blob saat hex display aktif |

### SPTableContentDataSource_Private_API (4553)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4555 | 4562 | `_contentValueForTableColumn:row:asPreview:` | Kembalikan preview atau nilai storage penuh |
| 4564 | 4570 | `_recordViewSelectedRow` | Kembalikan satu baris terpilih atau -1 |
| 4572 | 4576 | `_recordViewColumnForFieldID:` | Map field id ke `NSTableColumn` |
| 4578 | 4620 | `_updateRecordView` | Bangun ulang field record-view untuk baris terpilih |
| 4622 | 4647 | `_recordViewStringForValue:tableColumn:` | Format nilai untuk record view |

### SPTableContentFilter (4649)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4654 | 4667 | `makeContentFilterHaveFocus` | Tampilkan rule editor + fokus field pertama |

### TableView delegate methods (4670)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 4676 | 4692 | `tableView:didClickTableColumn:` | Mulai task sort saat header diklik |
| 4694 | 4774 | `tableViewSelectionDidChange:` | Simpan edit sebelumnya, update tombol/count, jalankan bundle trigger |
| 4779 | 4783 | `tableViewColumnDidMove:` | Refresh record view setelah kolom dipindah |
| 4788 | 4820 | `tableViewColumnDidResize:` | Persist lebar kolom baru ke pref |
| 4826 | 4957 | `tableView:shouldEditTableColumn:row:` | Load sel not-loaded; buka field editor blob; guard kolom generated |
| 4965 | 4974 | `tableView:pasteboardWriterForRow:` | Sediakan item pasteboard drag per-baris |
| 4979 | 5040 | `tableView:draggingSession:willBeginAtPoint:forRowIndexes:` | Lampirkan string SQL/tab + payload sel ke drag |
| 5045 | 5050 | `tableView:shouldSelectRow:` | Nonaktifkan seleksi saat working/editor terbuka |
| 5055 | 5093 | `tableView:sizeToFitWidthOfColumn:` | Autodetect lebar + hapus lebar tersimpan |
| 5098 | 5157 | `tableView:willDisplayCell:forTableColumn:row:` | Warnai sel NULL/unloaded/link/override (mutex) |
| 5166 | 5242 | `tableView:toolTipForCell:rect:tableColumn:row:mouseLocation:` | Tooltip image/geometry/text |

### Control delegate methods (5245)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 5247 | 5279 | `control:textShouldEndEditing:` | Validasi input hex/base formatter sebelum commit |
| 5286 | 5357 | `control:textShouldBeginEditing:` | Tooltip error edit; alihkan ke sheet field editor |
| 5363 | 5382 | `control:textView:doCommandBySelector:` | Tangkap Escape untuk cancel editing; delegasi ke SPCopyTable |

### Database content view delegate methods (5385)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 5387 | 5390 | `usedQuery` | Kembalikan query terakhir |
| 5395 | 5398 | `dataColumnDefinitions` | Kembalikan `dataColumns` |

### Methods for Column Display Format (5401)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 5403 | 5428 | `toggleDisplayOverrideFormatter:` | Terapkan/hapus override display format kolom, persist |
| 5431 | 5452 | `defaultColumnHeaderMenu()` *(static)* | Bangun context menu header (format UUID) |
| 5455 | 5470 | `currentFormatters()` *(static)* | Map key override tersimpan ke instance formatter |
| 5472 | 5478 | `knownColumnFormatters()` *(static)* | Dictionary formatter dikenal (hanya UUID) |

### SPTableHeaderViewDelegate Methods (5481)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 5483 | 5508 | `validateWithMenu:forTableColumn:` | Cek state menu; hanya tawarkan untuk BINARY(16) |

### UserDefault Helper Functions (5511)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 5513 | 5517 | `savedWidthForColumn()` *(static)* | Baca lebar tersimpan kolom |
| 5519 | 5521 | `dbHostPrefKey()` *(static)* | Bangun key pref `database@host` |

### Column Filter Methods (5524)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 5526 | 5558 | `columnFilterChanged:` | Parse term koma; bangun ulang kolom bila berubah |
| 5560 | 5583 | `rebuildTableColumnsWithFilter` | Pertahankan lebar, bangun ulang kolom dengan term filter |

### Teardown (5585)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 5587 | 5610 | `dealloc` | Hapus observer/KVO, cancel deferred action, destroy mutex |

### `@implementation ContentPaginationViewController` (5616)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 5623 | 5630 | `init` | Muat nib, seed page/maxPage ke 1 |
| 5632 | 5639 | `paginationGoAction:` | Teruskan action ke `target` |
| 5641 | 5644 | `makeInputFirstResponder` | Fokus field halaman |
| 5646 | 5653 | `isFirstResponderInside` | Apakah first responder di dalam view ini |

## 4. Temuan menonjol / UNKNOWN

- **Path count murah vs akurat (by design).** `updateNumberOfRows` (4080–4140) memakai `tableRowsCount` untuk tabel tanpa filter/limit (murah, `RowsCountAccurate = "y"`), selain itu `updateAccurateNumberOfRowsForCurrentTableForcingUpdate:NO` + fallback estimasi (`maxNumRowsIsEstimate`). `updateCountText` (1330–1383) merender estimasi dengan awalan `~`. `tableFilterString` diawali komentar `// TODO: this is called A LOT, optimize` (1329).
- **Logika pembangun WHERE terduplikasi.** `argumentForRow:ofTable:andDatabase:includeBlobs:` (1976–2049) dan `argumentForRow:excludingLimits:` (3292–3402) keduanya menyusun predikat identifikasi baris, dengan special-case `NSNull`, BIT (`b'…'`), geometry, `NSData` yang nyaris duplikat (2029–2041 vs 3370–3388; 2415–2423 vs 3379–3381).
- **`removeRow:` adalah method terbesar (2176–2565 ≈ 390 baris)** dengan tiga strategi delete inline: per-baris (2372–2398), PK satu kolom `IN (…)` chunk 256 KB (2399–2456), dan multi-PK `OR` (2457–2564), plus dua alur alert.
- **`_buildTableColumns:withFont:` (724–726)** hanya wrapper tipis delegasi ke varian `filterTerms:`.
- **`setFilterTableData:` (3940)** punya `;` nyasar sebelum body `{`.
- **Dead code.** `knownColumnFormatters` (5472–5478) hanya UUID; formatter HEX dikomentari (5475–5476). Header baris 262 punya prototipe `escapeFilterArgument:againstClause:` yang dikomentari tanpa implementasi. `deriveQueryString` mencatat "these class matches are not ever met at present" (3056–3057).
- **`ContentPaginationViewController`** adalah workaround sementara yang didefinisikan di `.m` ini saja (TODO 74–78).
- **`visibleColumns` (`.h:101`) dideklarasikan tapi tak direferensikan** di anchor yang dicari — **UNKNOWN** bila dipakai di tempat lain.
- **Pola thread-safety manual berulang**: `tableValuesLock` di-lock/unlock di `clearTableValues` (937–941), `updateResultStore:` (1249–1252), `autosizeColumns` (4148–4150), `objectValueForTableColumn:` (4384–4390), `willDisplayCell:` (5123–5129), `toolTipForCell:` (5190–5196); `dealloc` destroy mutex (5607).
- **`dealloc` mencetak ke console** (`NSLog(@"Dealloc called …")`, 5609) — sisa debug.
