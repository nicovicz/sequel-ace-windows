# Audit: `SPBundleEditorController.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/BundleSupport/SPBundleEditorController.m`. Baseline commit
`02c910932`.

- File: 2077 baris; header 146 baris.
- Peran: NSWindowController window Bundle Editor — memuat/menyimpan command
  bundle `.saBundle`/`.spBundle`, membangun + menggerakkan outline tree 4-level
  (scope → kategori → bundle), dan merutekan semua perubahan popup/text-field/
  shortcut ke dictionary bundle.
- Total: 66 definisi (1 di `SPBundleEditorShortcutRecorderCell`, 65 di controller).

Konstanta eksternal (`SPConstants.m:79,80,464,466`): `SPUserBundleFileExtension = @"spBundle"`,
`SPUserBundleFileExtensionV2 = @"saBundle"`, `SPBundleCurrentVersion = 2`, `SPBundleFileName = @"command.plist"`.

## 1. Struktur berkas

| Baris | Isi |
| --- | --- |
| 1–29 | header lisensi (Hans-Jörg Bibiko, 2010) |
| 31–41 | import |
| 43–55 | `#define`: `kBundleNameKey=@"bundleName"` (43), `kChildrenKey=@"_children_"` (44), index scope `kInputFieldScopeArrayIndex=0`/`kDataTableScopeArrayIndex=1`/`kGeneralScopeArrayIndex=2` (45–47), `kDisabledScopeTag=10` (48), string scope terlokalisasi (50–53), nama autosave splitview (55) |
| 57–74 | `SPBundleEditorShortcutRecorderCell : SRRecorderCell` — workaround issue #1755 (crash `removeTrackingRect:` macOS 13+) |
| 76–93 | class extension `SPBundleEditorController () <NSMenuItemValidation, NSControlTextEditingDelegate>` |
| 95 | `#pragma mark -` |
| 97–99 | `@implementation` + `@synthesize fileManager, prefs` |
| 2077 | `@end` |

## 2. Inventaris ivar/property

### `.h` (38–118)
| Baris | Grup | Ivar |
| --- | --- | --- |
| 40–69 | IBOutlet editor | `commandTextView`, `commandsOutlineView`, `authorTextField`, `contactTextField`, `descriptionTextView`, `nameTextField`, `tooltipTextField`, `categoryTextField`, `fallbackLabelField`, `withBlobLabelField`, `commandLabelField`, `inputPopupButton`, `inputFallbackPopupButton`, `outputPopupButton`, `scopePopupButton`, `triggerPopupButton`, `withBlobPopupButton`, tombol remove/add/save/cancel/help, `metaInfoSummary`, `displayMetaInfoButton`, `duplicateMenuItem`, `revealInFinderMenuItem`, `keyEquivalentField`, `disabledCheckbox`, `commandScrollView`, `splitView` |
| 71–73 | Sheet | `undeleteSheet`, `metaInfoSheet`, `undeleteTableView` |
| 75 | Outline | `commandBundleTreeController` |
| 77–81 | Model | `touchedBundleArray`, `deletedDefaultBundles`, `commandBundleTree`, `sortDescriptor`, `esUndoManager` |
| 83–85 | Path/nama | `bundlePath`, `draggedFilePath`, `oldBundleName` |
| 87–98 | 12 popup menu | `input*ScopePopUpMenu`, `output*ScopePopUpMenu`, `trigger*PopUpMenu`, `withBlobDataTablePopUpMenu`, … |
| 100–111 | 12 array nilai | `input*ScopeArray`, `output*ScopeArray`, `trigger*Array`, `withBlobDataTableArray`, `shellVariableSuggestions` |
| 113–117 | flag BOOL | `doGroupDueToChars`, `allowUndo`, `wasCutPaste`, `selectionChanged`, `isTableCellEditing` |

### `.m` property tambahan (90–91)
`NSFileManager *fileManager`, `NSUserDefaults *prefs` (synthesize 99, init 110–111).
**Catatan**: tak ada property untuk ivar `.h`; semua ivar polos. `selectionChanged` (`.h:116`) di-assign tapi **tak pernah dibaca** di file ini.

## 3. Semua `#pragma mark` (urut baris)

