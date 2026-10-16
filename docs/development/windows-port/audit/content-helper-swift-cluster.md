# Audit: Content-helper Swift cluster

Audit method-by-method klaster helper konten. `Start`/`End` = nomor baris di file
masing-masing. Baseline commit `02c910932`.

File: `TableContent/SACellFilterAction.swift` (201), `TableContent/SACellFilterMenuBuilder.swift` (333),
`TableContent/SACellFilterOperator.swift` (206), `TableContent/SATableReloadRetryPolicy.swift` (247),
`TableViews/SAFieldEditorCommitPolicy.swift` (239), `Controls/SPRuleFilterDropBox.swift` (398),
`DataImport/SACSVImportStreamDecoder.swift` (211).

---

## 1. `SACellFilterAction.swift` (201)

Menyimpan payload satu menu "Filter by Selected Value" + menambah aksi copy nilai sel ke tabel hasil.

### Tipe & konstanta
`SACellFilterAction` (18–62); `SACellValueCopyCoordinator` (65–201).
`rawMenuTag = 1_945_002` (66), `sqlMenuTag = 1_945_003` (67).

### `// MARK:`
**Tidak ada.**

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 19–36 | | `tableContent` (weak) / `columnName` / `operatorName` / `values` / `isNull` | Field |
| 46–61 | | `init(tableContent:columnName:operatorName:values:isNull:)` / `apply(_:)` | Bangun 1 wrapper per menu item + terapkan filter |
| 70 | 170 | `appendItems(to:event:table:tableStorage:columnDefinitions:connection:)` | Bangun + sisip item menu copy-as-value/SQL |
| 172 | 191 | `sqlLiteral(value:columnDefinition:connection:)` | Normalisasi geometry → data + quote satu literal |
| 193 | 200 | `removeItems(from:)` | Hapus item bertag sebelumnya |

### Temuan
- `appendItems` tekan seluruh menu saat content/custom-query view sibuk (79–86), pilih baris terklik bila belum (93–100), resolve kolom tampak → index storage via `SACellFilterColumnIdentifier.storageIndex` (102–106).
- Nilai SQL dikumpulkan hanya bila ada definisi kolom **dan** koneksi (114); satu nilai tak terepresentasi → abort mode SQL dengan `sqlValues = nil` (122–132) → item "as SQL" **disabled** (bukan parsial). Teks SQL di-defer: closure `SACellValueCopyAction` dipanggil hanya saat seleksi, return nil kecuali semua nilai sukses di-quote (153–159).
- Geometry di-unwrap ke `SPMySQLGeometryData.data()` sebelum quote (178–182); **UNKNOWN** semantik byte-nya.
- Sisip item beranker setelah tag `SPEditMenuCopyAsSQLNoAutoInc`, else sebelum item terakhir (144–146). Nilainya **UNKNOWN**.

---

## 2. `SACellFilterMenuBuilder.swift` (333)

Descriptor model + builder yang menghasilkan submenu cell-filter dan aksi copy nilai sel.

### Tipe
`SACellFilterMenuItemDescriptor` (16–50); `SACellValueCopyMenuItemDescriptor` (53–66); `SACellValueCopyMenuBuilder` (69–182); `SACellValueCopyAction` (185–225, `NSMenuItemValidation`); `SACellFilterMenuBuilder` (231–333).

### `// MARK:`
**Tidak ada.**

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 19–49 | | descriptor fields + `init(title:columnName:operatorName:values:isNull:)` | Filter descriptor |
| 54–65 | | copy descriptor fields + `init(title:text:isSQL:isEnabled:)` | Copy descriptor |
| 71–133 | | `menuItemDescriptors(columnName:rawValues:sqlLiterals:)` / `deferredMenuItemDescriptors(...)` / `shouldSuppressMenu(...)` / `makeMenuItemDescriptors(...)` / `selectedRows(current:clickedRow:)` | Builder |
| 144 | 166 | `sqlLiteral(value:typeGrouping:fieldType:quoteString:quoteData:)` | Render satu nilai → literal SQL |
| 169 | 181 | `canPrepareSQLLiteral(value:typeGrouping:fieldType:)` | Pre-cek literal tanpa koneksi |
| 186–224 | | `descriptor` / `textProvider` / init ×2 / `copy(_:)` / `validateMenuItem(_:)` | Copy action |
| 244–332 | | `filterMenu(column:value:isNull:)` / `menuItemDescriptors(column:value:isNull:)` / `menuItemDescriptors(columnName:typeGrouping:value:isNull:)` | Filter menu |

