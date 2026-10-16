# Audit: Filter-editor Swift cluster

Audit method-by-method tiga file klaster filter editor Swift. `Start`/`End` =
nomor baris di file masing-masing. Baseline commit `02c910932`.

File: `SPCustomQuerySQLClassifier.swift` (623 baris), `SPFilterRuleEditor.swift` (607),
`SARecordView.swift` (605). **Tak ada `// MARK:`** di ketiganya.

---

## File 1 — `SPCustomQuerySQLClassifier.swift`

- Peran: classifier prefiks SQL stateless (explain-safe / gate warning destruktif)
  plus helper regex konteks database, berbagi satu lexer strip-comment.

### Tipe & konstanta
`enum SPCustomQuerySQLClassifier` (namespace, 12–472) + nested `private struct CommentStrippingResult` (14–17);
`@objc final class SASQLDatabaseContext` (474–623).
`mutatingExplainAnalyzeStatements = {UPDATE,DELETE,INSERT,REPLACE}` (19–24);
`identifierPattern` (476); `identifierSeparator` (477); `useRegex` (478–480); `dropDatabaseRegex` (481–483).

### Tabel definisi (urut baris)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 15–16 | | `CommentStrippingResult.sql`/`hasIndeterminateExecutableComment` | Hasil lexer |
| 19–24 | | `mutatingExplainAnalyzeStatements` | Statement yang membuat EXPLAIN ANALYZE tak aman |
| 26 | 30 | `identifierSet` | Alfanumerik + `_` untuk batas keyword |
| 32 | 49 | `isQueryExplainable(_:)` | True hanya SELECT/WITH leading (allowBare:false) |
| 51 | 73 | `isQuerySafeWithoutDestructiveWarning(_:serverVersion:serverIsMariaDB:)` | Wajib aman di kedua bacaan backslash |
| 82 | 117 | `isQuerySafeWithoutDestructiveWarning(_:serverVersion:serverIsMariaDB:backslashEscapes:)` | Penilaian aman satu-bacaan; SHOW/SELECT/EXPLAIN |
| 128 | 138 | `stripSQLComments(_:serverVersion:serverIsMariaDB:)` | Stripper komentar publik (teks saja) |
| 144 | 274 | `stripSQLCommentsWithMetadata(_:serverVersion:serverIsMariaDB:backslashEscapes:)` | Lexer inti; strip komentar, pertahankan body executable |
| 277 | 279 | `isASCIIDigit(_:)` | Tes scalar 0–9 untuk gate versi |
| 281 | 305 | `shouldPreserveExecutableComment(...)` | Gate versi/MariaDB retensi comment executable |
| 307 | 314 | `hasLeadingSQLKeyword(_:in:allowBare:)` | Cek keyword prefiks dengan batas identifier |
| 318 | 341 | `isExplainAliasSafeWithoutWarning(_:alias:backslashEscapes:)` | Nilai verb target EXPLAIN [ANALYZE] |
| 358 | 435 | `sqlTokens(from:backslashEscapes:)` | Split statement EXPLAIN jadi token gaya server |
| 437 | 471 | `skipExplainModifiers(in:from:)` | Lewati modifier EXTENDED/PARTITIONS/FORMAT/INTO/FOR |
| 476–483 | | regex identifier/separator/use/dropDatabase | Pola |
| 486–488 | | `databaseNameChanged(from:to:)` | Banding nama DB |
| 491 | 516 | `requiresDatabaseNameCaseSensitivityLookup(...)` | True hanya untuk beda DROP beda-kapital |
| 519 | 553 | `databaseName(afterSuccessfulQuery:...)` | Resolve konteks DB setelah USE/DROP |
| 555 | 567 | `queryCouldChangeDatabaseContext(_:)` | Gate prefiks/comment murah sebelum strip |
| 569 | 588 | `databaseName(matchedBy:in:)` | Ekstrak + unquote capture regex pertama |
| 590 | 608 | `hasLeadingKeyword(_:in:at:)` | Keyword leading case-insensitive dengan batas |
| 610 | 614 | `isIdentifierCharacter(_:)` | `_`, `$`, atau alfanumerik |
| 616 | 622 | `makeRegex(pattern:)` | Kompilasi regex `preconditionFailure` saat error |

