# Audit: `SPFieldEditorController.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/SubviewControllers/SPFieldEditorController.m`. Baseline commit
`02c910932`.

- File: 1532 baris; header 263 baris.
- Peran: `NSWindowController` sheet field editor untuk mengedit satu nilai field —
  text, blob/image, hex, JSON, GEOMETRY, atau BIT — dengan preview QuickLook,
  open/save file, validasi panjang/format, lalu mengembalikan nilai ke caller.
- Total: 43 definisi method.

## 1. Inventaris ivar/property

### Ivar `.h` (57–191)
| Baris | Grup | Anggota |
| --- | --- | --- |
| 58–73 | IBOutlet editSheet | `editSheetProgressBar`, `editSheetSegmentControl`, `editSheetQuickLookButton`, `editImage`, `editTextView`, `hexTextView`, `jsonTextView`, scroll view, `editSheet` (SPWindow), tombol cancel/ok/open, `editSheetFieldName` |
| 75–82 | IBOutlet bitSheet | `bitSheet`, `bitSheetFieldName`, `bitSheetHexTextField`, `bitSheetIntegerTextField`, `bitSheetOctalTextField`, `bitSheetOkButton`, `bitSheetCloseButton`, `bitSheetNULLButton` |
| 83–146 | tombol bit | `bitSheetBitButton0` … `bitSheetBitButton63` (64 tombol) |
| 147–154 | label bit | `bitSheetBitLabel0/8/…/56` |
| 156–158 | konteks caller | `usedSheet`, `callerInstance`, `contextInfo` |
| 160–172 | state sheet/data | `sheetEditData`, `editSheetWillBeInitialized`, `_isBlob`, `_isEditable`, `_allowNULL`, `doGroupDueToChars`, `quickLookCloseMarker`, `encoding`, `fieldType`, `fieldEncoding`, `stringValue`, `tmpFileName`, `tmpDirPath` |
| 174–179 | undo/teks | `counter`, `maxTextLength`, `editTextViewWasChanged`, `allowUndo`, `wasCutPaste`, `selectionChanged` |
| 181–190 | prefs/QL/hasil | `prefs`, `qlTypes`, `editSheetReturnCode`, `_isGeometry`, `_isJSON`, `esUndoManager`, `editedFieldInfo` |

**Catatan:** `quickLookCloseMarker` (`.h:166`) **tak pernah direferensikan** di `.m` → vestigial.

### Property (`.h:193,200,207,212,217,223`) & synthesize (`.m:61-66`)
`editedFieldInfo`, `textMaxLength`→`maxTextLength`, `fieldType`, `fieldEncoding`, `displayFormatter` (`SABaseFormatter`), `allowNULL`→`_allowNULL`.

### Konstanta lokal
`.m:49-54` `typedef enum { TextSegment = 0, ImageSegment, HexSegment, JsonSegment } FieldEditorSegment`;
`.m:31-47` import; `.m:56-57` class-extension kosong. Tak ada `static`/`static const`.

## 2. Semua `#pragma mark` (urut baris)

`207, 794/795 (Drop methods), 812/813 (QuickLook), 904 (QLPreviewPanelController), 934 (QLPreviewPanelDataSource), 959 (QLPreviewPanelDelegate), 987, 1049/1050 (BIT Field Sheet), 1187/1188 (TextView delegate methods), 1426/1427 (UndoManager methods), 1464/1465 (UI Helper Methods)`.

## 3. Tabel method (43 definisi, urut baris)

### Sebelum 207
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 72 | 194 | `init` | Muat nib FieldEditorSheet; bangun menu QuickLook dari plist + pref |
| 199 | 205 | `dealloc` | Cancel perform tertunda; bersihkan editedFieldInfo |
| 209 | 211 | `maxLengthDateWithOverride` | Kembalikan override displayFormatter atau maxTextLength |
| 225 | 440 | `editWithObject:fieldName:usingEncoding:isObjectBlob:isEditable:withWindow:sender:contextInfo:` | Entry utama; validasi config, dispatch BIT vs editSheet, seed view |
| 442 | 468 | `buildLabelForField:` | Bangun label header dengan tipe/panjang/encoding/NOT NULL |
| 470 | 481 | `selectFont` | Pilih font editor teks dari pref |
| 486 | 573 | `segmentControllerChanged:` | Pindah segmen text/image/hex/JSON; isi view hex/JSON lazy |
| 578 | 586 | `openEditSheet:` | Tampilkan open panel untuk load file ke sheet |
| 591 | 610 | `saveEditSheet:` | Save panel; batasi ke PDF untuk image GEOMETRY |
| 617 | 702 | `closeEditSheet:` | Validasi panjang/format, hapus file temp QL, kembalikan hasil ke caller |
| 707 | 746 | `openPanelDidEnd:returnCode:contextInfo:` | Load file terpilih ke view text/image/hex |
| 751 | 792 | `savePanelDidEnd:returnCode:contextInfo:` | Tulis data (data/geometry/text) atomik ke URL |

