# Audit: Text-view cluster

Audit method-by-method klaster text view. `Start`/`End` = nomor baris di file
masing-masing. Baseline commit `02c910932`. Dua kelas subclass `NSTextView` langsung (tanpa intermediate bersama).

File: `SPBundleCommandTextView.m` (809), `SPEditSheetTextView.m` (324).

---

## 1. `SPBundleCommandTextView.m` (809, header 55)

Subclass `NSTextView` untuk field command Bundle Editor — line numbers, manajemen tab-stop, perintah indent/comment, drag-drop file, dan persistensi font per-delegate.

### Ivar/konstanta
`.h:35` `IBOutlet NSScrollView *commandScrollView`; `.h:37` `NSUserDefaults *prefs` (`awakeFromNib` 52); `.h:39` `BOOL textWasChanged` (set 768, dibaca undo/redo); `.h:40` `NoodleLineNumberView *lineNumberView` (dibuat 61). Tak ada `@property`/`#define`/`static const` di `.m`.
Konstanta eksternal: `SPCustomQueryEditorTabStopWidth` (44, 54, 377, 782); `@"BundleEditorFont"` (56, 57, 365, 366, 371, 796); `SPFileExtensionSQL` (709); `@"CorePasteboardFlavorType 0x54455854"` (605); `NSFileSize`/`NSFileType` (638, 639).

### `#pragma mark`
`109, 595, 656, 747, 764, 790` (semua divider kosong tanpa label).

### Method (23 definisi)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 41–81 | | `dealloc` / `awakeFromNib` | Teardown + setup prefs/font/ruler/observer/tab stop |
| 83 | 107 | `drawRect:` | Latar putih + highlight baris aktif saat on-screen |
| 117 | 172 | `shiftSelectionRight` | Indent baris caret/blok satu tab (dengan undo) |
| 180 | 247 | `shiftSelectionLeft` | Un-indent baris caret/blok satu tab bila mungkin |
| 254 | 309 | `commentOut` | Toggle comment `#`/`--` baris via regex |
| 311–345 | | `undo:` / `redo:` / `paste:` / `cut:` | Undo/redo + grouping |
| 357 | 409 | `setTabStops` | Bangun paragraph style tab-stop 256-px dari lebar pref |
| 415 | 477 | `wrapSelectionWithPrefix:` | Bungkus seleksi dengan karakter auto-pair |
| 479 | 543 | `keyDown:` | Tangani ⌘+/-/0, ⌘[/⌘], ⌘/, undo grouping, auto-pair |
| 550 | 593 | `doCommandBySelector:` | Pertahankan indentasi saat sisip newline |
| 601 | 654 | `performDragOperation:` | Sisip konten file drop atau path (⌘ toggle) |
| 662–680 | | `characterIndexOfPoint:` | Konversi titik view → index karakter |
| 686 | 745 | `insertFileContentOfFile:` | Baca file via UNIX `file`, deteksi encoding, append |
| 752–788 | | `validateMenuItem:` / `textDidChange:` / `boundsDidChangeNotification:` / `observeValueForKeyPath:...` | Menu + notifikasi |
| 793–807 | | `saveChangedFontInUserDefaults` / `changeFont:` | Font |

### Temuan
- **Indent**: `shiftSelectionRight` tangani caret-only (125–137) + blok (140–170), sisip `\t`, re-seleksi range melebar; register undo via `shouldChangeTextInRange:` (131, 161). Branch manual `\r`/`\n` (151–158). `shiftSelectionLeft` mirror, terima tab atau spasi (194–196, 220–222).
- **Comment**: `commentOut` (254–309) default token `#`, ganti `--` hanya bila buffer `#!...osascript` shebang (260–266). Pakai **RegexKitLite** (`isMatchedByRegex`, `replaceOccurrencesOfRegex`, `rangeOfRegex`) (277–296).
  - ⚠️ **UNKNOWN**: setelah `setSelectedRange:lineRange` (302) kode **append** (`appendAttributedString:` 303) padahal komentar bilang "Replace current line by (un)commented string" (299). Pola append-setelah-setSelection sama di 470 dan 578, 580.
