# Audit: `StringExtension.swift`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Other/Extensions/StringExtension.swift`. Baseline commit `02c910932`.

- File: 1093 baris.
- Peran: kumpulan extension `String`/`NSString` plus sebelas kelas helper `@objc`
  untuk serialisasi baris process-list, batas panjang edit berbasis code point,
  klasifikasi tipe field, estimasi tipe field, pelacakan kegagalan load tabel,
  dan normalisasi metadata charset.
- **Tidak ada `// MARK:`** di seluruh file (temuan navigasi).

## 1. Inventaris tipe/extension (per rentang baris)

| Baris | Nama | Jenis |
| --- | --- | --- |
| 11–27 | `StringProtocol` | extension |
| 29–236 | `String` | extension |
| 238–303 | `SPProcessListRowSerializer` | `@objc public class : NSObject` |
| 241–251 | `ProcessListColumnKey` | `private enum : String` (nested) |
| 309–321 | `SATextLimitDecision` | `@objc public enum : Int` |
| 327–429 | `SAFieldEditorEditLimit` | `@objcMembers public final class` |
| 436–510 | `SACellEditLimit` | `@objcMembers public final class` |
| 512–697 | `NSString` | `@objc extension` |
| 699–841 | `SPOptimizedFieldTypeEstimator` | `@objcMembers final class` |
| 807–812 | `IntegerRange` | struct nested |
| 843–988 | `SPFieldTypeClassifier` | `@objcMembers public final class` |
| 844–848 | `FieldTypeGroup` | `private enum : String` (nested) |
| 990–1017 | `SPTableLoadFailure` | `@objcMembers public final class` |
| 1019–1080 | `SPCharacterSetMetadataNormalizer` | `@objcMembers public final class` |
| 1085–1093 | `SABitFieldCommitValidation` | `@objcMembers public final class` |

## 2. Tabel method (urut baris, dikelompokkan per tipe)

### `extension StringProtocol` (11–27)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 23 | 26 | `containsAnyUnicodeScalar(of:)` | Apakah ada scalar Unicode terdaftar muncul |

### `extension String` (29–236)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 31 | 36 | `subscript(_: CountableRange<Int>)` | Substring untuk range int ter-clamp |
| 38 | 41 | `subscript(_: CountablePartialRangeFrom<Int>)` | Substring dari start ter-clamp |
| 43 | 50 | `slice(from:to:)` | Teks antara dua substring penanda, atau nil |
| 52 | 54 | `rawByteString(_:)` | Hex-encode `Data` uppercase |
| 57 | 62 | `dropPrefix(_:)` | Hapus prefiks bila ada |
| 66 | 71 | `dropSuffix(_:)` | Hapus sufiks bila ada |
| 77 | 84 | `hasPrefix(_:caseSensitive:)` | Tes prefiks dengan opsi case-insensitive |
| 90 | 97 | `hasSuffix(_:caseSensitive:)` | Tes sufiks dengan opsi case-insensitive |
| 101 | 111 | `separatedIntoLines()` | Pecah pada setiap line break Unicode |
| 114 | 122 | `separatedIntoLinesByCharset()` | Pecah pada titik-koma, buang bagian kosong |
| 130 | 142 | `format(_:)` | Substitusi tipe CVarArg ke format string |
| 144 | 146 | `isNumeric` (var) | True bila non-kosong dan semua char digit |
| 156 | 164 | `isPercentEncoded` (var) | True bila percent-decode mengubah string |
| 167 | 177 | `stringByExpandingTildeAsIfNotInSandbox` (var) | Expand `~` ke home dir nyata di luar sandbox |
| 181 | 225 | `stringByExpandingTildeAsIfNotInSandboxBackup` (var) | Fallback rewrite path `file://` untuk path tilde |
| 228 | 230 | `trimmedString` (var) | Trim whitespace + newline |
| 233 | 235 | `whitespacesTrimmedString` (var) | Trim spasi saja |

