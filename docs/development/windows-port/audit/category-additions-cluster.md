# Audit: CategoryAdditions cluster

Audit method-by-method klaster CategoryAdditions. `Start`/`End` = nomor baris di
file masing-masing. Baseline commit `02c910932`. "End" = baris `}` penutup body.

**Catatan tipe**: hanya 4 dari 5 target adalah kategori ObjC; `SPDataStorage.m` adalah
`@implementation` kelas biasa + class extension (`SPDataStorage.m:39–44`).

---

## 1. `SPDataAdditions.m` (633)

Kategori `NSData`: hashing (SHA-1), AES-128 password/key encrypt-decrypt, zlib compress/decompress, konversi hex, enumerasi baris.

### `@implementation`
`@implementation NSData (SPDataAdditions)` (51–623) — 13 method. Helper C `hexchar2nibble` (352) + `decodeValidHexSequence` (369) dideklarasikan di dalam blok; `LimitUInt32` (627) di luar blok setelah `@end` (623). Header `SPDataAdditions.h:38–59`. Forward decl helper `.m:47`.

### `#pragma mark`
`49, 625` (di luar kategori).

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 53 | 77 | `sha1Hash` | Digest SHA-1; di-chunk via LimitUInt32 untuk input >4 GiB |
| 79 | 95 | `dataEncryptedWithPassword:` | IV acak + key 128-bit dari SHA-1 password; delegasi ke varian key |
| 123 | 177 | `dataEncryptedWithKey:IV:` | AES-128-CBC encrypt dengan IV di depan + padding kustom |
| 179 | 186 | `dataDecryptedWithPassword:` | Turunkan key dari SHA-1 password; delegasi |
| 188 | 248 | `dataDecryptedWithKey:` | AES-128-CBC decrypt; validasi padding, plaintext atau nil |
| 250 | 290 | `decompress` | zlib inflate seluruh buffer ke mutable data membesar |
| 292 | 326 | `compress` | zlib deflate seluruh buffer per chunk 16 KiB |
| 331 | 344 | `dataToHexString` | String hex uppercase, tanpa separator |
| 352 | 358 | `static int hexchar2nibble(char)` | Satu digit hex → nilai, −1 bila invalid |
| 369 | 378 | `static void decodeValidHexSequence(...)` | Decode hex pair in-place (ketat) |
| 392 | 478 | `+sp_dataWithHexString:` | Parse hex, `0x`, `X'..'` jadi data |
| 483 | 544 | `dataToFormattedHexString` | Hexdump 16 byte/baris dengan offset + gutter ASCII |
| 549 | 554 | `stringRepresentationUsingEncoding:` | Decode, fallback ASCII, bisa nil |
| 561 | 573 | `shortStringRepresentationUsingEncoding:` | Preview cap 255 char + ellipsis |
| 575 | 621 | `enumerateLinesBreakingAt:withBlock:` | Pecah byte pada CR/LF/CRLF, block per baris |
| 627 | 633 | `LimitUInt32(NSUInteger)` | Clamp ke UINT32_MAX; NOOP di 32-bit |

### Temuan
- **Kontrak kripto non-standar terdokumentasi sendiri**: ABNF wire format (IV ‖ AES ‖ 4-byte BE plaintext length) di 97–122; formula padding 136; IV di blok pertama 141–142; tail length 147–150. Encrypt menolak input > `UINT32_MAX` (125–126) + wajib IV `kCCBlockSizeAES128`/key `kCCKeySizeAES128` (128–132) → `NSInvalidArgumentException`.
- ⚠️ **Key derivation = SHA-1 terpotong tanpa salt/iteration** (92, 182). **UNKNOWN** ekspektasi kekuatan.
- **Guard CVE-2016-4711** re-check ciphertext-in-place vs plaintext → `NSInternalInconsistencyException` (167–174).
- ⚠️ **Asimetri gagal decrypt**: padding gagal *return nil* (229–235, setelah `free`), sedangkan `dataLength` over-long *throw* (241–245) — jalur throw **tidak** free `decryptedBytes` (alokasi 205) → **leak**. Leak serupa `calloc` 140 bila `CCCrypt` gagal (167–170).
- ⚠️ **`decompress`/`compress` bypass `LimitUInt32`**: cast ke `uInt` (263, 303) → input >4 GiB truncate senyap (beda dari `sha1Hash` yang guard 58–74).
- **Kontrak hex (`+sp_dataWithHexString:`)**: kosong ⇒ data kosong (398); spasi/tab leading di-skip sebelum deteksi `0x` (405–410); MySQL `X'..'` butuh quote penutup + tail whitespace-only (412–441); digit ganjil di-pad kiri `'0'` (464–469); decode in-place (471–473); gagal lewat `fail_cleanup` free buffer (475–477).
- `dataToFormattedHexString` hardcode 16 byte/baris (487), byte `<0x20` → `'.'` (525) (byte high-bit **tidak** disanitasi), rely pada slot `calloc(buffLength+1)` untuk NUL (514, 529).
- `stringRepresentationUsingEncoding:` bisa return nil (551–553); cabang `@"-- cannot display --"` (565–567) jarang tercapai karena fallback ASCII biasanya hasilkan mojibake (**UNKNOWN** intent).
- `enumerateLinesBreakingAt:` yield range **tanpa** terminator (606), hormati `stop` (605–608), skip `\n` dari CRLF (611), emit baris akhir tak terterminasi (616–620).

