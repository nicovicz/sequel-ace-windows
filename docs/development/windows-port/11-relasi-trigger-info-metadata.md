# Relasi, trigger, Table Info, dan cache metadata tabel

Kontrak source untuk tab **Relations**, **Triggers**, pane **Table Info**,
tab **Table Info** penuh, dan lapisan cache metadata `SPTableData`. Sitasi
`file:line` merujuk checkout `02c910932`.

## 1. Inventaris komponen

| Komponen | File | Peran |
| --- | --- | --- |
| `SPTableData` | `Source/Controllers/DataControllers/SPTableData.{h,m}` | Cache metadata per tabel: kolom, constraint, trigger, status, encoding, CREATE syntax |
| `SPTableRelations` | `.../TableRelations/SPTableRelations.{h,m}` | Tab Relations: UI daftar FK + SQL add/remove FK |
| `SAForeignKeyReferenceRuleSupport` | `.../TableRelations/SAForeignKeyReferenceRuleSupport.swift` | Logika capability rule referensi FK MySQL 8.4+ |
| `SPTableTriggers` | `.../TableTriggers/SPTableTriggers.{h,m}` | Tab Triggers: UI daftar + SQL add/edit/delete trigger |
| `SPTableInfo` | `Source/Controllers/SubviewControllers/SPTableInfo.{h,m}` | Pane bawah "TABLE INFORMATION" + "ACTIVITIES" |
| `SPExtendedTableInfo` | `.../SPExtendedTableInfo.{h,m}` | Tab "Table Info" penuh (engine/encoding/collation/rows/size/comment/CREATE syntax) |
| `SPServerSupport` | `Source/Controllers/DataControllers/SPServerSupport.{h,m}` | Flag capability murni dari perbandingan versi |
| `SPDatabaseStructure` | `Source/Controllers/DataControllers/SPDatabaseStructure.{h,m}` | Cache struktur DB untuk navigator/autocompletion |
| `SPDatabaseData` | `Source/Controllers/DataControllers/SPDatabaseData.{h,m}` | Cache encoding/collation/storage engine/variabel server |
| `DatabaseProcessList.xib` | `Source/Interfaces/DatabaseProcessList.xib` | Owner `SPProcessListController` |
| `DatabaseServerVariables.xib` | `Source/Interfaces/DatabaseServerVariables.xib` | Owner `SPServerVariablesController` |

UI Relations/Triggers/TableInfo ada di dalam `Source/Interfaces/DBView.xib`
(tidak ada xib terpisah).

`SPTableLoadFailure` (dipakai `SPTableData` untuk negative caching) adalah kelas
Swift: `Source/Other/Extensions/StringExtension.swift:990-1017`.

## 2. Query pembacaan metadata

### 2.1 Kolom/constraint (relasi)

`SPTableData` menurunkan constraint dengan **memparse `SHOW CREATE TABLE`**,
bukan query `information_schema`:

- `SHOW CREATE TABLE `db`.`table`` bila ada database — `SPTableData.m:556`.
- `SHOW CREATE TABLE `table`` tanpa database — `SPTableData.m:559`.
- Untuk `information_schema`, CREATE disintesis dari `SHOW COLUMNS`:
  - `SHOW COLUMNS FROM `table`` — `SPTableData.m:652`.
  - `CREATE TEMPORARY TABLE `table` (\n...` per kolom `\`Field\` Type NULL DEFAULT,\n` — `SPTableData.m:660, 686`; dipotong via `dropSuffixWithSuffix:@",\n"` lalu `)` — `693-694`.
- View: `SHOW CREATE TABLE `view`` (`1042`), `SHOW COLUMNS FROM `view`` (`1083`).
- Ekstraksi FK dari CREATE terparse: `SPTableData.m:814-919` (`CONSTRAINT ... FOREIGN KEY (...) REFERENCES ...`); regex target referensi `` ^`([\w_.]+)`\.`([\w_.]+)`$ `` — `SPTableData.m:835`.
- Parser kolom (tipe/length/unsigned/zerofill/binary/charset/collate/NULL/AUTO_INCREMENT/DEFAULT/ON UPDATE/COMMENT/GENERATED): `SPTableData.m:1385-1596`; `SPCurrentTimestampPattern` (`SPConstants.m:526`).
- Encoding dari CREATE: `CHARSET=`/`CHARACTER SET=` — `974-976`; `DEFAULT` → `tableDocumentInstance.databaseEncoding` (`986-987`); tanpa klausa → encoding DB (`994-995`) selain itu `@"latin1"` (`997`).

