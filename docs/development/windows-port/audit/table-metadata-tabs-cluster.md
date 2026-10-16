# Audit: Table-metadata tab cluster

Audit method-by-method klaster tab metadata tabel. `Start`/`End` = nomor baris di
file masing-masing. Baseline commit `02c910932`.

File: `TableRelations/SPTableRelations.m` (761), `TableTriggers/SPTableTriggers.m` (694),
`TableRelations/SAForeignKeyReferenceRuleSupport.swift` (218), `SPTableInfo.m` (525),
`SPExtendedTableInfo.m` (738).

---

## 1. `SPTableRelations.m` (761)

Controller tab "Relations" — tampilkan FK tabel + sheet add/remove + SQL `ALTER TABLE … FOREIGN KEY`.

### Ivar
`.h:39–42` IBOutlet `tableDocumentInstance`, `tablesListInstance`, `tableDataInstance`, `tableList`;
`.h:44–49` `addRelationButton`, `removeRelationButton`, `refreshRelationsButton`, `labelTextField`, `relationsTableView`, `addRelationPanel`;
`.h:51–61` `constraintName`, `addRelationTableBox`, `columnPopUpButton`, `refDatabasePopUpButton`, `refTablePopUpButton`, `refColumnPopUpButton`, `onUpdatePopUpButton`, `onDeletePopUpButton`, `confirmAddRelationButton`, `dataProgressIndicator`, `progressStatusTextField`;
`.h:63–64` `detailErrorView`, `detailErrorText`; `.h:66–70` `connection`, `prefs`, `relationData`, `takenConstraintNames`; property `.h:73–74`.
Konstanta file-scope `SPRelationNameKey`…`SPRelationOnDeleteKey` `.m:43–49`.