---

## 2. `SPDataStorage.m` (590)

Store data editable yang membungkus `SPMySQLStreamingResultStore`; proxy baris belum diedit + simpan salinan mutable baris teredit.

### `@implementation`
`@interface SPDataStorage ()` (39–44) decl `_checkNewRow:` (41) + `_addRowUnsafeUnchecked:` (42);
`@implementation SPDataStorage` (46–590) **kelas biasa** — 23 method.
Helper C: `static inline NSMutableArray* SPDataStorageGetEditedRow(NSPointerArray*, NSUInteger)` (48–54).
Header `SPDataStorage.h:42–86`; inline IMP cache `SPDataStorageAddRow` (`.h:93`), `SPDataStorageReplaceRow` (101), `SPDataStorageObjectAtRowAndColumn` (109), `SPDataStoragePreviewAtRowAndColumn` (117).

### `#pragma mark`
`56 (Setting result store), 110/111 (Retrieving rows and cells), 239/240 (NSFastEnumeration), 289/290 (Adding and amending), 454 (Unloaded columns), 470 (Basic information), 509 (Delegate callback methods), 532, 561 (Private API)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 62 | 108 | `setDataStorage:updatingExisting:` | Tukar store, bersihkan baris teredit + kolom unloaded |
| 117 | 142 | `rowContentsAtIndex:` | Salinan dangkal baris teredit, else baris diproksi + placeholder |
| 147 | 173 | `cellDataAtRow:column:` | Satu sel dari baris teredit/placeholder/store |
| 179 | 209 | `cellPreviewAtRow:column:previewLength:` | Preview sel terpotong; baris teredit dapat cut 150/147 keras |
| 214 | 237 | `cellIsNullOrUnloadedAtRow:column:` | Uji NULL/unloaded lintas baris teredit+diproksi |
| 247 | 287 | `countByEnumeratingWithState:objects:count:` | NSFastEnumeration satu salinan baris |
| 297–452 | | `addRowWithContents:` / `insertRowContents:atIndex:` / `replaceRowAtIndex:withRowContents:` / `replaceObjectInRow:column:withObject:` / `removeRowAtIndex:` / `removeRowsInRange:` / `removeAllRows` | Mutasi baris |
| 460–507 | | `setColumnAsUnloaded:` / `count` / `columnCount` / `dataDownloaded` / `awaitDataDownloaded` | Info |
| 514 | 527 | `resultStoreDidFinishLoadingData:` | Sinkron count baris teredit, broadcast selesai |
| 534–588 | | `init` / `dealloc` / `_checkNewRow:` / `_addRowUnsafeUnchecked:` | Setup + privat |

### Temuan
- **Kontrak wrapper** (header 36–40): semua jalur baca cek `editedRows` dulu lalu store (121–131, 151–171, 183–207, 218–235, 260–268).
- **Race delegate terdokumentasi**: delegate di-set **setelah** swap ivar untuk hindari callback dari store tak dikenal, dengan re-check `dataDownloaded` kompensasi (98–107); guard callback tak cocok (517–520).
- **Lock tunggal `@synchronized(self)`** di 19 method (67, 120, 150, 182, 216, 253, 303, 325, 367, 384, 405, 426, 447, 462, 477, 487, 497, 516, 550). `dataDownloadedLock` = `NSCondition` terpisah (540) dipakai hanya `awaitDataDownloaded`/`resultStoreDidFinishLoadingData` (502–507, 524–526).
- **Kontrak call privat load-bearing**: "DO NOT CALL THIS METHOD UNLESS YOU CURRENTLY HAVE A LOCK ON SELF!!!" (563); "…UNLESS YOU HAVE CALLED `_checkNewRow:` FIRST!" (578–579).
- **`editedRowCount` ada karena `NSPointerArray` bisa memuat NULL**, jadi `count` tidak bisa dipakai sebagai panjang prefix (122, 152, 184, 218, 260, 385, 412).
- `_addRowUnsafeUnchecked:` menambah baris dummy ke store (587) agar count benar; `count` (478) ikut hitung dummy; `dataDownloaded` YES bila store nil (498).
- ⚠️ **Defect ordering bounds-check (baris teredit bypass cek kolom)**: di `cellDataAtRow:column:`, `cellPreviewAtRow:...`, `cellIsNullOrUnloadedAtRow:`, guard `columnIndex >= numberOfColumns` jalan **setelah** early-return baris teredit (152–163, 184–199, 218–229), sehingga `CFArrayGetValueAtIndex` pada baris teredit (156, 188, 222) **tak dijaga** → UB out-of-bounds.
- ⚠️ **`cellPreviewAtRow:column:previewLength:` mengabaikan `previewLength` untuk baris teredit** dan hardcode `>150`/`substringToIndex:147` (189–190), jalur diproksi tetap pakai `previewLength` (207).
- ⚠️ **`removeRowsInRange:` risiko unsigned underflow**: `NSUInteger i = MIN(editedRowCount, NSMaxRange(rangeToRemove)); while (--i >= rangeToRemove.location)` (433–434) wrap ke `NSUIntegerMax` bila `rangeToRemove.location == 0` + ada baris teredit → `removePointerAtIndex:` index raksasa. Reachability **UNKNOWN/kemungkinan dead**: tak ada caller di luar file (hanya `SPDataStorage.h:68`, `.m:424`, `:438`).
- ⚠️ **`replaceObjectInRow:column:withObject:` memutasi baris setelah keluar lock** (394–397) → write tak atomik vs pembaca konkuren; tanpa cek bounds `columnIndex`.
- `dealloc` hanya free bitmap + body locked kosong (550–557); tak bersihkan `dataStorage.delegate` (**UNKNOWN** bila store hidup lebih lama).

---

## 3. `SPFileManagerAdditions.m` (222)

Kategori `NSFileManager`: subdirektori Application Support, deteksi encoding best-guess, temp directory tanpa trailing slash.

### `@implementation`
`@implementation NSFileManager (SPFileManagerAdditions)` (43–222) — 4 method. Header `.h:31–37`.
State file-scope: enum error `DirectoryLocationError*` (35–39), `static NSString *DirectoryLocationDomain` (41).

### `#pragma mark`
**Tidak ada.**

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 50 | 53 | `applicationSupportDirectoryForSubDirectory:error:` | Entry convenience, paksa createIfNotExists:YES |
| 55 | 159 | `applicationSupportDirectoryForSubDirectory:createIfNotExists:error:` | Resolve/buat subdir App Support, lapor NSError |
| 171 | 207 | `detectEncodingforFileAtPath:` | Sniff BOM/encoding byte awal via SPFileHandle |
| 209 | 220 | `+temporaryDirectory` | NSTemporaryDirectory tanpa trailing slash, fallback /tmp |

