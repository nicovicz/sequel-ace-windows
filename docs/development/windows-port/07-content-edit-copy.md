# Content: edit baris, delete/duplicate, copy, dan formatter

Kontrak source untuk separuh **edit/copy** dari tab Content: aturan tampilan sel,
commit edit inline, SQL INSERT/UPDATE/DELETE, aksi copy, dan penanganan hasil
field editor. Sitasi `file:line` merujuk checkout `02c910932`.

File: `Source/Controllers/MainViewControllers/TableContent/SPTableContent.m`
bagian editing/insert/delete/duplicate/copy/format; `Source/Views/TableViews/SPCopyTable.m/.h`
(1807 baris); `Source/Views/SPDataCellFormatter.m/.h`; `SAFieldEditorCommitPolicy.swift`;
`SPDataAdditions.m/.h`; `SPDataStorage.m/.h`; `SPStringAdditions.m`;
`StringExtension.swift` (`SPFieldTypeClassifier`).

## 1. Aturan tampilan sel

**Placeholder NULL** — key `SPNullValue = @"NullValue"` (`SPConstants.m:135`), default
`"NULL"` (`PreferenceDefaults.plist:174-175`).
- Non-edit: `value isNSNull` → `[prefs objectForKey:SPNullValue]` (`SPTableContent.m:4407-4409`).
- Abu: `willDisplayCell` set `nullHighlightColor` untuk sel null/unloaded tak terpilih (`5135-5137`).
- Record view: placeholder sama (`4628-4630`).
- Copy pakai `nullString` (`SPCopyTable.m:257,276 / 394,414 / 805,817 / 893,896`).

**Binary/BLOB hex** — digerbangi key `SPDisplayBinaryDataAsHex = @"DisplayBinaryDataAsHex"`
(default `false`, `PreferenceDefaults.plist:224-225`), dibaca di `cellValueIsDisplayedAsHexForColumn:`
(`4533-4551`) yang mengembalikan YES untuk `typegrouping` `binary` atau `blobdata`.
- Display: `[NSString stringWithFormat:@"0x%@", [data dataToHexString]]` (`4421`), dipotong 255 byte + `…` bila lebih (`4418-4419`).
- `dataToHexString` uppercase `%02X` (`SPDataAdditions.m:331-344`).
- Binary non-hex: sel diedit → `stringRepresentationUsingEncoding:` (`4426`); selain itu
  `shortStringRepresentationUsingEncoding:` (`4428`).

**Geometry** — `isKindOfClass:[SPMySQLGeometryData class]` → `[value wktString]`
(`4403-4405`, `4625-4627`; `SPCopyTable.m:330, 834, 925`).

**Not-loaded** — `isSPNotLoaded` → `NSLocalizedString(@"(not loaded)")` (`4432-4433`, `4631-4633`).

**Truncation/preview**: `objectValueForTableColumn:` pakai `_contentValueForTableColumn:…asPreview:YES`
(panjang preview 150) kecuali sel sedang diedit (`4395-4400`, `4555-4562`);
`cellPreviewAtRow:…previewLength:` → `"<147 chars>..."` untuk string >150 (`SPDataStorage.m:189-191`);
saat loading sel render `@"..."` (`4392`); `shortStringRepresentationUsingEncoding:` batasi 255
char + fallback ASCII; `-- cannot display --` bila tak terdekode (`SPDataAdditions.m:561-573`).

**Long/multi-line → sheet vs inline** — `shouldUseFieldEditorForRow:column:checkWithLock:`
(`1438-1513`): `SPEditInSheetEnabled` → YES (`1441`); `typegrouping=="blobdata"` → YES (`1463-1464`);
NULL → NO (`1494-1498`); `SPEditInSheetForLongText` dan panjang > `SPEditInSheetForLongTextLengthThreshold`
(default 15, `PreferenceDefaults.plist:108-109`) → YES (`1501-1503`); `SPEditInSheetForMultiLineText`
dan ada newline → YES (`1506-1508`).

**Transformasi tampilan** — `SPDataCellFormatter attributedStringForObjectValue:` memetakan
`\n`→`¶`, `\r/U+0085/U+000B/U+000C`→`⁋` (`SPDataCellFormatter.m:92-108`);
`editingStringForObjectValue:` mengembalikan raw (`51-54`).