### `#pragma mark`
`68/69 (Initialisation), 123/124 (IB action methods), 383/384 (Tableview datasource methods), 402/403 (Task interaction), 434/435 (Other), 509/510 (TextField delegate methods), 530/531 (Tableview delegate methods), 560/561 (Private API), 751 (tanpa nama)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 71 | 80 | `init` | Alokasi relationData, prefs, takenConstraintNames |
| 85 | 121 | `awakeFromNib` | Konfigurasi tabel, registrasi observer prefs/tabel/task |
| 129 | 131 | `openRelationSheet:` | Present sheet add-relation |
| 136 | 140 | `closeRelationSheet:` | Akhiri + order out sheet |
| 145 | 236 | `confirmAddRelation:` | Validasi kolom ref, bangun + jalankan ALTER FK |
| 241 | 244 | `selectTableColumn:` | Bangun ulang kolom tersedia |
| 249 | 254 | `selectReferenceDatabase:` | Refresh tabel ref lalu kolom |
| 259 | 262 | `selectReferenceTable:` | Bangun ulang kolom tersedia |
| 267 | 308 | `addRelation:` | Reset kontrol sheet, isi popup, buka sheet |
| 313 | 337 | `removeRelation:` | Konfirmasi + drop FK terpilih |
| 342 | 345 | `refreshRelations:` | Refresh penuh relation-data |
| 350 | 381 | `tableSelectionChanged:` | Gating UI by engine/tipe tabel |
| 386–400 | | `numberOfRowsInTableView:` / `objectValueForTableColumn:row:` | Datasource (dim nama DB aktif) |
| 408–432 | | `startDocumentTaskForTab:` / `endDocumentTaskForTab:` | Disable/enable tombol |
| 441 | 470 | `relationDataForPrinting` | Array heading + baris untuk print |
| 475–507 | | `observeValueForKeyPath:…` / `validateMenuItem:` | KVO pref + validasi menu |
| 512 | 528 | `controlTextDidChange:` | Flag nama constraint duplikat merah |
| 536–558 | | `tableViewSelectionDidChange:` / `shouldEditTableColumn:row:` / `shouldSelectRow:` | Delegate |
| 563–624 | | `_serverRequiresStandardForeignKeyReferences` / `_singleColumnUniqueReferenceColumnsForTable:database:` / `_referenceColumnAllowsForeignKeyReference:table:database:` / `_showInvalidForeignKeyReferenceAlertForColumn:table:` | Aturan referensi FK MySQL 8.4 |
| 629–749 | | `_refreshRelationDataForcingCacheRefresh:` / `_updateAvailableTables` / `_updateAvailableTableColumns` | Data + popup |
| 753 | 759 | `dealloc` | Hapus observer |

### Temuan / UNKNOWN
- **SQL FK** (`confirmAddRelation:` 145–236): `ALTER TABLE … ADD [CONSTRAINT] FOREIGN KEY (…) REFERENCES db.table (…)` + opsional `ON DELETE`/`ON UPDATE` dari `onActions @[@"RESTRICT",@"CASCADE",@"SET NULL",@"NO ACTION"]` (180–190). Error ID 1005 → parse InnoDB-status via regex `stringByMatching:` (209–215); errno-121 → deteksi nama duplikat (218–221).
- **Delegasi Swift**: `_serverRequiresStandardForeignKeyReferences` (563–575) hardcode `serverVersionIsAtLeast84:YES` + `requiresStandardForeignKeyReferencesWithMariaDB:` karena kasus <8.4/MariaDB sudah return dini (565) — guard Swift (`SAForeignKeyReferenceRuleSupport.swift:6`) jadi redundan sebagian untuk call site ini.
- **Gating UI**: cek engine lowercase == `"innodb"` (363) menggerakkan enable tombol + teks label (355–378). Non-InnoDB → label "does not support relations".
- Sort derived dari `SPAlphabeticalTableSorting` (276, 742). Nama FK DB aktif dirender abu (`NSAttributedString`, 394–398).
- **Doc/behavior mismatch**: komentar atas `_updateAvailableTables` (665–668) bilang "updates the available table columns" padahal mengisi tabel.
- **Mungkin dead**: `progressStatusTextField` (`.h:61`) + `tableList` (`.h:42`) tak dirujuk di `.m`.

---

## 2. `SPTableTriggers.m` (694)

Controller tab "Triggers" — daftar trigger + add/edit/remove via `CREATE`/`DROP TRIGGER`.

### Ivar
`.h:37–41` outlet; `.h:43–48` tombol/tabel/sheet/label; `.h:50–53` `triggerNameTextField`, `triggerActionTimePopUpButton`, `triggerEventPopUpButton`, `triggerStatementTextView`; `.h:55–56` `addTriggerTableBox`, `confirmAddTriggerButton`; `.h:58–61` `connection`, `triggerData`, `prefs`; `.h:63` `isEdit`; `.h:66` `editedTrigger`; property `.h:69`.
Konstanta `SPTriggerName`…`SPTriggerSQLMode` (`.m:43–49`); enum `SPTriggerActionTimeTag`/`SPTriggerEventTag` (51–60); forward decl `TagForActionTime`/`TagForEvent` (62–63).

### `#pragma mark`
`81/82 (Initialisation), 179/180 (IB action methods), 347/348 (Tableview datasource methods), 366/367 (Task interaction), 398/399 (Other), 489/490 (Private API), 604/605 (Tableview delegate methods), 652/653 (Textfield delegate methods), 663, 675 (top-level setelah @end)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 84–93 | | `init` | Alokasi triggerData, bersihkan state edit |
| 98 | 136 | `awakeFromNib` | Konfigurasi tabel, observer prefs/statement/task |
| 142–177 | | `loadTriggers` / `resetInterface` | Aktifkan/bersihkan UI |
| 185 | 189 | `closeTriggerSheet:` | Akhiri sheet |
| 194 | 284 | `confirmAddTrigger:` | `DROP` lama (edit) lalu `CREATE TRIGGER` |
| 289–345 | | `addTrigger`/`removeTrigger`/`editTrigger`/`refreshTriggers` | Aksi |
| 350–396 | | `numberOfRowsInTableView:` / `objectValueForTableColumn:row:` / task start/end | Datasource + gating |
| 404–487 | | `observeValueForKeyPath:…` / `validateMenuItem:` / `triggerStatementTextDidChange:` / `triggerDataForPrinting` | Prefs/menu/print |
| 497–602 | | `_editTriggerAtIndex:` / `_toggleConfirmAddTriggerButtonEnabled` / `_refreshTriggerDataForcingCacheRefresh:` / `_openTriggerSheet` / `_reopenTriggerSheet:returnCode:contextInfo:` / `_addPreferenceObservers` / `_removePreferenceObservers` | Privat |
| 610–671 | | `tableViewSelectionDidChange:` / `willDisplayCell:…` / `shouldEditTableColumn:row:` / `shouldSelectRow:` / `controlTextDidChange:` / `dealloc` | Delegate + teardown |
| 677–694 | | `TagForActionTime` / `TagForEvent` (C) | Map string → enum tag |

### Temuan / UNKNOWN
- **SQL**: template `CREATE TRIGGER %@ %@ %@ ON %@ FOR EACH ROW %@` (200); jalur edit emit `DROP TRIGGER db.name` dulu karena "MySQL doesn't have ALTER TRIGGER" (202–218). Gagal → re-create trigger asli dari `editedTrigger` (265–277), dengan pengakuan in-code data user bisa hilang (273–275).
- **Workaround**: `_refreshTriggerDataForcingCacheRefresh:` trim byte NUL dari Statement via character set `'\0'` (544–550) karena beberapa versi MySQL menambahkannya.
- Gating tombol confirm butuh nama + statement non-kosong (525).
- `TagForActionTime`/`TagForEvent` return `-1` untuk nilai tak dikenal (683, 693); `selectItemWithTag:` lalu memilih apa pun. Notable.
- **Mungkin dead**: `_reopenTriggerSheet:returnCode:contextInfo:` (579–582) tak dipanggil di file (error path pakai `performSelector:@selector(_openTriggerSheet)` 215/279). `addTriggerTableBox` (`.h:55`) + `tableList` (`.h:41`) tak dipakai.

---

## 3. `SAForeignKeyReferenceRuleSupport.swift` (218)

Support type Swift untuk tab Relations: logika rule referensi FK MySQL 8.4 + orkestrator task penghapusan field yang bisa dibatalkan.

### Tipe
`SAForeignKeyReferenceRuleSupport` (kelas, baris 3, tanpa stored property);
`SAFieldRemovalQueryResult` (enum, 64–68: `succeeded`/`cancelled`/`failed`);
`SAFieldRemovalTask` (kelas, 74) dengan property 76–79 (`field`/`foreignKeyName`/`table`/`database`), static privat 81–82 (`initialCancellationRetryDelay=0.025`, `maximumCancellationRetryDelay=1`), state 84–90 (`stateLock`, `cancellationQueue`, `cancellationRequested`, `queryIsAdmitted`).

### `// MARK:`
**Tidak ada**.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 5 | 10 | `requiresStandardForeignKeyReferences(isMariaDB:serverVersionIsAtLeast84:restrictionQueryErrored:restrictionValue:)` | Putuskan apakah server menegakkan referensi FK standar |
| 13 | 37 | `singleColumnUniqueReferenceColumns(_:)` | Ekstrak nama kolom unik single dari baris SHOW INDEX |
| 39–51 | | `restrictionEnforcesStandardReferences(_:)` / `stringValue(_:)` | Helper |
| 53 | 61 | `integerValue(_:)` | Koersi Int null-safe |
| 93–105 | | `init(field:foreignKeyName:table:database:)` / `cancel()` | Konteks + cancel |
| 109 | 115 | `requestQueryCancellation(_:)` | Mulai percobaan cancel ber-backoff |
| 120 | 164 | `run(foreignKeyQuery:fieldQuery:foreignKeyFailure:fieldFailure:schemaRefresh:completion:)` | Urutkan query FK lalu field, refresh, complete di main |
| 169–195 | | `cancelAdmittedQuery(_:)` / `scheduleCancellationAttempt(after:nextRetryDelay:cancellation:)` | Cancel + retry backoff eksponensial (`min(nextRetryDelay*2, 1s)`) |
| 199 | 217 | `runQueryIfAllowed(afterPreviousCancellation:operation:)` | Jalankan query hanya bila tak cancel |

