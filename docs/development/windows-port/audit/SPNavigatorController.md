# Audit: `SPNavigatorController.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/SubviewControllers/SPNavigatorController.m`. Baseline commit
`02c910932`.

- File: 1309 baris; header 98 baris.
- Peran: singleton `NSWindowController` window "Navigator" — outline view
  connections → database → tabel → field + info table, dengan filter tree dan
  "sync to active document".
- Total: 52 definisi method + 2 deklarasi privat + 1 fungsi C static.

## 1. Inventaris ivar

Semua ivar gaya lama `@interface { }` (`.h:34-67`); **tak ada `@property`**.

| Ivar | `.h` | Catatan |
| --- | --- | --- |
| `outlineSchema2` (`SPNavigatorOutlineView*`) | 35 | outline utama |
| `navigatorWindow` (IBOutlet id) | 36 | **tak direferensikan** di `.m` → UNKNOWN/unused (window diakses via `[self window]`) |
| `infoTable` (IBOutlet id) | 37 | tabel info bawah |
| `searchField` (IBOutlet id) | 38 | filter |
| `syncButton` | 39 | toggle sync |
| `schemaStatusSplitView` (`SPSplitView*`) | 41 | split vertikal |
| `schema12SplitView` (IBOutlet id) | 42 | split horizontal |
| `prefs` | 44 | di-assign 134; **tak pernah dibaca** → dead store |
| `schemaData` | 45 | model tree inti |
| `schemaDataFiltered` | 46 | model tree terfilter |
| `allSchemaKeys` | 47 | daftar key flat untuk search/autocomplete |
| `infoArray` | 48 | baris tabel info |
| `updatingConnections` | 49 | koneksi sedang update |
| `expandStatus2` | 50 | state expand tersimpan |
| `cachedSortedKeys` | 51 | cache sortir node besar |
| `selectedKey2` | 52 | path seleksi tersimpan |
| `selectionViewPort2` (NSRect) | 53 | viewport tersimpan |
| `ignoreUpdate` (BOOL) | 54 | skip update struktur berikut |
| `isFiltered` (BOOL) | 55 | filter aktif |
| `isFiltering` (BOOL) | 56 | filter sedang berjalan (guard) |
| `connectionIcon`…`fieldIcon` (7 NSImage) | 58–64 | ikon sel |
| `NSDictionaryClass` (Class) | 66 | cache `[NSDictionary class]` |

## 2. Semua `#pragma mark`

`.m:162, 561/562 (IBActions), 738/739 (outline delegates), 1190/1191 (table delegates), 1251/1252 (others)`.

## 3. Tabel method (urut baris)

### Singleton/lifecycle (58–160)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 60 | 63 | `static compareStrings(…)` | Comparator case/localized untuk sort key tree |
| 68 | 84 | `+sharedNavigatorController` | Buat singleton (di main thread) |
| 86 | 92 | `+allocWithZone:` | Paksa semua alokasi lewat singleton |
| 94 | 114 | `init` | Init window nib + semua collection ivar |
| 116 | 120 | `dealloc` | Hapus self dari notification center |
| 125 | 125 | `copyWithZone:` | Kembalikan self untuk semantik singleton |
| 130 | 155 | `awakeFromNib` | Wire split view, drag type, ikon, observer |
| 157 | 160 | `windowFrameAutosaveName` | Nama autosave `SPNavigator` |

