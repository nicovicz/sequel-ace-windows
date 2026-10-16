# Audit: `SPIndexesController.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/SubviewControllers/SPIndexesController.m`. Baseline commit
`02c910932`.

- File: 1027 baris; header 112 baris.
- Peran: `NSWindowController` sheet "Add Index" + daftar index pada view Table
  Structure; membangun & menjalankan SQL `ALTER TABLE ... ADD/DROP INDEX`.
- Total: 25 definisi method.

## 1. Inventaris ivar/property + konstanta

### Ivar `.h:39–88`
| Baris | Grup | Ivar |
| --- | --- | --- |
| 41–44 | Controller | `dbDocument` (IBOutlet weak), `tableStructure`, `tablesList`, `tableData` |
| 47–49 | tabel index | `indexesTableView`, `addIndexButton`, `removeIndexButton` |
| 52–62 | sheet index baru | `indexTypePopUpButton`, `indexNameTextField`, `indexedColumnsTableView`, `indexedColumnsScrollView`, label, `indexSizeTableColumn`, tombol add/remove kolom, `confirmAddIndexButton`, `anchoredButtonBar` |
| 65–69 | advanced options | `indexAdvancedOptionsViewButton`, `indexAdvancedOptionsView`, `indexAdvancedOptionsViewLabelButton`, `indexStorageTypePopUpButton`, `indexKeyBlockSizeTextField` |
| 71–74 | state | `mainNibLoaded`, `isMyISAMTable`, `isInnoDBTable`, `table` |
| 76–77 | data | `fields`, `indexes`, `indexedFields`; `supportsLength`, `requiresLength` |
| 79–87 | lain | `prefs`, `connection`, `showAdvancedView`, `heightOffset`, `windowMinWidth`, `windowMinHeigth` (**salah eja**) |

Property: `table` (readwrite copy, `.h:93`), `connection` (readwrite strong, `.h:98`); synthesize `.m:72–73`.

### Konstanta lokal
`SPNewIndexIndexName=@"IndexName"` (46), `SPNewIndexIndexType=@"IndexType"` (47),
`SPNewIndexIndexedColumns=@"IndexedColumns"` (48), `SPNewIndexStorageType=@"IndexStorageType"` (49),
`SPNewIndexKeyBlockSize=@"IndexKeyBlockSize"` (50), `IndexesControllerKVOContext` (56).
`supportsLength` runtime (96–98): CHAR, VARCHAR, TINYTEXT, TEXT, MEDIUMTEXT, LONGTEXT, BINARY, VARBINARY, TINYBLOB, BLOB, MEDIUMBLOB, LONGBLOB.
`requiresLength` (101–103): TINYTEXT, TEXT, MEDIUMTEXT, LONGTEXT, TINYBLOB, BLOB, MEDIUMBLOB, LONGBLOB.
Menu tag `SPPrimaryKeyMenuTag`…`SPSpatialMenuTag` dari `SPConstants.h:204-208`.

## 2. Semua `#pragma mark`

`75, 141/142 (IB action methods), 449/450 (TableView datasource methods), 519/520 (ComboBoxCell datasource methods), 544/545 (TableView delegate methods), 562/563 (Accessor methods), 595/596 (Other methods), 651/652 (Private API methods), 1018`.

## 3. Tabel method (25 definisi, urut baris)

| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 77 | 107 | `init` | Set nama nib `IndexesView`, init array + list dukungan panjang |
| 109 | 139 | `awakeFromNib` | Setup nib sekali: font, gridline, empty double-click, observer KVO |
| 147 | 299 | `addIndex:` | Buka sheet Add Index, default tipe/kolom, eksekusi add saat OK |
| 304 | 330 | `removeIndex:` | Konfirmasi + dispatch penghapusan index terpilih di thread background |
| 335 | 358 | `chooseIndexType:` | Update field nama, storage-type, kolom size untuk tipe terpilih |
| 363 | 380 | `closeSheet:` | Sembunyikan view advanced/kolom size, akhiri sheet, bersihkan field block-size |
| 385 | 409 | `addIndexedField:` | Append field belum terindeks; tampilkan kolom size bila butuh panjang |
| 414 | 421 | `removeIndexedField:` | Hapus field terindeks terpilih + reload tabel |
| 426 | 447 | `toggleAdvancedIndexOptionsView:` | Show/hide view advanced + resize window |
| 452 | 455 | `numberOfRowsInTableView:` | Jumlah baris tabel index atau kolom terindeks |
| 457 | 472 | `tableView:objectValueForTableColumn:row:` | Nilai sel; koersi kolom Size ke number |
| 474 | 505 | `tableView:setObjectValue:forTableColumn:row:` | Tulis edit sel; `name` menukar dict field penuh, else set key |
| 507 | 517 | `tableView:willDisplayCell:forTableColumn:row:` | Placeholder sel Size "required"/"optional" |
| 525 | 528 | `numberOfItemsInComboBoxCell:` | Jumlah field belum terindeks |
| 533 | 542 | `comboBoxCell:objectValueForItemAtIndex:` | Nama field tersedia (belum terindeks) |
| 550 | 560 | `tableViewSelectionDidChange:` | Enable tombol remove berdasar seleksi |
| 570 | 581 | `setFields:` | Ganti array fields, reset indexed fields ke kolom pertama |
| 588 | 593 | `setIndexes:` | Ganti array indexes via `safeSetArray:` |
| 601 | 624 | `observeValueForKeyPath:ofObject:change:context:` | Reaksi pref gridline/font |
| 629 | 649 | `validateMenuItem:` | Enable Remove Index / Reset AUTO_INCREMENT / Remove Field |
| 657 | 660 | `_isFullTextIndexSelected` | True bila tag popup FULLTEXT |
| 665 | 694 | `_addAdditionalIndexTypes` | Tambah/hapus item SPATIAL & FULLTEXT per engine |
| 699 | 725 | `_reloadIndexedColumnsTableData` | Reload tabel kolom, toggle kolom size + tombol confirm |
| 732 | 831 | `_addIndexUsingDetails:` | Bangun + jalankan SQL `ALTER TABLE ... ADD` index |
| 838 | 896 | `_removeIndexUsingDetails:` | Drop FK bila diberi, lalu `DROP INDEX`/`DROP PRIMARY KEY` |
| 904 | 954 | `_removingIndexFailedWithForeignKeyError:` | Saat error 1553, cari FK dependen, tawarkan hapus keduanya |
| 962 | 1016 | `_resizeWindowForAdvancedOptionsViewByHeightDelta:` | Resize window sheet sambil pin kontrol/min size |
| 1020 | 1025 | `dealloc` | Hapus dua observer KVO `prefs` |