### Temuan / UNKNOWN
- `requiresStandardForeignKeyReferences` (5–10) false untuk MariaDB/<8.4, true bila query restriction error, else delegasi. Selector `@objc` (baris 4) cocok call site ObjC `SPTableRelations.m:571`.
- Derivasi unique-index (13–37): hanya `Non_unique == 0`, grup by `Key_name`, kecualikan multi-kolom/`Sub_part`, return set `Column_name`.
- Threading: `stateLock` (84); `run` dispatch `completion` di main, `sync`-blok bila sudah off-main (159–163); retry cancel backoff (191).
- **UNKNOWN**: tak ada consumer `SAFieldRemovalQueryResult.cancelled`/`SAFieldRemovalTask` di file yang diaudit (user-nya `SPTableStructure`, di luar scope).

---

## 4. `SPTableInfo.m` (525)

Controller pane bawah "Table Info / Activities" yang selalu terlihat — render baris status + jalankan aktivitas BASH.

### Ivar
`.h:38–41` outlet `tableList`/`tableListInstance`/`tableDataInstance`/`tableDocumentInstance`; `.h:43–45` `infoTable`/`activitiesTable`/`tableInfoScrollView`; `.h:47–48` `info`/`activities`; `.h:49` `_activitiesWillBeUpdated`.