### 2.2 Status tabel

`SPTableData.m:1183-1201` per `SPTableType`:

- `SPTableTypeProc` (`1186`): `SELECT * FROM information_schema.ROUTINES AS r WHERE r.SPECIFIC_NAME = '...' AND r.ROUTINE_SCHEMA = '...' AND r.ROUTINE_TYPE = 'PROCEDURE'`
- `SPTableTypeFunc` (`1191`): idem dengan `ROUTINE_TYPE = 'FUNCTION'`
- `SPTableTypeView` (`1196`): `SELECT * FROM information_schema.VIEWS AS r WHERE r.TABLE_NAME = '...' AND r.TABLE_SCHEMA = '...'`
- `SPTableTypeTable` (`1200`): `SHOW TABLE STATUS LIKE '...'` (dengan `assertingDatabaseContext:`)

Sanitisasi injeksi: escape backslash/quote `1177-1179`; regex backslash `:1199`.
Post-processing view menambah pseudo-key `Engine="View"`,
`Comment="No status information is available for views."`, `Name`, `Collation`
(dari `COLLATION_CONNECTION`), `CharacterSetClient` (dari `CHARACTER_SET_CLIENT`)
— `1259-1266`. MySQL <4.1 `Type` di-remap ke `Engine` (`1225-1227`).

### 2.3 Index (untuk validasi referensi FK)

- `SPTableRelations.m:592`: `SHOW INDEX FROM `db`.`table`` (atau `SHOW INDEX FROM `table`` tanpa db, `581-584`).
- Diparse oleh `SAForeignKeyReferenceRuleSupport.singleColumnUniqueReferenceColumns` (`:13-37`) — baca kolom `Non_unique`, `Key_name`, `Sub_part`, `Column_name`.

### 2.4 Trigger

- `SPTableData.m:1292-1293`: `` /*!50003 SHOW TRIGGERS WHERE `Table` = '...' */ `` (nama tabel via `tickQuotedString`), dengan `assertingDatabaseContext:`.

### 2.5 Hitung baris akurat

- `SELECT COUNT(1) FROM `table`` — `SPTableData.m:1364` dan `1239` (dipakai bila `SHOW TABLE STATUS` melaporkan `Rows` `NSNull`).

### 2.6 Server support / capability

- `SPTableRelations.m:567`: `SELECT @@session.restrict_fk_on_non_standard_key`.
- `SPDatabaseData.m:454`: `SHOW VARIABLES LIKE '...'` (satu variabel via `tickQuotedString`).

### 2.7 Struktur database (navigator/autocomplete)

- `SPDatabaseStructure.m:318`: `SHOW FULL COLUMNS FROM `table` FROM `db``.
- `SPDatabaseStructure.m:395`: `` SELECT SPECIFIC_NAME, ROUTINE_TYPE, DTD_IDENTIFIER, IS_DETERMINISTIC, SQL_DATA_ACCESS, SECURITY_TYPE, DEFINER FROM `information_schema`.`ROUTINES` WHERE `ROUTINE_SCHEMA` = '...' `` (hanya bila mayor server ≥ 5, `386`).

### 2.8 Encoding/collation/storage engine

- `SELECT * FROM `information_schema`.`collations` ORDER BY `collation_name` ASC` (`141`); fallback `SHOW COLLATION` (`145`).
- `SELECT * FROM `information_schema`.`collations` WHERE character_set_name = '...' ORDER BY `collation_name` ASC` (`182`); retry alias utf8/utf8mb3 (`188`, `557-562`); fallback `SHOW COLLATION WHERE `Charset` = '...'` (`572`).
- `SELECT * FROM `information_schema`.`character_sets` ORDER BY `character_set_name` ASC` (`309`); fallback `SHOW CHARACTER SET` (`313`); fallback hardcoded (`318`, `SPCharacterSetMetadataNormalizer.fallbackCharacterSetEncodings`, `StringExtension.swift:1061+`).
- `SHOW TABLES IN information_schema LIKE 'ENGINES'` (probe akses) (`276`).
- `SELECT Engine, Support FROM `information_schema`.`engines` WHERE SUPPORT IN ('DEFAULT', 'YES') AND Engine != 'PERFORMANCE_SCHEMA'` (`282`).
- Variabel engine default per versi: `table_type`/`storage_engine`/`default_storage_engine` (`416-425`).

## 3. Model cache `SPTableData` dan invalidasi

### 3.1 Ivar cache (`SPTableData.h:39-53`)