`95, 349, 1092/1093 (NSWindow delegate), 1119/1120 (SRRecorderControl delegate), 1165/1166 (TableView delegates), 1183/1184 (outline delegates), 1336/1337 (TableView (outline) delegate), 1449/1450 (Menu validation), 1480/1481 (OutlineView drag & drop delegate methods), 1539/1540 (NSTextView delegates), 1602/1603 (UndoManager methods)`.
**Tidak ada pragma** untuk blok privat 1640–2075 (9 helper `_` duduk di bawah heading `UndoManager methods`).

## 4. Tabel method (66 definisi, urut baris)

### `@implementation SPBundleEditorShortcutRecorderCell` (67–74)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 69 | 72 | `resetTrackingRects` | Override kosong sengaja; hindari crash tracking-rect #1755 |

### Init & setup (97–347)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 101 | 116 | `init` | Muat nib `BundleEditor`; init default/array/fileManager/prefs |
| 118 | 347 | `awakeFromNib` | Wire splitview, shortcut recorder, menu, array scope, list shell-var, tree |

### IBActions & bundle actions (355–991)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 355 | 377 | `inputPopupButtonChanged:` | Simpan key input source ke bundle; update view |
| 383 | 397 | `inputFallbackPopupButtonChanged:` | Simpan key input fallback ke bundle |
| 403 | 421 | `outputPopupButtonChanged:` | Simpan key output action ke bundle |
| 428 | 477 | `scopeButtonChanged:` | Simpan scope; pindah node ke scope/kategori baru |
| 483 | 503 | `triggerButtonChanged:` | Simpan key trigger; update view |
| 509 | 523 | `withBlobButtonChanged:` | Simpan key BLOB-handling untuk scope data-table |
| 528 | 534 | `duplicateCommandBundle:` | Beep kecuali satu baris terpilih; delegasi ke `addCommandBundle:` |
| 541 | 697 | `addCommandBundle:` | Duplikat bundle terpilih atau buat baru di scope/kategori |
| 702 | 759 | `removeCommandBundle:` | Konfirmasi, trashing folder bundle, catat default bundle terhapus |
| 764 | 785 | `revealCommandBundleInFinder:` | Pilih folder bundle di Finder (saBundle lalu spBundle) |
| 790 | 844 | `saveBundle:` | Save panel; copy/update bundle terpilih ke path dipilih |
| 849 | 852 | `showHelp:` | Buka URL help Bundle Editor terlokalisasi |
| 857 | 860 | `reloadBundles:` | Bangun ulang tree bundle dari disk |
| 865 | 868 | `showWindow:` | Front window via super saja |
| 870 | 874 | `performClose:` | Bangun ulang tree lalu tutup window |
| 876 | 902 | `undeleteDefaultBundles:` | Sheet restore default bundle terhapus; reload |
| 904 | 913 | `closeUndeleteDefaultBundlesSheet:` | Akhiri sheet undelete |
| 915 | 920 | `displayBundleMetaInfo:` | Tampilkan sheet meta-info; refresh summary saat tutup |
| 922 | 926 | `closeSheet:` | Akhiri sheet + order out |
| 931 | 991 | `saveAndCloseWindow:` | Commit edit, simpan bundle tersentuh, tutup, reload manager |

### Persistensi bundle (993–1090)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 997 | 1083 | `saveBundle:atPath:` | Tulis `command.plist` ke folder bundle; ROT13 `contact` |
| 1085 | 1090 | `cancelRowEditing` | Abort editing outline; bersihkan flag editing |

### NSWindow delegate (1093)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1095 | 1107 | `windowShouldClose:` | Blokir tutup saat editing cell tabel aktif |
| 1109 | 1117 | `windowWillClose:` | Hapus file drag temporer bila ada |

### SRRecorderControl delegate (1120)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1122 | 1125 | `shortcutValidator:isKeyCode:andFlagsTaken:reason:` | Selalu izinkan capture shortcut |
| 1127 | 1130 | `shortcutRecorderCell:isKeyCode:andFlagsTaken:reason:` | Selalu izinkan capture cell |
| 1132 | 1163 | `shortcutRecorder:keyComboDidChange:` | Konversi KeyCombo → string KeyBinding.dict, simpan |