## 4. Temuan menonjol / UNKNOWN

**SQL add index (`_addIndexUsingDetails:`, 732–831)**
- Berjalan dalam `@autoreleasepool`; re-check `saveRowOnDeselect` di main thread (736).
- `PRIMARY KEY` paksa nama kosong (750–752); tipe lain backtick-quote nama (754).
- Kerangka query di 784: `ALTER TABLE <table> ADD <indexType>`. Kecuali tipe `INDEX`/`PRIMARY KEY`, `" INDEX"` ditambahkan (781) → `UNIQUE INDEX`, `FULLTEXT INDEX`, `SPATIAL INDEX`.
- Klausa opsional: nama (787–790), `USING <storageType>` (793–796), kolom join `componentsJoinedByCommas` (799), `KEY_BLOCK_SIZE = <n>` (802–804).
- Prefiks panjang per-kolom hanya bila `Size` ada, tipe di `supportsLength`, dan bukan FULLTEXT → `` `col` (size) `` (771–772); else nama backtick (775). Field di `requiresLength` tanpa size **di-skip** (769).
- Eksekusi 807; alert error hanya bila tak dicancel (810–812); sukses reset table data + reload (813–818). Setelah blok, main-queue reset `indexedFields` ke field pertama + reload (822–827), lalu `endTask` (829).

**SQL remove index (`_removeIndexUsingDetails:`, 838–896)**
- Prasyarat FK opsional: bila `ForeignKey` ada, `ALTER TABLE <t> DROP FOREIGN KEY <fk>` (847); error → error sheet (850–857) tapi **tidak return** — eksekusi jatuh ke DROP index. **UNKNOWN** apakah disengaja.
- `PRIMARY` → `ALTER TABLE <t> DROP PRIMARY KEY` (861); else `ALTER TABLE <t> DROP INDEX <name>` (864–865).
- Error 869–886; sukses reset data/tabel 887–892; `endTask` 894.

**Gating SPATIAL/FULLTEXT**
- Flag engine di-set saat add dari `tableData statusValueForKey:@"Engine"` (155–156).
- `_addAdditionalIndexTypes` (665–694) **menghapus** item SPATIAL/FULLTEXT lama (667–673), lalu re-add SPATIAL **hanya MyISAM** (676–683) dan FULLTEXT bila **MyISAM atau (InnoDB && `supportsFulltextOnInnoDB`)** (686–693). `supportsFulltextOnInnoDB` dari `SPServerSupport.h:99`.
- Popup storage-type disabled untuk MyISAM/InnoDB (223, 352) dan SPATIAL (352); storage type tak ditulis ke query untuk SPATIAL (282). Kolom size disembunyikan saat FULLTEXT (355), placeholder "optional" untuk FULLTEXT (513–515), penegakan size dilewati untuk FULLTEXT (704–708, 765/769/771).

**Jalur error FK 1553**
- Pada error `DROP INDEX`, bila `[connection lastErrorID] == 1553` (`ER_DROP_INDEX_FK`) **dan** belum ada FK yang di-drop, worker hop ke main thread via `performSelectorOnMainThread:@selector(_removingIndexFailedWithForeignKeyError:)` (871–876). `lastErrorID` di `Querying & Preparation.h:70`.
- `_removingIndexFailedWithForeignKeyError:` (904–954, wajib UI thread per doc 902) kumpulkan kolom berbagi `Key_name` index (913–917), warn bila tak ada (920–923), sortir (925), scan `[tableData getConstraints]` (928) untuk FK yang kolomnya cocok (929–930). Match ambigu → alert + return (931–934); tak ada match → alert + return (938–941).
- Match unik → confirm alert "Delete Both" (943); konfirmasi panggil ulang `_removeIndexUsingDetails:` dengan `Key_name` + `ForeignKey` (947, 951) yang drop FK dulu (845–847).

**Lain**
- `setIndexes:` pakai `safeSetArray:` (592); `setFields:` tak guard `[fields objectAtIndex:0]` melebihi cek `count` (578).
- `addIndex:` pilih field awal yang belum terindeks, else fallback `fields[0]` (198–207) — **UNKNOWN** bila `fields` kosong (akan raise di `objectAtIndex:0`).
- Kolom terindeks duplikat dipangkas di handler completion sheet (236–253).
- KVO: didaftarkan di `awakeFromNib` (137–138), dihapus di `dealloc` (1022–1023; 1022 ada TODO varian `context:`). Observer font pakai `context:nil` (138) dicocokkan by keyPath (611).
- `_resizeWindowForAdvancedOptionsViewByHeightDelta:` (962–1016) pin autoresizing mask sementara; maksud pasangan `heightOffset` add/subtract di 997–1005 **UNKNOWN/non-obvious**.
- `dbDocument` `__weak` (`.h:41`) di-capture `self->dbDocument` dalam blok (231 dst).
