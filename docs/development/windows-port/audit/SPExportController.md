# Audit: `SPExportController.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/DataExport/SPExportController.m`. Baseline commit `02c910932`.

- File: 3982 baris; header `SPExportController.h` 357 baris.
- Peran: window-controller dialog Export — mengumpulkan opsi export
  (SQL/CSV/XML/Dot), enumerasi tabel, membuat operasi `SPExporter` per-format,
  melacak progress/cancel, dan persist/restore setelan export.
- Conformance di `.m:95`: `SPCSVExporterProtocol, SPSQLExporterProtocol, SPXMLExporterProtocol,
  SPDotExporterProtocol, SPPDFExporterProtocol, SPHTMLExporterProtocol, NSMenuItemValidation,
  NSControlTextEditingDelegate`. **Tidak ada** `SPExcelExporterProtocol`/delegate Excel.

## 1. Inventaris ivar/property

### Ivar `.h` (50–241)
| Baris | Grup | Isi |
| --- | --- | --- |
| 52–56 | Controller | `tableDocumentInstance`, `tableContentInstance`, `customQueryInstance`, `tablesListInstance`, `tableDataInstance` |
| 59–71 | Window export | `exporterView`, `exportButton`, `exportPathField`, `exportTableList`, `exportTypeTabBar`, `exportOptionsTabBar`, `exportInputPopUpButton`, `exportFilePerTableCheck`, tombol select/deselect/refresh, `exportTablelistScrollView`, `exportFilenameDividerBox` |
| 74–75 | Sheet error | `errorsWindow`, `errorsTextView` |
| 78–85 | Advanced options | `exportAdvancedOptionsViewButton`, `exportAdvancedOptionsView`, `exportAdvancedOptionsViewLabelButton`, `exportUseUTF8BOMButton`, `exportProcessLowMemoryButton`, `exportOutputCompressionFormatPopupButton`, `exportTableListButtonBar` |
| 88–92 | Sheet progress | `exportProgressWindow`, `exportProgressTitle`, `exportProgressText`, `exportFormatInfoText`, `exportProgressIndicator` |
| 95–99 | Custom filename | `exportCustomFilenameViewButton`, `exportCustomFilenameViewLabelButton`, `exportCustomFilenameView`, `exportCustomFilenameTokenField`, `exportCustomFilenameTokenPool` |
| 102–110 | SQL | `exportSQLIncludeStructureCheck`, `exportSQLIncludeDropSyntaxCheck`, `exportSQLIncludeContentCheck`, `exportSQLIncludeErrorsCheck`, `exportSQLBLOBFieldsAsHexCheck`, `exportSQLInsertNValueTextField`, `exportSQLInsertDividerPopUpButton`, `exportSQLIncludeAutoIncrementValueButton`, `exportSQLIncludeGeneratedColumnsCheck` |
| 113–118 | CSV | `exportCSVIncludeFieldNamesCheck`, `exportCSVFieldsTerminatedField`, `exportCSVFieldsWrappedField`, `exportCSVFieldsEscapedField`, `exportCSVLinesTerminatedField`, `exportCSVNULLValuesAsTextField` |
| 121–124 | XML | `exportXMLFormatPopUpButton`, `exportXMLIncludeStructure`, `exportXMLIncludeContent`, `exportXMLNULLValuesAsTextField` |
| 127 | Dot | `exportDotForceLowerTableNamesCheck` |
| 132–240 | Scalar | `mainNibLoaded`, `exportCancelled`, `exportToMultipleFiles`, `createCustomFilename`, `exportTableCount`, `currentTableExportIndex`, `exportTypeLabel`, `exportFilename`, `tables`, `connection`, `serverSupport`, `operationQueue`, `exporters`, `exportFiles`, `exportType`, `exportSource`, `showAdvancedView`, `showCustomFilenameView`, `prefs`, `previousConnectionEncoding`, `previousConnectionEncodingViaLatin1`, `serverLowerCaseTableNameValue`, `heightOffset1/2`, `windowMinWidth/Heigth`, `localizedTokenNames` |

### Property `.h` readwrite (246–263)
`exportCancelled` (246), `exportToMultipleFiles` (251), `connection` (256), `serverSupport` (257),
`appScopedBookmark` (259), `startTime` (260), `userChosenDirectory` (261),
`changeExportOutputPathPanel` (262), `bookmarks` (263).

