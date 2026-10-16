# Audit: PHP-serialized Swift cluster

Audit method-by-method dua file klaster PHP-serialized. `Start`/`End` = nomor
baris di file masing-masing. Baseline commit `02c910932`.

File: `Source/Model/SAPHPSerializedValue.swift` (762 baris),
`Source/Controllers/SubviewControllers/SPFieldEditorController+PHPSerialized.swift` (711).

**Tidak ada `// MARK:`**, `TODO`, `FIXME`, `HACK`, atau `XXX` di kedua file.

---

## File 1 — `SAPHPSerializedValue.swift`

- Peran: model layer — graf node PHP `serialize()`, parser rekursif ketat, dan
  serializer dengan emisi panjang yang akurat byte.

### Tipe & konstanta
`phpSerializedParserMaximumDepth = 512` (10, privat);
`private enum PHPSerializedASCII` (12–29, 16 byte); `private func phpSerializedIntegerValue(from:)` (31–41);
`@objc enum SAPHPSerializedValueType: Int` (43–54, 10 case: null/boolean/integer/double/string/array/object/customSerialized/enum/reference);
`@objcMembers final class SAPHPSerializedEntry` (56–62); `SAPHPSerializedValue` (64–419);
`SAPHPSerializedParser` (421–762). **Tak ada `// MARK:`**; seksi hanya batas kelas.

Peta byte tipe (12–29): `colon=58`, `semicolon=59`, `quote=34`, `openBrace=123`, `closeBrace=125`,
`null=78(N)`, `boolean=98(b)`, `integer=105(i)`, `double=100(d)`, `string=115(s)`, `array=97(a)`,
`object=79(O)`, `customSerialized=67(C)`, `enum=69(E)`, `reference=114(r)`, `objectReference=82(R)`.

### Tabel definisi (urut baris)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 10 | 10 | `phpSerializedParserMaximumDepth` | Konstanta 512; batas kedalaman |
| 13–28 | | 16 `PHPSerializedASCII.*` | Byte discriminator/delimiter |
| 31 | 41 | `phpSerializedIntegerValue(from:)` | Fungsi bebas; parse integer ketat clamp ke Int |
| 66–70 | | `type`, `scalarValue`, `serializedClassName`, `referenceType`, `children` | Payload node |
| 72 | 72 | `stringEncoding` | Encoding field untuk math panjang byte |
| 75 | 82 | `value(with:)` | Factory node bertipe + default kosong |
| 85 | 88 | `normalizedIntegerString(fromEditedString:)` | Trim teks editor; nil bila bukan integer PHP |
| 90 | 111 | `isValidPHPIntegerString(_:)` | Terima minus opsional lalu digit saja |
| 113 | 128 | `isValidPHPFloatString(_:)` | Terima INF/-INF/NAN + bentuk desimal |
| 130 | 177 | `isValidPHPDecimalFloatString(_:)` | Scan byte sign/digit/fraksi/eksponen |
| 180 | 182 | `isContainer()` | True hanya array + object |
| 185 | 191 | `isScalarEditable()` | True untuk null/bool/int/double/string |
| 194 | 217 | `typeLabel()` | Teks tipe human-readable |
| 220 | 231 | `displayValue()` | String inspector; kosong untuk container |
| 234 | 266 | `nextAvailableArrayKey()` | Key integer non-negatif berikutnya yang belum dipakai |
| 269 | 296 | `uniqueObjectPropertyName()` | Nama `new_property[_N]` menghindari key existing |
| 299 | 311 | `containsReference()` | Scan rekursif node r/R |
| 313 | 319 | `setStringEncodingRecursively(_:)` | Propagasi encoding ke semua descendant |
| 321 | 328 | `data(forSerializedString:encoding:errorMessage:)` | Encode teks / catat error encoding |
| 330 | 341 | `serializedString(forKey:encoding:errorMessage:)` | Emit prefiks key `i:`/`s:` dengan panjang byte |
| 344 | 346 | `serializedString()` | Wrapper ObjC-friendly tanpa out-param error |
| 349 | 352 | `serializedString(errorMessage:)` | Resolve encoding output lalu serialisasi |
| 354 | 418 | `serializedString(using:errorMessage:)` | Serializer rekursif semua 10 tipe |
| 423–427 | | `bytes`/`position`/`recursionDepth`/`stringEncoding`/`errorMessage` | State parser |
| 430 | 432 | `parseString(_:errorMessage:)` | Wrapper UTF-8 untuk caller ObjC |
| 435 | 464 | `parseString(_:encoding:errorMessage:)` | Validasi input, parse, tolak sisa byte |
| 466 | 472 | `currentByte()` | Byte aktif, atau 0 di akhir buffer |
| 474 | 482 | `consumeByte(_:)` | Assert byte berikutnya cocok; catat error |
| 484 | 498 | `readUntilByte(_:)` | Scan ke delimiter, majukan cursor, decode ASCII |
| 500 | 522 | `readBytesAsString(_:)` | Baca payload ber-panjang dengan fallback encoding |
| 524 | 533 | `parseValue()` | Entry ber-guard depth (naik/turun) |
| 535 | 741 | `parseValueAtCurrentDepth()` | Dispatch tipe induk (grammar) |
| 743 | 761 | `unsignedIntegerValue(from:)` | Parse UInt digit-saja untuk panjang/count |

