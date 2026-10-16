# Audit: `SPTableData.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/DataControllers/SPTableData.m`. Baseline commit `02c910932`.

- File: 1637 baris; header `SPTableData.h` 88 baris.
- Peran: data-controller/model yang mengambil, memparse, dan meng-cache struktur,
  status, serta trigger tabel/view terpilih untuk satu dokumen Sequel Ace
  (kolom, encoding, CREATE syntax, constraint, akurasi jumlah baris).
- Superclass: `NSObject` (`.h:35`).

## 1. Struktur

| Item | Baris |
| --- | --- |
| Header/import (`SPSQLParser`, `SPDatabaseDocument`, `SPTablesList`, `RegexKitLite`, `SPServerSupport`, `SPFunctions`, bridging Swift) | 1–42 |
| Class extension privat `@interface SPTableData ()` (8 method privat) | 44–55 |
| `@implementation` + blok ivar khusus impl | 57–61 |
| `@synthesize` | 63–64 |
| `@end` | 1637 |

## 2. Inventaris ivar/property

### `.h` (35–54)
| Ivar | Tipe | Baris | Catatan |
| --- | --- | --- | --- |
| `tableDocumentInstance` | `__weak SPDatabaseDocument*` (IBOutlet) | 36 | sumber encoding DB |
| `tableListInstance` | `SPTablesList*` (IBOutlet) | 37 | pemasok tabel/db/tipe terpilih |
| `columns` | `NSMutableArray*` | 39 | cache dict kolom |
| `columnNames` | `NSMutableArray*` | 40 | cache nama paralel |
| `constraints` | `NSMutableArray*` | 41 | cache constraint FK |
| `triggers` | `NSArray*` | 42 | lazy; nil = belum dimuat |
| `status` | `NSMutableDictionary*` | 43 | cache status tabel |
| `primaryKeyColumns` | `NSMutableArray*` | 44 | cache nama kolom PK |
| `tableEncoding` | `NSString*` | 46 | nil = belum dimuat |
| `tableCreateSyntax` | `NSString*` | 47 | nil = belum dimuat |
| `mySQLConnection` | `SPMySQLConnection*` | 49 | koneksi |
| `dataProcessingLock` | `pthread_mutex_t` | 51 | menjaga semua mutasi cache |
| `tableHasAutoIncrementField` | `BOOL` | 53 | di-set saat parsing field |

### Blok impl `.m` (58–61)
`_failedTableInformationLoad` `SPTableLoadFailure*` (59) — negative cache struktur;
`_failedStatusLoad` `SPTableLoadFailure*` (60) — negative cache status.

Property: `tableHasAutoIncrementField` (readonly assign, `.h:56`), `connection` (`.h:57`, synthesize ke `mySQLConnection` `.m:64`).
Statik/konstanta file-local: **tidak ada**. Helper negative-cache `SPTableLoadFailure` ada di Swift `StringExtension.swift:990` (`failureWithTableName:database:tableType:` 1002; `matchesTableName:database:tableType:` 1011).

## 3. Semua `#pragma mark`

`523, 1619, 1628/1629 (Private API)`. Hanya empat token; sebagian besar file (44–523 dan 523–1619) tanpa mark.

## 4. Tabel method (39 definisi, urut baris)

### Lifecycle, init & helper negative-cache (57–132)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 66 | 89 | `init` | Alokasi cache, register observer close, init mutex |
| 91 | 98 | `documentWillClose:` | Bersihkan koneksi saat dokumen pemilik tutup |
| 100 | 105 | `_shouldSkipTableInformationLoadForTable:database:tableType:` | Tes hit negative-cache struktur |
| 107 | 110 | `_recordTableInformationLoadFailureForTable:database:tableType:` | Catat identitas load struktur gagal |
| 112 | 115 | `_clearTableInformationLoadFailure` | Bersihkan negative cache struktur |
| 117 | 122 | `_shouldSkipStatusLoadForTable:database:tableType:` | Tes hit negative-cache status |
| 124 | 127 | `_recordStatusLoadFailureForTable:database:tableType:` | Catat identitas load status gagal |
| 129 | 132 | `_clearStatusLoadFailure` | Bersihkan negative cache status |