## 2. Alur commit edit inline & SQL UPDATE

**Entry/gate** — `tableView:shouldEditTableColumn:row:` (`4826-4957`): abort bila `isWorking`
(`4828`), bila `fieldEditor` terbuka (`4835`), bila `argumentForRow` kosong (`4839-4841`);
muat sel not-loaded via `SELECT %@ FROM %@ WHERE %@` (`4847-4861`); blokir kolom generated
(`4864-4867`); delegate ke sheet bila `shouldUseFieldEditorForRow` (`4873-4951`).

**begin-edit** — `control:textShouldBeginEditing:` (`5286-5355`): tooltip error berdasarkan
`numberOfPossibleUpdateRows` (`5299-5332`); bila sheet diperlukan, simpan `fieldEditorSelectedRange`,
`abortEditing`, buka sheet (`5337-5354`).

**Commit** — `tableView:setObjectValue:forTableColumn:row:` (`4442-4531`):
- combo-box konsultasikan `SAComboBoxSelectionTracker` (`4449-4455`);
- `SAFieldEditorCommitPolicy shouldIgnoreInlineCommitWithFieldEditorRequired:…` bisa return awal (`4473-4480`);
- view → `saveViewCellValue:` (`4483-4486`);
- mulai capture editing `oldRow`/`isEditingRow`/`currentlyEditingRow` (`4494-4499`);
- placeholder NULL → `[NSNull null]` bila kolom nullable (`4509-4511`);
- sel ditampilkan hex → `[NSData sp_dataWithHexString:]`, beep+abort bila invalid (`4512-4522`);
- simpan via `replaceObjectInRow:column:withObject:` (`4524`); objek kosong → `@""` (`4526-4528`).
- `control:textShouldEndEditing:` (`5247-5279`): formatter `getObjectValue:` (`5256-5258`); validasi hex (`5259-5276`).

**Menyimpan** — `saveRowOnDeselect` (`3225-3249`) → `endEditingFor:` (`3235`) → `saveRowToTable`
(`3242`). `saveRowToTable` (`3136-3217`): view return NO; baris identik short-circuit (`3145-3148`);
panel konfirmasi bila `SPQueryWarningEnabled` (`3156-3197`, diringkas ke `SPMaxQueryLengthForWarning`=1000,
`SPConstants.m:60`).

**Generasi SQL** — `deriveQueryString` (`3013-3124`):
- Skip sel `isSPNotLoaded` (`3030`) dan sel tak berubah untuk baris existing (`3034`).
- Aturan nilai: NULL saat `isNSNull`, atau float/integer/date kosong + nullable (`3043-3047`);
  geometry → `getGeomFromTextString` (`3050-3051`); `NSNumber` → `stringValue` (`3058-3059`);
  `NSData` → `escapeAndQuoteData:` (`3062-3063`); `isfunction` passthrough (`3066-3067`);
  bit → `b'%@'` (`3068-3069`); date `NOW()` (`3070-3071`); string `UUID()`/`UUID_v4()` passthrough
  (`3072-3073`); selain itu `escapeAndQuoteString:` (`3075`). Kolom `generatedalways` dikecualikan (`3085-3088`).
- INSERT (baris baru): `@"INSERT INTO %@ (%@) VALUES (%@)"` (tabel backtick, field `componentsJoinedAndBacktickQuoted`, nilai `", "`) (`3097-3098`).
- UPDATE: `@"UPDATE %@ SET "` + per field `@"%@ = %@"` join `", "` (`3106-3111`), lalu
  `@" WHERE %@"` dari `argumentForRow:-2` (`3112-3118`); WHERE kosong → `NSBeep` + return kosong (`3113-3117`).

**Identifikasi baris (WHERE)** — `argumentForRow:excludingLimits:` (`3292-3402`):
- Key = kolom dengan `Key == "PRI"` dari `SHOW COLUMNS FROM %@` (`3302-3316`); bila tak ada,
  semua kolom + `LIMIT 1` (`3319-3321`) dan blokir blob tersembunyi saat `SPLoadBlobsAsNeeded` (`3325-3326`).