### `SPProcessListRowSerializer` (238–303)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 241 | 251 | `ProcessListColumnKey` (enum) | Raw string key kolom baris `SHOW PROCESSLIST` |
| 262 | 284 | `serializedProcessRow(_:includeProgress:)` | Join nilai kolom baris proses jadi satu baris |
| 293 | 302 | `processValue(for:in:)` | Baca satu nilai kolom sebagai string (blank bila absen/null) |

### `SATextLimitDecision` (309–321)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 309 | 321 | `SATextLimitDecision` (enum) | Verdict edit: exempt / withinLimit / refuse / truncate |

### `SAFieldEditorEditLimit` (327–429)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 329 | 329 | `allowsEdit` (let) | Apakah edit sheet boleh jalan tanpa ubah |
| 334 | 334 | `fittingInsertion` (let) | Prefiks insert yang muat, nil bila diizinkan/tak ada |
| 344 | 348 | `init(allowsEdit:fittingInsertion:)` | Init privat; hanya `evaluate` membangun |
| 376 | 407 | `evaluate(text:replacing:with:limit:fieldType:)` | Putuskan edit field-editor terhadap batas code point |
| 389 | 392 | `decimalPointAllowance(inserting:)` | Helper nested: +1 untuk FLOAT bila titik desimal selamat |
| 422 | 428 | `isNullPlaceholderEdit(text:replacing:with:nullValue:)` | Deteksi edit yang menghasilkan placeholder NULL |

### `SACellEditLimit` (436–510)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 438 | 438 | `allowsEdit` (let) | Apakah edit sel boleh jalan tanpa ubah |
| 443 | 443 | `isExempt` (let) | Apakah tak ada aturan panjang berlaku |
| 448 | 448 | `replacementText` (let) | Teks pengganti sel, nil bila diizinkan/ditolak |
| 452 | 452 | `selectionLocation` (let) | Lokasi caret UTF-16 di `replacementText` |
| 462 | 468 | `init(allowsEdit:isExempt:replacementText:selectionLocation:)` | Init privat; hanya `evaluate` membangun |
| 487 | 509 | `evaluate(text:replacing:with:limit:fieldType:nullValue:)` | Putuskan edit sel, potong hanya insertion; delegasi ke `SAFieldEditorEditLimit` |

### `@objc extension NSString` (512–697)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 514 | 514 | `columnHeaderSplittingSpace` (static let) | Spasi pemisah nama+tipe kolom di header |
| 517 | 523 | `tableContentColumnHeaderString(columnName:columnType:columnTypesVisible:)` | Bangun header, tambah tipe bila terlihat |
| 529 | 531 | `rawByteString(data:)` | Bridge ObjC ke `String.rawByteString(_:)` |
| 538 | 540 | `dropPrefix(prefix:)` | Bridge `String.dropPrefix(_:)` |
| 548 | 550 | `characterCount()` | Hitung code point Unicode, bukan grapheme |
| 561 | 564 | `utf16Length(ofFirstCodePoints:)` | Panjang UTF-16 N code point pertama |
| 571 | 573 | `prefix(codePoints:)` | Prefiks N code point, tak memotong surrogate pair |
| 583 | 586 | `isNullPlaceholderOrItsStart(_:)` | True bila teks = placeholder NULL atau prefiksnya |
| 609 | 625 | `textLimitDecision(limit:nullValue:)` | Verdict panjang seluruh string untuk cell formatter |
| 630 | 632 | `dropSuffix(suffix:)` | Bridge `String.dropSuffix(_:)` |
| 634 | 636 | `hasPrefix(prefix:caseSensitive:)` | Bridge |
| 638 | 640 | `hasSuffix(suffix:caseSensitive:)` | Bridge |
| 642 | 644 | `separatedIntoLinesByCharsetObjC()` | Bridge |
| 646 | 648 | `trimWhitespacesAndNewlines()` | Bridge ke `trimmedString` |
| 650 | 652 | `trimWhitespaces()` | Bridge ke `whitespacesTrimmedString` |
| 654 | 656 | `stringByExpandingTildeAsIfNotInSandboxObjC()` | Bridge |
| 658 | 660 | `isNumeric()` | Bridge |
| 662 | 664 | `isPercentEncoded()` | Bridge |
| 666 | 668 | `separatedIntoLinesObjC()` | Bridge |
| 670 | 696 | `dateStringFromUnixTimestamp()` | Format timestamp Unix 10-digit sebagai ISO-8601 dalam ±100 tahun |