### Tambahan class-extension `.m`
`@property (readwrite, copy) NSString *exportDatabaseName` (`.m:96`, **hanya di extension**);
`@synthesize` 172–180; forward decl `IsOn`/`SetOnOff` (86, 91); helper C global 3964–3982.

## 2. Semua `#pragma mark` (urut baris)

Extension (95–168): `114 (SPExportFileUtilitiesPrivateAPI), 118 (SPExportControllerDelegate), 123 (SPExportSettingsPersistence), 164 (Shared Private)`.
Implementation (170–3960): `181/182 (Initialisation), 278/279 (Export methods), 371/372 (IB action methods), 988/989 (Other), 1008/1009 (Private API), 1307 (SPExportInitializer), 1876 (SPExportFileUtilitiesPrivateAPI), 2223 (SPExportFilenameUtilities), 2517 (SPExportInterfaceController), 2634 (SPExportControllerDelegate), 2636 (Table view datasource methods), 2657/2658 (Table view delegate methods), 2670/2671 (Tabview delegate methods), 2680/2681 (Token field delegate methods), 2794/2795 (Combo box delegate methods), 2804 (unnamed), 2939 (SPExportSettingsPersistence), 3644 (SPCSVExporterDelegate), 3732 (SPSQLExporterDelegate), 3778 (SPXMLExporterDelegate), 3875 (SPDotExporterDelegate), 3918 (SPPDFExporterDelegate), 3933 (SPHTMLExporterDelegate), 3948/3949 (Memory Management)`.
Setelah `@end`: `3962` (sebelum C global).

## 3. Tabel method (urut baris; `@implementation` mulai 170)

### Initialisation (181–182)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 187 | 237 | `init` | Set default, alokasi koleksi, bangun dict token nama file |
| 242 | 276 | `awakeFromNib` | Setup sekali: ukuran window, tab default, tweak kontrol |

### Export methods (278–279)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 288 | 347 | `exportTables:asFormat:usingSource:` | Tampilkan window export pra-pilih tabel/format/sumber; buka sheet |
| 354 | 360 | `openExportErrorsSheetWithString:` | Tampilkan teks error di sheet error |
| 365 | 369 | `displayExportFinishedNotification` | Post notifikasi "Export Finished" |

### IB action methods (371–372)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 384 | 414 | `exportData` | Buka dialog pilih sumber/tipe dari konteks aktif |
| 419 | 469 | `closeSheet:` | Validasi path, collapse subview, tutup sheet export |
| 471 | 507 | `setExportInput:` | Set sumber export + validasi ketersediaan |
| 512 | 515 | `switchInput:` | Terapkan sumber terpilih dari popup |
| 520 | 524 | `cancelExportForFile:` | Bungkus error file-handle sebagai sender cancel |
| 530 | 569 | `cancelExport:` | Cancel semua operasi exporter; mulai drain-and-cleanup async |
| 571 | 575 | `_waitUntilQueueIsEmptyAfterCancelling:` | Tunggu background sampai operation queue kosong |
| 577 | 599 | `_queueIsEmptyAfterCancelling:` | Hapus file parsial, sembunyikan progress, restore encoding |
| 601 | 610 | `_hideExportProgress` | Akhiri + sembunyikan sheet progress, stop indikator |
| 613 | 615 | `panel:shouldEnableURL:` | NSOpenSavePanelDelegate; selalu YES |
| 620 | 700 | `changeExportOutputPath:` | Buka panel direktori, validasi URL, simpan security bookmark |
| 705 | 790 | `refreshTableList:` | Bangun ulang array tabel (+ proc/function), pertahankan seleksi |
| 795 | 821 | `selectDeselectAllTables:` | Pilih/batal semua flag struktur/konten/drop |
| 826 | 829 | `changeExportCompressionFormat:` | Refresh nama file tampil setelah kompresi berubah |
| 834 | 851 | `toggleCustomFilenameFormatView:` | Show/hide view nama file kustom + resize window |
| 856 | 868 | `toggleXMLOutputFormat:` | Enable opsi XML per format MySQL vs Plain |
| 873 | 889 | `toggleAdvancedExportOptionsView:` | Show/hide box advanced + resize window |
| 894 | 908 | `toggleSQLIncludeStructure:` | Enable drop/auto-increment & kolom; revaluasi tombol export |
| 913 | 949 | `tableView:mouseDownInHeaderOfTableColumn:` | Klik header kolom = toggle semua baris untuk aspek itu |
| 954 | 959 | `toggleSQLIncludeContent:` | Show/hide kolom content; revaluasi tombol export |
| 964 | 969 | `toggleSQLIncludeDropSyntax:` | Show/hide kolom drop; revaluasi tombol export |
| 974 | 978 | `toggleNewFilePerTable:` | Update info format + token nama file tersedia |
| 983 | 986 | `exportCustomQueryResultAsFormat:` | Export hasil custom query pakai tag format item menu |