- `row >= 0` pakai sel saat ini; `row < 0` pakai `oldRow` (`3347-3353`).
- Per key: `isNSNull` → `"col IS NULL"` (`3355-3356`); `isSPNotLoaded` → abort (`3358-3360`);
  BIT → `escapeString:includingQuotes:NO` + `fmt = @"b'%@'"` (`3370-3373`); geometry →
  `escapeAndQuoteData:[data data]` (`3374-3376`); `NSData` → UUID string khusus selain `escapeAndQuoteData:`
  (`3378-3385`); selain itu `escapeAndQuoteString:` (`3386-3388`); append `@"%@ = %@"` (`3395`);
  `" LIMIT 1"` saat `setLimit && !excludeLimits` (`3399`).

## 3. Add / duplicate / delete

**Add row** — `addRow:` (`2054-2096`): bangun `newRow` dari `default` kolom — nil/`isNSNull`
→ `[NSNull null]` (`2065-2066`); default kosong + numeric/bit non-null → `@"0"` (`2067-2073`);
default bit `b'...'` dibuang (`2074-2076`); selain itu default (`2078`). In-memory saja;
INSERT dibuat lambat oleh `deriveQueryString`.

**Duplicate row** — `duplicateRow:` (`2101-2171`): tolak >1 terpilih (`2114-2117`); salin baris
dari storage (`2121`); dengan `SPLoadBlobsAsNeeded` fetch ulang: `SELECT * FROM %@ WHERE %@` (`2133`),
`SHOW COLUMNS FROM %@` (`2138`); `auto_increment` → `NSNull` (`2146-2147`); blob/text diisi ulang
(`2149-2153`); sisip di selected+1 (`2157`).

**Delete row** — `removeRow:` (`2176-2565`):
- Delete all: `DELETE FROM %@` (`2286`).
- Tanpa PK usable: per baris `DELETE FROM %@ WHERE %@` (`2380`); PK uniqueness diprobe dengan
  `SELECT COUNT(1) FROM %@` (`2356`) dan `SELECT * FROM %@ GROUP BY %@ LIMIT %ld,%ld` (`2364`).
- PK tunggal: `DELETE FROM %@ WHERE %@ IN (` (`2403`), nilai `escapeAndQuoteData:`/`escapeAndQuoteString:`
  (UUID khusus, `2413-2423`), separator `,` (`2438`), chunk 256000 char (`2428`), tutup `)"` (`2448`).
- Multi PK: `DELETE FROM %@ WHERE ` + `(%@)` join OR (`2462-2486`), chunk 64000 (`2477-2484`),
  `" OR "` akhir dipangkas (`2497`).

## 4. Aksi copy

**Tag menu** — `SPEditMenuCopy`=2001, `CopyWithColumns`=2002, `CopyAsSQL`=2003,
`CopyAsSQLNoAutoInc`=2004 (`SPCopyTable.m:57-60`); judul di `MainMenu.xib:205-217`
(`Copy with Column Names`, `Copy as SQL INSERT`, `Copy as SQL INSERT (no auto_inc)`).

**`copy:`** (`189-220`):
- As SQL → `rowsAsSqlInsertsOnlySelectedRows:YES skipAutoIncrementColumn:NO/YES skipGeneratedColumn:YES`
  (`196,199`); pasteboard `NSPasteboardTypeString` (`203-206`).
- Selain itu → `rowsAsTabStringWithHeaders:…onlySelectedRows:YES blobHandling:kBlobInclude` (`209`);
  deklarasikan `NSPasteboardTypeString` + `NSPasteboardTypeTabularText` (`214-217`).

**Tab copy** — `rowsAsTabStringWithHeaders:…` (`226-357`): header opsional join `"\t"` berakhir
`"\n"` (`241-249`); kolom per urutan terlihat (`252-254`); NULL→`nullString`+`\t` (`275-276`);
not-loaded→`"(not loaded)"` (`277-278`); `SABaseFormatter` (`279-283`); `NSData` hex `0x…` atau
decode dengan fallback ISO-Latin-1 (`284-295`); mode blob-as-file (`296-315`) selain `"BLOB\t"`;
geometry `wktString` (`317-331`); selain itu `description` dengan `\n`→`↵`, `\t`→`⇥` (`334`);
tab akhir dibuang, `"\n"` per baris (`343-346`), newline akhir dipangkas (`350-352`).