`columns`, `columnNames`, `constraints`, `triggers`, `status`, `primaryKeyColumns`,
`tableEncoding`, `tableCreateSyntax`, `tableHasAutoIncrementField`, plus
`dataProcessingLock` (`pthread_mutex_t`, init `SPTableData.m:85`).

### 3.2 Aturan populasi (lazy, saat aksesor pertama dipanggil)

- `tableEncoding` (`137-150`), `tableCreateSyntax` (`155-174`), `columns` (`179-193`), `columnNames` (`249-261`), `columnAtIndex:` (`268-281`), `columnWithName:` (`229-244`), `columnIsBlobOrText:` (`289-303`), `columnIsGeometry:` (`311-325`), `primaryKeyColumnNames` (`1601-1617`): semua fallback ke `updateInformationForCurrent{Table,View}` bila array relevan kosong.
- `getConstraints` (`198-201`): **mengembalikan cache saja, tanpa lazy load.**
- `triggers` (`206-222`): bila nil dan tipe == `SPTableTypeTable` → `updateTriggersForCurrentTable`; selain itu `@[]`.
- `statusValueForKey:` (`332-341`) / `statusValues` (`359-368`): bila `status` kosong → `updateStatusInformationForCurrentTable`.
- `updateInformationForCurrentTable` (`423-473`): bersihkan `columns`/`columnNames`/`constraints`, reset `tableHasAutoIncrementField=NO` dan `primaryKeyColumns`; lalu `informationForTable:fromDatabase:`; salin hasil.
- `updateInformationForCurrentView` (`479-521`): analog.
- `informationForTable:` membersihkan `constraints` lebih dulu demi tidak membocorkan constraint tabel sebelumnya (issue 1206) — `546-549`.
- `parseFieldDefinitionStringParts` menyetel `tableHasAutoIncrementField = YES` saat `AUTO_INCREMENT` — `1553-1555`.

### 3.3 API invalidasi

- `resetAllData` (`373-392`): bersihkan `columns`, `columnNames`, `status`, kedua failure record, nil `triggers`, `tableEncoding`, `tableCreateSyntax`.
- `resetStatusData` (`397-403`): hanya `status` + failure record status.
- `resetColumnData` (`408-417`): hanya `columns`, `columnNames` + failure record table-info.
- `setStatusValue:forKey:` (`351-354`): tulis langsung nilai (lebih akurat) dari luar.
- `_loopWhileWorking` (`1631-1635`): spin pada `pthread_mutex_trylock` + `usleep(10000)`; setiap aksesor publik memanggilnya dulu.

### 3.4 Negative caching

`SPTableLoadFailure` (triple tabel+database+tipe, `StringExtension.swift:990-1016`)
dicatat saat pengambilan CREATE/status gagal (`107-110, 124-127, 570-572, 620-622,
1049, 1074, 1088, 1207`) dan membuat `_shouldSkipTableInformationLoadForTable:` /
`_shouldSkipStatusLoadForTable:` kembali lebih awal (`100-105, 117-122`; dipakai di
`441-445, 486, 1164-1167`). Dibersihkan saat sukses (`633, 1136, 1220`) dan oleh
`resetAllData`/`resetStatusData`/`resetColumnData`.

### 3.5 Situs invalidasi eksternal

- Ganti tabel: `SPDatabaseDocument.m:5749` `[tableDataInstance resetAllData]` di `_loadTableTask`, lalu `updateStatusInformationForCurrentTable` (`5767`) dan `updateAccurateNumberOfRowsForCurrentTableForcingUpdate:NO` (`5836`); dispatch tab di `5800-5821`.
- `SPTableRelations.m:641, 708-709, 718`; `SPTableTriggers.m:540-541`; `SPExtendedTableInfo.m:108`; `SPTableContent.m:1547-1548`; `SPTablesList.m:2514`; `SPIndexesController.m:814, 888`; `SPTableStructure.m:845/859/861/1017/1617/1887/2324`.
- `SPDatabaseDocument.m:5106, 5239` mereset `databaseDataInstance` (`SPDatabaseData.resetAllData`, `SPDatabaseData.m:105-127`).

### 3.6 Threading

Semua mutasi cache di bawah `dataProcessingLock`; semua query sementara memaksa
`utf8mb4` bila encoding koneksi tidak diawali `utf8` (`storeEncodingForRestoration`/
`restoreStoredEncoding`): `534/541-544/635`, `1029/1036-1039/1135`, `1151/1170-1174/1269`,
`1286-1290/1309`.

## 4. Tab Relations

### 4.1 Key cache

