# Audit: `SPTextView.m`

Audit method-by-method. `Start`/`End` = nomor baris di `Source/Views/TextViews/SPTextView.m`.
Baseline commit `02c910932`.

- File: 3884 baris; header `SPTextView.h` 177 baris.
- Peran: subclass `NSTextView` untuk editor query/teks Sequel Ace — syntax
  highlighting, popup autocompletion SQL, snippet + placeholder mirror,
  autopairing, autoindent, auto-uppercase keyword, drag&drop, context menu.
- Total: 74 method Obj-C + 1 fungsi C (`_alphabeticSort`).

## 1. Konstanta/macro

### `SPTextView.m` atas file
| Baris | Isi |
| --- | --- |
| 31–54 | 19 `#import` + `sequel-ace-Swift.h`, `<SPMySQL/SPMySQL.h>`; termasuk `SPSyntaxParser.h` (47), `RegexKitLite.h` (40) |
| 56/57 | `#pragma mark -` / `attribute definition` |
| 59/60 | `kAPlinked @"Linked"` / `kAPval @"linked"` |
| 61/62 | `kLEXToken @"Quoted"` / `kLEXTokenValue @"isMarked"` |
| 63 | `kSQLkeyword @"s"` |
| 64/65 | `kQuote @"Quote"` / `kQuoteValue @"isQuoted"` |
| 66 | `kValue @"x"` |
| 67/68 | `kBTQuote @"BTQuote"` / `kBTQuoteValue @"isBTQuoted"` |
| 70/71 | `#pragma mark -` / `Constant definitions` |
| 73–75 | `SP_CQ_SEARCH_IN_MYSQL_HELP_MENU_ITEM_TAG 1000`, `SP_CQ_COPY_AS_RTF_MENU_ITEM_TAG 1001`, `SP_CQ_SELECT_CURRENT_QUERY_MENU_ITEM_TAG 1002` |
| 77 | `SP_SYNTAX_HILITE_BIAS 1500` |
| 78 | `SP_MAX_TEXT_SIZE_FOR_SYNTAX_HIGHLIGHTING 2000000` |

### `SPTextView.h` atas
| Baris | Isi |
| --- | --- |
| 31 | `#define SP_TEXT_SIZE_TRIGGER_FOR_PARTLY_PARSING 10000` |
| 32 | `#define SP_TEXT_SIZE_MAX_PASTE_LENGTH 15000` |
| 34–40 | forward `@class` |
| 42–46 | `typedef struct SnippetControlInfo { location, length, task }` |
| 48–52 | `typedef struct MirrorControlInfo { snippet, location, length }` |

## 2. Inventaris ivar/property

### Class-extension & static (`SPTextView.m:84–101`)
| Baris | Item |
| --- | --- |
| 84 | `@interface SPTextView () <NSFontChanging>` |
| 85 | forward decl `NSInteger _alphabeticSort(id, id, void *reverse);` |
| 86–88 | decl method privat: `_setTextSelectionColor:`, `_setTextSelectionColor:onBackgroundColor:`, `_positionCompletionPopup:relativeToTextAtLocation:` |
| 90 | `@property (assign) NSUInteger taskCount;` |
| 96–101 | 6 `static inline` helper: `SPRectTop/Bottom/Left/Right` (96–99), `SPPointDistance` (100), `SPPointOnLine` (101) |

`@synthesize` 105–121: `queryHiliteColor`, `queryEditorBackgroundColor`, `commentColor`,
`quoteColor`, `keywordColor`, `backtickColor`, `numericColor`, `variableColor`,
`otherTextColor`, `queryRange`, `shouldHiliteQuery`, `enableSyntaxHighlighting`,
`completionIsOpen`, `completionWasReinvokedAutomatically`, `syntaxHighlightingApplied`,
`taskCount`, `completionFuzzyMode`.