### `SPOptimizedFieldTypeEstimator` (699–841)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 701 | 703 | `integerFieldTypes` (static let) | Set nama tipe integer MySQL |
| 704 | 706 | `binaryFieldTypes` (static let) | Set nama tipe binary/blob MySQL |
| 707 | 709 | `stringFieldTypes` (static let) | Set nama tipe char/text MySQL |
| 712 | 721 | `normalizedFieldType(fromDefinition:)` | Ekstrak tipe dasar uppercase dari dict definisi field |
| 724 | 727 | `isIntegerFieldType(_:)` | Tes keanggotaan set integer |
| 730 | 733 | `isBinaryFieldType(_:)` | Tes keanggotaan set binary |
| 736 | 739 | `isStringFieldType(_:)` | Tes keanggotaan set string |
| 742 | 751 | `decimalNumber(fromStatValue:)` | Parse nilai statistik jadi `NSDecimalNumber` atau nil |
| 754 | 760 | `unsignedIntegerValue(fromStatValue:)` | Parse nilai statistik jadi `UInt` non-negatif |
| 763 | 803 | `maxBytesPerCharacter(forFieldDefinition:tableEncoding:availableEncodings:)` | Resolve byte max/char dari metadata charset atau by nama |
| 806 | 840 | `estimatedIntegerType(forMinimum:maximum:)` | Pilih tipe integer terkecil yang muat rentang min/max |
| 807 | 812 | `IntegerRange` (struct) | Batas signed/unsigned satu tipe kandidat |

### `SPFieldTypeClassifier` (843–988)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 844 | 848 | `FieldTypeGroup` (enum) | Grup: bit / integer / float |
| 850 | 865 | `unquotedFieldTypes` (static let) | Set nama tipe yang disisip ke SQL unquoted |
| 876 | 885 | `shouldBeUnquoted(fieldTypeGroup:fieldType:)` | Putuskan nilai kolom masuk SQL unquoted |
| 895 | 900 | `isBitField(fieldTypeGroup:fieldType:)` | Putuskan kolom MySQL `BIT` |
| 916 | 921 | `unquotedSQLLiteral(for:fieldTypeGroup:fieldType:)` | Bangun literal SQL untuk nilai unquoted |
| 928 | 931 | `bitLiteral(for:)` | Format nilai BIT sebagai literal biner `b'...'` |
| 941 | 947 | `decimalString(forBitString:)` | Konversi bit display string ke string desimal |
| 959 | 966 | `filterValue(for:targetTypeGrouping:)` | Sesuaikan nilai mentah untuk memfilter kolom target |
| 969 | 973 | `validatedBitString(from:)` | Kembalikan nilai bila string digit 0/1 non-kosong |
| 977 | 987 | `baseTypeToken(of:)` | Ekstrak token dasar uppercase dari tipe terdeklarasi |

### `SPTableLoadFailure` (990–1017)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 991 | 991 | `tableName` (let) | Nama tabel yang gagal dimuat |
| 992 | 992 | `databaseName` (let) | Nama database load gagal |
| 993 | 993 | `loadTableType` (let) | Kode integer jenis load gagal |
| 995 | 1000 | `init(tableName:databaseName:tableType:)` | Init privat; pakai factory |
| 1003 | 1009 | `failure(withTableName:database:tableType:)` | Factory bangun record kegagalan, blank kan nil |
| 1012 | 1016 | `matches(tableName:database:tableType:)` | Banding identitas tersimpan vs koordinat tabel |