`SPTableRelations.m:43-49`: `name`, `columns`, `fk_database`, `fk_table`, `fk_columns`,
`on_update`, `on_delete`. Key constraint dari parser: `name`, `columns`, `ref_database`,
`ref_table`, `ref_columns`, `update`, `delete` (`SPTableData.m:820/831/838/839/843/855/864/877`).

### 4.2 Kolom tabel (DBView.xib, `TableRelationsTableView` id 5548)

| identifier | header | xib |
| --- | --- | --- |
| `name` | Name | `2206-2207` |
| `columns` | Columns | `2218-2219` |
| `fk_database` | FK Database | `2230-2231` |
| `fk_table` | FK Table | `2242-2243` |
| `fk_columns` | FK Columns | `2254-2255` |
| `on_update` | On Update | `2266-2267` |
| `on_delete` | On Delete | `2278-2279` |

`fk_database` diwarnai abu bila sama dengan DB aktif (`395-398`). Data printing
(`relationDataForPrinting`) memuat name/columns/fk_table/fk_columns/on_update/on_delete
(`441-470`).

### 4.3 Jalur load

`_refreshRelationDataForcingCacheRefresh:` (`629-662`): bersihkan `relationData` +
`takenConstraintNames`; hanya untuk `SPTableTypeTable` (`639`); opsional
`updateInformationForCurrentTable` (`641`); baca `[tableDataInstance getConstraints]`
(`643`); petakan tiap constraint, default `ref_database` = DB aktif (`650`) dan
`update`/`delete` = `@""` (`653-654`); catat nama constraint lowercase di
`takenConstraintNames` (`657`) untuk validasi nama duplikat (`512-528`, teks merah +
tombol Add disabled).

### 4.4 Caveat engine/view

`tableSelectionChanged:` (`350-381`):

- Relations aktif hanya bila `statusValueForKey:@"Engine"` lowercase == `"innodb"` **dan**
  `tablesListInstance.tableType == SPTableTypeTable` (`363`).
- Label aktif: `"Relations for table: %@"` (`366`).
- Disabled: tombol Add/Refresh + table view disabled (`373-375`); label
  `"This table currently does not support relations. Only tables that use the InnoDB storage engine support them."` untuk tabel nyata, `@""` untuk lainnya (`377`).

### 4.5 Alur pemilihan DB/tabel/kolom referensi

1. `addRelation:` (`267-308`): guard `isWorking`/non-tabel (`270`); judul box `"Table: %@"` (`273`); isi popup kolom sumber dari `[tableDataInstance columnNames]`, opsi sort alfabetis (`276-277`, `SPAlphabeticalTableSorting`); isi `refDatabasePopUpButton` dari `[tableDocumentInstance allDatabaseNames]` (`291`), pilih DB aktif (`293`); reset `constraintName`/ON DELETE/ON UPDATE ke index 0 (`299-301`); `_updateAvailableTables` (`296`); buka sheet (`307`).
   - `allDatabaseNames` = database **user** saja (`SPDatabaseDocument.m:936-939`), diisi dari `partition.userDatabases` (`:620`); DB sistem array terpisah (`619, 944-947`).
2. `selectReferenceDatabase:` (`249-254`) → `_updateAvailableTables` lalu `_updateAvailableTableColumns`.
3. `_updateAvailableTables` (`669-696`): disable popup tabel/kolom + confirm (`677-679`); jalankan `SELECT table_name FROM information_schema.tables WHERE table_type = 'BASE TABLE' AND engine = 'InnoDB' AND table_schema = '...' ORDER BY table_name ASC` (`684`); tambah item (`687-689`); enable kembali (`692-694`).
4. `selectReferenceTable:` / `selectTableColumn:` (`241-262`) → `_updateAvailableTableColumns`.
5. `_updateAvailableTableColumns` (`702-749`): bersihkan + reload cache kolom tabel aktif (`708-709`); ambil `columnInfo` kolom sumber (`711`); reset cache + load info tabel referensi (`718-719`); kumpulkan kolom kandidat yang `type`-nya sama (`732-737`), dan — bila server mensyaratkan referensi standar — namanya ada di himpunan unique single-column (`723-725, 734`); isi popup dan enable confirm hanya bila non-kosong (`741-748`).

### 4.6 ON UPDATE / ON DELETE (add)

`confirmAddRelation:` (`145-236`):