### Cache accessor / lazy loader (134–368)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 137 | 150 | `tableEncoding` | Kembalikan encoding tabel, load bila belum cache |
| 155 | 174 | `tableCreateSyntax` | Kembalikan CREATE syntax, load bila belum cache |
| 179 | 193 | `columns` | Kembalikan array dict kolom, load bila kosong |
| 198 | 201 | `getConstraints` | Kembalikan constraint FK ter-cache (tanpa load) |
| 206 | 222 | `triggers` | Kembalikan trigger, load hanya untuk tabel |
| 229 | 244 | `columnWithName:` | Kembalikan dict kolom by nama |
| 249 | 261 | `columnNames` | Array nama kolom ter-cache, load bila kosong |
| 268 | 281 | `columnAtIndex:` | Dict kolom di index, load bila kosong |
| 289 | 303 | `columnIsBlobOrText:` | True bila grouping kolom text/blob |
| 311 | 325 | `columnIsGeometry:` | True bila grouping kolom geometry |
| 332 | 341 | `statusValueForKey:` | Nilai status, load bila kosong |
| 351 | 354 | `setStatusValue:forKey:` | Timpa nilai status ter-cache |
| 359 | 368 | `statusValues` | Dict status, load bila kosong |

### Reset cache (370–417)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 373 | 392 | `resetAllData` | Flush semua cache + kedua negative cache |
| 397 | 403 | `resetStatusData` | Flush cache status + negative cache-nya |
| 408 | 417 | `resetColumnData` | Flush cache kolom + negative cache struktur |

### Populasi info tabel/view (419–521)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 423 | 473 | `updateInformationForCurrentTable` | Load + cache struktur tabel aktif (mutex) |
| 479 | 521 | `updateInformationForCurrentView` | Load + cache struktur view aktif (mutex) |

### Parsing & fetch struktur (525–1007)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 532 | 640 | `informationForTable:fromDatabase:` | Jalankan SHOW CREATE TABLE + parse struktur |
| 642 | 646 | `createTableSyntaxFromView:withSyntaxResult:` | Wrapper convenience pakai DB terpilih |
| 648 | 722 | `createTableSyntaxFromView:fromDatabase:withSyntaxResult:` | Bangun syntax CREATE-TEMP-TABLE dari SHOW COLUMNS untuk view |
| 737 | 1007 | `parseCreateStatement:ofType:` | Parse CREATE statement → kolom/key/encoding dict |

### Info view (1009–1139)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1017 | 1021 | `informationForView:` | Wrapper convenience pakai DB terpilih |
| 1023 | 1139 | `informationForView:fromDatabase:` | Bangun data kolom view dari SHOW CREATE/COLUMNS |

### Status & trigger (1141–1314)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1144 | 1274 | `updateStatusInformationForCurrentTable` | Query + cache status tabel/view/routine |
| 1279 | 1314 | `updateTriggersForCurrentTable` | Query + cache trigger tabel aktif |

### Row count & parsing field (1316–1596)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1323 | 1377 | `updateAccurateNumberOfRowsForCurrentTableForcingUpdate:` | Opsional jalankan COUNT(1) agar jumlah baris akurat |
| 1385 | 1596 | `parseFieldDefinitionStringParts:` | Parse token definisi field → dict detail |

### PK accessor (1598–1617)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1601 | 1617 | `primaryKeyColumnNames` | Nama kolom PK, load info bila kosong |

### Lifecycle/private (1619–1637)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1621 | 1626 | `dealloc` | Lepas koneksi, destroy mutex |
| 1631 | 1635 | `_loopWhileWorking` | Spin-wait (trylock/usleep) menunggu processing lain selesai |

## 5. Temuan menonjol

**Model caching & invalidasi**
- **Lazy sentinel.** Setiap accessor panggil `_loopWhileWorking` lalu cek sentinel: `columns`/`columnNames` cek `count == 0` (`185`, `253`), `status` cek `count == 0` (`337`, `364`), `tableEncoding`/`tableCreateSyntax`/`triggers` cek `== nil` (`142`, `161`, `212`). Hasil yang dimuat-tapi-kosong akan memicu fetch ulang karena `count == 0` tak terbedakan dari "belum dimuat" — **UNKNOWN** apakah disengaja.
- **Satu mutex** `dataProcessingLock` di-lock di `updateInformationForCurrentTable` (425), `…View` (481), `updateStatusInformationForCurrentTable` (1149), `updateTriggersForCurrentTable` (1281); `_loopWhileWorking` (1631–1635) poll `pthread_mutex_trylock` + `usleep(10000)`.
- `resetColumnData` (408–417) membersihkan `columnNames` tapi **tidak** `constraints` (410–416).