### Temuan / UNKNOWN
- **Aturan backslash ganda**: tanpa `sql_mode`, classifier membaca query dengan backslash ON dan OFF dan wajib keduanya aman (58–72); caller yang pre-split pakai overload satu-bacaan (75–87).
- **Strip komentar berbasis scalar** (bukan Character) sengaja agar combining mark setelah quote tak menyembunyikan delimiter (150–152, 349–351).
- **Bentuk komentar**: `#` sampai EOL (187–194); `--` hanya bila diikuti whitespace/control via `SASQLCommentSyntax.isCommentWhitespace` (196–206); `/* … */` (208–263); executable `/*! … */` (211–212) + MariaDB-only `/*M! … */` (213–218). Marker dalam quote/backtick dipertahankan (161–185). Aturan harus terus mirror `SADatabaseAssertion` SPMySQLFramework (124–127).
- **Gate versi**: digit leading setelah `/*!` di-parse sebagai versi wajib (231–237); retensi via `shouldPreserveExecutableComment` (281–305). **Flag indeterminate** di-set saat serverVersion nil + body version-gated/MariaDB-only (238–241); kedua entry safety menolak (39, 96). Body executable di-strip rekursif (249–258).
- **Aturan konservatif EXPLAIN ANALYZE**: `EXPLAIN ANALYZE WITH …` → tak aman (CTE tak bisa diparse, 333–338); selain itu tak aman hanya bila verb akhir di `mutatingExplainAnalyzeStatements` (340). `isQueryExplainable` juga larang SELECT/WITH bare (48). Unwrapping parenthesis leading di kedua classifier (42–44, 101–103).
- **Jalur konteks DB** hanya strip komentar untuk USE/DROP leading atau komentar leading (555–567); regex anchored + tangani identifier ter-quote dengan unescape quote-ganda (569–588). DROP nama persis → nil; match case-insensitive hanya saat nama case-insensitive (543–550).
- **UNKNOWN**: `CommentStrippingResult` tanpa `Equatable`/visibilitas publik; tak ada dokumentasi kenapa `makeRegex` pakai `preconditionFailure` (616–622).

---

## File 2 — `SPFilterRuleEditor.swift`

- Peran: UI filter rule tab Content — policy drop-target, pelacakan baris seeded,
  policy resize/drop-zone layout, dan subclass `NSRuleEditor` drag-and-drop.

### Tipe & konstanta
`@objc protocol SPFilterRuleEditorDropHandler` (15–43); `enum SARuleFilterContextMenu` (48–69);
`SARuleFilterPendingStarter` (84–254) + nested `private final class SAWaitingRow` (87–109);
`SARuleFilterVisibilityPolicy` (258–274); `SARuleFilterConjunctionRowPresentation` (278–291);
`@objc enum SARuleFilterResizeAction` (295–302, `none`/`immediate`/`deferred`);
`SARuleFilterResizePolicy` (321–339); `SARuleFilterDropZoneLayoutMetrics` (344–361);
`SARuleFilterDropZoneLayoutPolicy` (366–424); `SPFilterRuleEditor: NSRuleEditor` (434–582);
`private final class SPFilterRuleEditorHighlight: NSView` (589–607).

Konstanta: `preferenceKey = "RuleFilterShowDropZone"` (367); `buttonBarHeight = 31` (397);
`deferredResizeDelay = 0.2` (323); `rowDropType` (435).

### Tabel definisi (urut baris)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 22–42 | | protocol methods `appendFilter`/`replaceFilter`/`addEmptyFilterRow`/`addEmptyFilterGroup` | Kontrak drop handler |
| 51 | 68 | `SARuleFilterContextMenu.menu(for:)` | Bangun menu konteks Add Filter/Add Group |
| 90–108 | | `SAWaitingRow.*` | Ref checkbox weak + closure clear + init + endTracking |
| 114–253 | | `waiting`, `begin(with:clearingRestoredMark:)`, `forgetCheckbox`, `forgetRow`, `isCheckbox`, `row(in:)`, `reusableEmptyRow`, `enableIfRow`, `waitingRows`, `isReusableEmptyRow` | Pelacakan baris seeded |
| 265 | 273 | `shouldAddStarterRule(...)` | Putuskan seed starter rule |
| 281–290 | | `explainerText`, `isConjunctionChoice(_:)` | Presentasi baris konjungsi |
| 323–338 | | `deferredResizeDelay`, `action(rowCount:previousRowCount:)` | Policy resize |
| 345–360 | | `SARuleFilterDropZoneLayoutMetrics` fields + init | Metrics layout |
| 367–423 | | `preferenceKey`, `defaultsKey`, `metrics(editorVisible:...userDefaults:)`, `buttonBarHeight`, `metrics(editorVisible:...showDropZonePreference:)` | Policy layout drop zone |
| 435–581 | | `rowDropType`, `highlightOverlay`, `init(frame:)`, `init?(coder:)`, `menu(for:)`, `draggingEntered/Updated/Exited`, `prepareForDragOperation`, `performDragOperation`, `topLevelOrdinal(forRow:)`, `concludeDragOperation`, `dragOperation(for:)`, `row(for:)`, `showHighlight`, `clearHighlight` | Subclass NSRuleEditor drag-drop |
| 590–606 | | `SPFilterRuleEditorHighlight.init`/`init?(coder:)`/`hitTest`/`draw` | Overlay highlight transparan mouse |