- Validasi kolom referensi untuk aturan MySQL 8.4+; gagal → alert, abort (`160-165`).
- Bentuk: `ALTER TABLE `thisTable` ADD [`CONSTRAINT `name` `]FOREIGN KEY (`thisColumn`) REFERENCES `thatDatabase`.`thatTable` (`thatColumn`)` (`167-178`).
- `NSArray *onActions = @[@"RESTRICT", @"CASCADE", @"SET NULL", @"NO ACTION"];` (`180`).
- Tambah ON DELETE/ON UPDATE hanya bila `selectedTag >= 0` (`183-190`). Tag xib: `Restrict`=0 (tanpa tag), `Cascade`=1, `Set NULL`=2, `No Action`=3, item pertama placeholder tag `-1` (`DBView.xib:3387-3401` onUpdate id 5614; `3408-3422` onDelete id 5622).
- Eksekusi dengan `assertingDatabaseContext:currentDatabase` (`193`).
- Saat error: bila `lastErrorID == 1005` jalankan `SHOW ENGINE INNODB STATUS` + regex `latest foreign key error\s+-----*\s+[0-9: ]*(.*?)\s+-----` (`209-215`); errno 121 + "already exists" mencatat nama sebagai taken + revalidasi (`218-221`).
- Sukses → `_refreshRelationDataForcingCacheRefresh:YES` (`234`).

### 4.7 Remove

`removeRelation:` (`313-337`): alert prompt, lalu per baris terpilih (terbalik)
`ALTER TABLE `thisTable` DROP FOREIGN KEY `name`` (`322`) dengan `assertingDatabase:`
DB aktif (`324`); abort pada error pertama (`330`); refresh dengan forced cache clear (`334`).

### 4.8 Capability rule referensi FK (MySQL 8.4+)

- `_serverRequiresStandardForeignKeyReferences` (`563-575`): NO untuk MariaDB atau < 8.4.0 (`565`); selain itu query `SELECT @@session.restrict_fk_on_non_standard_key` (`567`).
- `SAForeignKeyReferenceRuleSupport.requiresStandardForeignKeyReferences` (`:5-10`): false untuk MariaDB atau bukan ≥8.4; true bila query restriction error; selain itu `restrictionEnforcesStandardReferences`.
- `restrictionEnforcesStandardReferences` (`:39-45`): true bila nilai kosong/hilang atau bukan di `["0","OFF","FALSE"]` (uppercase/trim).
- `singleColumnUniqueReferenceColumns` (`:13-37`): pertahankan baris `Non_unique == 0`; grup per `Key_name`; hanya key satu baris dengan `Sub_part` kosong/absen; kembalikan himpunan `Column_name`.
- Alert referensi tidak valid: judul `"Referenced column needs a unique key"`, pesan `"MySQL 8.4 and newer require a foreign key to reference a full unique or primary key. Add a single-column unique key to %@.%@ or choose another referenced column."` (`SPTableRelations.m:621-622`).

## 5. Trigger

### 5.1 Baca

`SPTableData.updateTriggersForCurrentTable` (`1279-1314`): `` /*!50003 SHOW TRIGGERS WHERE `Table` = '...' */ `` (`1292-1293`), paksa utf8mb4, cache di `triggers` (`1307`). Error → alert `"Error retrieving trigger information"` / `"An error occurred while retrieving the trigger information for table '%@'. Please try again.\n\nMySQL said: %@"` (`1299`).

`SPTableTriggers` membaca `[tableDataInstance triggers]` dan menyalin key
`Trigger`→TriggerName, `Event`→TriggerEvent, `Timing`→TriggerActionTime,
`Statement`→TriggerStatement (trim NUL, `544-550`), `Definer`, `Created`, `sql_mode` (`553-561`).

### 5.2 Kolom tabel trigger (DBView.xib id 6701, outlet `triggersTableView` `4540`)

`TriggerName`→"Trigger" (`2385-2386`), `TriggerEvent`→"Event" (`2397-2398`),
`TriggerActionTime`→"Timing" (`2409-2410`), `TriggerStatement`→"Statement" (`2421-2422`),
`TriggerDefiner`→"Definer" (`2433-2434`), `TriggerCreated`→"Created" (`2445-2446`),
`TriggerSQLMode`→"SQL Mode" (`2457-2458`). NULL dirender sebagai `[prefs objectForKey:SPNullValue]` (`359-361`) dan abu (`618-627`).

### 5.3 Add/Edit (`confirmAddTrigger:` `194-284`)