### Temuan
- **Aturan merge (`sqlLiteral`)**: `NSNull → "NULL"` (151); klasifikasi unquoted via `SPFieldTypeClassifier.shouldBeUnquoted` → `unquotedSQLLiteral` (152–154); `Data` diperlakukan teks bila field type `UUID` atau grouping `textdata`/`string`, else binary-quoted (156–163); fallback quote `String(describing:)` (165).
- **Routing NULL (307–316)**: string kosong ATAU nilai nil dipaksa ke operator NULL karena `filterValues: [""]` kolaps ke placeholder di `SPRuleFilterController.m:1814-1829` (komentar 301–302). Sel `BIT`: bit-string valid → desimal (`00000101` → `5`); nilai bit bukan bit-string → degradasi NULL-only (310–316).
- Item SQL disabled kecuali `sqlLiteralCount == rawValues.count` (78, 97).
- `selectedRows` pertahankan seleksi bila sudah memuat baris terklik, else pilih hanya baris terklik (131–133).
- `SACellValueCopyAction.copy` deklarasikan `.string` (dan `.tabularText` untuk non-SQL), tulis via keduanya (212–219).
- `canPrepareSQLLiteral` probe dengan quoter no-op → `""`, jadi hanya deteksi kegagalan struktural, bukan kegagalan quoting koneksi (174–180). **UNKNOWN**: semantik `SPFieldTypeClassifier.*`.
- `filterMenu` item display-only dengan `action: nil`; menu konteks nyata pakai `menuItemDescriptors` (236, 252).

---

## 3. `SACellFilterOperator.swift` (206)

Katalog/matriks statis operator cell-filter yang valid per type grouping kolom.

### Tipe & konstanta
`SACellFilterOperator` (15–197); `SACellFilterOperatorPair` (200–206, test-only).
`filterDefinitionGroups` (91); `numberTypeGroupings: Set = ["bit","integer","float"]` (154); `stringTypeGroupings: Set = ["string","textdata","enum"]` (155); `supportedTypeGroupings` (157–168); `nullOperators` `IS NULL`/`IS NOT NULL` (170–173); `catalog` (175–196).

### `// MARK:`
**Tidak ada.**

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 17–43 | | `menuTitle` / `serializedName` / `valueCount` / `typeGroupings` / init | Entry katalog |
| 53 | 59 | `operators(for:)` | Semua operator untuk satu grouping |
| 71 | 88 | `operators(for:cellIsNull:)` | Operator ter-filter state nilai |
| 91 | 91 | `filterDefinitionGroups` | Nama group plist bawaan |
| 105–152 | | `ruleFilterGroup(forTypeGrouping:)` / `filterDefinitionGroup(forTypeGrouping:)` / `allAdvertisedPairs()` | Pemetaan grouping |
| 154–196 | | `numberTypeGroupings` / `stringTypeGroupings` / `supportedTypeGroupings` / `nullOperators` / `catalog` | Data statis |
| 202–205 | | `typeGrouping` / `op` (Pair) | Pairing test |

### Temuan
- Grouping unknown/nil/kosong → `[]` (54–56). Sel NULL hanya operator 0-arg (73–76); sel non-NULL simpan operator nilai **dan** NULL agar grouping binary/blob/geometry tak kehilangan `IS NOT NULL` (78–87).
- `ruleFilterGroup` — date→date; string/binary/textdata/blobdata/enum→string; bit/integer/float→number; geometry→spatial; else `""` (105–117). `filterDefinitionGroup` — hanya `bit` berbeda (`bit`→`bit`), lain ikut `ruleFilterGroup` (133–134).
- **Rasional bit** (komentar 120–128): `bit` pakai `CAST('<value>' AS DECIMAL(65,30))` untuk komparasi eksak hingga 2^64 + hindari miss BIT ter-index.
- **UNKNOWN**: apakah entry katalog persis match `ContentFilters.plist` saat runtime — file hanya menyatakan niatnya (13–14, 137–142); tanpa parsing plist di sini.