### Ivar `SPTextView.h:55–109`
| Baris | Grup |
| --- | --- |
| 56–58 | outlet `tableDocumentInstance`, `tablesListInstance`, `customQueryInstance` |
| 60–66 | flag fitur: `autoindentEnabled`, `autocompleteEnabled`, `autopairEnabled`, `autoindentIgnoresEnter`, `autouppercaseKeywordsEnabled`, `delBackwardsWasPressed`, `autohelpEnabled` |
| 67–71 | `lineNumberView`, `textBufferSizeIncreased`, `showMySQLHelpFor` |
| 73–74 | `scrollView`, `completionPopup` |
| 76–79 | `prefs`, `mySQLConnection`, `mySQLmajorVersion` |
| 81–88 | state snippet: `snippetControlArray[20]`, `snippetMirroredControlArray[20]`, `snippetControlCounter`, `snippetControlMax`, `currentSnippetIndex`, `mirroredCounter`, `snippetWasJustInserted`, `isProcessingMirroredSnippets` |
| 90–93 | state completion: `completionIsOpen`, `completionWasReinvokedAutomatically`, `completionWasRefreshed`, `completionParseRangeLocation` |
| 95–106 | warna + `NSRange queryRange`, `shouldHiliteQuery`, `enableSyntaxHighlighting` |
| 108 | `syntaxHighlightingApplied` |

Property `.h:111–126` (9 warna 111–119; sisanya 120–126). Deklarasi method header 128–175.
Struct lokal di `awakeFromNib`: `struct csItem { NSString *p; SEL m; } colorSetup[]` (176–194).

## 3. Semua `#pragma mark` (urut baris)

`56/57 (attribute definition), 70/71 (Constant definitions), 80, 1202/1203 (user actions), 1561/1562 (snippet handler), 2262/2263 (event management), 3279/3280 (context menu), 3458/3459 (delegates), 3573/3574 (drag&drop), 3789 (tanpa judul), 3821/3822 (Private API)`.

## 4. Tabel method (urut baris)

### Blok A — pre-pragma (setelah `@implementation` 103)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 123 | 249 | `awakeFromNib` | Init pref, warna tema, default, line-number view, observer KVO |
| 251 | 255 | `setConnection:withVersion:` | Simpan objek koneksi MySQL + versi mayor server |
| 260 | 334 | `observeValueForKeyPath:ofObject:change:context:` | Handler KVO menerapkan perubahan pref editor |
| 336 | 356 | `doSyntaxHighlightingWithForceWrapper:` | Debounce highlight paksa; mulai task progress untuk teks besar |
| 366 | 677 | `suggestionsForSQLCompletionWith:dictMode:browseMode:withTableName:withDbName:` | Bangun list completion unik terurut dari keyword/kata/tabel/field |
| 679 | 721 | `doAutoCompletion` | Trigger autocompletion tertunda saat mengetik |
| 723 | 728 | `refreshCompletion` | Panggil ulang list completion sekali setelah refresh |
| 730 | 1004 | `doCompletionByUsingSpellChecker:fuzzyMode:autoCompleteMode:` | Entry completion inti; parse prefiks `db.table.field`, bangun/pop popup |
| 1009 | 1063 | `extractTablesFromCurrentQuery` | Ekstrak nama tabel dari FROM/JOIN query untuk prioritas |
| 1065 | 1091 | `getCurrentQueryRange` | Hitung range query aktif di sekitar caret |
| 1093 | 1096 | `getLineNumberForCharacterIndex:` | Kembalikan nomor baris 1-based |
| 1101 | 1117 | `isNextCharMarkedBy:withValue:` | Cek atribut teks char berikut == nilai |
| 1119 | 1151 | `isCaretAdjacentToAlphanumCharWithInsertionOf:` | Deteksi caret diapit alfanumerik untuk lewati autopair |
| 1153 | 1170 | `isCaretAtIndentPositionIgnoreLineStart:` | Cek caret hanya di indentasi awal |
| 1172 | 1200 | `areAdjacentCharsLinked` | Cek char di kedua sisi caret adalah auto-pair linked |