- Template (`200`): `CREATE TRIGGER %@ %@ %@ ON %@ FOR EACH ROW %@` → `` CREATE TRIGGER `name` BEFORE|AFTER INSERT|UPDATE|DELETE ON `table` FOR EACH ROW <statement> `` (`251-256`).
- Mapping action-time: `Before`=tag 0→`BEFORE`, `After`=tag 1→`AFTER` (`224-233`; `DBView.xib:3612-3613`). Event: `Insert`=0→`INSERT`, `Update`=1→`UPDATE`, `Delete`=2→`DELETE` (`236-247`; `DBView.xib:3626-3628`).
- **Edit = DROP + CREATE** (MySQL tak punya `ALTER TRIGGER`): `DROP TRIGGER `db`.`oldName`` (`206-208`) sebelum re-create (`202-204`). DROP gagal → alert `"Unable to delete trigger"` / `"The selected trigger couldn't be deleted.\n\nMySQL said: %@"`, sheet dibuka ulang (`212-218`).
- CREATE gagal: bila editing, re-create trigger asli dari `editedTrigger` (`265-277`), lalu alert `"Error creating trigger"` / `"The specified trigger was unable to be created.\n\nMySQL said: %@"`, buka ulang sheet (`278-280`).
- Lalu `_refreshTriggerDataForcingCacheRefresh:YES` (`283`).
- Edit meng-cache asli di `editedTrigger` dan ubah label tombol jadi `"Save"` (`497-517`); add mereset field dan label `"Add"` (`289-301`). Confirm aktif hanya bila nama dan statement non-kosong (`523-526`).

### 5.4 Delete (`removeTrigger:` `306-329`)

`DROP TRIGGER `db`.`name`` (`315`) dengan `assertingDatabaseContext:database` (`317`); abort pada error pertama (`322`) alert `"Unable to delete trigger"` (`320`); refresh forced (`326`).

### 5.5 Gating

`loadTriggers` (`142-162`): hanya `SPTableTypeTable`, selain itu `resetInterface` (`149-150`); label `"Triggers for table: %@"` (`153`). `_refreshTriggerDataForcingCacheRefresh:` dipaksa = `resetAllData` + `updateTriggersForCurrentTable` (`539-542`). Action-time/event tak dikenal → `SPLog`, return `-1` (`677-684`, `686-694`).

## 6. Table Info

### 6.1 Pane bawah — `SPTableInfo`

Terdaftar untuk `SPTableChangedNotification` dan `SPTableInfoChangedNotification`
(`SPConstants.m:275-276`).

- Tanpa tabel: header `"INFORMATION"` + `"multiple selection"` bila multi-pilih (`138-141`).
- Header tabel `"TABLE INFORMATION"` (`152`); baca `[tableDataInstance statusValues]`; kosong → `"error occurred"` (`160-163`).
- Baris: `created: %@` ← `Create_time` (`169`); `updated: %@` ← `Update_time` (InnoDB tak ada → baris dilewati) (`170`); `engine: %@` (dilewati bila `Engine` `NSNull`) (`173-175`); `rows: %@` (akurat) atau `rows: ~%@` (estimasi) dari `Rows`+`RowsCountAccurate` (`190-191`); `size: %@` ← `Data_length` via byte formatter (`195-207`); `encoding: %1$@ (%2$@)` (`%2` = `Collation` tanpa prefix `charset_`) atau `encoding: %1$@` (`210-220`); `auto_increment: %@` (`222-236`); `create_options: %@` bila non-kosong (`239-242`).
- Proc/Func: header `"PROCEDURE INFORMATION"`/`"FUNCTION INFORMATION"`; `created:`←`CREATED`, `updated:`←`LAST_ALTERED`, `data access: %@ (%@)` deterministic/non-deterministic, `return type: %@` (`DTD_IDENTIFIER`, fungsi saja), `execution privilege: %@` (`SECURITY_TYPE`), `definer: %@` (`247-294`).
- View: header `"VIEW INFORMATION"`; hanya bila `DEFINER` non-null: `definer:`, `execution privilege:`, `is updatable:`, `check option:`, `character set client:`, `collation connection:` (`296-344`).
- Mekanik pane: tinggi header 25 (`416-419`), header = group row (`490-494`), klik header toggle pane ACTIVITIES (`443-462`). String header `"TABLE INFORMATION"` (`83`), `"ACTIVITIES"` (`79, 376`).

### 6.2 Tab penuh — `SPExtendedTableInfo.loadTable:` (`235-442`)

Key status (`48-61`): `Engine`, `Row_format`, `Rows`, `Avg_row_length`, `Data_length`,
`Max_data_length`, `Index_length`, `Data_free`, `Auto_increment`, `Create_time`,
`Update_time`, `Collation`, `Comment`.