- **Undo**: `undo:`/`redo:` (311–335) panggil `[[self undoManager] undo]` hingga 3×, loop selama `textWasChanged` tetap `NO`, untuk skip record undo no-op (317–319). Undo group difinalisasi dengan poking `setWasCutPaste` delegate (306–308, 340–342, 350–352, 585–587); delegate di-cast ke `SPBundleEditorController` (307).
- **Drag-drop**: `performDragOperation:` (601–654). Delegasi super bila pasteboard bawa file URL + `CorePasteboardFlavorType 0x54455854` (605–606). Hanya satu item; multi-file ditolak + alert (615–618). Bila `draggingSourceOperationMask == 4` (⌘ ditekan) → sisip **path** saja (629–632); else cek `NSFileTypeRegular` + ukuran file, konfirmasi >1 MB (642–645), lalu `insertFileContentOfFile:` (644/647). **UNKNOWN**: literal `4` dipakai alih-alih simbol `NSDragOperation` bernama (629).
- **Sisip file**: `insertFileContentOfFile:` (686–745) spawn `/usr/bin/file -Ib` via `NSTask` (699–703) untuk tebak encoding, map `utf-8`/`iso-8859-1`/`us-ascii` (718–723), fallback deteksi Cocoa untuk octet-stream/utf-16 (716–717, 727–728). Konten di-append (733, 739).
- **Auto-pair**: `wrapSelectionWithPrefix:` (415–477) dukung `(` `"` `` ` `` `'` `{` `[` + curly quotes `0x201c`/`0x2018` (428–458); return `NO` untuk panjang 0 atau non-editable (421).
- **Tab stops**: `setTabStops` (357–409) sementara paksa editable (374–375) sekitar mutasi atribut, lalu restore (408). `tabStopWidth` floor 1 (378). Di-trigger KVO `SPCustomQueryEditorTabStopWidth` (782–783).
- **Newline indent**: `doCommandBySelector:` (550–593) juga tangkap numpad-Enter via `keyCode == 0x4C` (554) + short-circuit `super` hindari linebreak ganda (590).
- **Line numbers**: `NoodleLineNumberView` dipasang ke `commandScrollView` di `awakeFromNib` (61–65); background layout dimatikan agar baca range lebih aman (75); repaint scroll via `boundsDidChangeNotification:` (775–778).
- ⚠️ **Font**: `saveChangedFontInUserDefaults` guard pada **nama kelas delegate** string equality `@"SPBundleEditorController"` (795) — cek tipe-by-string (design-smell). `changeFont:` (800–807) juga guard `prefs && [self font] != nil`.

---

## 2. `SPEditSheetTextView.m` (324, header 38)

Subclass `NSTextView` untuk sheet field-editor — grouping undo/redo, ukuran font, drag-drop file, persistensi font `SPFieldEditorController`.

### Ivar/konstanta
`.h:32` `BOOL textWasChanged` (set 106, dibaca 45, 52, 58, 65). Tak ada `@property`/`#define`/`static const` di `.m`.
Konstanta eksternal: `@"CorePasteboardFlavorType 0x54455854"` (167); `SPFileExtensionSQL` (271); `@"FieldEditorSheetFont"` (313); `NSFileSize`/`NSFileType` (202, 203).

### `#pragma mark`
**Tidak ada.**