### Other (988–989)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 991 | 994 | `tableListChangedAlertDidEnd:returnCode:contextInfo:` | Stub kosong (dead?) |
| 999 | 1006 | `validateMenuItem:` | Enable menu export hasil query hanya bila ada data |

### Private API (1008–1009)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1014 | 1094 | `_switchTab` | Konfigurasi ulang UI per tipe/sumber export; query setelan case |
| 1099 | 1126 | `_checkForDatabaseChanges` | Warning bila jumlah tabel berubah, else init export |
| 1133 | 1151 | `_displayExportTypeOptions:` | Animasi visibilitas tab opsi + lebar list tabel |
| 1156 | 1182 | `_updateExportFormatInformation` | Tampilkan catatan format tak bisa diimpor |
| 1187 | 1214 | `_updateExportAdvancedOptionsLabel` | Bangun judul ringkas advanced options |
| 1221 | 1281 | `_toggleExportButton:` | Hitung enablement tombol export dari dict state UI |
| 1286 | 1295 | `_toggleExportButtonOnBackgroundThread` | Snapshot state check SQL, spawn evaluasi background |
| 1302 | 1305 | `_toggleExportButtonWithBool:` | Terapkan enablement ke tombol export di main thread |

### SPExportInitializer (1307)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1313 | 1339 | `startExport` | Cache encoding, buka sheet progress, antrikan exporter pertama |
| 1344 | 1368 | `exportEnded` | Sembunyikan progress, restore query mode/encoding, notify selesai |
| 1373 | 1466 | `initializeExportUsingSelectedOptions` | Kumpulkan data/tabel per sumber, set label, mulai export |
| 1476 | 1744 | `exportTables:orDataArray:` | Bangun exporter CSV/SQL/XML/Dot + file export |
| 1752 | 1809 | `initializeCSVExporterForTable:orDataArray:` | Konfigurasi exporter CSV + penamaan per-file |
| 1817 | 1874 | `initializeXMLExporterForTable:orDataArray:` | Konfigurasi exporter XML + penamaan per-file |

### SPExportFileUtilitiesPrivateAPI (1876)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1884 | 1897 | `outputEncodingForCurrentExportType` | Kembalikan encoding output per-format via SAExportOutputEncoding |
| 1904 | 1935 | `writeCSVHeaderToExportFile:` | Tulis baris header host/database/waktu ke file CSV |
| 1942 | 1976 | `writeXMLHeaderToExportFile:` | Tulis prolog/komentar/root tag XML |
| 1984 | 2193 | `errorCreatingExportFileHandles:` | Klasifikasi kegagalan pembuatan file; alert replace/skip/cancel |
| 2198 | 2221 | `_openExportSheet` | Buka sheet export, simpan setelan, dispatch ke init/check |

### SPExportFilenameUtilities (2223)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2228 | 2247 | `updateDisplayedExportFilename` | Refresh teks label "Customize Filename" |
| 2249 | 2268 | `customFilenamePathExtension` | Turunkan ekstensi dari token field |
| 2270 | 2299 | `isTableTokenAllowed` | Putuskan apakah token {table} valid untuk seleksi |
| 2304 | 2341 | `updateAvailableExportFilenameTokens` | Bangun ulang token pool; buang {table} terlarang |
| 2343 | 2353 | `currentAllowedExportFilenameTokens` | Kembalikan objek token dari pool |
| 2360 | 2386 | `generateDefaultExportFilename` | Bangun nama default by sumber + tanggal + ekstensi |
| 2393 | 2432 | `currentDefaultExportFileExtension` | Kembalikan ekstensi format termasuk sufiks kompresi |
| 2443 | 2515 | `expandCustomFilenameFormatUsingTableName:` | Ekspansi token (host/db/table/date); sanitasi |

### SPExportInterfaceController (2517)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2525 | 2576 | `_resizeWindowForCustomFilenameViewByHeightDelta:` | Animasi tinggi window untuk custom filename view |
| 2584 | 2632 | `_resizeWindowForAdvancedOptionsViewByHeightDelta:` | Animasi tinggi window untuk advanced options |