### Grammar (disitir)
- Dispatch (544–741): null → wajib colon (551) → cabang `b` (555), `i` (569), `d` (583), `s` (597), `a` (613), `O` (647), `C` (688), `E` (710), `r`/`R` (726), else unsupported (739).
- `N;` konsumsi semicolon langsung tanpa colon (544–548). `s:len:"bytes";` (597–606). `a:count:{k v}` wajib key int/string (626–629). `O:len:"cls":count:{k v}` (667–670). `C:len:"cls":payloadLen:{payload}` (688–707). `E:len:"Case";` (710–723). `r:`/`R:` id numerik wajib (726–736).
- Serializer mirror semua 10 tipe (356–417).
- **Tidak diimplementasi**: `S` (escaped string), `o` (custom object PHP 7.4 dihapus) → keduanya error unsupported (739). **UNKNOWN** apakah payload server app ini pernah mengemit varian itu.

### Error string (terlokalisasi)
323, 438, 443, 453, 458, 476, 491, 504, 520, 526, 537, 560, 574, 588, 627, 668, 739, 750, 756 (lihat pesan persis di source).

### Temuan/koreksi
1. **Panjang byte benar**: `s:` (366–369), class name `O:` (385–388), payload/class `C:` (403–408), enum case (411–414), key (336–340).
2. **Konsumsi colon bersama**: semua tipe non-null konsumsi satu colon sebelum cabang (551).
3. **Count container tak bounds-checked**: `for _ in 0..<count` `UInt` (622, 663) bergantung baca berikutnya gagal; count besar tak bisa spin karena byte habis (536–539). Observasi robustness.
4. **`readUntilByte` bisa gagal senyap**: bila `String(data:encoding:.ascii)` gagal → return nil **tanpa** set `errorMessage` (495–497) → caller lapor generik (453).
5. **Cabang kemungkinan unreachable**: fallback `readBytesAsString` berakhir `.isoLatin1` (513) yang men-decode semua byte, sehingga jalur `"Serialized string could not be decoded as text."` (520) tampak unreachable. **UNKNOWN** bila defensif.
6. **Asimetri casing float**: `INF`/`-INF`/`NAN` persis (118–120); kapitalisasi lain ditolak (122–125), tapi `Double(string)` tetap wajib (127).
7. **Serializer tak normalisasi numerik**: `d:` tulis `scalarValue` verbatim (362–363), `i:` substitusi `0` saat kosong (361), `b:` kolaps apa pun ≠ `"1"` jadi `0` (359). Perilaku round-trip presisi **UNKNOWN**.
8. **Default class name**: serializer substitusi `stdClass` (384) sedangkan `typeLabel()` tampilkan sufiks kosong (209).
9. `containsReference()` (299–311) ada agar editor bisa blokir mutasi struktural saat ada node r/R; serializer emit `referenceType` atau `r` id default 1 (416).

---