### Blok B — user actions (1203)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1205 | 1239 | `printDocument:` | Print seleksi atau delegasikan ke dokumen |
| 1241 | 1247 | `printOperationDidRun:success:contextInfo:` | Terapkan ulang syntax highlighting setelah print |
| 1252 | 1255 | `showMySQLHelpForCurrentWord:` | Buka MySQL Help untuk seleksi/kata aktif |
| 1261 | 1304 | `wrapSelectionWithPrefix:suffix:` | Bungkus seleksi dengan prefiks/sufiks |
| 1306 | 1318 | `copyAsRTF` | Copy range terpilih ke pasteboard sebagai RTF |
| 1320 | 1324 | `selectCurrentQuery:` | Delegasi "select current query" ke custom query controller |
| 1329 | 1367 | `selectLineNumber:ignoreLeadingNewLines:` | Pilih + scroll ke nomor baris |
| 1369 | 1442 | `shiftSelectionRight` | Indentasi baris terpilih dengan tab/soft-indent |
| 1444 | 1541 | `shiftSelectionLeft` | Unindent baris terpilih |
| 1543 | 1546 | `appendString:` | Append string di akhir text storage |
| 1548 | 1551 | `insertString:atIndex:` | Sisip string di index |
| 1553 | 1559 | `insertString:intoRange:` | Sisip attributed string di range dengan notifikasi undo |

### Blok C — snippet handler (1562)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1567 | 1574 | `endSnippetSession` | Reset counter snippet/mirror + akhiri sesi |
| 1579 | 1731 | `showCompletionListFor:atRange:fuzzySearch:` | Bangun popup untuk token `$SP_ASLIST_*` |
| 1736 | 1804 | `processMirroredSnippets` | Propagasi teks snippet primer ke placeholder mirror |
| 1806 | 1895 | `selectCurrentSnippet` | Pilih placeholder snippet aktif + scroll |
| 1897 | 2181 | `insertAsSnippet:atRange:` | Parse/ekspansi snippet (termasuk shell command, mirror); sisip teks |
| 2186 | 2255 | `checkForCaretInsideSnippet` | Validasi caret tetap di range snippet; else akhiri sesi |
| 2257 | 2260 | `isSnippetMode` | Apakah sesi snippet aktif |

### Blok D — event management (2263)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 2268 | 2291 | `mouseDown:` | Cancel timer autohelp/completion saat interaksi mouse |
| 2293 | 2580 | `keyDown:` | Handler tombol pusat: ESC/F5 completion, TAB snippet/indent, autopair, ⌘font |
| 2582 | 2599 | `paste:` | Paste lalu highlight syntax di bawah batas ukuran |
| 2601 | 2613 | `moveWordRight:` | Gerak word-right melewati bagian dipisah titik |
| 2615 | 2620 | `moveWordLeft:` | Gerak word-left melewati titik |
| 2622 | 2627 | `moveWordLeftAndModifySelection:` | Perluas seleksi word-left melewati titik |
| 2629 | 2641 | `moveWordRightAndModifySelection:` | Perluas seleksi word-right melewati titik |
| 2643 | 2675 | `deleteBackward:` | Hapus auto-pair linked simetris, else backspace biasa |
| 2677 | 2740 | `doCommandBySelector:` | Intersepsi `insertNewline:` untuk autoindent |
| 2742 | 2745 | `setAutoindent:` | Set flag autoindent |
| 2747 | 2750 | `setAutoComplete:` | Set flag autocomplete |
| 2755 | 2758 | `autoindent` | Kembalikan flag autoindent |
| 2764 | 2767 | `setAutoindentIgnoresEnter:` | Set flag ignore-Enter saat autoindent |
| 2772 | 2775 | `autoindentIgnoresEnter` | Kembalikan flag tersebut |
| 2780 | 2783 | `setAutopair:` | Set flag autopair |
| 2788 | 2791 | `autopair` | Kembalikan flag autopair |
| 2796 | 2799 | `setAutohelp:` | Set flag autohelp |
| 2804 | 2807 | `autohelp` | Kembalikan flag autohelp |
| 2812 | 2815 | `setAutouppercaseKeywords:` | Set flag auto-uppercase keyword SQL |
| 2820 | 2823 | `autouppercaseKeywords` | Kembalikan flag tersebut |
| 2829 | 2846 | `autoHelp` | Tampilkan MySQL Help untuk kata/seleksi setelah delay idle |
| 2848 | 2864 | `removeSyntaxHighlighting` | Hapus atribut warna foreground + lex token |
| 2866 | 3114 | `doSyntaxHighlightingWithForce:` | Highlight berbasis lexer (parsial untuk teks besar) + auto-uppercase |
| 3116 | 3158 | `setTabStops` | Bangun ulang tab stop paragraph-style dari lebar pref |
| 3160 | 3212 | `drawViewBackgroundInRect:` | Gambar highlight query + latar placeholder snippet |
| 3214 | 3277 | `roundedBezierPathAroundRange:` | Bangun bezier path rounded di sekitar range teks |