---

## 4. `SATableReloadRetryPolicy.swift` (247)

Membatasi full reload tabel self-triggered yang lepas kendali + koordinasi kapan task reload boleh mulai.

### Tipe & konstanta
`SATableReloadRetryPolicy` (41–110); `SATableReloadCoordinator` (124–247). `defaultLimit = 3` (44).

### `// MARK:`
**Tidak ada.**

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 44–62 | | `defaultLimit` / `limit` / `lock` / `attemptsByTable` / init ×2 | Kebijakan budget |
| 70–109 | | `shouldReload(forTable:)` / `attemptCount(forTable:)` / `reset(forTable:)` / `resetAll()` | Counter |
| 126–143 | | `policy` / `lock` / `loadDepth` / `reloadTaskIsRunning` / `reloadIsNoted` / init ×2 | State koordinator |
| 146–176 | | `loadDidBegin()` / `loadDidEnd()` / `reloadTaskDidBegin()` / `reloadTaskDidEnd()` | Siklus load/task |
| 188–246 | | `noteFullReload(forTable:)` / `takeNoteForReloadTask()` / `takeNoteWhenIdle()` / `reset(forTable:)` / `resetAll()` / `attemptCount(forTable:)` | Catatan reload |

### Temuan
- Tabel boleh reload sendiri maks `limit` (default 3); limit negatif clamp 0 (54–56). `shouldReload` naik hanya saat return `true` (75–79).
- Tabel tanpa nama berbagi key string kosong → load tanpa nama tak bisa reload tanpa henti (66–67, 71).
- **Coalescing**: dua load melaporkan mismatch sama → berbagi satu catatan pending; `noteFullReload` kedua return `true` tanpa membelanjakan budget (192–196).
- `reloadTaskDidBegin` reset `reloadIsNoted = false` (163), membuang catatan sisa dari load sibling sebelum task mulai.
- **Gate**: `takeNoteWhenIdle` syarat `reloadIsNoted && !reloadTaskIsRunning && loadDepth == 0` (219); `takeNoteForReloadTask` hanya butuh catatan pending (207).
- Semua state dijaga `NSLock`; `attemptCount`/`reset` pada koordinator delegasi ke policy **tanpa** lock koordinator sendiri (227–229, 244–246).
- **UNKNOWN**: call site di `SPTableContent` yang menggerakkan `loadDidBegin/End`, `noteFullReload`, `takeNote*` tak ada di file ini.

---

## 5. `SAFieldEditorCommitPolicy.swift` (239)

Autentikasi callback edit combo-box terhadap snapshot tabel + putuskan apakah inline commit diabaikan.

### Tipe & konstanta
`enum SAComboBoxSelectionState` (10–15: notTracked/current/deferred/invalidated); `SADeferredComboBoxEdit` (20–31); `SAComboBoxSelectionTracker` (35–206); `SAFieldEditorCommitPolicy` (208–239).

### `// MARK:`
**Tidak ada.**

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 22–30 | | field deferred edit + init | Record edit tertunda |
| 37–68 | | `lock` / `dataGeneration` / `reloadsInProgress` / `popupGeneration` / `openingValue` / `pendingSelection` / `deferredEdit` / `tableDataWillChange()` / `tableColumnModelWillChange()` / `advanceTableGeneration()` / `tableDataReloadWillBegin()` | State + generasi |
| 73–128 | | `tableDataReloadDidFinish()` / `comboBoxWillOpen(with:)` / `comboBoxSelectionDidChange(_:)` / `comboBoxDidClose(with:)` | Siklus popup |
| 131 | 165 | `consumeSelection(matching:tableColumn:row:)` | Klasifikasi seleksi → state, defer bila perlu |
| 169–205 | | `takeDeferredEditIfReady()` / `discardPendingSelection()` / `clearPendingSelection()` / `valuesMatch(_:_:)` | Konsumsi |
| 213 | 238 | `shouldIgnoreInlineCommit(fieldEditorRequired:cell:proposedValue:storedValue:displayValue:popupSelectionState:)` | Putuskan commit diabaikan |