### SPExportControllerDelegate (2634)
Table view datasource (2636):
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2638 | 2641 | `numberOfRowsInTableView:` | Jumlah tabel di list export |
| 2643 | 2646 | `tableView:objectValueForTableColumn:row:` | Nilai sel (nama tabel atau flag) |
| 2648 | 2655 | `tableView:setObjectValue:forTableColumn:row:` | Simpan edit sel, refresh token/tombol/info format |

Table view delegate (2658):
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2660 | 2663 | `tableView:shouldTrackCell:forTableColumn:row:` | Hanya track sel untuk export table list |
| 2665 | 2668 | `tableView:willDisplayCell:forTableColumn:row:` | Render sel dengan font sistem kecil |

Tabview delegate (2671):
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2673 | 2678 | `tabView:didSelectTabViewItem:` | Reassign content view + reconfigure via `_switchTab` |

Token field delegate (2681):
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2686 | 2691 | `tokenField:styleForRepresentedObject:` | Style default token; plain untuk teks |
| 2693 | 2715 | `tokenField:writeRepresentedObjects:toPasteboard:` | Serialisasi token/teks ke pasteboard plist+string |
| 2717 | 2747 | `tokenField:readFromPasteboard:` | Bangun ulang token/teks dari pasteboard, sanitasi newline |
| 2753 | 2756 | `tokenField:shouldAddObjects:atIndex:` | Delegasi split mixed-content saat token ditambah |
| 2758 | 2765 | `tokenField:displayStringForRepresentedObject:` | Nama tampil terlokalisasi untuk token |
| 2770 | 2773 | `tokenField:representedObjectForEditingString:` | Kembalikan editing string apa adanya |
| 2779 | 2792 | `controlTextDidChange:` | Update preview nama file; jadwalkan tokenisasi tertunda |

Combo box delegate (2795):
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2797 | 2802 | `comboBoxSelectionDidChange:` | Refresh nama file saat combo terminator CSV berubah |

Blok unnamed (2804):
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2810 | 2877 | `_updateTokensForMixedContent:` | Split/merge teks + konversi `{token}` valid jadi objek |
| 2879 | 2937 | `_tokenizeCustomFilenameTokenField` | Sintesis event key Return untuk memicu tokenisasi |

### SPExportSettingsPersistence (2939)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2944 | 2952 | `+describeExportSource:` | Enum→string sumber export |
| 2954 | 2960 | `+copyExportSourceForDescription:to:` | String→enum sumber export |
| 2962 | 2975 | `+describeExportType:` | Enum→string tipe export |
| 2977 | 2987 | `+copyExportTypeForDescription:to:` | String→enum tipe export (HTML/PDF/Excel dikomentari) |
| 2989 | 2997 | `+describeCompressionFormat:` | Enum→string format kompresi |
| 2999 | 3005 | `+copyCompressionFormatForDescription:to:` | String→enum format kompresi |
| 3007 | 3014 | `+describeXMLExportFormat:` | Enum→string format XML |
| 3016 | 3021 | `+copyXMLExportFormatForDescription:to:` | String→enum format XML |
| 3023 | 3030 | `+describeSQLExportInsertDivider:` | Enum→string divider insert SQL |
| 3032 | 3037 | `+copySQLExportInsertDividerForDescription:to:` | String→enum divider insert SQL |
| 3042 | 3095 | `importCurrentSettings:` | Buka panel, baca plist, terapkan setelan, tampilkan error |
| 3097 | 3129 | `exportCurrentSettings:` | Save panel, serialisasi setelan ke plist, tulis file |
| 3131 | 3151 | `currentCustomFilenameAsArray` | Serialisasi token field ke array string/dict |
| 3153 | 3182 | `setCustomFilenameFromArray:` | Restore token field dari array, validasi token |
| 3184 | 3219 | `currentSettingsAsDictionary` | Serialisasi semua setelan export ke satu dict |
| 3221 | 3320 | `applySettingsFromDictionary:error:` | Validasi + terapkan dict setelan ke UI |
| 3322 | 3341 | `exporterSettings` | Dispatch ke getter setelan per-format; throw untuk HTML/PDF/Excel |
| 3343 | 3362 | `applyExporterSettings:` | Dispatch ke setter per-format; throw untuk HTML/PDF/Excel |
| 3364 | 3375 | `csvSettings` | Dict setelan CSV |
| 3377 | 3389 | `applyCsvSettings:` | Terapkan setelan CSV ke UI |
| 3391 | 3394 | `dotSettings` | Dict setelan Dot |
| 3396 | 3400 | `applyDotSettings:` | Terapkan setelan Dot ke UI |
| 3402 | 3411 | `xmlSettings` | Dict setelan XML |
| 3413 | 3426 | `applyXmlSettings:` | Terapkan setelan XML ke UI |
| 3428 | 3452 | `sqlSettings` | Dict setelan SQL |
| 3454 | 3479 | `applySqlSettings:` | Terapkan setelan SQL ke UI |
| 3481 | 3500 | `exporterSpecificSettingsForSchemaObject:ofType:` | Dispatch getter per-objek; throw untuk HTML/PDF/Excel |
| 3502 | 3521 | `applyExporterSpecificSettings:forSchemaObject:ofType:` | Dispatch setter per-objek; throw untuk HTML/PDF/Excel |
| 3523 | 3527 | `dotSpecificSettingsForSchemaObject:ofType:` | Kembalikan nil (Dot = seluruh database) |
| 3529 | 3532 | `applyDotSpecificSettings:forSchemaObject:ofType:` | No-op (komentar "should never be called") |
| 3534 | 3546 | `xmlSpecificSettingsForSchemaObject:ofType:` | Flag konten ya/tidak tabel |
| 3548 | 3560 | `applyXmlSpecificSettings:forSchemaObject:ofType:` | Set flag konten tabel |
| 3562 | 3574 | `csvSpecificSettingsForSchemaObject:ofType:` | Flag konten ya/tidak tabel |
| 3576 | 3588 | `applyCsvSpecificSettings:forSchemaObject:ofType:` | Set flag konten tabel |
| 3590 | 3620 | `sqlSpecificSettingsForSchemaObject:ofType:` | Array flag struktur/konten/drop tabel |
| 3622 | 3642 | `applySqlSpecificSettings:forSchemaObject:ofType:` | Terapkan flag struktur/konten/drop ke baris tabel |