**Negative caching (desain eksplisit)**
- Dua cache independen (59–60), ber-key `(tableName, database, tableType)` via Swift `SPTableLoadFailure`.
- Dicatat saat: SHOW CREATE gagal (572), create-syntax NULL/permission-denied (622), error fetch view (1049, 1074, 1088), error query status (1207). Dikonsultasi sebelum requery (442, 486, 1164). Dibersihkan saat sukses (633, 1136, 1220) dan oleh reset.
- **Asimetri/kemungkinan bug:** `informationForView:fromDatabase:` membersihkan failure cache saat sukses (1136) tapi **tidak** mencatat failure untuk cabang "SHOW CREATE nil tanpa error" (1067–1070) — hanya `NSLog`.

**Parser**
- `parseCreateStatement:ofType:` (737–1007) — header memperingatkan hanya aman untuk output `SHOW CREATE`, bukan input user (734–735). Pakai `SPSQLParser` untuk trim sadar bracket/quote. Menangani field definition (769), nama field backtick ter-escape (786–794), FK `CONSTRAINT ... FOREIGN KEY ... REFERENCES` termasuk offset ON UPDATE/DELETE (814–919), PRIMARY KEY (930–948), UNIQUE (952–964). Constraint tak dikenal dilewati (MariaDB CHECK, 920–924).
  - **Aritmetika index rapuh:** merujuk `parts` di index tetap 1,2,3,4,6,7 + offset 9/10/12/13 (817–916) — bergantung spasi token persis.
  - **Fallback encoding:** scan `CHARSET=`/`CHARACTER SET=` (974–990), "default" → encoding dokumen, else encoding dokumen, else `latin1` (986–998).
  - Efek samping: append dict FK langsung ke ivar `constraints` (918) dan return salinan (1004).
- `parseFieldDefinitionStringParts:` (1385–1596) dipakai CREATE parsing (801) dan string tipe `SHOW COLUMNS` (1109). Set `typegrouping` (1444–1476), default kolom (1480–1487), flag unsigned/zerofill/binary/charset/collate (1496–1525), generated (1528–1540), null/auto_increment/default/on-update-timestamp/comment (1543–1585), token tak dikenal → `unparsed` (1589–1592). Set `AUTO_INCREMENT` juga set ivar `tableHasAutoIncrementField = YES` (1555) — efek samping tersembunyi.

**Risiko duplikasi `constraints`**
`constraints` di-append di dalam `parseCreateStatement` (918). `informationForTable:` membersihkannya sebelum dipakai untuk hindari bocor constraint tabel sebelumnya (issue 1206, 546–549).

**Encoding & koneksi**
- Beberapa method sementara memaksa `utf8mb4` bila encoding koneksi tak diawali `utf8`, via `storeEncodingForRestoration`/`restoreStoredEncoding` (534–544, 1029–1039, 1151–1174, 1286–1290). Restore tak seimbang mungkin di beberapa jalur early-return: `informationForTable` return nil di 611–616 (restore) tapi jalur error 590 restore hanya `if (changeEncoding)` dalam `isConnected`.

**Status**
- Bercabang per `SPTableType`: Proc/Func/View query `information_schema` (1183–1197), Table pakai `SHOW TABLE STATUS LIKE` (1198–1201). `MyISAM` → `RowsCountAccurate=@"y"`, else `@"n"` (1230–1234). `Rows` NULL → fallback `SELECT COUNT(1)` (1238–1249) + alert saat gagal. Status view ditambah key sintetik (1253–1267).
- Escaping nama tabel/db manual via `replaceOccurrencesOfString` + regex (1177–1200) — **UNKNOWN** kenapa regex `\\\\(?=\\Z|[^'])` diperlukan.

**Policy row count**
`updateAccurateNumberOfRowsForCurrentTableForcingUpdate:` membaca pref `SPTableRowCountQueryLevel`/`SPTableRowCountCheapSizeBoundary` (default 5242880) (1347–1352), menghormati `SPRowCountFetchNever`/`IfCheap` (1354–1360). Post `SPTableInfoChangedNotification` saat sukses (1374).

**Concurrency/UI (inkonsisten)**
Beberapa method `SPMainQSync(^{ ... [NSAlert ...]})` (587, 713, 1052), yang lain panggil `NSAlert` langsung (624, ~1080, 1247) — **UNKNOWN** apakah aman off-main.

**UNKNOWN**
- Apakah perilaku re-fetch sentinel `count == 0` disengaja.
- Alasan cabang nil-syntax tanpa `_recordTableInformationLoadFailureForTable` (1067–1070).
- Rasional offset aksi FK dua-tahap `nextOffs = 12/13` (857–916).
- Jaminan thread-affinity untuk `NSAlert` langsung vs `SPMainQSync`.