### `SPCharacterSetMetadataNormalizer` (1019–1080)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 1020–1023 | | `charsetNameKeys`/`descriptionKeys`/`defaultCollationKeys`/`maxLengthKeys` (static let) | Varian key dikenali |
| 1026 | 1059 | `normalizedCharacterSetEncodings(fromRows:)` | Normalisasi baris charset ke key kanonik, dedup by nama |
| 1062 | 1068 | `fallbackCharacterSetEncodings()` | Baris fallback hardcoded utf8mb4/utf8/latin1 |
| 1070 | 1079 | `firstNonEmptyString(in:keys:)` | Nilai non-kosong pertama di antara key kandidat |

### `SABitFieldCommitValidation` (1085–1093)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 1087 | 1092 | `isValid(text:fieldType:nullValue:)` | Validasi nilai sel ter-commit untuk field BIT |

## 3. Temuan menonjol / UNKNOWN

**`SPFieldTypeClassifier` (843–988)** — kelas terpenting. Memusatkan aturan literal SQL sehingga `SPCopyTable.m:605,680`, `SACellFilterMenuBuilder.swift:152-153,310-311`, `SPFilterRuleTextField.swift:68-69`, dan `SPTableContent.m:2841` sepakat soal quoting/BIT. `shouldBeUnquoted` (876–885) mempercayai grouping dulu, fallback ke set lokal `unquotedFieldTypes` (850–865) — komentar mengakui grouping bisa "missing or wrong" (868–869). Jalur BIT mengubah string display `0`/`1` ke literal biner (`bitLiteral` 928–931) atau string desimal (`decimalString` 941–947) agar MySQL tak membaca `00000101` sebagai 101. `baseTypeToken` (977–987) titik normalisasi tunggal (`INT(10) UNSIGNED` → `INT`).

**`SACellEditLimit`/`SATextLimitDecision` (309–321, 436–510)** — sistem panjang edit dua lapis menghitung **code point** Unicode (unit MySQL), bukan grapheme Swift (543–547). `SACellEditLimit.evaluate` (487–509) delegasi ke `SAFieldEditorEditLimit.evaluate` (376–407) dan hanya memotong insertion, mempertahankan teks setelah caret (431–435). `SATextLimitDecision` dipakai `SPDataCellFormatter.m:222-244` via `textLimitDecision` (609–625). Pengecualian placeholder NULL (493–496, 583–586) adalah mekanisme agar user bisa mengetik "NULL" ke kolom pendek. Konsumen: `SPDataCellFormatter.m:148`, `SPFieldEditorController.m:1276,1304`.

**`SPTableLoadFailure` (990–1017)** — value object immutable dipakai sebagai state ObjC di `SPTableData.m:59-60,109,126`; dicocokkan via `matches` (1012–1016). Tak ada UNKNOWN.

**`SPCharacterSetMetadataNormalizer` (1019–1080)** — merekonsiliasi casing kolom berbeda dari server dengan menerima empat set key (1020–1023) + emit ulang key uppercase kanonik, dedup by nama (1033–1035). Dipanggil `SPDatabaseData.m:487,492`. `fallbackCharacterSetEncodings` (1062–1068) hardcode tiga encoding — **UNKNOWN** kenapa tepat tiga ini.

**`SABitFieldCommitValidation` (1085–1093)** — guard commit BIT, dipanggil `SPDataCellFormatter.m:58`. Komentar 1083–1084: sengaja **lebih ketat** dari jalur mengetik: prefiks NULL lolos saat editing tapi tak boleh di-commit sebagai literal biner. **UNKNOWN**: cabang input kosong (1090–1091) "keeps the existing save policy, which writes the bit zero" — policy itu di luar file ini.

**Lintas**: `SPProcessListRowSerializer` (238–303) dan `SPOptimizedFieldTypeEstimator` (699–841) juga dideklarasikan di sini; konsumen ObjC (`NSDictionaryExtension.swift:42`, `SPProcessListController.m:80`; `SPTableStructure.m:498-585`).

**Temuan navigasi**: file 1093 baris dengan 11 tipe tanpa satu pun `// MARK:`.