### Temuan
- Dua entry point, satu implementasi; bentuk 2-arg forward dengan create YES (52).
- ⚠️ **Out-parameter di-deref di luar guard NULL → crash debug-only**: `SPLog(@"…%@", [*errorOut localizedDescription])` di 89 dan 128 **setelah** blok `if (errorOut) { … }` (71–88, 110–127). Di RELEASE inert karena `SPLog` = `do { if (0) NSLog(…); } while (0)` (`SPConstants.h:761–768`); di DEBUG deref live. **UNKNOWN** apakah ada caller NULL.
- Komposisi path `<AppSupport>/<CFBundleExecutable>[/<subDirectory>]` (68, 95, 98, 101–102); "no path found" → `DirectoryLocationErrorNoPathFound` (70–88); file di lokasi → `…FileExistsAtLocation` (108–131); gagal buat → teruskan `NSError` (135–143); `create == NO` + dir hilang → nil tanpa error (144–146).
- ⚠️ **Deteksi encoding**: doc klaim baca 5 KB (163) tapi implementasi baca **500 byte** (179) → mismatch. Cek BOM index `bytes[0…3]` **sebelum** verifikasi `startData.length` (182–197) → file <4 byte dibaca melewati ujung buffer (UB; memori milik NSData). `NSUTF16LittleEndianStringEncoding` sengaja mengecualikan signature UTF-32LE (182 vs 192).
- `temporaryDirectory` strip tepat satu `/` akhir (215–217); bila `NSTemporaryDirectory()` return `"/"` hasilnya string kosong (**UNKNOWN/edge**); fallback `@"/tmp"` (213–214).