### `#pragma mark`
`43/44 (Initialisation), 87 (tanpa nama), 351/352 (TableView datasource methods), 516 (tanpa nama)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 46–85 | | `init` / `awakeFromNib` | Alokasi array + registrasi notifikasi + seed header pane |
| 93–124 | | `removeActivity:` / `updateActivities` | Rebuild list aktivitas |
| 132 | 349 | `tableChanged:` | Bangun baris info untuk tipe table/proc/func/view |
| 357–462 | | `numberOfRowsInTableView:` / `objectValueForTableColumn:row:` / `heightOfRow:` / `shouldSelectRow:` / `shouldEditTableColumn:row:` | Datasource/delegate |
| 469–514 | | `toolTipForCell:…` / `isGroupRow:` / `willDisplayCell:…` | Tooltip/ikon |
| 521 | 523 | `dealloc` | Hapus observer |

### Temuan / UNKNOWN
- **Builder bercabang** (`tableChanged:` 132–349): empat cabang — table (150–245), proc/func (247–294), view (296–346). Masing-masing guard `![tableStatus count]` dengan menambah baris error + return (160–164, 260–264, 306–310).
- Null handling pervasive (`isNSNull`/`unboxNull`), baris Create/Update time hilang lewat `safeAddObject` (169–170). Modifier `RowsCountAccurate`: prefix `~` untuk estimasi (190–191).
- `shouldEditTableColumn:` toggle activity pane via `setActivityPaneHidden:` (448, 452); `_activitiesWillBeUpdated` menekan draw sel saat update (382); tooltip "Cancel" bila `mouseLocation.x` dalam 30pt tepi kanan (474).
- **UNKNOWN/dead**: `return @"";` tak terjangkau setelah else-chain di 409; ivar `tableList` (`.h:38`) tak dipakai.

---

## 5. `SPExtendedTableInfo.m` (738)

Controller tab "Table Info" detail — engine/encoding/collation/comment/create syntax + `ALTER TABLE`.

### Ivar
`.h:40–44` `tableDocumentInstance`, `tablesListInstance`, `tableDataInstance`, `databaseDataInstance`, `tableSourceInstance`; `.h:46` `resetAutoIncrementResetButton`; `.h:48–57` 10 field nilai; `.h:59–60` `tableCommentsTextView`, `tableCreateSyntaxTextView`; `.h:62–64` 3 popup; `.h:66,68` `selectedTable`, `connection`; property `.h:71–73`.
Konstanta `SPUpdateTableType*` (`.m:45–46`); nama field status `SPMySQLEngineField`…`SPMySQLCommentField` (49–61). Synthesize 73–75.

### `#pragma mark`
`99/100 (IBAction methods), 226/227 (Other), 597/598 (Task interaction), 651/652 (Private API), 730 (tanpa nama)`.

