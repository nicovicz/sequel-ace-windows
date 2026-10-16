# Audit: `SPEditorPreferencePane.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/Preferences/Panes/SPEditorPreferencePane.m`. Baseline commit
`02c910932`.

- File: 1018 baris; header 84 baris.
- Peran: controller pane preferensi **Query Editor**: tabel warna syntax, font
  editor, dan manajemen file colour-theme `*.spTheme` (import/export/duplicate/
  rename/remove).
- Total: 36 definisi method.

## 1. Inventaris ivar/property + konstanta

### Ivar publik `.h:40–67`
| `.h` | Tipe | Nama | Peran |
| --- | --- | --- | --- |
| 42 | IBOutlet NSWindow | `enterNameWindow` | Sheet penamaan theme (`.m:201`) |
| 43 | IBOutlet NSWindow | `editThemeListWindow` | Sheet daftar theme (`.m:347`) |
| 45–50 | IBOutlet NSTextField | `enterNameLabel`, `enterNameInputField`, `enterNameAlertField`, `colorThemeName`, `colorThemeNameLabel`, `editorFontName` | (`.m:197-199, 211-213, 425-442, 671, 688, 359`) |
| 52–56 | IBOutlet NSButton/MenuItem | `themeNameSaveButton`, `editThemeListTable`, `removeThemeButton`, `duplicateThemeButton`, `saveThemeMenuItem` | (`.m:221, 345, 537, 675, 689`) |
| 58–59 | IBOutlet NSTableView/NSMenu | `colorSettingTableView`, `themeSelectionMenu` | (`.m:119-132, 331-332, 486, 533, 545, 607, 1009`; menu dibangun `.m:382-417`) |
| 61–63 | `NSArray`/`NSUInteger` | `editorColors`, `editorNameForColors`, `colorRow` | (`.m:78-90`, `.m:92-104`, `.m:478, 611`) |
| 65–66 | `NSString`/`NSArray` | `themePath`, `editThemeListItems` | (`.m:74`, `.m:76, 230, 260, 343`) |

### Class-extension `.m:51–59`
51 `@interface SPEditorPreferencePane () <NSControlTextEditingDelegate, NSFontChanging>`;
52–55 forward decl `_checkForUnsavedTheme`, `_getAvailableThemes`, `_saveColorThemeAtPath:`,
`_loadColorSchemeFromFile:`; 57 `@property fileManager` (synthesize `.m:63`).

### Konstanta file-local
`SPSaveColorScheme=@"SaveColorScheme"` (41, **dead**), `SPDefaultColorSchemeName=@"Default"` (42, `.m:309`),
`SPDefaultColorSchemeNameLC=@"default"` (43, `.m:563, 674`), `SPCustomColorSchemeName=@"User-defined"` (44, `.m:174, 255, 462, 515`),
`SPCustomColorSchemeNameLC=@"user-defined"` (45, `.m:431, 563, 674, 741`),
`#define SP_EXPORT_COLOR_SCHEME_NAME_STRING` = "MyTheme" (47, `.m:148`).

## 2. Semua `#pragma mark`

`65/66 (Initialisation), 135/136 (IB action methods), 350/351 (Public API), 470/471 (Font panel methods), 528/529 (TableView datasource methods), 602/603 (TableView delegate methods), 658/659 (TextField delegate methods), 696/697 (Preference pane protocol methods), 736/737 (Private API)`.

## 3. Tabel method (36 definisi, urut baris)

| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 68 | 108 | `init` | Seed file manager, dir theme, list theme, array key warna/label |
| 113 | 133 | `awakeFromNib` | Pasang `SPColorWellCell` di tabel warna; set latar tabel |
| 138 | 156 | `exportColorScheme:` | Save panel (.spTheme) → `_saveColorThemeAtPath:` |
| 158 | 180 | `importColorScheme:` | Open panel terima spTheme/tmTheme → load, tandai theme User-defined |
| 182 | 191 | `loadColorScheme:` | Muat file theme bernama dari judul sender; simpan nama theme |
| 193 | 217 | `saveAsColorScheme:` | Sheet penamaan; buat dir Themes; simpan + pilih theme baru |
| 219 | 242 | `duplicateTheme:` | Copy theme terpilih ke `<name>_copy.spTheme` |
| 244 | 272 | `removeTheme:` | Hapus file theme terpilih; bersihkan pref theme aktif bila cocok |
| 274 | 278 | `closePanelSheet:` | Akhiri sheet sender dengan tag-nya sebagai return code |
| 283 | 292 | `showCustomQueryFontPanel:` | Buka font panel dengan target pref font editor |
| 294 | 298 | `resetSystemFont:` | Reset pref font editor ke system font |
| 303 | 335 | `setDefaultColors:` | Restore 11 warna dari default registration-domain |
| 340 | 348 | `editThemeList:` | Buka sheet edit daftar theme + refresh baris |
| 356 | 361 | `updateDisplayedEditorFontName` | Terapkan font tersimpan ke field font; reload tabel warna |
| 363 | 377 | `delayStepperChanged:` | Warn via tooltip bila delay auto-complete < 0.1 |
| 382 | 417 | `updateColorSchemeSelectionMenu` | Bangun ulang submenu theme dari disk + item Default/Edit |
| 422 | 468 | `updateDisplayColorThemeName` | Tampilkan/validasi nama theme aktif; turunkan invalid ke User-defined |
| 476 | 489 | `colorClick:` | Buka color panel bersama untuk baris warna diklik |
| 494 | 518 | `colorChanged:` | Simpan warna teredit, tandai theme User-defined, refresh UI |
| 523 | 526 | `validModesForFontPanel:` | Batasi font panel ke mode size & collection |
| 531 | 541 | `numberOfRowsInTableView:` | Jumlah baris per tabel: key warna atau nama theme |
| 543 | 553 | `tableView:objectValueForTableColumn:row:` | Nilai sel: string nama warna atau warna ter-arsip; nama theme |
| 555 | 600 | `tableView:setObjectValue:forTableColumn:row:` | Rename file theme setelah validasi keunikan + nama reserved |
| 605 | 624 | `tableView:shouldEditTableColumn:row:` | Klik tabel warna buka color panel alih-alih editing |
| 626 | 656 | `tableView:willDisplayCell:forTableColumn:row:` | Gaya sel nama warna pakai warna editor aktif |
| 666 | 694 | `controlTextDidChange:` | Validasi nama theme baru; toggle tombol Save + field alert |
| 699 | 702 | `preferencePaneView` | View pane |
| 704 | 711 | `preferencePaneIcon` | SF Symbol terminal, atau image toolbar legacy pra-macOS 11 |
| 713 | 716 | `preferencePaneName` | Judul pane "Query Editor" |
| 718 | 721 | `preferencePaneIdentifier` | `SPPreferenceToolbarEditor` |
| 723 | 726 | `preferencePaneToolTip` | Tooltip pane |
| 728 | 734 | `preferencePaneWillBeShown` | Refresh menu theme, nama theme, display font |
| 739 | 759 | `_checkForUnsavedTheme` | Prompt Proceed/Cancel bila theme belum tersimpan; tutup color panel |
| 761 | 793 | `_getAvailableThemes` | Daftar file `.spTheme` di dir theme, ekstensi dilepas |
| 795 | 895 | `_saveColorThemeAtPath:` | Serialisasi warna aktif ke file theme plist XML |
| 897 | 1016 | `_loadColorSchemeFromFile:` | Parse plist theme, terapkan entri warna utama + opsional |

