# Audit: `SPSQLParser.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Other/Parsing/SPSQLParser.m`. Baseline commit `02c910932`.

- File: 1110 baris; header 311 baris.
- Peran: subclass `NSMutableString` ("class cluster") yang melakukan parsing SQL-aware
  streaming: lookup sadar quote/comment/bracket, splitting, trimming, penanganan
  DELIMITER, dan normalisasi query.
- Total: 59 definisi method. Tidak ada fungsi C/file-local static.

## 1. Inventaris ivar + konstanta

### Ivar (`.h:57–72`, tanpa `@property`)
| Ivar | `.h` | Fungsi |
| --- | --- | --- |
| `NSMutableString *string` | 58 | backing storage |
| `unichar *stringCharCache` | 59 | buffer char cache (calloc) |
| `unichar parsedToChar` | 60 | char terakhir diquery (reset 430, 1018, 1106) |
| `NSInteger parsedToPosition` | 61 | posisi scan ter-cache (431, 1019, 1107) |
| `NSInteger charCacheStart` / `charCacheEnd` | 62–63 | rentang cache; `-1` = kosong (1020, 1089, 1104) |
| `BOOL ignoreCommentStrings` | 64 | bila YES comment diperiksa, bukan dilewati (88, 760, 768, 774) |
| `BOOL containsCRs` | 65 | lacak `\r` di luar quote (77, 782, 922, 1026) |
| `BOOL noBackslashEscapes` | 66 | mode NO_BACKSLASH_ESCAPES (105, 219, 294, 876, 1027) |
| `BOOL supportDelimiters` | 68 | aktifkan parsing DELIMITER (100, 619, 662, 719, 792, 1022) |
| `NSString *delimiter` | 69 | token DELIMITER aktif (814, 1023) |
| `NSUInteger delimiterLengthMinusOne` | 70 | panjang delimiter − 1 (379, 407, 815, 821, 1024) |
| `BOOL lastMatchIsDelimiter` | 71 | flag match delimiter (bukan char) (375, 403, 708, 825, 1025) |

### Konstanta/macro `.m`
`CHARACTER_CACHE_LENGTH` = 1500 (47); `CHAR_SQUOTE`/`CHAR_DQUOTE`/`CHAR_BTICK`/`CHAR_BS`/`CHAR_CR`/`CHAR_LF` (49–54);
`STRING_SQUOTE`/`STRING_DQUOTE`/`STRING_BS`/`STRING_LF` (56–59). Class extension `@interface SPSQLParser ()` (36–40) dengan `_clearCharCache` (38).

## 2. Semua `#pragma mark`

`.m:67/68 (Parser information), 80/81 (Parser behaviour setting), 108/109 (SQL-aware utility methods), 353/354 (Trimming or retrieving strings from the front), 447/448 (…from one specified character to another), 557/558 (Splitting strings), 671/672 (SQL-aware character lookups), 946/947 (Required and primitive methods…), 1070 (Private API)`. Header punya `#pragma mark -` nyasar di `.h:289`.

## 3. Tabel method (59 definisi, urut baris)

| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 75 | 78 | `containsCarriageReturns` | Kembalikan flag `containsCRs` ter-cache |
| 86 | 89 | `setIgnoreCommentStrings:` | Set apakah comment diperiksa vs dilewati |
| 98 | 101 | `setDelimiterSupport:` | Enable/disable parsing DELIMITER |
| 103 | 106 | `setNoBackslashEscapes:` | Toggle perilaku NO_BACKSLASH_ESCAPES |
| 114 | 171 | `deleteComments` | Buang comment `--`, `#`, `/* */` dari string in place |
| 173 | 188 | `isQuotedString` | Uji apakah seluruh string satu token ter-quote |
| 193 | 230 | `unquotedString` | Lepas quote sekitar + un-escape isi |
| 236 | 239 | `+normaliseQueryForExecution:` | Overload convenience; backslash escape aktif (238) |
| 241 | 351 | `+normaliseQueryForExecution:noBackslashEscapes:` | Trim ujung, konversi CR/CRLF non-quoted → LF |
| 359 | 362 | `trimToCharacter:inclusively:` | Shortcut; abaikan quoted string (361) |
| 368 | 381 | `trimToCharacter:inclusively:ignoringQuotedStrings:` | Hapus sampai kemunculan char pertama |
| 386 | 389 | `stringToCharacter:inclusively:` | Shortcut; abaikan quoted string (388) |
| 395 | 408 | `stringToCharacter:inclusively:ignoringQuotedStrings:` | Kembalikan prefiks sampai char pertama |
| 414 | 417 | `trimAndReturnStringToCharacter:trimmingInclusively:returningInclusively:` | Shortcut; abaikan quote (416) |
| 423 | 445 | `trimAndReturnStringToCharacter:...ignoringQuotedStrings:` | Kembalikan + hapus prefiks sampai char |
| 455 | 458 | `stringFromCharacter:toCharacter:inclusively:` | Shortcut; tanpa bracket-skip (457) |
| 464 | 467 | `stringFromCharacter:toCharacter:inclusively:skippingBrackets:` | Shortcut + bracket-skip (466) |
| 473 | 476 | `stringFromCharacter:toCharacter:inclusively:ignoringQuotedStrings:` | Shortcut + opsi quote (475) |
| 481 | 500 | `stringFromCharacter:...skippingBrackets:ignoringQuotedStrings:` | Rentang antara char from/to berimbang |
| 506 | 509 | `trimAndReturnStringFromCharacter:trimmingInclusively:returningInclusively:` | Shortcut; tanpa bracket-skip (508) |
| 515 | 518 | `trimAndReturnStringFromCharacter:...skippingBrackets:` | Shortcut + bracket-skip (517) |
| 524 | 527 | `trimAndReturnStringFromCharacter:...ignoringQuotedStrings:` | Shortcut + opsi quote (526) |
| 533 | 555 | `trimAndReturnStringFromCharacter:...skippingBrackets:ignoringQuotedStrings:` | Kembalikan + hapus rentang from/to |
| 563 | 566 | `splitStringByCharacter:` | Shortcut; tanpa bracket-skip, abaikan quote (565) |
| 572 | 575 | `splitStringByCharacter:skippingBrackets:` | Shortcut + bracket-skip (574) |
| 581 | 584 | `splitStringByCharacter:ignoringQuotedStrings:` | Shortcut + opsi quote (583) |
| 589 | 626 | `splitStringByCharacter:skippingBrackets:ignoringQuotedStrings:` | Pecah ke array substring di batas delimiter |
| 631 | 669 | `splitStringIntoRangesByCharacter:` | Pecah ke array `NSValue` range di batas |
| 680 | 683 | `firstOccurrenceOfCharacter:ignoringQuotedStrings:` | Shortcut: dari index −1, tanpa bracket (682) |
| 691 | 694 | `firstOccurrenceOfCharacter:afterIndex:ignoringQuotedStrings:` | Shortcut: dari index, tanpa bracket (693) |
| 702 | 841 | `firstOccurrenceOfCharacter:afterIndex:skippingBrackets:ignoringQuotedStrings:` | Scanner SQL-aware inti; bracket, quote, comment, DELIMITER |
| 848 | 900 | `endIndexOfStringQuotedByCharacter:startingAtIndex:` | Cari quote penutup (backslash/doubling) |
| 905 | 944 | `endIndexOfCommentOfType:startingAtIndex:` | Cari akhir comment `#`/`--`/`/* */` |
| 949 | 956 | `init` | Init `string`, panggil `initSQLExtensions` |
| 957 | 963 | `initWithBytes:length:encoding:` | Backing string dari bytes |
| 964 | 970 | `initWithBytesNoCopy:length:encoding:freeWhenDone:` | Backing dari bytes dipinjam |
| 971 | 977 | `initWithCapacity:` | Backing dengan capacity |
| 978 | 984 | `initWithCharactersNoCopy:length:freeWhenDone:` | Backing dari unichar dipinjam |
| 985 | 987 | `initWithContentsOfFile:` | Delegasi UTF-8, tanpa error (986) |
| 988 | 994 | `initWithContentsOfFile:encoding:error:` | Backing dari file |
| 995 | 1001 | `initWithCString:encoding:` | Backing dari C string |
| 1002 | 1009 | `initWithFormat:` | Variadik; bangun via `arguments:` (1005) |
| 1010 | 1016 | `initWithFormat:arguments:` | Backing dari format + va_list |
| 1017 | 1028 | `initSQLExtensions` | Reset semua ivar state parser ke default |
| 1029 | 1031 | `length` | Panjang backing string |
| 1032 | 1038 | `characterAtIndex:` | Accessor bounds-checked; overflow return `0x00` + log (1033–1035) |
| 1039 | 1041 | `description` | Deskripsi backing string |
| 1042 | 1044 | `replaceOccurrencesOfString:withString:options:range:` | Teruskan replace ke backing |
| 1045 | 1051 | `setString:` | Set backing, reset state delimiter, bersihkan cache |
| 1052 | 1055 | `replaceCharactersInRange:withString:` | Replace range lalu bersihkan char cache |
| 1056 | 1059 | `deleteCharactersInRange:` | `super` lalu bersihkan cache |
| 1060 | 1063 | `insertString:atIndex:` | `super` lalu bersihkan cache |
| 1064 | 1068 | `dealloc` | Free `stringCharCache` bila dialokasi (1067) |
| 1077 | 1093 | `_charAtIndex:` | Fetch char mentah ter-cache; refill buffer 1500 char |
| 1099 | 1108 | `_clearCharCache` | Free cache + reset cursor cache/parse |

