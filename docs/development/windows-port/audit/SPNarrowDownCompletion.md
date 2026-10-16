# Audit: `SPNarrowDownCompletion.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Model/SPNarrowDownCompletion.m`. Baseline commit `02c910932`.

- File: 1165 baris; header 110 baris.
- Peran: popup autocompletion inkremental `NSWindow` borderless (turunan TextMate
  TMDIncrementalPopUp) yang memfilter saran DB/schema/keyword dan menyisipkan
  hasil ke `SPTextView`.
- Helper `SACompletionWindowLayout.swift` (55 baris) menyediakan sizing window
  (`windowSizeForTableContentSize:maximumWidth:scrollerStyle:`, dipakai `.m:292, 745`).
- Total: 21 definisi method.

## 1. Inventaris ivar/property

Semua state ivar (tanpa `@property`). Basis `.h:36-80` dan `.m:57-59`.

| Ivar | Tipe | `.h` |
| --- | --- | --- |
| `suggestions` | `NSArray *` | 38 |
| `mutablePrefix` / `staticPrefix` | `NSMutableString *` / `NSString *` | 39–40 |
| `currentDb` | `NSString *` | 41 |
| `filtered` | `NSArray *` | 42 |
| `theTableView` | `NSTableView *` | 43 |
| `caretPos` | `NSPoint` | 44 |
| `isAbove`/`closeMe`/`caseSensitive`/`dictMode`/`triggerMode`/`fuzzyMode`/`cursorMovedLeft`/`commaInsertionMode`/`autoCompletionMode`/`oneColumnMode`/`isQueryingDatabaseStructure`/`autocompletePlaceholderWasInserted` | BOOL | 45–56 |
| `originalFilterString` | `NSMutableString *` | 57 |
| `backtickMode` | `NSInteger` | 58 |
| `tableFont` | `NSFont *` | 59 |
| `theCharRange`/`theParseRange` | `NSRange` | 60–61 |
| `theAliasName` | `NSString *` | 62 |
| `stateTimer` | `NSTimer *` | 64 |
| `syncArrowImages`/`currentSyncImage` | `NSArray *`/`NSUInteger` | 65–66 |
| `timeCounter` | `NSUInteger` | 68 |
| `theView` | `SPTextView *` | 70 |
| `maxWindowWidth`/`tableContentWidth` | `CGFloat` | 72–73 |
| `spaceCounter` | `NSInteger` | 74 |
| `textualInputCharacters` | `NSMutableCharacterSet *` | 76 |
| `databaseStructureRetrieval` | `SPDatabaseStructure *` | 78 |
| `prefs` | `NSUserDefaults *` | 79 |
| `startedIntercepting` | BOOL (class extension) | `.m:58` |

Konstanta lokal: `SPAutoCompletePlaceholderName = @"Placeholder"` (`.m:48`);
`SPAutoCompletePlaceholderVal = @"placholder"` (`.m:49`, **salah eja**).
Eksternal: `SPNarrowDownCompletionMaxRows = 15` (`SPConstants.m:59`, dipakai `.m:742`).

## 2. Semua `#pragma mark`

`.m:45/46 (attribute definition)` — satu-satunya. Implementasi dibagi oleh komentar banner `// ===`.

## 3. Tabel method (21 definisi, urut baris)

### `@implementation NSTableView (MovingSelectedRow)` (70–108)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 72 | 106 | `SP_NarrowDownCompletion_canHandleEvent:` | Intersepsi tombol arrow/page/home/end; pindah seleksi; sisip placeholder |