### TableView delegates (1166)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1168 | 1171 | `numberOfRowsInTableView:` | Jumlah baris = count `deletedDefaultBundles` |
| 1173 | 1176 | `tableView:objectValueForTableColumn:row:` | Nama bundle default terhapus |
| 1178 | 1181 | `tableView:shouldEditTableColumn:row:` | Editing disabled di tabel undelete |

### Outline delegates (1184)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1186 | 1189 | `outlineView:isItemExpandable:` | Expandable iff dict mengandung `_children_` |
| 1191 | 1204 | `outlineView:numberOfChildrenOfItem:` | Jumlah child untuk node dict/array |
| 1206 | 1213 | `outlineView:objectValueForTableColumn:byItem:` | Tampilkan `bundleName` objek |
| 1215 | 1218 | `outlineView:isGroupItem:` | Dict dengan `_children_` = group item |
| 1220 | 1224 | `outlineView:shouldCollapseItem:` | Larang collapse root |
| 1226 | 1229 | `outlineView:shouldExpandItem:` | Izinkan expand non-leaf |
| 1231 | 1235 | `outlineView:shouldSelectItem:` | Root level 0 tak selectable |
| 1237 | 1245 | `outlineView:shouldEditTableColumn:item:` | Editing hanya bila depth seleksi > 2 |
| 1251 | 1274 | `outlineViewSelectionDidChange:` | Ingat nama lama, toggle state input, refresh view |
| 1276 | 1280 | `outlineView:shouldShowOutlineCellForItem:` | Sembunyikan disclosure cell di level root |
| 1282 | 1285 | `outlineView:shouldShowCellExpansionForTableColumn:item:` | Cell expansion disabled |
| 1287 | 1334 | `outlineView:toolTipForCell:rect:tableColumn:item:mouseLocation:` | Tooltip per level |

### TableView (outline) delegate (1337)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1342 | 1354 | `control:textView:doCommandBySelector:` | Tangkap Esc/complete; abort editing tanpa maju baris |
| 1356 | 1447 | `controlTextDidEndEditing:` | Tangani rename kategori/bundle dan pindah kategori di disk/tree |

### Menu validation (1450)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1455 | 1478 | `validateMenuItem:` | Enable item menu bundle hanya untuk seleksi leaf tunggal |

### OutlineView drag & drop (1481)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1486 | 1488 | `draggingSession:sourceOperationMaskForDraggingContext:` | Kembalikan `NSDragOperationMove` (komentar bilang copy) |
| 1493 | 1537 | `outlineView:writeItems:toPasteboard:` | Serialisasi leaf bundle ke file temp; mulai drag Finder |

### NSTextView delegates (1540)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1545 | 1548 | `textViewDidChangeSelection:` | Tandai command text view untuk redraw (highlight baris) |
| 1553 | 1586 | `textDidChange:` | Grouping undo per burst mengetik/cut-paste |
| 1591 | 1600 | `textView:completions:forPartialWordRange:indexOfSelectedItem:` | Completion shell-variable prefix-matched |

### UndoManager methods (1603)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1608 | 1614 | `undoManagerForTextView:` | Buat+return `esUndoManager` (lazy) |
| 1619 | 1622 | `setWasCutPaste` | Set flag group undo cut/paste |
| 1627 | 1630 | `setAllowedUndo` | Flag tertunda mengizinkan group undo berikut |
| 1635 | 1638 | `setDoGroupDueToChars` | Set flag grouping undo karena input karakter |

### Private helpers, tanpa pragma (1640–2075)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1640 | 1766 | `_initTree` | Bersihkan tree, baca direktori bundle, sisip ke scope/kategori |
| 1771 | 1920 | `_updateBundleDataView` | Sinkron semua popup/field + visibilitas dari bundle terpilih |
| 1925 | 1943 | `_updateBundleMetaSummary` | Bangun string ringkasan "(author) description" |
| 1948 | 1951 | `_currentSelectedObject` | representedObject baris outline terpilih |
| 1956 | 1959 | `_currentSelectedNode` | NSTreeNode terpilih |
| 1964 | 1980 | `_arrangedScopeIndexForScopeIndex:` | Map index scope model → index arranged (tersortir) |
| 1985 | 1995 | `_scopeIndexForArrangedScopeIndex:` | Map arranged → model |
| 2000 | 2023 | `_enableBundleDataInput:bundleEnabled:` | Enable/disable kontrol editor per seleksi+enabled |
| 2029 | 2075 | `_arrangedCategoryIndexForScopeIndex:andCategory:` | Cari/buat node kategori, kembalikan arranged index |

