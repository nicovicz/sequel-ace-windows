# Audit: Parsing + ThirdParty cluster

Audit method-by-method klaster Parsing + ThirdParty. `Start`/`End` = nomor baris di
file masing-masing. Baseline commit `02c910932`.

File: `Parsing/SPCSVParser.m` (649), `Parsing/SPJSONFormatter.m` (368),
`MGTemplateEngine/MGTemplateEngine.m` (619), `MGTemplateEngine/MGTemplateStandardMarkers.m` (631),
`Views/NoodleLineNumberView.m` (566), `Views/YRKSpinningProgressIndicator.m` (375).

---

## 1. `SPCSVParser.m` (649)

Parser CSV capable streaming (bukan subclass NSMutableString) — field/line terminator multi-karakter, quote + escape string, NSNull replacement, lazy trim.

### Ivar/konstanta
Semua ivar di `.h`: `csvString` (56); posisi/counter `trimPosition`/`parserPosition`/`totalLengthParsed`/`csvStringLength`/`fieldCount` (58–62); token `nullReplacementString`/`fieldEndString`/`lineEndString`/`fieldQuoteString`/`escapeString` (64–68); varian ter-escape (69–72); panjang (73–76); `skipCharacterSet` (77); `NSScanner *csvScanner` (78, **tak pernah dirujuk di `.m`**); boolean `escapeStringIsFieldQuoteString`/`useStrictEscapeMatching` (80–81).
`#define SPCSVPARSER_TRIM_ENACT_LENGTH 250000` (`.h:52`, dipakai `.m:533`). Default token di `_initialiseCSVParserDefaults` (501–510): `,`/`\n`/`"`/`\`, escaped `\\,` `\\\n` `\\"` `\\\\`.

### `#pragma mark`
`40/41 (Retrieving data from the CSV string), 323/324 (Adding new data to the string), 346/347 (Basic information), 382/383 (Setting the terminator, quote, escape and null character replacement strings), 485/486 (Init and internal update methods), 623`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 47–74 | | `array` / `getRowAsArray` | Reset + ambil array 2D / baris berikut |
| 87 | 321 | `getRowAsArrayAndTrimString:stringIsComplete:` | Parser baris inti: quote, escape, null, padding |
| 329–344 | | `appendString:` / `setString:` | Streaming append + set ulang |
| 352–380 | | `length` / `string` / `parserPosition` / `totalLengthParsed` | Info |
| 390–483 | | `setFieldTerminatorString:convertDisplayStrings:` / `setLineTerminatorString:` / `setFieldQuoteString:` / `setEscapeString:` / `setNullReplacementString:` / `setEscapeStringsAreMatchedStrictly:` / `lineTerminatorString` | Setting token |
| 491–523 | | `_initialiseCSVParserDefaults` | Default ivar + skip set |
| 529–544 | | `_updateState` | Lazy trim prefix terpakai lewat threshold |
| 551–586 | | `_convertDisplayString:` / `_updateSkipCharacterSet` | Konversi display string + skip set |
| 593–631 | | `_getDistanceToString:` / `_moveParserPastSkippableCharacters` / `init` | Helper + init |
| 633–647 | | `initWithString:` / `initWithContentsOfFile:encoding:error:` | Init |

### Temuan
- **Aturan escape**: di dalam field ter-quote, status escape diputuskan dengan menghitung escape string yang match **mundur** dari quote (127–152); ganjil ⇒ escaped (135). Fallback non-strict mengaktifkan doubling Excel `""` (139, 145–152). Cek terminator field/line mirror ini (217–244).
- **Identitas quote/escape**: saat escape == quote, penanganan doubled-quote + jalur append alternatif hindari double-unescape (131, 160–165, 271).
- **NULL**: literal `\N` selalu jadi NSNull; `nullReplacementString` hanya match sel unquoted (257–258).
- **Row padding**: baris pertama set `fieldCount`; baris pendek di-pad `[SPNotLoaded notLoaded]` (313–317).
- **Streaming**: baris tak lengkap di-rollback via `startingParserPosition` saat `!stringComplete` (103, 282–285).
- ⚠️ **UNKNOWN/dead**: ivar `NSScanner *csvScanner` (`.h:78`) tanpa referensi di `.m` — vestigial.
- Baris kosong di-skip dengan rekursi bukan loop (290–302) — risiko kedalaman pada input patologis; **UNKNOWN** apakah disengaja.