## 4. Temuan menonjol / UNKNOWN

**Yang dimiliki pane**
- **Key warna (11)** di `editorColors` (`78–90`): `SPCustomQueryEditorTextColor`, `…BackgroundColor`, `…CaretColor`, `…CommentColor`, `…SQLKeywordColor`, `…NumericColor`, `…QuoteColor`, `…BacktickColor`, `…VariableColor`, `…HighlightQueryColor`, `…SelectionColor` (baca 484, 506, 546, 616, 651).
- **Pref font**: `SPCustomQueryEditorFont` (baca 122, 290, 358, 632; tulis 296).
- **Pref nama theme**: `SPCustomQueryEditorThemeName` (`SPConstants.m:187`) — tulis 174, 187, 213, 255, 309, 462, 515, 588; baca 254, 424, 431, 438, 587, 741.
- **Delay auto-complete**: `delayStepperChanged:` (363–377) hanya tooltip warning; **tidak** menulis pref apa pun — binding stepper di tempat lain. **UNKNOWN** key-nya dari file ini.
- Nilai sentinel: `"Default"`/`"User-defined"` (42, 44); bentuk lowercase untuk perbandingan case-insensitive (43, 45).

**Default**: `setDefaultColors:` ambil default vendor dari `[prefs volatileDomainForName:NSRegistrationDomain]` (311) — bergantung `-registerDefaults:` dipanggil (komentar 311), lalu copy 11 key (313–329) + set nama theme `"Default"` (309). **UNKNOWN** nilai default literal (bukan di file ini). Theme bundled di-seed ke disk di luar pane (`SPAppController.m:1552-1567`); pane hanya enumerasi (761–793).

**Wiring kontrol**
- Outlet nib `.h:42-59`; tiga tombol/menu `removeThemeButton`, `duplicateThemeButton`, `saveThemeMenuItem` **tak direferensikan di `.m`** — ter-wire langsung di nib.
- Kolom 1 tabel warna dapat `SPColorWellCell` editable dengan action `colorClick:` (124–130). Color panel pakai satu `NSColorPanel` bersama dengan action `colorChanged:` (480–483, 612–615); `colorRow` membawa baris terpilih (478, 611).
- Font-panel routing: `showCustomQueryFontPanel:` set action font manager `changeDefaultFont:` + `fontChangeTarget` ke `SPPrefFontChangeTargetEditor` (286–288).
- Item submenu theme dibangun runtime targeting `loadColorScheme:` (400), `editThemeList:` (410), `setDefaultColors:` (384–385). Identifier pane `SPPreferenceToolbarEditor` (720). `validModesForFontPanel:` hanya size + collection (525).

**Observasi/risiko**
- `SPSaveColorScheme` (41) **tak dipakai** (dead).
- `themePath` nil ditoleransi: `_getAvailableThemes` return `@[]` bila dir hilang (764, 792); `init` pass `error:nil` (74) → lookup gagal senyap.
- Operasi file di `duplicateTheme:`/`removeTheme:`/rename pass `error:nil` (225–226, 251, 580) — gagal hanya ditandai `NSBeep()`/reload.
- Load theme defensif: type-check root dict, array `settings`, kind per-dict, log + skip bentuk tak terduga (919–1006); return NO + warn bila `actuallyLoaded == 0` (1008–1013).
- Format simpan = plist XML 1.0 via `NSPropertyListSerialization` (882–885), atomik (892). Key utama `background/caret/foreground/lineHighlight/selection` (803–815, 931–937) + entri bernama `Comment/String/Keyword/User-defined constant/Number/Variable` (820–877, 939–946). Asimetri: import `tmTheme` diterima (168) tapi parsing mengasumsikan bentuk plist *ini*, bukan XML TextMate — **UNKNOWN** apakah `tmTheme` benar-benar bisa dimuat.