### SPCSVExporterDelegate (3644)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3646 | 3666 | `csvExportProcessWillBegin:` | Progress indeterminate + teks "Fetching data" |
| 3668 | 3707 | `csvExportProcessComplete:` | Tulis header tabel berikut / tutup file; antrikan exporter berikut |
| 3709 | 3725 | `csvExportProcessWillBeginWritingData:` | Switch progress ke determinate "Writing data" |
| 3727 | 3730 | `csvExportProcessProgressUpdated:` | Update indikator progress |

### SPSQLExporterDelegate (3732)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3734 | 3741 | `sqlExportProcessWillBegin:` | Set judul/teks progress "Exporting SQL"/"Dumping" |
| 3743 | 3751 | `sqlExportProcessComplete:` | Akhiri export; tampilkan sheet error bila ada |
| 3753 | 3761 | `sqlExportProcessProgressUpdated:` | Update nilai indikator progress |
| 3763 | 3771 | `sqlExportProcessWillBeginFetchingData:` | Teks fetch per-tabel, reset indikator |
| 3773 | 3776 | `sqlExportProcessWillBeginWritingData:` | Teks writing per-tabel |

### SPXMLExporterDelegate (3778)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3780 | 3801 | `xmlExportProcessWillBegin:` | Progress indeterminate + teks fetch per-tabel |
| 3803 | 3850 | `xmlExportProcessComplete:` | Tulis tag penutup XML; antrikan exporter berikut / akhiri |
| 3852 | 3855 | `xmlExportProcessProgressUpdated:` | Update indikator progress di main thread |
| 3857 | 3873 | `xmlExportProcessWillBeginWritingData:` | Switch progress ke determinate "Writing data" |

### SPDotExporterDelegate (3875)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3877 | 3886 | `dotExportProcessWillBegin:` | Set judul/teks progress Dot, stop spinner |
| 3888 | 3891 | `dotExportProcessComplete:` | Akhiri export Dot |
| 3893 | 3896 | `dotExportProcessProgressUpdated:` | Update indikator progress |
| 3898 | 3906 | `dotExportProcessWillBeginFetchingData:forTableWithIndex:` | Teks fetch per-tabel untuk Dot |
| 3908 | 3916 | `dotExportProcessWillBeginFetchingRelationsData:` | Teks fetch relations; mulai spin |