**CSV copy** — `rowsAsCsvStringWithHeaders:…` (`363-495`): header dibungkus `"…"`, `"` internal→`""` (`383`);
nilai dibungkus `"…",` dipisah koma; NULL dibungkus quote (`414`); formatter (`417-420`);
`NSData` hex/decoded (`422-432`); geometry (`455-468`); selain itu `description` dibungkus quote
**tanpa** escape quote internal (`472`); koma akhir dibuang, `"\n"` per baris (`481-484`).

**INSERT copy** — `rowsAsSqlInsertsOnlySelectedRows:skipAutoIncrementColumn:skipGeneratedColumn:` (`507-760`):
- Pass kolom (`541-619`): skip generated bila diminta (`550`); deteksi auto-increment (`553-587`);
  klasifikasi `kColType` =0 numeric/BIT (`605`), =2 blobdata/textdata (`608`), =3 geometry (`611`), =1 default.
- Awal: `@"INSERT INTO %@ (%@)\nVALUES\n"` dengan `selectedTable` atau `"<table>"`, `componentsJoinedAndBacktickQuoted:` (`628-629`).
- Per baris `"\t("` (`637`); NULL→`"NULL"` (`669-671`); numeric/bit→`SPFieldTypeClassifier.unquotedSQLLiteralForValue:` (`680`);
  string/text/blob: UUID/textdata/string `NSData` decode UTF-8 (`692-699`), `NSData`→`escapeAndQuoteData:` (`701-702`),
  selain itu `escapeAndQuoteString:[cellData description]` (`704`); geometry→`escapeAndQuoteData:[cellData data]` (`710`);
  join `", "` (`727`); tutup `"),\n"` (`740,744`).
- Sel not-loaded di-fetch via `SELECT %@ FROM %@ WHERE %@` (`661-665`); abort bila WHERE kosong (`655-658`).
- Chunking: blok `INSERT INTO…` baru saat >250000 char (`732-737`); `",\n"` akhir dipangkas (`752-755`); `";\n"` final (`757`).
- `SPFieldTypeClassifier`: set `unquotedFieldTypes` dan `shouldBeUnquoted`/`unquotedSQLLiteral` (BIT → `b'…'`) (`StringExtension.swift:850-931`).

**Helper tampilan sel copy** — `displayStringForRow:column:` (`865-928`): NULL→nullString;
not-loaded; formatter; hex `0x…`; `NSData` decoded (fallback ISO-Latin-1); geometry `wktString`;
selain `description`. `isNullAtRow:column:` (`930-942`).

**Copy nilai sel (context menu)** — `SACellValueCopyCoordinator.appendItemsToMenu:…`
(`SACellFilterAction.swift:69-170`): kumpulkan `rawValues`; deskriptor "Copy '<col>' Values"
join `"\n"` dan "Copy '<col>' Values as SQL" join `", "` (`SACellFilterMenuBuilder.swift:117-118, 77, 123`);
`sqlLiteral` NULL→`"NULL"`, unquoted numeric/BIT, UUID/textdata/string decode UTF-8 selain
`escapeAndQuoteData` (`:144-166`).

## 5. Hasil field editor (sheet) & coercion tipe

**Membuka sheet** — `shouldEditTableColumn:` (`4904-4948`): set `colName`/`usedQuery`/`tableSource`,
`setTextMaxLength:fieldLength`, `setFieldType:`, `setFieldEncoding:` (charset kecuali binary),
`setAllowNULL:` (`4896-4898`); kolom hex → sheet non-editable dengan limit panjang penuh (`4927-4930`);
`editWithObject:…isObjectBlob:isEditable:withWindow:sender:contextInfo:` dengan `rowIndex`/`columnIndex` (`4941-4948`).

**Hasil** — `processFieldEditorResult:contextInfo:` (`3543-3593`): baca `rowIndex`/`columnIndex` (`3549-3552`);
capture `oldRow`+state (`3557-3561`); placeholder NULL → `[NSNull null]` bila nullable (`3563-3567`);
view → `saveViewCellValue:forTableColumn:row:` (`3569-3575`); tabel →
`replaceObjectInRow:…withObject:[data copy]` + `_updateRecordView` (`3578-3580`).