### Method (13 definisi)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 38 | 41 | `init` | Override trivial return `[super init]` |
| 43–67 | | `undo:` / `redo:` | Ulangi undo/redo hingga perubahan teks nyata |
| 69–87 | | `paste:` / `cut:` | Buka undo group lalu panggil super |
| 92–107 | | `validateMenuItem:` / `textDidChange:` | Enable menu + flag perubahan |
| 109 | 158 | `keyDown:` | Tangani ⌥ dead keys, ⌘+/-/0 font, undo grouping |
| 164 | 218 | `performDragOperation:` | Sisip konten file drop atau path (⌘ toggle) |
| 224–242 | | `characterIndexOfPoint:` | Titik view → index karakter |
| 248 | 307 | `insertFileContentOfFile:` | Baca file via UNIX `file`, deteksi encoding, append |
| 310–322 | | `saveChangedFontInUserDefaults` / `changeFont:` | Font |

### Temuan
- **Undo**: `undo:`/`redo:` (43–67) logika byte-identik dengan kelas Bundle — ulangi panggilan undo/redo selama `textWasChanged` `NO` (49–53, 62–66). `validateMenuItem:` gate pada `canUndo`/`canRedo` (95–100). Undo group difinalisasi dengan cast delegate ke `SPFieldEditorController` + `setWasCutPaste` (73, 83) dan `setDoGroupDueToChars` (154).
- **Key handling/dead keys**: `keyDown:` (109–158) berbeda dari kelas Bundle — eksplisit lolos saat hanya ⌥ ditekan atau tak ada karakter (dukung komposisi aksen/dead-key) (113–120). **Tidak** implementasi `shiftSelection*`, `commentOut`, `wrapSelectionWithPrefix:` (tanpa auto-pair). Shortcut font ⌘+/⌘-/⌘0 → `makeTextSizeLarger/Smaller/StandardSize` (126–145) — kategori di `SPTextViewAdditions.h:53–55`. **UNKNOWN**: beda dari Bundle, `keyDown:` tak override font via `[self font]` guard; hanya panggil save (129, 136, 143).
- ⚠️ **Drag-drop**: `performDragOperation:` (164–218) **near-exact copy** versi Bundle (167–215 vs 605–651): guard pasteboard type sama, limit satu item, `sourceOperationMask == 4` sisip path (193), konfirmasi 1 MB (206–209), lalu `insertFileContentOfFile:` (208, 211). **UNKNOWN/risiko duplikasi**: dua implementasi diduplikasi lintas file alih-alih dibagi.
- ⚠️ **Sisip file**: `insertFileContentOfFile:` (248–307) juga duplikat verbatim (`/usr/bin/file -Ib` 261–265; peta encoding 278–287); append di 295, 301.
- ⚠️ **Font**: `saveChangedFontInUserDefaults` (310–314) guard nama kelas delegate string equality `@"SPFieldEditorController"` (312), simpan key `@"FieldEditorSheetFont"` (313). `changeFont:` (317–322) **tanpa** guard `prefs`/nil-font, beda dari Bundle (802).
- **Hilang vs kelas Bundle**: tanpa `awakeFromNib`, `drawRect:`, `setTabStops`, line-number view, `boundsDidChangeNotification:`, `observeValueForKeyPath:`, `dealloc`. `init` (38–41) no-op passthrough — **UNKNOWN** kenapa didefinisikan.

---

## Ringkasan
| Aspek | SPBundleCommandTextView | SPEditSheetTextView |
| --- | --- | --- |
| Baris (`.m`) | 809 | 324 |
| Method | 23 | 13 |
| `#pragma mark` | 6 (kosong) | 0 |
| Ivar (`.h`) | 4 | 1 |
| Cast delegate | `SPBundleEditorController` | `SPFieldEditorController` |
| Key pref font | `@"BundleEditorFont"` | `@"FieldEditorSheetFont"` |
| Indent/comment | Ya | Tidak |
| Auto-pair | Ya | Tidak |
| Drag-drop | Ya (601) | Ya (164) — duplikat |
| Sisip file | Ya (686) | Ya (248) — duplikat |
| Line numbers | Ya (NoodleLineNumberView) | Tidak |