### Temuan / UNKNOWN
- **Policy drop: replace vs append.** Rule editor sendiri hanya **replace** rule existing; appending didelegasikan ke `SPRuleFilterDropBox` agar target sendiri yang mendisambiguasi (426–433).
- **Target drop valid** butuh pasteboard membawa `rowDropType` (531), delegate adopt protokol (532), baris top-level `parentRow(forRow:) == -1` (550) dan tipe `.simple` (551); baris compound/nested ditolak. Frame baris dihitung dari `rowHeight`, bukan hit-testing view, karena NSRuleEditor memakai ulang display view antar baris (526–529).
- **Mapping visible-index → root-ordinal** esensial karena `row` adalah index flat termasuk subrow; handler mengalamatkan root children (492–503).
- **Highlight baris** ditambah frontmost + hit-test-transparent; overlay stroke-only agar kontrol baris tetap interaktif (555–573, 584–588, 598–600).
- **Pelacakan baris seeded** (`SARuleFilterPendingStarter`): tiap baris seeded dilacak terpisah + weak; baris seeded mulai **unchecked** karena baris kosong checked membuat preview WHERE menampilkan `column = ''` (71–83). `reusableEmptyRow` hanya reuse baris seeded yang masih kosong (183–198); `isReusableEmptyRow` butuh top-level `.simple`, checkbox off, field teks kosong semua (245–253).
- **Aturan seed starter**: `(!visibilityWasApplied || !wasVisible || tableChanged) && willBeVisible && editorIsEmpty` (265–273); policy ada agar setter visibilitas idempoten selama reload (256–263).
- **Policy resize**: baris bertambah → `.immediate`; berkurang → `.deferred` setelah 0.2 s; tetap → `.none` (304–320, 333–338). Caller harus cancel deferred resize tertunda (319–320).
- **Layout drop zone**: preferensi hilang pertahankan perilaku (default tampil) (363–365, 382–384). Bar bawah selalu direservasi saat UI filter terlihat; drop zone tersembunyi menyusutkan bar ke `buttonBarHeight` 31 (395–397, 409–413). Rule editor dapat margin atas 1pt + min height 29 (414–415).
- **UNKNOWN**: dua overload `metrics` beda hanya label parameter (`userDefaults:` vs `showDropZonePreference:`) + visibilitas; yang internal tak `@objc` dan mungkin dipanggil test, tapi tak ada call site di file ini (366–423). `SPCellValuePasteboard`/`SPRuleFilterDropBox` di luar file (435, 484–491).

---

## File 3 — `SARecordView.swift`

- Peran: panel record-view SwiftUI (tabel field, editing inline, search) + overlay
  controller AppKit, resize handle, dan dukungan toolbar/keyboard.

### Tipe & konstanta
`SARecordField` (10–24); `enum SARecordViewFocusTarget` (26–29); `enum SARecordViewFocusAction` (31–35);
`@objc final class SARecordViewEditSupport` (37–69); `@objc final class SARecordViewColumnMapping` (77–91);
`final class SARecordViewModel: ObservableObject` (93–192); `private struct SARecordView: View` (194–296);
`private final class SARecordViewResizeHandle: NSView` (298–308);
`private final class SARecordViewOverlayView: NSView` (310–368);
`@objc final class SARecordViewToolbarSupport` (370–457);
`@objc final class SARecordViewController` (459–605).

Konstanta: `contentIdentifier = "SwitchToTableContentToolbarItemIdentifier"` (371),
`queryIdentifier = "SwitchToRunQueryToolbarItemIdentifier"` (372),
`itemIdentifier = "RecordViewToolbarItemIdentifier"` (374),
width `minWidth 240`/`idealWidth 320` (209), default overlay 320/min 240 (518),
keyCode 49 = space, 123 = left, 124 = right (391, 423, 436).