**Coercion saat simpan**:
- `saveViewCellValue:` (`3595-3707`): `NSNumber`→`stringValue`; `NSData`→`escapeAndQuoteData:`;
  `CURRENT_TIMESTAMP` passthrough; NULL placeholder→`"NULL"`; geometry→`getGeomFromTextString`;
  bit→`b'%@'`; date `NOW()`; selain itu `escapeAndQuoteString:`. SQL:
  `@"UPDATE %@.%@ SET %@.%@.%@ = %@ %@"` (`3652-3654`).
- Row-level `deriveQueryString` coercion (`3041-3088`).
- Formatter: `SPDataCellFormatter getObjectValue:forString:errorDescription:` validasi BIT via
  `SABitFieldCommitValidation` (`SPDataCellFormatter.m:56-64`).
- Penekan inline-commit: `SAFieldEditorCommitPolicy.shouldIgnoreInlineCommit…` YES untuk
  `.deferred`/`.invalidated`, atau saat sheet field-editor diperlukan kecuali combo box commit
  nilai berubah yang cocok stored/display (`SAFieldEditorCommitPolicy.swift:212-238`); provenance
  dilacak `SAComboBoxSelectionTracker` (`35-206`).

## 6. String yang terlihat pengguna (separuh edit/copy)

**SPTableContent.m**: `"You can only copy single rows."` (`2115`); dialog delete:
`"Delete Selected Row"`/`"Delete Selected Rows"`/`"Delete ALL ROWS IN TABLE"` (`2187-2193`),
`"Reset AUTO_INCREMENT after deletion\n(only for Delete ALL ROWS IN TABLE)?"` (`2208`),
`"Are you sure you want to delete the selected row from this table? This action cannot be undone."` (`2214`),
`"Are you sure you want to delete the selected %ld rows from this table? This action cannot be undone."` (`2218`);
`"Double checking as you have 'Show warning before executing a query' set in Preferences"` (`2254`);
`"Couldn't delete rows.\n\nMySQL said: %@"` (`2306`); `"Unexpected number of rows removed!"` (`2512`),
`"One additional row was removed!"` (`2517`), `"One row was not removed."` (`2527`),
`" You should also add a primary key to this table!"` (`2534`); `"/* WARNING: No rows have been affected */\n"` (`2906`);
`"Unable to write row"` + `"MySQL said:\n\n%@"` + tombol `"Edit row"`/`"Discard changes"` (`2994`);
`"Do you really want to proceed with this query?\n\n %@"` (`3168`); `"You can't hide blob and text fields when working with tables without index."` (`3326`);
`"Field is not editable. Field has no or multiple table or database origin(s)."` (`3607`);
`"Couldn't write field.\nMySQL said: %@"` (`3659`); `"The current field \"%@\" is a generated column and therefore cannot be edited."` (`4865`);
`"Invalid hexadecimal value"` + `"A valid hex string may only contain the numbers 0-9 and letters A-F (a-f). …"` (`5268-5272`).

**SPCopyTable.m**: `"Cannot find auto_increment column name"` (`572`); `"Filter by Selected Value"` (`147`); `"Bundles"` (`1164`).

**SPDataCellFormatter.m**: `"For BIT fields only “1” or “0” are allowed."` (`59,196`);
`"Maximum text length is set to %ld."` (`178,231`); `"Maximum text length is set to %ld. Inserted text was truncated."` (`171,240`).

**SACellFilterMenuBuilder.swift**: `"Copy %@ Values"` (`117`); `"Copy %@ Values as SQL"` (`123`).

**SPDataAdditions.m**: `"-- cannot display --"` (`566`).

## 7. UNKNOWN

- Logika keputusan panjang/BIT ada di Swift (`SACellEditLimit`, `SATextLimitDecision`) yang
  direferensikan dari `SPDataCellFormatter.m:148-153, 224` — di luar focus set.
- Delete/duplicate view: `removeRow:`/`addRow:` guard pada `SPTableTypeTable` (`2059, 3139`);
  sel view disimpan via `saveViewCellValue:`.
- INSERT hasil `duplicateRow:` diproduksi lambat oleh `deriveQueryString` saat save; tidak ada
  string SQL duplicate khusus.