### Core block (162–559)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 164 | 169 | `syncMode` | Apakah seleksi outline sinkron ke dokumen aktif |
| 171 | 187 | `restoreExpandStatus` | Re-expand baris dari `expandStatus2` tersimpan |
| 189 | 208 | `saveSelectedItems` | Catat key item terpilih + rect viewport |
| 210 | 253 | `selectPath:` | Expand/scroll/pilih baris outline untuk schema path |
| 255 | 282 | `restoreSelectedItems` | Restore seleksi + posisi scroll |
| 284 | 287 | `setIgnoreUpdate:` | Set flag `ignoreUpdate` |
| 289 | 333 | `removeConnection:` | Buang data schema koneksi bila window terakhir tutup |
| 335 | 339 | `_setSyncButtonOn` | Enable ulang tombol sync + trigger aksinya |
| 341 | 397 | `selectInActiveDocumentItem:fromView:` | Pilih db/tabel yang diklik di dokumen aktif |
| 399 | 404 | `updateNavigator:` | Handler notifikasi; majukan ke update di main thread |
| 406 | 478 | `updateEntriesForConnection:` | Bangun ulang struktur tersimpan satu koneksi |
| 480 | 488 | `schemaPathExistsForConnection:andDatabase:` | Cek key db ada di `schemaData` |
| 490 | 495 | `dbStructureForConnection:` | Kembalikan salinan dict struktur koneksi |
| 497 | 505 | `allSchemaKeysForConnection:` | Array key schema flat untuk koneksi |
| 513 | 549 | `getUniqueDbIdentifierFor:andConnection:ignoreFields:` | Resolve term ke identitas db/tabel untuk completion |
| 551 | 554 | `isUpdatingConnection:` | Apakah koneksi sedang update |
| 556 | 559 | `isUpdating` | Apakah ada koneksi sedang update |

### IBActions (561)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 564 | 590 | `reloadAllStructures:` | Bersihkan state + re-query struktur di background |
| 592 | 594 | `outlineViewAction:` | Stub aksi outline kosong |
| 596 | 695 | `filterTree:` | Bangun struktur terfilter dari pola search |
| 697 | 705 | `reloadAfterFiltering` | Reload background + expand tree terfilter |
| 707 | 736 | `syncButtonAction:` | Expand + pilih path db/tabel dokumen aktif |

### outline delegates (739)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 741 | 758 | `outlineViewItemDidExpand:` | Catat expand di `expandStatus2` |
| 760 | 775 | `outlineViewItemDidCollapse:` | Hapus expand dari `expandStatus2` |
| 777 | 833 | `outlineView:child:ofItem:` | Kembalikan child ke-N; pakai cache key tersortir untuk node besar |
| 835 | 846 | `outlineView:isItemExpandable:` | Expandable kecuali proc/function |
| 848 | 863 | `outlineView:numberOfChildrenOfItem:` | Jumlah child dari model terfilter atau utama |
| 865 | 975 | `outlineView:objectValueForTableColumn:byItem:` | Teks sel + ikon untuk kolom name/type |
| 977 | 983 | `outlineView:isGroupItem:` | Gaya group kecuali baris level-3 |
| 985 | 1000 | `outlineView:heightOfRowByItem:` | Baris spacer 5pt ringkas untuk separator level-3 |
| 1002 | 1005 | `outlineView:shouldExpandItem:` | Selalu izinkan expand |
| 1007 | 1021 | `outlineView:shouldSelectItem:` | Larang pilih baris separator level-3 |
| 1025 | 1029 | `outlineView:shouldEditTableColumn:item:` | Perlakukan double-click sebagai select-in-document |
| 1031 | 1074 | `outlineViewSelectionDidChange:` | Bangun baris tabel info untuk item terpilih |
| 1076 | 1082 | `outlineView:didClickTableColumn:` | Collapse pane status saat header diklik |
| 1090 | 1100 | `outlineView:pasteboardWriterForItem:` | Buat pasteboard writer drag per-item |
| 1102 | 1138 | `outlineView:draggingSession:willBeginAtPoint:forItems:` | Lampirkan path, CREATE stmt, teks ke drag |
| 1148 | 1172 | `navigatorDragCarriesTextForItem:inOutlineView:` | Predikat: apakah drag akan membawa teks |
| 1178 | 1188 | `schemaPathForItem:inOutlineView:` | Schema path item dengan connection ID dilepas |

### table delegates (1191)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1193 | 1200 | `numberOfRowsInTableView:` | Jumlah baris tabel info |
| 1202 | 1206 | `tableView:heightOfRow:` | Baris dummy pertama 5pt, lain 16pt |
| 1208 | 1214 | `tableView:shouldSelectRow:` | Baris tabel info tak selectable |
| 1216 | 1229 | `tableView:willDisplayCell:forTableColumn:row:` | Ikon/indentasi sel info |
| 1231 | 1238 | `tableView:objectValueForTableColumn:row:` | String baris info |
| 1240 | 1249 | `tableView:didClickTableColumn:` | Expand pane status saat header info diklik |