### Tabel definisi (urut baris)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 11–23 | | `SARecordField.id`/`name`/`value`/`previewValue` | Field + preview terflatten 4096 |
| 39 | 68 | `SARecordViewEditSupport.validate(_:with:)` | Validasi formatter; jalur parsial `SABaseFormatter` |
| 85 | 90 | `SARecordViewColumnMapping.tableColumn(forFieldID:in:)` | Resolve field ID → kolom tabel aktif |
| 94–106 | | `SARecordViewModel` published props + callback `beginEditing`/`validateEditing`/`commitEditing` | State model |
| 108–130 | | `visibleFields`, `canFocusTable`, `update(fields:selectedRowCount:)`, `clear()` | Aksesor + mutasi |
| 132–191 | | `requestEdit`/`requestFocus`/`commitEdit`/`validatedEditDraft`/`updateEditDraft`/`editorFocusChanged`/`cancelEdit` | Alur editing |
| 195–295 | | `SARecordView` body/content/recordFieldsTable/fieldMenu/field/emptyState/copy + FocusState | View SwiftUI |
| 299–367 | | `SARecordViewResizeHandle` + `SARecordViewOverlayView` methods | Resize overlay |
| 371–456 | | `SARecordViewToolbarSupport` ids + `hostIdentifier`/`shouldHandleSpace`/`focusAction`/`makeToolbarItem` | Toolbar/keyboard |
| 460–604 | | `SARecordViewController` props + `deinit`/`installOverlay`/`isVisible`/`setShowHandler`/`toggle`/`update`/`fields`/`clear`/`setEditingHandlers` | Controller |

### Temuan / UNKNOWN
- **Install overlay**: `installOverlay` wire monitor keyDown lokal (481), baca lebar tersimpan default 320/min 240 clamp ke lebar parent (517–518), bangun container di tepi kanan menjangkau bottomInset..topInset (519–524), separator border 1px, set resized view + right inset, tambah hosting SwiftUI, tambah resize handle 10pt, mulai tersembunyi (526–547).
- **Policy key monitor**: space (keyCode 49, tanpa modifier, bukan repeat) toggle record view — diterima saat tabel first responder, atau first responder adalah view non-teks di dalam record view (385–406). `focusAction`: panah kanan dari tabel dengan `canFocusRecordView` → `.recordView` (123–128); dari dalam record view, panah kiri → `.resultTable`, cmd-F → `.filter`, dan text-field/text-view first responder menekan ini (430–441); else nil (442).
- **Policy resize overlay**: drag pakai loop `nextEvent` manual (bukan tracking loop); lebar dipersist saat left mouse up (317–331). Lebar clamp `min(max(w,240), parent.bounds.width)` (355); lebar resized view = `frame.minX - resizedFrame.minX - 1`, floored 0 (365). `hide()` restore target mengisi parent minus insets (341–349).
- **Editing inline**: draft edit di-bind via computed `validatedEditDraft` (bukan closure `Binding`) agar SwiftUI bisa bandingkan keypath stabil (`@Bindable` butuh macOS 14) (158–168). Editing dibersihkan saat focus hilang via `editorFocusChanged` (183–185). Sel value tukar TextField/Text pada `editingFieldID` + focus async saat appear (244–261).
- **Identifikasi field by column index, bukan posisi layar**, sehingga reorder/hide kolom tak salah alamat edit (71–76, 85–90); lookup bangun `NSUserInterfaceItemIdentifier(String(fieldID))` + tolak ID negatif (86–89).
- **Preview truncation**: prefiks 4097, flag truncation di atas 4096, flatten newline→spasi dan tab→spasi, append `…` saat truncated (15–23).
- **Validasi**: hanya `SABaseFormatter` pakai jalur partial-string validity via `UnsafeMutablePointer<NSString>` dengan seleksi UTF-16 di akhir string (44–60); formatter lain pakai `isPartialStringValid(_:newEditingString:errorDescription:)` (62–67).
- **Toolbar host mapping**: tab index 1 → content, 2 → query, else nil (377–383). Toolbar item pakai simbol `sidebar.right` dengan fallback `NSListViewTemplate` + selector `toggleRecordView:` (446–456).
- **UNKNOWN**: `SABaseFormatter` dirujuk (44) tapi tak didefinisikan di sini; `update(with:selectedRowCount:)` + `fields(from:)` mengharapkan dict dengan key literal `id`/`name`/`value` (583–585) — sisi produsen di luar file ini.

---

## Ringkasan klaster
- **Nol `// MARK:`** di ketiga file.
- **Constraint mirroring komentar**: aturan comment/quote classifier harus tetap sinkron dengan `SADatabaseAssertion` SPMySQLFramework (`SPCustomQuerySQLClassifier.swift:124–127, 526–528`) — coupling lintas-modul yang tak ditegakkan type system.
- **Kelas policy diekstrak untuk testability**: file 2 memisahkan tipe keputusan murni (`SARuleFilterVisibilityPolicy`, `SARuleFilterResizePolicy`, `SARuleFilterDropZoneLayoutPolicy`) dari view AppKit, semuanya `@objc` untuk caller ObjC legacy.
- **UNKNOWN klaster**: file test tak diperiksa, jadi apakah overload policy + edge case classifier benar-benar tercoverage tak bisa dikonfirmasi pada kedalaman ini.