### Blok E — context menu (3280)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3285 | 3402 | `menuForEvent:` | Bangun context menu (MySQL Help, copy RTF, item bundle) |
| 3404 | 3437 | `validateMenuItem:` | Enable/judul item menu berdasar seleksi |
| 3439 | 3456 | `selectionRangeForProposedRange:granularity:` | Redraw highlight query saat seleksi mulai/berakhir |

### Blok F — delegates (3459)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3464 | 3566 | `textStorage:didProcessEditing:range:changeInLength:` | Delegate pasca-edit: highlight, snippet, autopair, jadwal autocomplete |
| 3568 | 3571 | `validModesForFontPanel:` | Batasi font panel ke mode size & collection |

### Blok G — drag&drop (3574)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3580 | 3680 | `performDragOperation:` | Tangani path file/string yang di-drop, sisip isinya |
| 3682 | 3704 | `characterIndexOfPoint:` | Konversi titik view → index karakter via layout manager |
| 3706 | 3768 | `insertFileContentOfFile:` | Deteksi encoding file via `file`, sisip isinya |
| 3770 | 3773 | `changeColor:` | Override no-op mencegah perubahan warna |
| 3775 | 3787 | `changeFont:` | Terapkan font panel + persist ke pref |

### Blok H — tanpa judul (3789)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3791 | 3819 | `dealloc` | Hapus observer KVO, cancel deferred call, tutup popup |

### Blok I — Private API (3822)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 3827 | 3830 | `NSInteger _alphabeticSort(id, id, void *reverse)` **(fungsi C)** | Comparator case-insensitive untuk sort completion |
| 3836 | 3854 | `_setTextSelectionColor:onBackgroundColor:` | Pre-multiply alpha warna seleksi terhadap background |
| 3856 | 3859 | `_setTextSelectionColor:` | Varian convenience pakai background aktif |
| 3865 | 3882 | `_positionCompletionPopup:relativeToTextAtLocation:` | Posisi popup completion di koordinat layar caret |

## 5. Temuan menonjol / UNKNOWN

**A. Snippet + mirror (subsistem terbesar)**
- Kapasitas array C tetap: `snippetControlArray[20]`/`snippetMirroredControlArray[20]` (`.h:81–82`); snippet terbatas `0…18` (`.m:1952–1956, 2061–2065`), mirror dibatasi `COUNT_OF` (2074–2079).
- `insertAsSnippet:atRange:` (1897–2181) method terbesar: ekspansi `${N:hint}`, variabel `$SP_SELECTED_TABLES`/`$SP_SELECTED_TABLE`/`$SP_SELECTED_DATABASE` (1974–2007), eksekusi `$( … )` via `SPBundleCommandRunner runBashCommand:` (2017–2033), penelusuran mirror (2051–2118), unescape `\${…}`/`\$N` (2126–2143), dibungkus `@try/@catch` (1919, 2176–2180).
- Propagasi mirror `processMirroredSnippets` (1736–1804) membaca `completionParseRangeLocation` yang di-set di method lain (998, dipakai 1775) — coupling lintas-method.
- Guard sesi `snippetWasJustInserted` (1941; cek delegate 3497) dan `isProcessingMirroredSnippets` (1740, 1797, 3497).