## 4. Temuan menonjol / UNKNOWN

**Scanner inti — `firstOccurrenceOfCharacter:afterIndex:skippingBrackets:ignoringQuotedStrings:` (702–841)**
- **Bracket depth**: `(` naik, `)` turun `bracketingLevel` (750–756); match hanya saat `bracketingLevel <= 0` (721, 727).
- **Quirk fall-through (UNKNOWN/bug potensial)**: cabang `case ')'` (755–756) **tanpa `break;`** setelah `bracketingLevel--` (756), jatuh ke `case '-'` (759). Untuk `)` yang char berikutnya `-`-`-` + whitespace, bisa salah-deteksi comment start. Maksud tak jelas — **UNKNOWN**.
- **Scanning DELIMITER**: bila `supportDelimiters && delimiter` (719), tes match membandingkan string delimiter via `safeSubstringWithRange:` (720).
- **Pengenalan command DELIMITER** (787–834): hanya pada `d`/`D` (787–788), wajib `supportDelimiters`, ≥11 sisa char, whitespace word-start (792–794), lalu jalan `e`,`l`,`i` (796–804) sebelum regex RegexKitLite `^(delimiter[ \t]+(\S+))(?=\s|\Z)` (805, 812); match → `lastMatchIsDelimiter = YES` (825).
- **Skip comment** digerakkan `ignoreCommentStrings` (760, 768, 774) → `endIndexOfCommentOfType:` (764, 769, 777). **Skip quote** → `endIndexOfStringQuotedByCharacter:` (741); quote tak tertutup → `NSNotFound` dengan `parsedToPosition = stringLength-1` (742–745). **CR capture** set `containsCRs` (782).

**Skip comment/quote**
- `endIndexOfStringQuotedByCharacter:startingAtIndex:` (848–900): doubling backtick di-skip (857–863) — didokumentasikan "backslash+backtick is not an escape sequence" (865–867); akhir single/double quote di-escape oleh backslash **berjumlah ganjil** (876–882) dan doubling (888–891). `NSNotFound` = tak tertutup (899).
- `endIndexOfCommentOfType:startingAtIndex:` (905–944): tangani tiga `SPCommentType` (`.h:74-78`); `SPDoubleDashComment` sengaja jatuh ke `SPHashComment` (914–919); return `stringLength-1` bila tak ada terminator (943).
- `deleteComments` (114–171) tulis ulang in place; buang `--[\s]` (137–147), `#` (149–156), `/* */` (159–168) dengan skip quoted string (126–134).

**`normaliseQueryForExecution:` (236–351)**: overload 236–239 forward `noBackslashEscapes:NO` (238). Metode utama 241–351 trim whitespace ujung (247–257) via `safeSubstringWithRange:` (256), jalan mencatat posisi `\r` **di luar** quoted string (263–327) dengan aturan paritas backslash (294–301) + doubling (306–309) inline (duplikat logika §skip quote), lalu konversi CRLF→LF + CR tunggal→LF iterasi mundur (330–348).

**Char cache (1077–1093, 1099–1108)**: `_charAtIndex:` **raw tanpa bounds-check** (1074–1075) yang lazy (re)fill buffer `CHARACTER_CACHE_LENGTH` (1500) via `CFStringGetCharacters` (1087–1088), free buffer sebelumnya (1082–1084). `_clearCharCache` dipanggil dari `setString:` (1050), `replaceCharactersInRange:` (1054), `deleteCharactersInRange:` (1058), `insertString:` (1062). Kelas **tidak thread-safe** (`.h:54`).

**UNKNOWN/eksternal**
- `safeSubstringWithRange:` (214, 256, 407, 442, 499, 552, 610, 720) — kategori `NSString` di `SPStringAdditions.h:87`; `safeString(...)` (1034) di `SPConstants.h:812`; `SPLog(...)` (1034).
- Overflow `characterAtIndex:` return `(unichar)0x00` + `FIXME: what should this return?` (1035).
- `dealloc` (1064–1068) free hanya `stringCharCache`; ownership `delimiter` (`[[NSString alloc] init…]` di 814) tak dilepas di sini — di bawah ARC ini nil; **UNKNOWN** apakah non-ARC.