### Setup/teardown (111–408)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 114 | 145 | `init` | Bangun window, default ivar, font tabel, image sync-arrow |
| 147 | 157 | `dealloc` | Cancel request tertunda, invalidate timer, lepas delegate tabel |
| 159 | 170 | `close` | Invalidate timer, set closeMe, beri tahu view completion ditutup |
| 172 | 196 | `updateSyncArrowStatus` | Tick timer: siklus frame arrow, poll state query DB-structure |
| 198 | 207 | `reInvokeCompletion` | Teardown + minta view refresh completion setelah delay |
| 209 | 316 | `initWithItems:alreadyTyped:staticPrefix:additionalWordCharacters:caseSensitive:charRange:parseRange:inView:dictMode:tabTriggerMode:fuzzySearch:backtickMode:selectedDb:caretMovedLeft:autoComplete:oneColumn:alias:withDBStructureRetriever:` | Init designated: terapkan semua flag mode, ukur kolom/window, arm timer |
| 318 | 341 | `setCaretPos:` | Simpan caret, clamp horizontal, flip popup atas/bawah caret |
| 343 | 408 | `setupInterface` | Bangun scroll view, tabel 5 kolom, chrome/appearance window |

### TableView DataSource/Delegate (410–620)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 413 | 416 | `numberOfRowsInTableView:` | Jumlah `filtered` suggestion |
| 418 | 491 | `tableView:toolTipForCell:rect:tableColumn:row:mouseLocation:` | Tooltip hover per kolom |
| 493 | 500 | `tableView:selectionIndexesForProposedSelection:` | Cegah pilih baris dummy "fetching" row 0 |
| 502 | 573 | `tableView:dataCellForTableColumn:row:` | Sediakan cell pop-up/token untuk kolom list/type/path |
| 575 | 620 | `tableView:objectValueForTableColumn:row:` | Nilai cell image/name/list/type/path |

### Filtering (622–766)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 625 | 637 | `checkSpaceForAllowedCharacter` | Izinkan spasi sebagai input bila ada saran memuatnya |
| 642 | 766 | `filter` | Filter by prefix/fuzzy/spellcheck; resize window; reload tabel |

### Convenience (768–787)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 771 | 774 | `filterString` | Konkatenasi `staticPrefix` + `mutablePrefix` |
| 776 | 787 | `rectOfMainScreen` | Frame screen dengan origin (0,0) |

### Run popup-menu (789–940)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 792 | 802 | `orderFront:` | Jalankan filter, tampilkan window, jadwalkan `watchUserEvents` |
| 804 | 940 | `watchUserEvents` | Loop intersepsi event: routing key/mouse, filter, insertion |

### Action (942–1163)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 945 | 986 | `insertAutocompletePlaceholder` | Sisip sufiks saran terpilih bertag atribut abu-abu |
| 988 | 1034 | `removeAutocompletionPlaceholderUsingFastMethod:` | Hapus placeholder by range (cepat) atau scan atribut |
| 1036 | 1088 | `insert_text:` | Ganti working char range; append snippet argumen function/proc |
| 1090 | 1153 | `completeAndInsertSnippet` | Commit item terpilih; tangani schema path, backtick, comma mode |
| 1155 | 1159 | `adjustWorkingRangeByDelta:` | Geser lokasi char/parse range sebesar delta |
| 1161 | 1163 | `hasStartedIntercepting` | Apakah intersepsi event sudah mulai |

## 4. Temuan menonjol / UNKNOWN

**Event loop (`watchUserEvents`, 804–940)**
- Pakai `nextEventMatchingMask:untilDate:[NSDate distantFuture] inMode:NSDefaultRunLoopMode dequeue:YES` dalam loop `while(!closeMe)` (809–813) — loop intersepsi gaya modal, bukan callback run-loop.
- Set `startedIntercepting = YES` hanya setelah event pertama diambil (814); `if(!event) continue;` (816) bisa spin bila `nextEventMatchingMask:` return nil.
- Presedensi routing: navigasi tabel (`SP_NarrowDownCompletion_canHandleEvent:` 819) → cabang key-down (822–914) → mouse (915–928) → fallthrough `sendEvent:` (929–931).
- ESC (keyCode 53): dengan Control toggle `fuzzyMode`, else tutup (854–869). Commit: Return/Enter/→/Tab (saat `!triggerMode`) → `completeAndInsertSnippet` (870–872). Backspace hapus prefiks + re-filter (873–890).
- Tail: hapus placeholder, beri tahu view, `close`, lalu `usleep(70)` meredam ESC cepat (934–939) — **UNKNOWN** implikasi platform (blok main thread).