### Temuan
- **Commit policy**: state `.deferred`/`.invalidated` **selalu** abaikan commit (221–223). Bila field editor tak perlu → commit dihormati (224–226). Untuk field editor perlu, hanya `NSComboBoxCell` dengan seleksi `.current` yang boleh lanjut; else diabaikan (227–229). Bahkan lalu commit diabaikan bila nilai usul == `storedValue`/`displayValue` (231–237).
- **Aturan generasi**: data/kolom berubah → bump `dataGeneration` (47–60); reload mulai **tanpa** bump karena query bisa gagal tanpa mengubah snapshot (62–68).
- **Deferral**: seleksi nilainya match tapi bertumpang reload in-flight → simpan `SADeferredComboBoxEdit` + return `.deferred` (146–161). `popupGeneration != dataGeneration` → `.invalidated` (142–145); nilai mismatch → `.notTracked` saat idle / `.invalidated` saat reload aktif (146–149).
- `takeDeferredEditIfReady` butuh `reloadsInProgress == 0` + generasi match; bersihkan pending bila generasi bergerak (173–179).
- `comboBoxDidClose` sengaja pertahankan provenance hanya saat pending == nilai final **dan** berbeda dari nilai pembuka (kasus Escape) (120–127).
- **UNKNOWN**: callback AppKit mana di `SPTableContent`/`SPCopyTable` yang memanggil selector `@objc` ini.

---

## 6. `SPRuleFilterDropBox.swift` (398)

Drop zone rule-filter permanen + preview WHERE live, dengan helper murni untuk teks, kebijakan klik, dan layout.

### Tipe & konstanta
`SARuleFilterPreviewFormatter` (15–25); `enum SARuleFilterDropBoxClickAction` (28–35); `enum SARuleFilterDropBoxClickPolicy` (38–55); `enum SARuleFilterDropBoxLayout` (59–95) + `struct Frames` (61–66); `SPRuleFilterDropBox: NSView` (113–398).
`rowDropType` (114); `promptText` (120); `promptTooltip` (185); `previewTooltip` (187).