- Cabang view/tanpa tabel (`257-319`): disable popup engine/encoding/collation; untuk view tambah item `"View"`, tampilkan CREATE yang dipretty (`createViewSyntaxPrettifier`, `SPStringExtAdditions.m:262`) + `";"`, dan bila ada isi popup encoding dari `CharacterSetClient` dan collation dari `Collation` (`310-316`); semua field baris/size dikosongkan.
- Popup engine dari `[databaseDataInstance getDatabaseStorageEngines]` (`321, 330-341`); else `"Not available"` (`353`). Engine yang tak terdaftar tetap disisipkan (`347-350`).
- Popup encoding dari `getDatabaseCharacterSetEncodings`, judul `"<DESCRIPTION> (<CHARACTER_SET_NAME>)"` (`356-376`), else `"Not available"` (`378`).
- Popup collation dari `getDatabaseCollationsForEncoding:[tableDataInstance tableEncoding]` (`381-394`).
- Nilai via `_formatValueWithKey:inDictionary:` (`399-409`)/`_formatValueWithKey` (`685-728`): byte size (`Data_length`/`Max_data_length`/`Index_length`/`Data_free`), tanggal (`Create_time`/`Update_time`) dengan `NSDateFormatter mysqlDateTimeString:`, angka (`Rows`/`Avg_row_length`/`Auto_increment`); **`Rows` diberi prefix `~` bila `RowsCountAccurate` false** (`715-718`); gagal/kosong → `"Not available"` (`722-727`).
- Comment dari `Comment` (`411-421`); create-syntax dari `[tableDataInstance tableCreateSyntax]` + `";"` (`423-436`); tombol Reset AUTO_INCREMENT muncul hanya bila `Auto_increment` ada dan non-null (`439-441`).
- Mutator: `ALTER TABLE `t` ENGINE = <newType>` (`668`), `ALTER TABLE `t` CHARACTER SET = <enc>` (`155`, nilai diparse dari judul popup via `^.+\\((.+)\\)$`, `149`), `ALTER TABLE `t` COLLATE = <coll>` (`180`), `ALTER TABLE `t` COMMENT = <quoted>` (`563`). Edit comment bisa digerbangi konfirmasi `SPQueryWarningEnabled` (`578-589`). Semua mutator memanggil `reloadTable:` → `resetAllData` + `loadTable:` (`105-112`); ganti engine memanggil `[tableDocumentInstance loadTable:ofType:]` (`679`).
- Guard skema sistem: kontrol disabled bila DB `information_schema`, `performance_schema`, atau `mysql` (`629-648`).

### 6.3 Fallback MyISAM / COUNT(1)

- `RowsCountAccurate` = `"y"` **hanya** bila `Engine == "MyISAM"`, selain itu `"n"` (`SPTableData.m:1230-1234`).
- Bila `Rows` `NSNull`: `SELECT COUNT(1) FROM `table`` (`1238-1239`); sukses → set `Rows` + `RowsCountAccurate="y"` (`1241-1244`); gagal → alert `"Querying row count failed"` / `"An error occurred while trying to determine the number of rows for “%@”.\nMySQL said: %@ (%lu)"` (`1247`).
- `updateAccurateNumberOfRowsForCurrentTableForcingUpdate:` (`1323-1377`): no-op sukses untuk non-tabel (`1335-1337`); bila tidak dipaksa dan `RowsCountAccurate` truthy → return (`1343-1345`); baca pref `SPTableRowCountQueryLevel` dan `SPTableRowCountCheapSizeBoundary` (init `SPRowCountFetchAlways`/`5242880` di `1349-1350`, lalu ditimpa default `1351-1352`; enum `SPConstants.h:98-100`, key `SPConstants.h:322-323`); skip bila level `SPRowCountFetchNever`, atau `SPRowCountFetchIfCheap` dengan `Data_length` nil/absen atau `>=` boundary (`1354-1360`); selain itu `SELECT COUNT(1) FROM `table`` (`1364`), simpan `Rows`+`RowsCountAccurate="y"` (`1370-1371`), post `SPTableInfoChangedNotification` di main thread (`1374`).
- Content view juga menulis balik count dari luar: `SPTableContent.m:4088-4089, 4136-4137`.

## 7. Deteksi capability server

`SPServerSupport` (murni perbandingan versi, tanpa I/O — `SPServerSupport.h:36-47`):