**B. Entry point autocompletion**
- `doAutoCompletion` (679) timer-driven, dijadwalkan dari edit delegate (3471–3473) dan `keyDown:`; dibatalkan di 685–687, 927–930, 2310–2313, 3472.
- `refreshCompletion` (723) digerbangi `completionWasRefreshed` (725–726, reset 2335).
- `doCompletionByUsingSpellChecker:fuzzyMode:autoCompleteMode:` (730–1004) membangun `SPNarrowDownCompletion` dengan 17 argumen (979–996); bail untuk parse range > 100000 char (773, 862).
- `showCompletionListFor:atRange:fuzzySearch:` (1579) berisi **kode legacy yang dikomentari + `// TODO HansJB`** (1598–1623).

**C. Integrasi lexer — eksternal/UNKNOWN**
- `doSyntaxHighlightingWithForce:` (2866–3114) memakai global/fungsi **tak dideklarasikan di file ini**: `yyuoffset`, `yyuleng`, `yylex()`, `yy_switch_to_buffer`, `yy_scan_string` (3007–3008, 3015, 3052) — dari `SPSyntaxParser.h`/output lex; **UNKNOWN** dari file ini.
- Variabel `NSRange visibleRange;` (2903) hanya di-assign bila `textStore.editedMask != 1` (2920–2928); digerbangi `if(!visibleRange.length)` di 2930 — cabang `editedMask == 1` hanya log "returning" (2927) **tanpa return**. **UNKNOWN/kontrol alur mencurigakan.**
- Uppercase keyword ditunda satu siklus token dengan atribut `kSQLkeyword` (3060–3101); digerbangi `textBufferSizeIncreased && !delBackwardsWasPressed` (3066–3067).

**D. Nama method berisiko — UNKNOWN**
- `_setTextSelectionColor:` (3836, 3856) dan `_positionCompletionPopup:…` (3865) memakai prefiks underscore gaya Apple; konformance `NSFontChanging` dijelaskan (82–83) tapi tak ada dokumentasi apakah `_setTextSelectionColor` bentrok dengan method privat AppKit. Flag **UNKNOWN/risiko**.
- `_alphabeticSort` (3827–3830) mengabaikan parameter `reverse` sepenuhnya.

**E. Lain**
- `#pragma mark -` (3789) tanpa judul; `changeColor:`/`changeFont:`/`characterIndexOfPoint:`/`insertFileContentOfFile:` ditempatkan di bawah pragma `drag&drop` yang menyesatkan.
- Magic number: `screenPosition.x -= 26` (3879); bezier `kappa 0.5522847498`, `radius 6`, `horzInset -3`, `vertInset 0.3f` (3217–3220); `connectDisconnectedPartsWithLine = NO` (3221) tampak inert — **UNKNOWN** pemakaiannya.
- `awakeFromNib` punya loop color-map dengan `goto retry` fallback ke default vendor untuk data pref invalid (200–220) + `#pragma clang diagnostic ignored "-Warc-performSelector-leaks"` (207–210).
- Batas keras: `SP_TEXT_SIZE_MAX_PASTE_LENGTH` 15000 (2588), `SP_MAX_TEXT_SIZE_FOR_SYNTAX_HIGHLIGHTING` 2000000 (2887, 3175), `SP_TEXT_SIZE_TRIGGER_FOR_PARTLY_PARSING` 10000 (347, 2899).
- Selector helper dipakai tapi tak didefinisikan di sini: `getRangeForCurrentWord` (766, 2357, 3414), `delegateDocument` (476, 1211), `delegateTablesList` (1624, 1633, 1639), `lineRangesForRange:`, `backtickQuotedString` — didefinisikan di kategori lain; **UNKNOWN** lokasinya dari file ini.