### Method
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 80 | 97 | `awakeFromNib` | Registrasi observer task + table-info-changed |
| 105 | 112 | `reloadTable:` | Reset cache + reload info tabel |
| 117–224 | | `updateTableType:` / `updateTableEncoding:` / `updateTableCollation:` / `resetAutoIncrement:` / `tableRowAutoIncrementWasEdited:` / `control:textView:doCommandBySelector:` | Aksi (dengan revert saat error) |
| 235 | 442 | `loadTable:` | Isi semua field/popup/syntax; tangani view |
| 449 | 542 | `tableInformationForPrinting` | Dict print termasuk HTML create syntax |
| 548 | 595 | `textDidEndEditing:` | Persist COMMENT via `ALTER TABLE` |
| 603–649 | | `startDocumentTaskForTab:` / `endDocumentTaskForTab:` | Gating (disable untuk skema sistem) |
| 657–728 | | `_updateDisplayedInfo:` / `_changeCurrentTableTypeFrom:to:` / `_formatValueWithKey:inDictionary:` | Privat |
| 732 | 736 | `dealloc` | Hapus observer |

### Temuan / UNKNOWN
- **SQL**: comment `ALTER TABLE %@ COMMENT = %@` dengan `escapeAndQuoteString:` (563); engine `ALTER TABLE %@ ENGINE = %@` (668); encoding `… CHARACTER SET = %@` (155); collation `… COLLATE = %@` (180).
- **Gating skema sistem**: `endDocumentTaskForTab:` hitung `isSystemSchemaDb` untuk `information_schema`/`performance_schema`/`mysql` (629–631) + disable kontrol (634–648).
- Comment dikonfirmasi via `SPQueryWarningEnabled` (578–592), trim whitespace sebelum banding lama/baru (555–562).
- **View**: `loadTable:` deteksi `Engine == "View"`, load `createViewSyntaxPrettifier`, disable popup (257–280). `resetAutoIncrementResetButton` muncul hanya bila `Auto_increment` non-null (439–441).
- `_formatValueWithKey:inDictionary:` (685–728): byte via `NSByteCountFormatter` (699), tanggal via `mysqlDateTimeString:` (706), angka via decimal formatter (713); prefix `~` saat `RowsCountAccurate` false (716–717); fallback "Not available" (726).
- ⚠️ **Kemungkinan bug**: `loadTable:` baris 425 memakai `[[tableCommentsTextView string] length]` sebagai panjang range pada `tableCreateSyntaxTextView` — text view tidak cocok (berbeda dari baris 268 yang benar). Perilaku saat comment lebih panjang/pendek dari syntax **UNKNOWN**.
- **UNKNOWN**: `tableSourceInstance` bertipe `id` (`.h:44`), hanya di-message `resetAutoIncrement:`/`setAutoIncrementTo:` (201, 211).

## Ringkasan lintas-file
- Dua controller tab (`SPTableRelations`, `SPTableTriggers`) + `SPExtendedTableInfo` berbagi bentuk gating `start/endDocumentTaskForTab:` identik, masing-masing ber-key identifier `SPMainToolbarTable*` sendiri (`SPTableRelations.m:411/424`, `SPTableTriggers.m:375/388`, `SPExtendedTableInfo.m:606/620`).
- Hanya `SPTableRelations` + `SPExtendedTableInfo` bridge ke Swift (`SAForeignKeyReferenceRuleSupport` 571, 608); `SPExtendedTableInfo.m` hanya import `sequel-ace-Swift.h` (41) tanpa pemakaian simbol Swift → **UNKNOWN**.