### SPPDFExporterDelegate (3918) — STUB
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3920 | 3922 | `pdfExportProcessWillBegin:` | STUB KOSONG |
| 3924 | 3927 | `pdfExportProcessComplete:` | Hanya panggil `exportEnded` |
| 3929 | 3931 | `pdfExportProcessWillBeginWritingData:` | STUB KOSONG |

### SPHTMLExporterDelegate (3933) — STUB
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3935 | 3937 | `htmlExportProcessWillBegin:` | STUB KOSONG |
| 3939 | 3942 | `htmlExportProcessComplete:` | Hanya panggil `exportEnded` |
| 3944 | 3946 | `htmlExportProcessWillBeginWritingData:` | STUB KOSONG |

### Memory Management (3948–3949)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3950 | 3958 | `dealloc` | Stop akses security-scoped; bersihkan serverSupport |

### C global (3962+)
| Start | End | Function | Tujuan |
| --- | --- | --- | --- |
| 3964 | 3967 | `IS_TOKEN(id)` | Uji objek `SPExportFileNameTokenObject` |
| 3969 | 3972 | `IS_STRING(id)` | Uji objek `NSString` |
| 3974 | 3977 | `IsOn(NSButton *)` | Konversi state tombol → `NSNumber` boolean |
| 3979 | 3982 | `SetOnOff(NSNumber *, NSButton *)` | Set state tombol dari boolean `NSNumber` |

## 4. Temuan menonjol / UNKNOWN

**HTML/PDF/Excel tidak diimplementasi**
- Stub HTML `htmlExportProcessWillBegin:` (3935) dan `…WillBeginWritingData:` (3944) **body kosong**; hanya `…Complete:` panggil `exportEnded` (3939).
- Stub PDF sama (3920, 3929, 3924).
- Excel: tak ada section delegate maupun method.
- Engine hanya menangani SQL/CSV/XML/Dot: `_switchTab` mengomentari HTML/PDF (`//BOOL isHTML`, `//BOOL isPDF`) dan mengecualikannya dari `enable` (1030–1034); `initializeExportUsingSelectedOptions` dan `currentDefaultExportFileExtension` jatuh ke `raise`.
- Getter/setter persistensi menandai HTML/PDF/Excel sebagai belum diimplementasi via `@throw ... @"exportType not implemented!"` (`exporterSettings` 3333–3339, `applyExporterSettings:` 3354–3360, `exporterSpecificSettingsForSchemaObject:ofType:` 3492–3498, `applyExporterSpecificSettings:…` 3513–3519); `copyExportTypeForDescription:to:` punya case PDF/HTML/Excel **dikomentari** (2983–2985).

**Exception**
- `[NSException raise:NSInvalidArgumentException format:@"unsupported exportType=%lu",exportType]` di `initializeExportUsingSelectedOptions` (1451) dan `currentDefaultExportFileExtension` (2415).
- `NSInternalInconsistencyException` di `customFilenamePathExtension` (2264), `tokenField:writeRepresentedObjects:toPasteboard:` (2708), `tokenField:readFromPasteboard:` (2735), `_tokenizeCustomFilenameTokenField` (2914).

**Lain-lain**
- **`goto` di kode nyata**: `setExportInput:` pakai `goto set_input;` dengan komentar `//MARK: a goto in the WILD!` (484, label 491).
- **Method stub dead/empty**: `tableListChangedAlertDidEnd:returnCode:contextInfo:` (991–994) body kosong; **UNKNOWN** apakah masih dipanggil.
- **No-op by design**: `applyDotSpecificSettings:forSchemaObject:ofType:` (3529–3532) kosong; `dotSpecificSettingsForSchemaObject:ofType:` selalu nil ("Dot is a graph of the whole database").
- **Kemungkinan bug laten (UNKNOWN intent)**: `writeCSVHeaderToExportFile:` menulis nama tabel dari `[[tables firstObject] firstObject]` (1932) alih-alih tabel yang sedang diexport.
- **Property privat** `exportDatabaseName` hanya di class-extension (96).
- **Penempatan pragma keliru**: `_openExportSheet` (2198) setelah blok `SPExportFileUtilitiesPrivateAPI`; `_waitUntilQueueIsEmptyAfterCancelling:`/`_queueIsEmptyAfterCancelling:` (571/577) dan `_hideExportProgress` (601) ada di region "IB action methods".
- **Hack timing `exportEnded`**: menyembunyikan progress tertunda untuk export < 2 s (1348–1352).