## 5. Temuan menonjol / UNKNOWN

**Struktur tree bundle (4 level, dict `_children_`)**
- Root `commandBundleTree` = `NSMutableDictionary` (awakeFromNib 138–146): **Level 0** root `{bundleName:"BUNDLES", _children:[3 scope dict]}` (141–142); **Level 1** scope urutan tetap **Input Field (0), Data Table (1), General (2)** — macro index 45–47 dipakai sebagai index *tak-tersortir* (1643–1645, 1681–1744); **Level 2** kategori `{bundleName, _children_}` dibuat lazy (1689–1695); **Level 3** bundle (leaf, tanpa `_children_`).
- **Urutan arranged (tersortir) berbeda dari urutan model** → dua helper konversi (1964, 1985) dan swap bolak-balik di `scopeButtonChanged:` (451–472) dan `controlTextDidEndEditing:` (1416–1446).

**`saveBundle:atPath:` (997–1083)**
- Nil `aPath` → path dari `bundleName` + ekstensi per versi: V2 `.saBundle` bila `bundleVersion.intValue > 1`, else legacy `.spBundle` (1026).
- File payload selalu `command.plist` (1045). **ROT13 saat tulis** (1050–1052) untuk `SPBundleFileContactKey`.
- Bandingkan `[saveDict description]` vs plist di disk dan **return YES dini saat sama** (1067–1068) — diff berbasis deskripsi string, bukan struktural.
- ⚠️ **Bug**: hasil `writeToURL:error:` hanya `SPLog`; method **return YES walau write gagal** (1077–1082). Pemanggil (610, 836, 970, 1522) memperlakukan write gagal sebagai sukses.
- `kBundleNameKey` dibuang dari dict tersimpan (1055).

**ROT13 `contact` round-trip**: tulis 1050–1052; baca `_initTree` 1674–1676 (ROT13 lagi) → load→save identitas. `rot13` di `SPStringAdditions.m:139`. **UNKNOWN** apakah maksudnya anti-spam (tak ada komentar menyatakan intent keamanan).

**Lain-lain**
- ⚠️ **Bug early-return `_arrangedCategoryIndexForScopeIndex:andCategory:` (2071):** `return 0;` tak-bersyarat di akhir iterasi pertama loop luar `for` → hanya child scope pertama yang pernah discan; juga bila nama bundle cocok tapi loop kategori dalam selesai tanpa hit, kontrol jatuh ke 2071 dan return 0 alih-alih `returnIndex`.
- **Scan scope `outlineView:toolTipForCell:…` (1291–1302):** menghitung `k` di seluruh scope child-node; **UNKNOWN** apakah label scope tooltip selalu benar.
- Asimetri toggle menu: `_enableBundleDataInput:…` (2000–2023) tidak enable/disable `withBlobPopupButton`/`duplicateMenuItem`/`revealInFinderMenuItem` (ditangani `validateMenuItem:` 1455).
- `closeUndeleteDefaultBundlesSheet:` pakai `[sender tag]` sebagai return code (907) sedangkan `undeleteDefaultBundles:` hanya reaksi bila `returnCode == 1` (881).
- `selectionChanged` (`.h:116`) di-set NO (1571) tapi tak pernah dibaca — dead state.
- `validateMenuItem:` (1466–1468) men-disable blok aksi saat shortcut recorder jadi first responder.
- **Temp-file drag**: ditulis ke temporary directory (1511), dihapus di `windowWillClose:` (1112–1114); file temp basi dihapus di awal drag berikutnya (1499–1502).
- **UNKNOWN**: `SPBundleEditorShortcutRecorderCell` (63–74) subclass `SRRecorderCell` tapi tak ada kode di sini yang membuktikan cell class-nya benar-benar dipasang ke `keyEquivalentField` (detail nib).