### Drop methods (795)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 801 | 810 | `dropImage:` | Reset semua view saat image dihapus |

### QuickLook (813)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 818 | 824 | `quickLookFormatButton:` | Panggil QuickLook untuk item menu tipe terpilih |
| 833 | 872 | `createTemporaryQuickLookFileOfType:treatAsText:` | Tulis data ke file temp untuk generator QuickLook |
| 881 | 902 | `invokeQuickLookOfType:treatAsText:` | Buat file temp, front QLPreviewPanel |

### QLPreviewPanelController (904)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 909 | 914 | `beginPreviewPanelControl:` | Klaim QL panel sebagai delegate/dataSource |
| 919 | 924 | `endPreviewPanelControl:` | Lepas tanggung jawab QL panel (no-op) |
| 929 | 932 | `acceptsPreviewPanelControl:` | Selalu YES |

### QLPreviewPanelDataSource (934)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 941 | 944 | `numberOfPreviewItemsInPreviewPanel:` | Selalu 1 item preview |
| 951 | 957 | `previewPanel:previewItemAtIndex:` | Kembalikan URL file temp sebagai item preview |

### QLPreviewPanelDelegate (959)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 971 | 979 | `previewPanel:sourceFrameOnScreenForPreviewItem:` | Frame 5×5 di tengah main window untuk zoom |

### `#pragma mark -` (987)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 992 | 1013 | `processPasteImageData` | Tangani image di-paste: set image + data TIFF blob/hex |
| 1020 | 1047 | `processUpdatedImageData:` | Tangani image di-drop/berubah; nil reset view |

> Catatan: `processPasteImageData` ditulis `-(void)` tanpa spasi (992) sehingga tak match anchor `^[-+] (`; tetap definisi nyata (deklarasi `.h:244`).

### BIT Field Sheet (1050)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1055 | 1091 | `updateBitSheet` | Hitung ulang bit string/int/hex/oktal dari checkbox |
| 1096 | 1143 | `bitSheetOperatorButtonWasClicked:` | Operasi all-1/all-0/negate/shift/rotate by tag sender |
| 1148 | 1151 | `bitSheetSelectBit0:` | Fokus tombol bit 0 (komentar: tak bekerja) |
| 1157 | 1177 | `setToNull:` | Toggle NULL; enable/disable kontrol bit, update sheet |
| 1182 | 1185 | `bitSheetBitButtonWasClicked:` | Update bitSheet setelah bit di-toggle |

### TextView delegate methods (1188)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1193 | 1244 | `controlTextDidChange:` | Parse field int/hex → checkbox bit |
| 1250 | 1346 | `textView:shouldChangeTextInRange:replacementString:` | Paksa panjang/format max; potong insert berlebih |
| 1351 | 1373 | `textViewDidChangeSelection:` | Konversi edit teks ke string data; bersihkan image/hex/JSON |
| 1378 | 1390 | `textView:doCommandBySelector:` | Tangkap cmd/fn+return untuk tutup sheet, bukan newline |
| 1396 | 1424 | `textDidChange:` | Kelola grouping undo berdasar tipe/cadence cut-paste |

### UndoManager methods (1427)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1432 | 1438 | `undoManagerForTextView:` | Buat+return `esUndoManager` (lazy) |
| 1443 | 1446 | `setWasCutPaste` | Flag cut/paste untuk grouping undo |
| 1451 | 1454 | `setAllowedUndo` | Flag tertunda izinkan group undo saat jeda mengetik |
| 1459 | 1462 | `setDoGroupDueToChars` | Flag grouping undo berdasar jumlah karakter |

### UI Helper Methods (1465)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1467 | 1476 | `showHexText:` | Show/hide view hex + scrollview, sembunyikan lain |
| 1478 | 1487 | `showEditText:` | Show/hide view edit text |
| 1496 | 1509 | `selectJsonSegmentIfValueIsJSON:` | Auto-pilih segmen JSON bila nilai container JSON |
| 1511 | 1520 | `showJsonText:` | Show/hide view JSON |
| 1522 | 1530 | `showImage:` | Show/hide view image |