- `initWithMajorVersion:minor:release:` menyimpan triple dan memanggil `evaluate` (`70-82`).
- `evaluate` (`99-110`): `_invalidate` dulu (`102`); `isMySQL5 = (major == 5)` (`104`); `isMySQL8 = (major >= 8)` (`105`); `supportsFractionalSeconds = isEqualToOrGreaterThan 5.6.4` (`108`); `supportsFulltextOnInnoDB = isEqualToOrGreaterThan 5.6.4` (`109`).
- `_compareServerMajorVersion:...` (`194-217`): major → minor → release, numerik.
- `isEqualToOrGreaterThanMajorVersion:minor:release:` (`125-133`); `description` (`140-161`) refleksi ivar berprefix `supports`.
- Diinstansiasi sekali per koneksi di `SPDatabaseDocument.m:420-422` dari `[mySQLConnection serverMajorVersion/minorVersion/releaseVersion]`; disebar ke `databaseDataInstance` (`476`), export controller (`500`), charset helpers (`696, 737`), user manager (`2071`), `SPCharsetCollationHelper` di `SPTablesList.m:401`.
- Konsumen: `SPDatabaseData.m:419, 423`; `SPIndexesController.m:686`; `SPUserManager.m:192, 230, 318, 434`.
- Cek ad-hoc di subsistem ini: `SPTableRelations.m:565` (`isMariaDB` + `serverVersionIsGreaterThanOrEqualTo:8 minorVersion:4 releaseVersion:0`).

## 8. String yang terlihat pengguna

`SPTableData.m`: `"An error occurred while retrieving the information for table '%@'. Please try again.\n\nMySQL said: %@"` (`575, 703`); judul `"Error retrieving table information"` (`588, 714`); `"Permission Denied"` / `"The creation syntax could not be retrieved due to a permissions error. ..."` (`624, 1075`); `"Error while parsing CREATE TABLE syntax"` / `"“%@” couldn't be parsed. You can edit the column setup but the column will not be shown in the Content view; ..."` (`782`); `"An error occurred while retrieving status data.\n\nMySQL said: %@"` (`1210`); `"Querying row count failed"` (`1247`); `"Error retrieving trigger information"` (`1299`); log `"Skipping unrecognized CONSTRAINT in CREATE stmt: %@"` (`923`).

`SPTableRelations.m`: `"Table: %@"` (`273`); `"Relations for table: %@"` (`366`); `"This table currently does not support relations. Only tables that use the InnoDB storage engine support them."` (`377`); `"Delete relation"` / `"Are you sure you want to delete the selected relations? This action cannot be undone."` (`316`); `"Unable to delete relation"` (`328`); `"Error creating relation"` (`225, 229`); `"Referenced column needs a unique key"` (`621-622`); menu `"Delete Relations"`/`"Delete Relation"` (`501`).

`SPTableTriggers.m`: `"Triggers for table: %@"` (`153`); tombol `"Add"` (`298`)/`"Save"` (`512`); `"Unable to delete trigger"` (`213, 320`); `"Error creating trigger"` (`278`); `"Delete trigger"` / `"Are you sure you want to delete the selected triggers? This action cannot be undone."` (`308`).

`SPTableInfo.m`: label `created:`/`updated:`/`engine:`/`rows:`/`rows: ~`/`size:`/`encoding:`/`auto_increment:`/`create_options:` (`169-241`); header `PROCEDURE`/`FUNCTION`/`VIEW INFORMATION` (`250-298`).

`SPExtendedTableInfo.m`: `"Change table type"` / `"Are you sure you want to change this table's type to %@? ... This action cannot be undone."` (`135`); `"Error changing table encoding"` (`164`); `"Error changing table collation"` (`189`); `"Error changing table comment"` (`574`); `"Execute SQL?"` / `"Do you really want to proceed with this query?\n\n %@"` (`579-582`); `"Error changing table type"` (`674`); `"Not available"` (`353, 378, 393, 726`); `"View"` (`264`).

`SPDatabaseData.m`: `"Unable to get database collations for given encoding"` (`205`); `"Limited Character Set Metadata"` / `"The server did not provide character set metadata. ..."` (`498-499`).

## 9. UNKNOWN

- Registrasi default preferensi `SPTableRowCountQueryLevel` (apakah absen berarti NEVER atau ALWAYS) — dua inisialisasi di `SPTableData.m:1349-1352` bertentangan bila key absen. **UNKNOWN**.
- `progressStatusTextField` di `SPTableRelations.h:61`: outlet dideklarasikan tapi tak dipakai di `.m`. **UNKNOWN**.
- Rendering NULL untuk `Created` di `triggerDataForPrinting` (`454-487`) tak punya guard NSNull. **UNKNOWN**.
- Arti persis `tickQuotedString` vs `backtickQuotedString` belum dibaca definisinya. **UNKNOWN**.