## File 2 — `SPFieldEditorController+PHPSerialized.swift`

- Peran: extension AppKit pada `SPFieldEditorController` ObjC — editor tree modal
  "Edit PHP Serialized Data as Tree" yang parse, memutasi, dan re-serialisasi field.

### Tipe & batasan
`private enum PHPSerializedEditorAssociation` (11–24, 12 key associated-object sebagai token alamat); `extension SPFieldEditorController: NSOutlineViewDataSource, NSOutlineViewDelegate` (26–711). Conformance ditambahkan ke controller bersama, jadi tiap delegate method mengecek identitas `phpSerializedOutlineView` (635, 649, 663, 673, 693, 701).

Batasan/wiring: sheet 820×520 (248), min 650×420 (254), lebar kolom key/type/value 170/130/260 (270/273/276), item popup tipe String/Integer/Float/Boolean/Array/Null (299–306), lebar outline constraint 470 (378), Apply key `"\r"` (360), auto-open delay 0.15 s (caller `SPFieldEditorController.m:429`).

### Tabel definisi (urut baris)
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 12–23 | | 12 `PHPSerializedEditorAssociation.*` | Key associated-object |
| 27–85 | | 12 computed `phpSerialized*` (menuItem/sheet/outlineView/valueTextView/typePopup/selectionLabel/updateButton/addButton/deleteButton/rootEntry/selectedEntry/automaticallyOpened) | Aksesor associated |
| 87–113 | | `phpSerializedEditTextView`/`HexTextView`/`EditSheet`/`Encoding`/`IsEditable`/`IsJSON`/`IsGeometry` | Akses KVC ke base class |
| 115 | 118 | `currentPHPSerializedText` | Teks view edit base class |
| 121 | 134 | `setupPHPSerializedEditorMenuItem(in:)` | Tambah separator + item menu tree-editor disabled |
| 137 | 142 | `resetPHPSerializedEditorState()` | Bersihkan state tree + disable item menu |
| 144 | 146 | `setPHPSerializedSheetEditData(_:)` | KVC write `sheetEditData` base class |
| 148 | 150 | `setPHPSerializedEditTextViewWasChanged(_:)` | KVC write flag dirty |
| 152 | 154 | `showPHPSerializedTooltip(_:)` | Tampilkan pesan validasi via `SPTooltip` |
| 156 | 163 | `phpSerializedEditorFont()` | Font `selectFont` base, else fallback monospaced |
| 165 | 183 | `populatePHPSerializedEditorFromCurrentText(showingError:)` | Parse teks + bungkus hasil di root entry sintetik |
| 185 | 190 | `assignPHPSerializedParent(for:)` | Wire parent link rekursif (untuk delete) |
| 193 | 197 | `refreshPHPSerializedEditorAvailability()` | Enable item menu hanya untuk data parse-able/editable/non-JSON |
| 200 | 216 | `openPHPSerializedEditorIfCurrentTextIsStructured()` | Auto-open sekali untuk container/custom payload |
| 219 | 240 | `openPHPSerializedEditor(_:)` | Populate, bangun sheet, expand tree, present sebagai sheet |
| 242 | 411 | `buildPHPSerializedEditorSheetIfNeeded()` | Bangun panel, kolom, inspector, tombol, constraint (lazy) |
| 413 | 422 | `keyLabel(for:)` | Format key `root`/`[index]`/nama properti |
| 424 | 459 | `updatePHPSerializedInspector(for:)` | Sinkron kontrol inspector + enablement ke seleksi |
| 461 | 530 | `commitPHPSerializedSelectedValue(showingError:)` | Validasi + terapkan tipe popup dan edit teks |
| 533 | 542 | `updatePHPSerializedSelectedValue(_:)` | Handler update-selected; reload tree + inspector |
| 545 | 576 | `addPHPSerializedChild(_:)` | Append child string kosong + key tergenerate |
| 579 | 598 | `deletePHPSerializedEntry(_:)` | Hapus child terpilih + reselect parent |
| 601 | 623 | `applyPHPSerializedEditor(_:)` | Commit, serialisasi ke teks field, tutup sheet |
| 626 | 632 | `cancelPHPSerializedEditor(_:)` | Akhiri sheet dengan return code cancel |
| 634 | 710 | `outlineView(_:numberOfChildrenOfItem:)`/`child:ofItem:`/`isItemExpandable:`/`objectValueFor:byItem:`/`selectionShouldChange(in:)`/`outlineViewSelectionDidChange(_:)` | Delegate/datasource outline |