---

## 4. `SPStringAdditions.m` (683)

Kategori `NSString`/`NSMutableString`/`NSMutableAttributedString`: helper format, escape HTML/XML, quoting identifier/literal MySQL, helper pencarian/edit-safety.

### `@implementation`
`NSString (SPStringAdditions)` (40–634) — 21 method; `NSMutableString (SPStringAdditions)` (650–670) — 3; `NSMutableAttributedString (SPStringAdditions)` (672–683) — 1.
Header `NSString` (`.h:54–112`), `NSMutableString` (114–128), `NSMutableAttributedString` (130–132). Helper `static NSInteger _smallestOf(...)` (decl 38, def 639–648, di antara blok).

### `#pragma mark`
**Tidak ada.**

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 45–115 | | `+stringForByteSize:` / `+stringForTimeInterval:` | Format manusiawi |
| 120–135 | | `+stringWithNewUUID` / `contains:` / `sha256Hash` | Helper kecil |
| 139 | 177 | `rot13` | Transform ROT13 ASCII-only |
| 182–225 | | `HTMLEscapeString` / `XMLEscapeStringForContent` | Escape |
| 233–255 | | `backtickQuotedString` / `tickQuotedString` / `replaceUnderscoreWithSpace` | Quoting |
| 262 | 301 | `createViewSyntaxPrettifier` | Reformas CREATE VIEW via capture regex |
| 310 | 335 | `lineRangesForRange:` | Range baris yang sebagian tertutup range |
| 337–439 | | `sp_stringByReplacingCharactersInSet:withString:` / `summarizeToLength:withEllipsis:` / `stringByRemovingCharactersInSet:options:` / `safeSubstringWithRange:` / `stringByRemovingCharactersInSet:` | Edit teks |
| 444 | 492 | `levenshteinDistanceWithWord:` | Edit distance case-insensitive via matriks calloc |
| 497 | 519 | `getGeomFromTextString` | Bangun ST_GeomFromText dengan SRID opsional |
| 521 | 632 | `nonConsecutivelySearchString:matchingRanges:` | Subsequence search dengan merge range match |
| 652–668 | | `setStringOrNil:` / `appendStringOrNil:` / `safeDeleteCharactersInRange:` (NSMutableString) | Nil-safe |
| 674 | 681 | `safeDeleteCharactersInRange:` (NSMutableAttributedString) | Guard sama |
| 639 | 648 | `_smallestOf(...)` | Min 3 NSInteger |

### Temuan
**Kontrak quoting (diminta eksplisit):**
- `backtickQuotedString` (233–236) hanya gandakan `` ` `` + bungkus backtick (identifier, tidak di-escape backslash). Memadai untuk identifier.
- ⚠️ `tickQuotedString` (244–247) hanya gandakan `'` + bungkus quote tunggal. **Tidak** meng-escape backslash, NUL, newline, `\Z`. Dengan parsing MySQL default (non-`NO_BACKSLASH_ESCAPES`), input memuat `\` bisa meng-escape quote penutup. Ini **literal quoter, bukan identifier quoter** — dokumentasi bersama kedua method menyesatkan; escaping tak cukup untuk nilai arbitrer. **UNKNOWN** apakah caller mengandalkannya untuk literal (tak ada caller di file ini).
- Keduanya tak menyediakan varian sadar charset/binary, tak menangani NUL embedded.

