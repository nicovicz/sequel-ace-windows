# Views AppKit, parser, formatter, utilitas, dan third-party

Kontrak source untuk komponen UI AppKit kustom, parser/formatter/utilitas, dan
kode third-party vendored. Sitasi `file:line` merujuk checkout `02c910932`.

## 1. Subclass NSView/NSControl kustom

### Window & root view
- **`SPWindow : NSWindow`** (`SPWindow.h:31`): `isResizable`/`isMiniaturizable`/`isReleasedWhenClosed` = YES (`:42,46,50`); `sendEvent:` mengubah ⌥⎋ jadi popup menu konteks di `visibleRect` first responder (`:61-96`), ⇧⌘}/⇧⌘{ → notifikasi next/prev-tab (`:102,108`), ⌥⌘←/→ sama (`:116,122`); `undoManager` per-tab delegasi ke `SPTableDocument` terdepan (`:141`); `canBecomeMainWindow` untuk sheet `isSheetWhichCanBecomeMain` (`:155`); `toggleToolbarShown:` post `SPWindowToolbarDidToggleNotification` (`:172`).
- **`SPFillView : NSView`** (`SPFillView.h:36`): rect warna solid; warna dari "User Defined Runtime Attribute" string via `setSystemColorOfName:` (`:35-43`); `drawRect:` `NSRectFill` (`:46`).
- **`SPImageView : NSImageView`** (`SPImageView.h:39`): drag-drop file URL/NSFileContents/TIFF/PICT → data ke `SPImageViewDelegate` (`:42-94`); PICT→PNG via `SAImageRenderer`.
- **`SPGeometryDataView : NSView`** (`SPGeometryDataView.h:31`): gambar geometry MySQL (POINT/LINESTRING/POLYGON/GEOMETRYCOLLECTION) dari dict bbox+koordinat; `thumbnailImage` + `pdfData` (`:243,265`).

### Split view & record-view
- **`SPSplitView : NSSplitView`** (`SPSplitView.h:36`): collapse subview dianimasikan (0.25 s, ×10 dengan ⇧) via `NSTimer` 30 fps (`:196-255`); mesin layout `adjustSubviews` kustom dengan min/max + springs/struts, tiga pass fallback + redistribusi sisa (`:331-450`); constraint min/max per-index (`:283,300`); "additional drag handle view" untuk divider pertama (`:266`). Helper `SPSplitViewHelperView` (`:992-1074`) dan `SPSplitViewAnimationRetainCycleBypass` (`:1081-1102`). Komentar menandai `// TODO: leaks` (`:250,729,735`).
- **`SARecordViewResizeHandle`/`SARecordViewOverlayView`** (private NSView, `SARecordView.swift:298-368`): chrome Record View berbasis SwiftUI, cursor resize, drag loop modal, persist width ke `UserDefaults`.

### Control
- **`SPColorSelectorView : NSView`** (`:45`): hingga 7 dot warna + hover; KVO bind/unbind `selectedTag` + cleanup dealloc (`:97,114,129,444`); 8 `NSTrackingArea` (`:206-242`); delegate `colorSelectorDidChange:` (`:77`).
- **`SPComboPopupButton : NSPopUpButton`**: dua zona (title vs arrow) dengan divider gradient (`:88-149`); reroute target/action ke `actionTarget/actionSelector` (`:151-179`); nested `SPComboPopupButtonCell` (`:224-306`).
- **`SPFontPreviewTextField : NSTextField`** (`:30`): `setFont:` render "Name, N.N pt" (`:39`).
- **`SPFilterRuleEditor : NSRuleEditor`** (Swift): drop target `com.sequel-ace.cell-row`; tipe logika `SPFilterRuleEditorDropHandler`, `SARuleFilterContextMenu`, `SARuleFilterPendingStarter`, `SARuleFilterVisibilityPolicy`, `SARuleFilterConjunctionRowPresentation`, `SARuleFilterResizePolicy`, `SARuleFilterDropZoneLayoutPolicy`.
- **`SPFilterRuleTextField : NSTextField`** (Swift) — seam kosong; konstanta `SPCellValuePasteboard` + `rowPayload(...)`.
- **`SPRuleFilterDropBox : NSView`** (Swift): drop zone dengan state prompt/preview WHERE, ⌥-click group, filter menu; override `hitTest`, `resetCursorRects`, `draw`, `mouseDown`, `resizeSubviews`; policy `SARuleFilterPreviewFormatter`, `SARuleFilterDropBoxClickPolicy`, `SARuleFilterDropBoxLayout`.

### Outline view
- **`SPFavoritesOutlineView : NSOutlineView`** (`:31`): `justGainedFocus` (`:49,58`); `menuForEvent:` pilih baris dulu (`:69`); Return/Enter + Tab di `keyDown:` (`:85`); `mouseDown:` simpan `itemForDoubleAction` (`:110`); dot edit kustom (`:147`).
- **`SPNavigatorOutlineView : NSOutlineView`** (`:31`): Enter/Return pilih di dokumen aktif (`:41`).
- **`SPOutlineView : NSOutlineView`** (`:31`): `menuForEvent:` bundle-editor-aware (`:39-73`); `keyDown:` mulai edit pada Return (`:80`).

### Text view
- **`SPBundleCommandTextView : NSTextView`**: gutter nomor baris via `NoodleLineNumberView`; tab stop dari pref; highlight baris aktif; indent/comment; `keyDown:` ukuran font/indent/comment; drag-drop file dengan konfirmasi >1 MB.
- **`SPEditSheetTextView : NSTextView`** (`:31`): helper undo/redo, ⌥ dead-key, ⌘+/⌘-/⌘0, drag-drop dengan deteksi encoding via `/usr/bin/file`.

### Accessory & table view
- **`SPEncodingPopupAccessory : NSObject`** (`:35`): bangun `NSPopUpButton` encoding dari `EncodingPopupView.nib`; `enabledEncodings` urut by Mac-compatible encoding (`:85`).
- **`SPWindowTabAccessory : NSView`** (Swift): strip warna + judul + lock SSL; `update(color:isSSL:)`, `setTitle(title:)`, layout SnapKit.
- **`SPTableView : NSTableView`** (`:31`): `tabEditingDisabled` gate `_allowTabbingIntoCells` (API privat Apple, `:196`); `emptyDoubleClickAction` (`:315`); `keyDown:` Return/Enter/Escape/Tab (`:201-277`).
- **`SPCopyTable : SPTableView`** (`:54`): sumber copy/drag-out (tab/CSV/SQL-INSERT), `draggedRowsAsTabString`, `displayStringForRow:column:`, `isNullAtRow:column:`, autodetect lebar kolom (`:195,214`), keputusan field-editor-vs-in-cell (`:240`).
- **`SATableHeaderView : NSTableHeaderView`** (Swift): `menu(for:)` per kolom via `SATableHeaderViewDelegate.validate(menu:forTableColumn:)` (`:9,18`).
- **`SATablesListView : SPTableView, NSTextInputClient`** (Swift): fuzzy type-ahead mengganti type-select AppKit, IME via `SAMarkedTextInputState`; badge search `SAClickThroughVisualEffectView` (`:895`).
- **`HyperlinkTextField : NSTextField`** (Swift, `:13`): field link-styled, cursor pointing-hand, `mouseDown` buka `href`.
- **`SAWebView : NSViewRepresentable`** + **`SAWKWebView : WKWebView`** (`SAWebView.swift:166,348`): override `willOpenMenu`, `keyDown`, `magnify`; coordinator menangani skema `sequelace://`, `sp-reveal-file://`, `sp-open-file://`, panel JS (`:249-345`).

## 2. Subclass NSCell & formatter

- **`ImageAndTextCell : NSTextFieldCell`** (`:39`): gambar `NSImage` leading + indentasi, reservasi ruang di `editWithFrame:`/`selectWithFrame:`, `cellSize` (`:74,89,104,154,185`). Base untuk favorites/activity/table-name cell.
- **`SPFavoriteTextFieldCell : ImageAndTextCell`** (`:33`): pill `labelColor` rounded hue-boosted (`:35-49`).
- **`SPActivityTextFieldCell : ImageAndTextCell`** (`:33`): baris aktivitas dua baris (nama + info) + cancel-button di kanan (`SPTextLinkRectFromCellRect` = kanan −30 `:50`); `trackMouse:` batalkan aktivitas bash via kill + hapus baris (`:216-283`).
- **`SPTableTextFieldCell : ImageAndTextCell`** (`:33`): `noteButton` kanan non-editable opsional; `drawInteriorWithFrame:` delegasi ke `SATableListCellRenderer` (`:71-76`).
- **`SPTextAndLinkCell : NSTextFieldCell`** (`:31`): tombol link-arrow trailing; `hitTestForEvent:` pisah link vs editable (`:169`); `trackMouse:` jalankan target/action + catat `lastLinkColumn/Row` (`:194,262,270`); state `SPLinkDrawState` (`SPConstants.h:152-154`).
- **`SPComboBoxCell : NSComboBoxCell`** (`:61`): instrumentasi popup via notifikasi privat (`:34-36`), `-popUp:` privat (`:38-39,76`); `SPComboBoxCellDelegate` (`:38-41`).
- **`SPPopUpButtonCell : NSPopUpButtonCell`** (`:43`): `objectValue`/`setObjectValue:` map ke `representedObject` item.
- **`SPTableFooterPopUpButtonCell : NSPopUpButtonCell`**: `drawImageWithFrame:` gambar `alternateImage` saat highlight.
- **`SPColorWellCell : NSActionCell`** (`:31`): swatch warna + checker alpha + warna `objectValue` (`:35`).
- **`SPPillAttachmentCell : NSTextAttachmentCell`** (`:49`): pill bergradien + border; tiga varian `drawWithFrame:` (`:51,56,61,73,91`).
- **`SATableListCellRenderer` (NSObject, Swift)** (`:10`): layout + draw kolom nama/komentar; `layout(in:nameWidth:)` (`:19`), `draw(...)` (`:41`).
- **Formatter**: `SABaseFormatter : Formatter` (`:9`) base sheet; `SAUuidFormatter : SABaseFormatter` (`:9`) `<Data16> ↔ "XXXXXXXX-XXXX-…"` uppercase, `maxLengthOverride`=36 (`:45`), `getObjectValue:` kembalikan `NSNull` untuk kosong/placeholder.

## 3. Parser/formatter/utilitas

### Parsing
- **`SPCSVParser : NSObject`** (`SPCSVParser.h:54`): CSV scanner dari depan `NSMutableString`; multi-char terminator, quote, escape, streaming append, null-replacement. `\N` atau null-replacement tak ter-quote → `NSNull`; baris pendek di-pad `[SPNotLoaded notLoaded]`; lazy trim di `SPCSVPARSER_TRIM_ENACT_LENGTH` = 250000 (`:44`); escape quote ganda gaya Excel kecuali `setEscapeStringsAreMatchedStrictly:`=YES.
- **`SPTableFilterParser : NSObject`** (`:33`): substitusi `$BINARY`, `$CURRENT_FIELD`, `${...}` dan escape 1-2 argumen; `filterString` (`:59`) → fragmen WHERE atau nil (beep) saat jumlah placeholder tak cocok.
- **`SPJSONFormatter : NSObject`** (`:97`): pretty-printer/unformatter berbasis tokenizer C (`SPJSONTokenizerInit` `:71`, `SPJSONTokenizerGetNextToken` `:95`); `stringByFormattingString:` (`:114`) sisip break setelah `,`/`{`/`[`, sebelum `}`/`]`, tab/level, `indentWidth` clamp 1-32; `stringByUnformattingString:` (`:122`).
- **`SAJSONValueDetector` (Swift)**: `isJSONContainer(_:)` deteksi objek/array JSON (maks 5 MB); mengecualikan skalar top-level.

### Extensions (`Source/Other/Extensions/`)
- **`SAArchiving`** (`:25`): tulis keyed+secure (`:31,37`), baca keyed dulu lalu fallback legacy `NSUnarchiver` (`:46,53,59`).
- **`StringExtension.swift`**: helper string (`:11,29`); `SPProcessListRowSerializer` (`:238`); `SATextLimitDecision` (`:309`), `SAFieldEditorEditLimit` (`:327`), `SACellEditLimit` (`:436`); `SPOptimizedFieldTypeEstimator` (`:699`), `SPFieldTypeClassifier` (`:843`), `SPTableLoadFailure` (`:990`), `SPCharacterSetMetadataNormalizer` (`:1019`), `SABitFieldCommitValidation` (`:1085`).
- **`StringRegexExtension.swift`** (`:12,33`): `captureGroup(regex:)` pengganti RegexKitLite.
- **`BundleExtension.swift`** (`:11`): `appName/version/bundleIdentifier/build/isSnapshotBuild/isMASVersion/versionString/githubReleaseTag/checkForNewVersion/saURLScheme`.
- **`ByteCountFormatterExtension`** (`:11`), **`CollectionExtension`** (`:11,28,34,40` — termasuk `PinnedTableMigrationPlanner`), **`DateComponentsFormatterExtension`** (`:11`), **`DateExtension`** (`:11,37`), **`DateFormatterExtension`** (`:15-129`, `mysqlDateTimeString(...)` UTC/POSIX/Gregorian pinned), **`DispatchQueueExtension`** (`:11`), **`FileManagerExtension`** (`:11`), **`NSAlertExtension`** (`:11,47,75` — `SABookmarkPathNormalizer`, `SABookmarkAlertContent`, banyak factory `NSAlert`), **`NSDictionaryExtension`** (`:11,39`), **`NSImageExtensions`** (`:11`), **`NSViewExtension`** (`:11`), **`NumberFormatterExtension`** (`:11`), **`PopupButtonExtensions`** (`:11,24`), **`UserDefaultsExtension`** (`:12` — `saveFont/getFont` + migrasi descriptor legacy).

### Utility
- **`SAImageRenderer`** (`:18,23`): render `NSImageRep` → PNG.
- **`SATypeAheadMatcher`** (`:19`): type-ahead inkremental reset 0.3 s; tier prefix→substring→subsequence; fold case/diacritic/width.
- **`SADatabaseScopedValueCache`** (`:13`): cache satu-nilai per nama database.
- **`SAKeyboardShortcut`** (`:17`): matcher chord exact (cmd/ctrl/opt/shift); preset `commandF` (`:38`).
- **`SAAppearancePreference`** (`:17`): terapkan pref "Appearance" ke `NSApp` hanya saat berubah (`:51`).
- **`SANotificationCenter`** (`:25`): `postNotification(title:body:)` (`:32`) via UserNotifications; auth lazy.
- **`SAMarkedTextInputState`** (`:18`): buffer marked-text IME.
- **`SPIdMenu : NSMenu`** (`:41`): `menuId` IB yang bertahan pada copy/archive.
- **`PanelOptions`** (`:13`): option bag panel; `jsonStringWithPrettyPrint:`.

### CategoryAdditions (ObjC)
`SPArrayAdditions` (join/backtick/comma/CSV, `safeObjectAtIndex:`); `SPCategoryAdditions` (umbrella di PCH); `SPColorAdditions` (hex color); `SPDataAdditions` (sha1/AES/hex/compress/line-enumerate `SPLineTerminator`); `SPDataBase64EncodingAdditions`; **`SPDataStorage`** (`:42`, wrapper editable atas `SPMySQLStreamingResultStore`); `SPDateAdditions` (monotonic time); `SPFileManagerAdditions` (app-support dir, deteksi encoding); `SPMainThreadTrampoline` (`onMainThread`); `SPMutableArrayAdditions` (`reverse`/`unique`/`safe*`); `SPNSMutableDictionaryAdditions`; **`SPNotLoaded`** (sentinel singleton + `isSPNotLoaded` `:38,82`); `SPObjectAdditions` (`cast:`, `isNSNull`, `unboxNull`); `SPPointerArrayAdditions`; `SPScreenAdditions`; **`SPStringAdditions`** (byte-size, UUID, rot13, sha256, HTML/XML escape, backtick/tick quote, `levenshteinDistanceWithWord:`, `safeSubstringWithRange:`, `nonConsecutivelySearchString:matchingRanges:`); `SPTaskAdditions` (`SPlaunch`/`SPterminate`); `SPTextViewAdditions` (transform seleksi, `insertNULLvalue:`, `executeBundleItemForInputField:`); `SPThreadAdditions` (`detachNewThreadWithName:`, `SPCountedObject.instanceId`, `SPCtxt`); `SPWindowAdditions` (`resizeForContentView:`).

## 4. Third-party vendored

- **RegexKitLite** (`Source/ThirdParty/RegexKitLite/`): regex pada string; dipakai luas (27 pengimpor), mis. `SPTextView.m`, `SPBundleCommandTextView.m`, `SPTableFilterParser.m`, `SPSQLParser.m`, `SPStringAdditions.m`.
- **MGTemplateEngine** (`Source/ThirdParty/MGTemplateEngine/`) dengan `ICUTemplateMatcher`: mesin template; dipakai `SPAppController.m:51`, `SPDatabaseDocument.m:76,6265,6293`, `SPHelpViewerClient.m:35,72`.
- **NSMutableArray-MultipleSort** (`:49`): shell-sort dengan array berpasangan; dipakai `SPConnectionController.m`.
- **SPReachability** (Tony Million): reachability SystemConfiguration; dipakai `SPConnectionController.{h,m:387}`.
- **NoodleLineNumberView** (`NSRulerView`): gutter nomor baris; dipakai `SPTextView` + `SPBundleCommandTextView`.
- **YRKSpinningProgressIndicator**: spinning progress `NSView`.
- **NSNotificationCenterThreadingAdditions**: helper threading notification center (via `SPCategoryAdditions.h`).
- **Localization** (`xibLocalizationPostprocessor.m:10`): tool CLI pasca-proses `.strings` XIB; tidak dilink ke app.

## 5. String yang terlihat pengguna (terpilih)

- `SPWindow.m:187,189` — "Hide Toolbar"/"Show Toolbar". `SPWindowTabAccessory.swift` — "Connection Secured via SSL".
- `SPDataCellFormatter.m` — "For BIT fields only “1” or “0” are allowed.", "Maximum text length is set to %ld. Inserted text was truncated.", "Maximum text length is set to %ld.".
- `SAWebView.swift:260,269,283` — "JavaScript" (judul alert/confirm/prompt); "OK"/"Cancel".
- `SARecordView.swift:450-452` — "Record View"/"Toggle Record View"; `:224-228` — "Select one row to view it"/"Select only one row"/"This record has no fields"; menu "Copy Value"/"Edit Value".
- `SAUuidFormatter.swift:14,18` — "Invalid UUID Character in: %@"/"Invalid UUID: %@".
- `SPBundleCommandTextView.m`/`SPEditSheetTextView.m` — "Only one dragged item allowed.", "Do you really want to proceed with %@ of data?", "Couldn't read the file content of".

## 6. UNKNOWN

- `SPURLAdditions.h` mendeklarasikan kategori `NSURL` kosong (implementasi tak diperiksa) — **UNKNOWN** tujuannya.
- Beberapa string tooltip didefinisikan di file interface di luar `Source/Views` (mis. `SPTooltip`) — dimiliki di tempat lain.