### Gate commit (`commitPHPSerializedSelectedValue`, 461–530)
1. Guard no-op: return true bila tak ada seleksi / node bukan scalar-editable (462–466) — termasuk customSerialized/enum/reference/container (`SAPHPSerializedValue.swift:185–191`).
2. String → assign teks verbatim (471–473).
3. Integer → `normalizedIntegerString` else tooltip "Integer values may only contain digits and an optional leading minus sign." (475–482).
4. Float → non-empty (485–490) + `isValidPHPFloatString` else "Float values must be a valid number, INF, -INF, or NAN." (491–499).
5. Boolean → terima `1/true/yes` + `0/false/no/empty` else "Boolean values must be true/false or 1/0." (500–511).
6. Array → diblokir saat ada reference ("Cannot change serialized structure while PHP references are present.", 513–518), else tipe jadi `.array` + anak dibersihkan (519–523).
7. Fallback (Null) → reset node ke `.null` payload kosong (524–527).

Catatan: cabang Array hanya reachable dari node scalar karena guard 463, jadi `children.removeAllObjects()` (523) normalnya operasi pada list kosong. Apakah promosi scalar→Array dimaksudkan **UNKNOWN**.

### Gate keamanan reference
| Baris | Gate |
| --- | --- |
| 430 | `canEditStructure` false saat root `containsReference()` |
| 435 | Tombol Delete disabled untuk root atau tree ber-reference |
| 513–518 | Konversi Array ditolak dengan tooltip |
| 551–554 | Add child ditolak: "Cannot add entries while PHP references are present." |
| 586–589 | Delete ditolak: "Cannot delete entries while PHP references are present." |

### Temuan / UNKNOWN
1. **Tak ada stored property** — semua state via `objc_setAssociatedObject` (28–85) karena extension Swift tak bisa tambah storage; key `UInt8` nol sebagai alamat (12–23).
2. **Andalkan KVC internal ObjC** — enam property baca internal base class by string KVC (88, 92, 96, 100, 104, 108, 112); rename ivar itu akan break diam-diam.
3. **Custom/enum/reference view-only** — tak ada di popup tipe (299–306), inspector fallback ke item 0 (451), text view `isEditable = canEditScalar && …` (432) false untuk mereka, commit short-circuit (462–466).
4. **Tak ada Object di popup & tak bisa edit key** — `keyLabel` hanya format (413–422); tak bisa ganti class name object lewat sheet ini.
5. **Validasi saat ganti seleksi** — `selectionShouldChange` return hasil commit (697), jadi edit tertunda divalidasi saat klik baris lain.
6. **Auto-open one-shot + digerbangi** — butuh `automaticallyOpened == false` + item menu enabled (201–205), hanya untuk container/`.customSerialized` (208–209), set flag sebelum buka (214–215).
7. **Gate availability** — enabled hanya bila parse sukses, field editable, bukan JSON/geometry (196).
8. **Identity guard load-bearing** — karena extension menambah conformance ke controller yang mungkin host outline lain, tiap method return nilai netral bila `outlineView !== phpSerializedOutlineView` (635, 649, 663, 673, 693, 701).
9. **Par-ulang penuh** — `refreshPHPSerializedEditorAvailability` + `populatePHPSerializedEditorFromCurrentText` masing-masing re-parse seluruh teks (167, 195, 208); profil biaya pada blob besar **UNKNOWN**.
10. **UNKNOWN**: apakah base class baca `editTextView.string` vs `sheetEditData` saat return hasil sheet (base baca `sheetEditData` di `SPFieldEditorController.m:685`), jadi dual write (612–614) defensif; apakah `phpSerializedSelectedEntry` bisa dangling setelah delete untuk seleksi bersarang.