## 4. Temuan menonjol / UNKNOWN

**Dispatch mode sheet**
- BIT menang bila `fieldType` uppercase == `"BIT"` → `bitSheet` (243–280). Selain itu `editSheet` dengan `_isBlob = (!_isJSON && isFieldBlob)` — hex/image sengaja ditekan untuk JSON (286). `_isJSON` dari `SPMySQLJsonType` (240); `_isGeometry` dari `"GEOMETRY"` (239); `isBinary` dari `"BINARY"/"VARBINARY"` (287).
- Cabang input: `displayFormatter` → text (337–342); `NSData` → hex/binary `0x…` (343–357); `SPMySQLGeometryData` → text+thumbnail, hex disabled (358–366); else → JSON reformat/pretty opsional (367–388).

**Segmen hex**: view hex dibersihkan saat init (334); diisi lazy saat pertama pilih Hex via `dataToFormattedHexString` (499–507).

**Editor JSON**: validasi `NSJSONSerialization` (hasil tak dipakai, error menggerakkan cabang, 539), lalu pretty-print via `SPJSONFormatter` menghormati `SPCustomQuerySoftIndent`/`Width` (555–564); invalid → localized "Invalid JSON" (517,529,542,567). Saat tutup, format JSON dilepas via `stringByUnformattingString:` (688–691). `selectJsonSegmentIfValueIsJSON:` (1496–1509) auto-pilih segmen JSON bahkan untuk kolom non-JSON; pakai Swift `SAJSONValueDetector isJSONContainer:` (1497).

**Editor BIT**: sampai 64 bit: `maxBit = min(maxTextLength, 64)` (261,1058,1100). Tombol via KVC `valueForKeyPath:@"bitSheetBitButton%lu"` (265,271,1078). Operator tag 0-6 (all-1, all-0, negate, shift-L, shift-R, rotate-L, rotate-R) (1102–1141). Field int/hex diparse `strtoull` (1202) / `scanHexLongLong` (1226). NULL (`setToNull:` 1157–1177, 1060–1067). Seed checkbox dari kanan bit string tersimpan (263–268).

**QuickLook**: menu dibangun saat init dari `EditorQuickLookTypes.plist` bundle + pref `SPQuickLookTypes`; item PHP-serialized via Swift `setupPHPSerializedEditorMenuItemInMenu:` (128–187). File temp `SequelProQuickLook<counter%2>.<ext>` (838/837/890); saat tutup semua `SequelProQuickLook*` di `tmpDirPath` dihapus (670–679).

**Jalur commit (`closeEditSheet:`)**: `editSheetReturnCode = 1` hanya untuk `editSheetOkButton` (setelah validasi) atau `bitSheetOkButton` saat editable (624–667). Return data `(returnCode && _isEditable) ? (_isGeometry ? [editTextView string] : sheetEditData) : nil` (685). JSON di-unformat (688–691); displayFormatter di-reparse (692–696). Dikirim via delegate opsional `processFieldEditorResult:contextInfo:` (698–700; protokol `.h:258-263`). Validasi panjang menghitung code point, dengan toleransi titik desimal FLOAT + pengecualian placeholder NULL (626–649).

**Cross-file/Swift (tak didefinisikan di sini)**: `setupPHPSerializedEditorMenuItemInMenu:` (185), `resetPHPSerializedEditorState` (238), `refreshPHPSerializedEditorAvailability` (428, 1371), `openPHPSerializedEditorIfCurrentTextIsStructured` (429), `SAFieldEditorEditLimit` (1276, 1304), `SAJSONValueDetector isJSONContainer:` (1497), `SAArchiving fontFromData:` (476).

**Flag/pertanyaan**
- `quickLookCloseMarker` (`.h:166`) **tak dipakai** → dead.
- `bitSheetCloseButton` (`.h:81`) dideklarasikan tapi tak direferensikan di `.m`.
- `bitSheetSelectBit0:` didokumentasikan tak bekerja (1146).
- `previewPanel:…transitionImageForPreviewItem:` dan `handleEvent:` adalah stub QL yang dikomentari (962–964, 982–985).
- `openPanelDidEnd:`/`savePanelDidEnd:` pakai signature `contextInfo:` deprecated tapi dipanggil dari panel block-based (582–585, 606–609).
- `SPLog` dalam cabang `#ifdef DEBUG DumpObjCMethods` (530–533).