**Catatan lain:**
- ⚠️ `levenshteinDistanceWithWord:` (444–492) — komentar klaim case-insensitive **dan** trimmed (442–443), tapi dua hasil `stringByTrimmingCharactersInSet:` dibuang (449–450) → trimming tak pernah terjadi: divergensi komentar/impl. Lowercase saja (452–453); return `CGFloat` padahal nilainya `NSInteger` (455, 488); idiom `n++/m++` → index 1-based (460); free matriks (462, 486); return `0.0f` untuk kasus kosong (491).
- ⚠️ `nonConsecutivelySearchString:matchingRanges:` (521–632) — **latent defect**: `NSRange left = NSMakeRange(my.location - prev.length, prev.length)` (608) tak dijaga untuk `my.location < prev.length`; `substringWithRange:` berikutnya (609) akan raise `NSRangeException`. Reachability **UNKNOWN**.
- ⚠️ `safeSubstringWithRange:` (422–431) **return receiver** pada input mencurigakan alih-alih raise, bertentangan dengan komentarnya sendiri (424). Guard asimetris: location tepat `length` dengan char terakhir di range → return seluruh string.
- `summarizeToLength:withEllipsis:` (375–387) potong dulu (379), lalu mundur ke whitespace dan bisa return **lebih pendek** dari `length` (382–384, ellipsis U+2026).
- `lineRangesForRange:` return array **serialisasi `NSString`** (`NSStringFromRange`, 323, 330), bukan `NSValue`.
- Escape: `HTMLEscapeString` escape `&`, `<`, `>`, `"` tapi bukan `'` (186–200); `XMLEscapeStringForContent` sengaja pertahankan quotes (208–224) — keduanya **bukan** XML-attribute-safe.
- `getGeomFromTextString` (497–519) return `@"NULL"` bila tak ada `")"`/panjang <5 (501), SRID split di `)` terakhir (504–517); uji pakai `self` tapi nilai dipakai `geomStr` trimmed (499).
- `sha256Hash` (130–135) bergantung `NSData dataToHexString` (uppercase) (`SPDataAdditions.m:331–344`) via header transitif (**UNKNOWN** header mana).

---

## 5. `SPTextViewAdditions.m` (834)

Kategori `NSTextView`: seleksi word/line/bracket, transform case + normalisasi Unicode, pindah baris, sisip NULL, jalankan bundle, context menu, zoom trackpad.

### `@implementation`
`@implementation NSTextView (SPTextViewAdditions)` (43–834) — 22 method. Satu-satunya `@implementation`.
`menuForEvent:` (738) + `magnifyWithEvent:` (822) diimplementasi di sini tapi **tak dideklarasikan** di `.h:31–57` (override NSView/NSResponder).