**Filtering (`filter`, 642–766)**
- Prefiks kosong: dict mode tak ada hasil, mode lain lewatkan semua (709–711).
- Dict mode: `NSPredicate` `match BEGINSWITH` (+ completion spellchecker) (647–659).
- Fuzzy non-dict: regex greedy `.*?` antar karakter; `.`→`SPUniqueSchemaDelimiter`, escape bracket; match `display` ATAU `path` (662–690); backtick pada filter string dilewati (672).
- Non-fuzzy: predicate `BEGINSWITH` case-(in)sensitive dengan fallback `match == NULL → display` (691–700).
- `@catch(id ae)` luas di sekitar filtering regexp: log + set `closeMe` (702–706).
- No-match: autoComplete tutup; else re-invoke spellchecker bila sufiks `.`; else sisip baris "No item found" (713–730).
- Auto-complete exact-match shortcut tutup bila satu hasil == filter (732–735).
- Baris dummy "fetching" disisip di index 0 saat DB structure querying (737–738).
- Geometri window: cap `SPNarrowDownCompletionMaxRows` (15), flip atas/bawah via caret (740–761).

**Snippet insertion (`insert_text:`, 1036–1088)**
- Re-anchor `theCharRange` ke `originalFilterString.length` (1039); clamp ke panjang string (1042–1044).
- `backtickMode == 100` special-case backtick-quoted, perluas seleksi menelan backtick sekitar (1053–1068).
- Completion function/procedure: bila `SPCustomQueryFunctionCompletionInsertsArguments` diset dan item ber-image `func`/`proc`, append snippet `(...)` via `SPQueryController argumentSnippetForFunction:` + `insertAsSnippet:atRange:`; argumen kosong `()` menggeser caret kiri (1082–1087).

**Placeholder (945–1034)**
- Insertion di-skip tanpa baris terpilih atau di fuzzy mode (947).
- Hanya sufiks di luar `originalFilterString` disisipkan, lowercased (964–972).
- Placeholder bertag atribut `SPAutoCompletePlaceholderName` = `@"Placeholder"` dan value `SPAutoCompletePlaceholderVal` = `@"placholder"` (**typo**, 49), plus foreground alpha 30% (978–982).
- Manipulasi `textStorage` langsung agar tak mengotori UndoManager (970–972, 1000–1001).
- Fast removal pilih `theCharRange` (offset 1 di backtick mode) dan replace `originalFilterString` (992–1001).
- Slow removal scan `textStorage` untuk atribut placeholder dalam blok `beginEditing`/`endEditing`, pakai `longestEffectiveRange` dua-tahap (1002–1031). Logika paling rumit; **UNKNOWN** apakah `scanPosition = attributeResultRange.location` bisa loop selamanya pada layout atribut patologis (1028).

**Interplay timer/DB-structure**
- `stateTimer` dijadwalkan 0.07 s saat retriever struktur sudah querying (310–312); tick siklus image sync-arrow + `timeCounter` (172–196). Setelah 20 tick, bila tak querying lagi, invalidate timer + `reInvokeCompletion` di main thread, set `closeMe` (180–192).

**Completion commit (`completeAndInsertSnippet`, 1090–1153)**
- Shift+click item `isRef` menyisip schema path penuh dengan prefiks DB aktif dilepas (1110–1124).
- Item `isRef` hormati `SPCustomQueryEditorCompleteWithBackticks` + set `backtickMode = 100` (1127–1135).
- Insert dengan Control append `", "` + arm `commaInsertionMode` agar list tetap terbuka (1142–1149).

**UNKNOWN**
- Banner komentar `.m:624` memuat karakter mojibake; teks maksud tak pulih dari source (kemungkinan "space").
- `.m:939` `usleep(70)` rasional hanya didokumentasikan longgar.
- `.m:182`/`308` makna ganda `isQueryingDatabaseStructure` (ivar vs panggilan retriever live) — bergantung timer; tak ada sinkronisasi.
- `.m:618` raise `NSInternalInconsistencyException` untuk identifier kolom invalid — perilaku runtime UNKNOWN (dianggap unreachable).