---

## 2. `SPJSONFormatter.m` (368)

Pretty-printer/compactor JSON lenient di atas tokenizer byte buatan tangan; eksplisit **bukan** validator.

### Ivar/konstanta
Tanpa ivar (utility static). Tipe/konstanta di `.h`: `enum SPJSONToken` (33–44); `enum SPJSONContext` (46–49); `struct SPJSONTokenizerState {str,len,pos,ctxt}` (51–56); `struct SPJSONTokenInfo` (58–62). Forward decl `static char GetNextANSIChar(...)` (33, definisi 243). **Tanpa `#pragma mark`.**

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 37 | 160 | `+stringByFormattingString:useSoftIndent:indentWidth:` | Pretty-print JSON dengan indent terkonfigurasi |
| 162–165 | | `+stringByFormattingString:` | Format default (hard tab, width 1) |
| 167 | 230 | `+stringByUnformattingString:` | Compact JSON; spasi tunggal setelah `:`/`,` |
| 243–368 | | `GetNextANSIChar` / `SPJSONTokenizerInit` / `SPJSONTokenizerGetNextToken` (C) | Tokenizer |

### Temuan
- **Aturan tokenizer**: `GetNextANSIChar` skip byte high-bit (hanya byte ASCII control-plane ditokenisasi); UTF-8 multibyte = konten opaque (243–253). Header memperingatkan "DO NOT try to build a parser/syntax validator".
- **Dua konteks**: di `JSON_STRING_CONTEXT` hanya `"` dan `\` penting; `\` buta skip byte berikutnya (274–303). `JSON_TOK_STRINGDATA` membawa byte string mentah tanpa quote (278–284).
- **Token OTHER**: rentetan byte non-kontrol (whitespace/angka/literal) → `JSON_TOK_OTHER` (305–317, 352–358).
- **Formatting**: newline sebelum `}`/`]` kecuali didahului `{`/`[`/`,` (65–68); newline setelah `,`/`{`/`[` kecuali next empty closer (143–150); spasi setelah `:` (122). `indentWidth` clamp 1…32 (73–75).
- **Unformatting** tambah satu spasi setelah `:` dan `,` "to match MySQL" (207, 211) — **UNKNOWN** kenapa sengaja beda dari compact sejati.
- **Memori**: token di-wrap `initWithBytesNoCopy:…freeWhenDone:NO` menunjuk ke `stateInfo.str` (132, 217); tokenizer memegang `UTF8String` non-retained (`.h:64-67`) — lifetime formatter+tokenizer terkoppel.

---

## 3. `MGTemplateEngine.m` (619)

Engine templating yang scan template, resolve variabel/block/filter, render output. Asal: Matt Gemmell, "Copyright 2008 Instinctive Code" (.m:4-5). Version "1.0.1" 9 Feb 2014 (57–58).

### Ivar/konstanta
Extension `@interface MGTemplateEngine (PrivateMethods)` (30–38). Ivar: `_openBlocksStack` (42), `_globals` (43), `_outputDisabledCount` (44), `_templateLength` (45), `_filters` (46), `_markers` (47), `_templateVariables` (48), `_literal` (49). Property publik di `.h:48-58`. `#define` default (13–28): `DEFAULT_MARKER_START "{%"`, `DEFAULT_MARKER_END "%}"`, `DEFAULT_EXPRESSION_START "{{"`, `DEFAULT_EXPRESSION_END "}}"`, `DEFAULT_FILTER_START "|"`, `DEFAULT_LITERAL_START "literal"`, `DEFAULT_LITERAL_END "/literal"`.

### `#pragma mark`
`52 (Creation and destruction), 91 (Managing persistent values.), 108 (Configuration and extensibility.), 144 (Delegate), 191 (Utilities.), 353 (Processing templates.), 617 (Properties, kosong)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 54–89 | | `+engineVersion` / `+templateEngine` / `-init` | Creation |
| 93–106 | | `setObject:forKey:` / `addEntriesFromDictionary:` / `objectForKey:` | Globals |
| 110–142 | | `loadMarker:` / `loadFilter:` | Register handler |
| 146–189 | | `reportError:code:continuing:` / `reportBlockBoundaryStarted:` / `reportTemplateProcessingFinished` | Delegate |
| 193–351 | | `valueForVariable:parent:parentKey:` / `setValue:forVariable:forceCurrentStackFrame:` / `resolveVariable:` / `templateVariables` | Resolusi variabel |
| 355 | 604 | `processTemplate:withVariables:` | Loop render utama |
| 606–615 | | `processTemplateInFileAtPath:withVariables:` | Load file + process |

### Temuan
- **Wiring init**: marker/filter standar dibuat + di-load di `init` (84–85), tapi `matcher` **tidak** diset di sini — `processTemplate:` error code 7 bila tak dikonfigurasi (383–386). **UNKNOWN** siapa set matcher di repo ini.
- **Resolusi variabel**: coba cepat `valueForKeyPath:` di bawah `@try` (230–235); gagal → walk segment iteratif, indeks array numerik via scan digit `NSScanner` (258–300). Fallback `_templateVariables` → `_globals` → frame open-block (208–223).
- **Loop render**: append teks pra-marker saat output aktif (397–400); mode literal via `literalStartMarker`/`literalEndMarker` (410–427); block start/end jaga `_openBlocksStack` pakai key `BLOCK_*` (485–541); sisa open block = error code 6 (582–590).
- **Error code**: 0 simultaneous block start/end, 1 bad block end, 2 invalid marker, 3 invalid variable, 4 missing end markers, 5 invalid resume range, 6 unclosed block, 7 no matcher.

---

## 4. `MGTemplateStandardMarkers.m` (631)

Implementasi `MGTemplateMarker` untuk tag bawaan `for/section/if/now/comment/load/cycle/set`. Asal: Matt Gemmell, 13/05/2008.

### Ivar/konstanta
Ivar (73–80): `__weak engine`, `forStack`, `sectionStack`, `ifStack`, `commentStack`, `cycles`. `#define` per-tag (10–68).

### `#pragma mark`
**Tidak ada** (hanya komentar separator `//===`).

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 82–121 | | `initWithTemplateEngine:` / `markers` / `endMarkersForMarker:` | Setup + deklarasi marker |
| 123 | 564 | `markerEncountered:withArguments:inRange:blockStarted:blockEnded:outputEnabled:nextRange:currentBlockInfo:newVariables:` | Tangani semua marker standar |
| 566–629 | | `currentBlock:matchesTopOfStack:` / `argIsTrue:` / `argIsNumeric:integerValue:checkVariables:` / `engineFinishedProcessingTemplate` | Helper |

### Temuan
- **Dispatch marker**: satu chain `if/else` raksasa di `markerEncountered:` (128–561).
- **`for`**: range ("to") vs enumerator ("in") dari arg[1] (131–134); `reversed` opsional butuh tepat 4 arg (136–138). Kondisi invalid tetap mulai block tapi disable output via `FOR_STACK_DISABLED_OUTPUT` (215–225). `FOR_END` putuskan loop-back pakai enumerator atau banding index + rewind `nextRange` dari frame stack (228–307); loop nested dapat `parentLoop` (205–212).
- **Operator `if`**: `and/&&`, `or/||`, `=/==`, `!=/<>`, `>`, `<`, `>=`, `<=`, `%`, `equalsstring/notequalsstring`, `hasprefix/nothasprefix` (358–412). Arg 1–3; prefix `not` untuk 2 arg (335–339). Operand non-numerik koersi 0/1 by resolvability (350–355). `else` flip output sekali (442–451); `/if` selalu set `blockEnded` walau stack tak match (454–469).
- **`now`** return `[NSDate date]` (471–472). **`comment`** mulai hanya bila argless + disable output (474–491).
- ⚠️ **`load`** pakai `NSClassFromString` + instansiasi kelas apa pun yang conform `MGTemplateFilter`/`MGTemplateMarker` (511–527) — permukaan reflection/deserialization; **UNKNOWN** apakah input template pernah tak-terpercaya.
- **`cycle`** di-key `NSStringFromRange(markerRange)` sehingga evaluasi ulang di posisi sumber sama memutar arg (529–551). **`set`** hanya bila tepat 2 arg + output aktif (552–560).
- **UNKNOWN**: `currentBlock:matchesTopOfStack:` banding hanya range start-marker (570–572) — nested range identik tak terbedakan (dianggap mustahil).

---

## 5. `NoodleLineNumberView.m` (566)

Subclass `NSRulerView` menggambar gutter nomor baris editor + seleksi multi-baris klik/drag. Asal: Paul Kim, "Copyright (c) 2008 Noodlesoft, LLC"; fork ini "removes marker functionality and adds selection by clicking on the ruler" (29–31).

### Ivar/konstanta
Ivar di `.h:33-60`: `lineIndices`, `currentNumberOfLines`, font/warna, `maxHeightOfGlyph`/`maxWidthOfGlyph`, `maxWidthOfGlyph1…8`, `currentRuleThickness`, `textAttributes`, `dragSelectionStartLine`, `numberClass`, layout objects. Property `alternateTextColor` (`.h:64`) + `backgroundColor` (`.h:65`) di-synthesize (67–68). Extension (56–63).
Key coding `NOODLE_*` (39–42); `DEFAULT_THICKNESS 22.0f`/`RULER_MARGIN 5.0f`/`RULER_MARGIN2` (46–48); `typedef NSRange (*RangeOfLineIMP)(...)` (50, **UNKNOWN/ tampak tak dipakai**).

### `#pragma mark`
`37 (NSCoding methods), 44, 54, 110, 181, 418, 462/463 (PrivateAPI)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 70–108 | | `initWithScrollView:` / `awakeFromNib` / `dealloc` | Setup |
| 112–179 | | `setFont:` / `font` / `setTextColor:` / `textColor` / `setClientView:` | Font/warna/client |
| 183 | 196 | `textDidChange:` | Invalidasi indeks baris pada perubahan panjang teks |
| 198 | 245 | `lineNumberForLocation:` | Map y → nomor baris 1-based |
| 247–273 | | `lineNumberForCharacterIndex:` | Binary-search indeks baris |
| 275 | 354 | `drawHashMarksAndLabelsInRect:` | Gambar nomor baris terlihat flush-kanan |
| 356–416 | | `mouseDown:` / `mouseDragged:` | Seleksi baris per-klik + extend drag + autoscroll |
| 420–460 | | `initWithCoder:` / `encodeWithCoder:` | Coding |
| 465–564 | | `lineIndices` / `calculateLines` / `setRuleThicknessNumber:` / `updateGutterThicknessConstants` | Indeks + lebar gutter |

### Temuan
- **Gutter logic**: `calculateLines` walk string dengan `getLineStart:end:contentsEnd:forRange:` (498–503) + append indeks akhir bila teks berakhir newline (506–509). Nomor baris **disabled untuk teks > 3.000.000 char** (488–492).
- **Lebar adaptif**: lebar gutter dipilih dari `maxWidthOfGlyph2…8` by threshold jumlah baris 100/1k/10k/100k/1M/10M/100M else 100 (512–527); resize di-defer via `performSelector:afterDelay:0.0` (536–538). `setRuleThicknessNumber:` dokumentasikan kenapa NSInvocation ditinggalkan (545–551).
- **Drawing**: cari range karakter terlihat, tiap baris ambil array rect + gambar nomor, center vertikal, rata kanan berdasar jumlah digit (300–345). Kompensasi `textContainerInset` (291; path klik 216–217, issue #2236).
- **Seleksi**: `mouseDown:` catat `dragSelectionStartLine` + pilih baris penuh (356–381); `mouseDragged:` normalisasi urutan start/end, seleksi, lalu `autoscroll:` (383–416). Pakai `safeObjectAtIndex:` (232, 262, 318, 373, 406).
- ⚠️ **UNKNOWN/dead**: ivar `numberClass` di-assign `[NSNumber class]` (90) tapi tak pernah dibaca; `maxWidthOfGlyph1` dihitung (556) tapi pemilihan thickness mulai `maxWidthOfGlyph2` (513); typedef `RangeOfLineIMP` (50) tak dipakai.
- **Caching**: `textDidChange:` hanya invalidasi saat `editedMask != 1` (edit atribut saja mempertahankan cache) (190–192).

---

## 6. `YRKSpinningProgressIndicator.m` (375)

`NSView` spinner kustom: mode indeterminate (sirip berputar) + determinate (arc), animasi timer atau thread. Asal: "Original drawing code by Kelan Champagne; forked by Rowan Beentje…" (4–7).

### Ivar/konstanta
Semua ivar di `.h`: `_position`/`_numFins` (34–35); `_isAnimating`/`_animationTimer`/`_animationThread` (37–39); `_foreColor`/`_backColor`/`_drawBackground`/`_shadow` (41–44); `_fadeOutAnimationTimer`/`_isFadingOut` (46–47, **vestigial**); determinate `_isIndeterminate`/`_currentValue`/`_maxValue` (50–52); `_usesThreadedAnimation` (54). Extension (34–41). Tanpa property (accessor ditulis tangan). `@interface` derive `NSView` (`.h:32`).

### `#pragma mark`
`153/154 (Subclass), 251 (Not Implemented), 260/261 (Accessors)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 45–78 | | `initWithFrame:` / `dealloc` / `viewDidMoveToWindow` | Setup + lifecycle |
| 80 | 151 | `drawRect:` | Gambar sirip (indeterminate) atau circle+arc (determinate) |
| 156–249 | | `updateFrame:` / `animateInBackgroundThread` / `startAnimation:` / `stopAnimation:` / `actuallyStartAnimation` / `actuallyStopAnimation` | Animasi |
| 253–258 | | `setStyle:` | Tolak style non-spinning |
| 263–373 | | accessor (foreColor/backColor/drawBackground/shadow/isIndeterminate/doubleValue/maxValue/usesThreadedAnimation) | Accessor |

### Temuan
- **Drawing (indeterminate)**: 12 sirip (`_numFins = 12`, 50) digambar dengan translate CTM ke center + rotasi per sirip, alpha menurun 1/numFins per langkah (104–132); state statis alpha 0.2 (123–124).
- **Determinate**: outline circle + arc terisi sweep `90 → 90−360·(_currentValue/_maxValue)`, searah jarum jam, wedge ke center (134–148).
- **Mode animasi**: threaded mode memutar `NSThread` `animateInBackgroundThread` pada delay tergantung ukuran `38000 + 2000*(height/32)` µs (179) hingga thread di-cancel (182–189); timer mode pakai timer 0.05 s repeating di common/default/event-tracking modes (220–228).
- **Lifecycle safety**: `actuallyStartAnimation` selalu `actuallyStopAnimation` dulu (209–210); `viewDidMoveToWindow` stop animasi saat window nil untuk putus retain timer→self→timer (71–77).
- **Accessor side-effect**: `setDoubleValue:` paksa indeterminate→determinate (334–337); `setIndeterminate:NO` stop animasi (323); `setUsesThreadedAnimation:` restart animasi (362–366); `setStyle:` assert NO untuk style non-spinning (255–257).
- ⚠️ **UNKNOWN/dead**: `_fadeOutAnimationTimer` (`.h:46`) tak pernah dirujuk di `.m`; `_isFadingOut` set NO di `initWithFrame:` (52) + tak pernah dibaca; `poolFlushCounter` (180, 185–188) tak dipakai untuk aksi apa pun — sisa autorelease-pool flush yang ditinggalkan.

## Ringkasan
- Dua file Parsing berbagi filosofi "tanpa validasi syntax penuh": CSV parse struktural tapi toleran quote buruk; JSON eksplisit "much too lenient" untuk validasi.
- Keempat file ThirdParty menyatakan author/tahun upstream di komentar kepala; NoodleLineNumberView + YRK juga dokumentasikan modifikasi fork Sequel.