### `#pragma mark`
`816/817 (multi-touch trackpad support)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 50–106 | | `getRangeForCurrentWord` / `selectCurrentWord:` / `selectCurrentLine:` | Seleksi |
| 111 | 190 | `selectEnclosingBrackets:` | Seleksi teks dalam pasangan bracket tak berimbang terdekat |
| 195–337 | | `doSelectionUpperCase:` / `LowerCase:` / `TitleCase:` / `doDecomposedStringWithCanonicalMapping:` / `…CompatibilityMapping` / `doPrecomposedStringWithCanonicalMapping` / `doRemoveDiacritics` / `doPrecomposedStringWithCompatibilityMapping:` | Transform case + normalisasi Unicode |
| 345 | 393 | `doTranspose:` | Transpose char bersebelahan / reverse seleksi |
| 398–500 | | `insertNULLvalue:` / `moveSelectionLineUp:` / `moveSelectionLineDown:` / `makeTextSizeLarger` / `makeTextSizeSmaller` / `makeTextStandardSize` | Edit + font |
| 502 | 733 | `executeBundleItemForInputField:` | Bangun env, jalankan bundle bash, dispatch output action |
| 738 | 814 | `menuForEvent:` | Bangun context menu dengan submenu Bundles |
| 822 | 832 | `magnifyWithEvent:` | Pinch trackpad zoom ukuran font |

### Temuan
- **Override tak terdeklarasi**: `menuForEvent:` + `magnifyWithEvent:` tanpa counterpart di header — risiko override-conflict **UNKNOWN**.
- ⚠️ **Local tak terinisialisasi di `selectEnclosingBrackets:`**: `unichar co, cc;` tak diinisialisasi (115); scan maju hanya menugaskan bila menemukan bracket penutup tak berpasangan (124–148, assign 128/135/142). Bila scan tak menemukan closer tak berimbang, `cc`/`co` dibaca di 154/156 dan dipakai sebagai karakter match loop mundur (159–170) → **UB**, bukan sekadar no-op. Early-out 171 terjadi **setelah** pembacaan itu.
- **Idiom seleksi-based editing**: transform set seleksi lalu append replacement ke `textStorage` (199–201, 211–213, 223–225, 281–282, 304, 326, 390). `moveSelectionLineUp:`/`Down:` tambah string kosong untuk paksa layout (425, 430, 452) — no-op edit sebagai hack; **UNKNOWN** apakah masih perlu.
- ⚠️ **`doTranspose:` sengaja belum lengkap**: TODO "not yet combining-diacritics-safe" (343, 381); bungkus `@try/@catch(id ae)` (351–377); reverse seleksi via `%C` (385–390).
- **Eksekusi bundle (`executeBundleItemForInputField:`, 502–733)** — satu-satunya method dengan side effect eksternal: index dari `tag - 1000000` (504), fallback path bundle dari tooltip sender (510–512), muat bundle (520–526), wajib command non-kosong (528–537), resolve input source/fallback (569–582), tulis file input di samping `SPBundleTaskInputFilePath` + sufiks UUID (543, 636–641), isi ~20 variabel env termasuk edited column/table/query (584–631), jalankan `SPBundleCommandRunner runBashCommand:` (649–656), hapus file input (547, 658), dispatch output action (660–732). Error code 9 ditekan ("command was killed") (669).
  - **Deteksi tipe/identity by string class-name**: `SPTextView` (549), `SPFieldEditorController` (590), `SPCopyTable` (598), suffix nama kelas data source `Content` (602). Rapuh; **UNKNOWN** untuk delegate lain.
  - File input write atomik UTF-8 (638–641); error write → alert + return (643–647). File dihapus apa pun hasil run (658).
  - Reuse window HTML by UUID bundle (686–703). Snippet insertion butuh `insertAsSnippet:atRange:` else tooltip (711–716); `ReplaceContent` pilih seluruh teks dulu (719–721); `ReplaceSelection` iris range dengan bounds string sebelum `shouldChangeTextInRange:` (724–728).
- `menuForEvent:` hapus item Bundles existing + separator (tag `10000000`, 744–749), early-return bila `connectionID == @"_"` (751–754), bangun ulang kategori/item jadi submenu (757–810), set tag item `1000000 + i` (802) — counterpart dari 504; target tiap item = first responder key window (797).
- `magnifyWithEvent:` skip resize untuk `SPCopyTable`/delegate `NSTableView` via perbandingan nama kelas (825–826), dead zone ±5.0 `deltaZ` (828–831).
- Method font sementara paksa editability sekitar `setFont:` + restore (472–475, 485–488, 496–499); `makeTextStandardSize` hardcode 11.0f (498), varian smaller clamp 4pt (484).

---

## Ringkasan
| File | Baris | Blok | Method | `#pragma mark` |
| --- | --- | --- | --- | --- |
| `SPDataAdditions.m` | 633 | 1 kategori + 3 C | 13 | 2 |
| `SPDataStorage.m` | 590 | extension + 1 kelas + 1 static inline | 23 | 12 |
| `SPFileManagerAdditions.m` | 222 | 1 kategori | 4 | 0 |
| `SPStringAdditions.m` | 683 | 3 kategori + 1 static | 25 | 0 |
| `SPTextViewAdditions.m` | 834 | 1 kategori | 22 | 2 |
| **Total** | **2962** | — | **87** | **16** |

Temuan bernilai tertinggi: jalur `CFArrayGetValueAtIndex` baris teredit tak dijaga (`SPDataStorage.m:156, 188, 222`); loop underflow `removeRowsInRange:` (`433–437`, kini tak terjangkau); `co`/`cc` tak terinisialisasi di `selectEnclosingBrackets:` (`SPTextViewAdditions.m:115, 154–170`); deref `*errorOut` DEBUG-only (`SPFileManagerAdditions.m:89, 128`); escaping literal MySQL tak lengkap di `tickQuotedString` (`SPStringAdditions.m:244–247`).