### others (1252)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1254 | 1307 | `tableInfoLabelForIndex:ofType:` | Label info field/proc/func terlokalisasi |

## 4. Temuan menonjol / UNKNOWN

**Build/sort cache**
- `outlineView:child:ofItem:` (777–833) hanya mengaktifkan cache key tersortir bila node punya **> 50 key** dan ada parent (791–793); menyimpan key tersortir di `cachedSortedKeys` ber-key deskripsi `parentKeys firstObject` (806–816). Cache dibersihkan di `updateEntriesForConnection:` (417) dan `reloadAllStructures:` (579).
- Sortir via `compareStrings` (`localizedCompare:`, 60–63) dengan `sortedArrayUsingFunction:`; ada `;;` ganda di 62 (harmless).
- **UNKNOWN:** cache hanya di-key deskripsi `firstObject` parent, bukan per view (filtered vs full); cabang cache digerbangi `!isFiltered` (793), jadi tak bisa kolid — tapi coupling implisit.

**Filter**
- `filterTree:` (596–695) lowercase pola, `.`→`SPUniqueSchemaDelimiter` (599), bangun salinan terpangkas `schemaData` ke `schemaDataFiltered` (636–679), pertahankan `"  struct_type  "` (664–666). Panjang min 2 (630). Tak jalan saat `isFiltering` (630).
- Reload di thread terpisah `SPNavigatorController update after filtering` (681) → `reloadAfterFiltering` (697–705) yang reload + expand baris 0. **UNKNOWN/risiko:** `reloadAfterFiltering` memanggil `[outlineSchema2 reloadData]` off main thread (701–702) — potensi pelanggaran main-thread AppKit.
- `filterTree:` membungkus logika dalam `@try/@catch` menampilkan tooltip saat gagal (683–693).

**Ikon**
- Dibuat di `awakeFromNib` (145–151), diterapkan di `objectValueForTableColumn:` via `[[tableColumn dataCell] setImage:…]` (883, 903–935). Baris field pakai `fieldIcon` (932); baris dummy `"dummy-small"` return nil (935–936). Tabel info pakai `"table-property"` (1224).
- **UNKNOWN:** `connectionIcon` di 883 untuk level-0; `databaseIcon` dipakai untuk db nyata dan "not queried yet" (918, 927).

**Sync**
- `syncMode` (164–169) YES hanya bila window terlihat, sync button on, dan ≥2 baris terpilih. `syncButtonAction:` (707–736) matikan filter, bangun path dokumen aktif, `selectPath:` (733).
- `selectInActiveDocumentItem:fromView:` (341–397) sementara men-disable sync button sekitar pemilihan dokumen lalu re-enable via `_setSyncButtonOn` setelah 0.1 s (379–386), dengan `TODO` soal rendering GUI sync-mode (377–378).

**Drag & drop**
- Writer per-item (1090–1100) menolak item tanpa schema path/teks; payload whole-drag (path ter-archive, `CREATE TABLE … SELECT`, teks koma) dilampirkan di `draggingSession:willBeginAtPoint:forItems:` (1102–1138). Delegasi ke Swift `SADragPasteboard` (1099, 1117, 1126, 1137, 1166, 1186).
- **UNKNOWN:** `SELECT * FROM` pakai `componentsJoinedByPeriodAndBacktickQuoted` (1123–1126) sedangkan teks drag pakai `…IgnoreFirst` (1134).

**Lifecycle struktur**
- `updateEntriesForConnection:` (406–478) guard `ignoreUpdate` early-return reset (409–412), prune db terhapus (441–445), tulis placeholder `&DEL&no data loaded yet` bila struktur absen (455).
- `removeConnection:` (289–333) skip penghapusan bila >1 dokumen berbagi connection ID (301–309).
- `updateNavigator:` (399–404) hop ke main thread. **UNKNOWN:** `connectionName` di-assign dari `connectionID` (425–426) sehingga cek mismatch nama/ID di 428 efektif membandingkan nilai dengan dirinya sendiri.

**Dead ivar**
- `navigatorWindow` (`.h:36`) tak direferensikan; `prefs` (`.h:44`) di-assign (134) tak dibaca. **UNKNOWN** apakah dipakai via KVC/binding di `Navigator.xib`.