### `// MARK:`
`356 (NSDraggingDestination)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 19–54 | | `previewText(clause:)` / `action(showingPreview:optionPressed:)` | Teks + kebijakan klik |
| 63–94 | | `Frames.label` / `.indicator` / `frames(bounds:...)` | Layout |
| 114–223 | | `rowDropType` / `dropHandler` / `promptText` / `label` / `setPreviewClause(_:)` / `isShowingPreview` / `previewClause` / `copyPreviewClause(_:)` / tooltip ×2 / `menuIndicator` / `isDragHovering` / init ×2 / `commonInit()` | View |
| 228–354 | | `menu(for:)` / `filterMenu()` / `resizeSubviews` / `placeSubviews` / `acceptsFirstResponder` / `hitTest` / `resetCursorRects` / `draw(_:)` / `mouseDown(with:)` | Interaksi |
| 358–397 | | `draggingEntered` / `draggingUpdated` / `draggingExited` / `prepareForDragOperation` / `performDragOperation` / `concludeDragOperation` / `acceptsDrop(_:)` | Drag |

### Temuan
- **Drop box**: baris grid yang di-drop membawa property-list dengan key `SPCellValuePasteboard.rowColumnNameKey`/`rowValueKey`/`rowValueKindKey`; nilai null bila kind == `rowValueKindNull` (379–387). Drop dengan nama kolom kosong/handler hilang → `false` (378–385).
- **Kebijakan klik**: preview tampil → `showFilterMenu`; else ⌥ → `addFilterGroup`, plain → `addFilterRow` (49–54). Diterapkan pada mouse-up dalam bounds, ⌥ di-sample saat press (330, 337–347).
- **Layout**: mode preview taruh label inset 10 + chevron kanan + gap 4 (80–85); mode prompt center + clamp lebar (tanpa ellipsis, `.byClipping`) (88–93).
- **Sengaja tak pakai `layout()`**: komentar 253–265 — penempatan frame di `resizeSubviews` untuk hindari feedback loop Auto Layout.
- `hitTest` return `self` untuk titik dalam bounds agar chevron `NSImageView` tak menelan klik (285–293).
- **Hover drawing**: drag-hover pakai fill `selectedContentBackgroundColor` + stroke `controlAccentColor` 1.5pt; else dashed `separatorColor` 1.0pt `[4,3]` (303–316).
- **UNKNOWN**: isi `SARuleFilterContextMenu.menu(for:)` + signature `SPFilterRuleEditorDropHandler.appendFilter/addEmptyFilterRow/addEmptyFilterGroup`.

---

## 7. `SACSVImportStreamDecoder.swift` (211)

Decode stream byte CSV inkremental ke teks, split hanya pada batas terminator + konsumsi BOM.

### Tipe & konstanta
`SACSVImportStreamDecoder` (20–211). `errorDomain` (22), `undecodableErrorCode = 1` (23); BOM byte di `byteOrder` (169–173: utf8 EF BB BF, utf16LE FF FE, utf16BE FE FF, utf32LE FF FE 00 00, utf32BE 00 00 FE FF).

### `// MARK:`
`95 (Scanning), 131 (Byte order and BOM handling)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 22–38 | | `errorDomain` / `undecodableErrorCode` / `requestedEncoding` / `lineTerminator` / `resolvedEncoding` / `codeUnitWidth` / `terminatorBytes` / `buffer` / `scanPosition` / `hasResolvedPreamble` | State |
| 44 | 51 | `init(encoding:lineTerminator:)` | Set encoding + code-unit width |
| 60 | 93 | `text(byAppending:endOfInput:)` | Append data, return baris terdekode lengkap |
| 101 | 129 | `scanForTerminators()` | Cari offset setelah terminator lengkap terakhir |
| 136–162 | | `preambleLength` / `resolvePreamble()` | Panjang + resolve preamble |
| 168–209 | | `byteOrder(for:leading:)` / `codeUnitWidth(for:)` | Byte order + stride |

### Temuan
- **Algoritma**: append ke `buffer` (61); tahan sampai `preambleLength` byte (atau `endOfInput`) sebelum resolve preamble (63–66). `segmentEnd` = seluruh buffer di EOF, else offset setelah terminator lengkap terakhir, else return `""` (68–75).
- Decode `buffer.prefix(segmentEnd)`; gagal → throw NSError `undecodableErrorCode` (77–88); lalu simpan tail + rebase `scanPosition` (90–91).
- **Scan terminator**: return nil bila tak ada byte terminator (103); scan hanya selama `position + codeUnitWidth <= count` (batas code-unit) (113); prefix terminator terpotong tepi buffer → break loop agar diselesaikan chunk berikutnya (115–117); match naik `position += terminatorLength`, mismatch naik `codeUnitWidth` (116–122); simpan `scanPosition = position` (127).
- **BOM/byte order**: `preambleLength` = 3 (utf8), 2 (utf16), 4 (utf32), else 0 (136–147). `resolvePreamble` set encoding + strip BOM cocok, lalu encode `lineTerminator` (155–161). UTF-8 BOM di-strip eksplisit (176–177); bentuk utf16/utf32 byte-order-agnostic resolve LE/BE dari BOM, default big-endian bila BOM absen (178–189); varian fixed-endian hanya drop BOM cocok (182–193).
- **Stride scan**: 2 untuk utf16, 4 untuk utf32, else 1 (201–209).
- **UNKNOWN**: init `@objc` terima `encoding: UInt` (raw `NSStringEncoding`) (41–46); tanpa validasi panjang terminator terhadap code-unit width.

## Ringkasan lintas-file
- Stack cell-filter berlapis: `SACellFilterOperator` (matriks operator) → `SACellFilterMenuBuilder.menuItemDescriptors` (descriptor) → `SACellFilterAction`/`SACellValueCopyCoordinator` (apply/copy). `SACellFilterMenuBuilder.swift:318` panggil `SACellFilterOperator.operators(for:cellIsNull:)`.
- `SPRuleFilterDropBox` memakai key drop baris `SPCellValuePasteboard` yang sama dengan fitur copy/cell-filter (379–387).
- **UNKNOWN (eksternal, di luar scope)**: `SPTableContent`, `SPCustomQuery`, `SPCopyTable`, `SPFieldTypeClassifier`, `SPFilterRuleEditorDropHandler`, `SARuleFilterContextMenu`, `SPMySQLGeometryData`, `ContentFilters.plist`.
