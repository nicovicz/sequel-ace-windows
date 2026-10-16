# Audit: `SPUserManager.m`

Audit method-by-method. `Start`/`End` = nomor baris di
`Source/Controllers/SubviewControllers/SPUserManager.m`. Baseline commit `02c910932`.

- File: 2001 baris; header `SPUserManager.h` 279 baris (termasuk 5 helper C `static inline` di `.h:38–170`).
- Peran: `NSWindowController` sheet yang membaca `mysql.user`/`mysql.db`/`mysql.global_grants`/`mysql.global_priv`
  ke model Core Data in-memory, memungkinkan edit user/host/resource-limit/privilege,
  dan menerjemahkan validasi Core Data insert/update/delete menjadi
  `CREATE/DROP/RENAME USER`, `ALTER USER`, `GRANT`, `REVOKE`. Juga delegate+datasource tree/table view.
- Total: 71 definisi di `@implementation`.

## 1. Inventaris ivar/property

### Ivar `.h` (173–223)
| Baris | Item |
| --- | --- |
| h:174–176 | `persistentStoreCoordinator`, `managedObjectModel`, `managedObjectContext` |
| h:177 | `privColumnToGrantMap` (di-set di `init`, `.m:115–129`) |
| h:179–181 | `connection`, `databaseDocument` (__weak), `serverSupport` |
| h:183–205 | IBOutlet: `splitView`, `outlineView`, `tabView`, `treeController`, `privsSupportedByServer`, `grantedController`/`availableController`, `schemasTableView`/`grantedTableView`/`availableTableView`, `addSchemaPrivButton`/`removeSchemaPrivButton`, `maxUpdatesTextField`/`maxConnectionsTextField`/`maxQuestionsTextField`, `userNameTextField`, `errorsSheet`/`errorsTextView` |
| h:207–212 | `schemas`, `grantedSchemaPrivs`, `availablePrivs`, `treeSortDescriptors`, `treeSortDescriptor` |
| h:214–222 | BOOL `isSaving`/`isInitializing`/`mariaDBGlobalPrivilegeAccessDataAvailable`/`mySQLDynamicPrivilegeDataAvailable`, `mySQLDynamicPrivilegeGrantOptionsByAccount`, `errorsString`, `requiresPost576PasswordHandling` |

### `@synthesize` (87–98) & class extension (53–83)
Synthesize: `connection`, `databaseDocument`, `privsSupportedByServer`, MOC/model/coordinator,
`schemas`, `grantedSchemaPrivs`, `availablePrivs`, `treeSortDescriptors`, `serverSupport`,
`isInitializing`. Extension `@interface SPUserManager () <NSMenuItemValidation>` (52–53) +
28 forward decl (54–81). **`contextDidChange:` (1164) TIDAK dideklarasikan** di sini.

### Static/helper file-level
`SPTableViewNameColumnID = @"NameColumn"` (44); tab id `General`/`Global Privileges`/`Resources`/`Schema Privileges` (46–49);
helper `.h`: `SPUserManagerPrivilegeOperationErrorMessageForServerError` (38–88),
`SPUserManagerMySQLDynamicPrivilegeKeysRequiringUnderscoreGrantNames` (90–100),
`SPUserManagerGrantNameForPrivilegeKey` (102–110),
`SPUserManagerApplyMySQLDynamicPrivilegeSupportAvailability` (112–120),
`SPUserManagerShouldPreserveMySQLDynamicPrivilegeGrantOption` (122–126),
`SPUserManagerGlobalShowCreateRoutinePrivilegeSupportKey` (128–131),
`SPUserManagerShouldUseAllPrivilegesShortcut` (133–136),
`SPUserManagerMariaDBGlobalOnlyPrivilegeKeysRequiringGlobalPrivAccess` (138–152),
`SPUserManagerApplyMariaDBGlobalPrivilegeSupportAvailability` (154–170).

## 2. Semua `#pragma mark` (urut baris)

`.m:100/101 (Initialisation), 734/735 (General IBAction methods), 1145/1146 (Notifications), 1169/1170 (Core data notifications), 1501/1502 (Private API), 1729 (SPUserManagerDelegate), 1731 (TableView Delegate Methods), 1831/1832 (Tab View Delegate methods), 1895/1896 (Outline view Delegate Methods), 1979 (SPUserManagerDataSource), 1995`.

## 3. Tabel method (urut baris)

### Initialisation (100–732)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 103 | 144 | `init` | Muat nib, `privColumnToGrantMap`, array, observe db created/removed/renamed |
| 146 | 150 | `_mappedPrivilegeKeyForKey:` | Map key kolom mysql → key sintaks grant |
| 152 | 160 | `_privilegeKeyForServerPrivilegeName:` | Lowercase, spasi→`_`, tambah `_priv`, lalu map |
| 162 | 180 | `_supportedPrivilegeKeysForEntityName:` | Key atribut `_priv` entitas difilter dukungan server |
| 182 | 185 | `_accountKeyForUser:host:` | Key dict `"user\nhost"` untuk cache priv |
| 187 | 222 | `_mariaDBGlobalPrivilegeAccessDataByAccount` | Parse JSON `access` bitmask dari `mysql.global_priv` per akun |
| 224 | 277 | `_mySQLDynamicPrivilegeDataByAccount` | Baca `mysql.global_grants` ke set key per akun + map grant-option |
| 279 | 305 | `_initializeMariaDBGlobalPrivsForChild:accessValue:` | Dekode bit akses 20–39 ke SPUserMO |
| 307 | 313 | `_initializeMySQLDynamicPrivsForChild:privilegeKeys:` | Set tiap key priv dinamis YES |
| 315 | 325 | `_shouldPreserveMySQLDynamicGrantOptionForPrivilege:user:host:` | Apakah grant bawa `WITH GRANT OPTION` |
| 331 | 357 | `windowDidLoad` | Setup UI sekali: tab, min split, image cell, initializer |
| 363 | 440 | `_initializeUsers` | Query `mysql.user`, bangun tree, muat map dukungan `SHOW PRIVILEGES` |
| 446 | 548 | `_initializeTree:` | Baca `mysql.db`, buat parent/child SPUser, schema priv, save |
| 553 | 572 | `_initializeAvailablePrivs` | Isi `availablePrivs` dari entitas `Privileges` × map dukungan |
| 577 | 585 | `_initializeSchemaPrivs` | Reset `schemas` dari `allDatabaseNames`, reload tabel |
| 590 | 621 | `_initializeChild:withItem:` | Salin baris dict ke child via KVC (`_priv`/`max*`/lain) |
| 629 | 669 | `_initializeSchemaPrivsForChild:fromData:` | Buat MO `Privileges` untuk baris host cocok |
| 675 | 681 | `managedObjectModel` | Model bundle merged (lazy) |
| 689 | 702 | `persistentStoreCoordinator` | Coordinator dengan store in-memory (lazy) |
| 708 | 725 | `managedObjectContext` | MOC main-queue (lazy) + register observer `contextWillSave:` |
| 727 | 732 | `beginSheetModalForWindow:completionHandler:` | Present window sebagai sheet + callback |

### General IBAction methods (734–1143)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 740 | 752 | `doCancel:` | Discard editing, rollback context, akhiri sheet |
| 757 | 796 | `doApply:` | Commit editing, save context (picu SQL), `FLUSH PRIVILEGES`, tampilkan error |
| 801 | 815 | `checkAllPrivileges:` | Set semua priv SPUser yang didukung YES |
| 820 | 834 | `uncheckAllPrivileges:` | Set semua priv SPUser yang didukung NO |
| 839 | 856 | `addUser:` | Tambah parent user baru + child `localhost`, fokus field nama |
| 861 | 877 | `removeUser:` | Propagasi user asli ke child, kosongkan host, hapus |
| 882 | 897 | `addHost:` | Tambah child host, jadwalkan `editNewHost` |
| 902 | 905 | `editNewHost` | Edit tertunda baris outline terpilih |
| 910 | 925 | `removeHost:` | Salin user parent ke child, hapus; warn bila user tanpa host |
| 930 | 941 | `addSchemaPriv:` | Pindah avail→granted, enable priv |
| 946 | 957 | `removeSchemaPriv:` | Pindah granted→avail, disable priv |
| 963 | 974 | `doubleClickSchemaPriv:` | Rutekan double-click ke add/remove schema priv |
| 979 | 1037 | `refresh:` | Konfirmasi, reset MOC, drop/re-add store, reinit tree |
| 1039 | 1074 | `_setSchemaPrivValues:enabled:` | Fetch/insert MO Privileges, set nilai, link ke host |
| 1076 | 1080 | `_clearData` | Reset MOC (**tanpa pemanggil — dead**) |
| 1085 | 1097 | `validateMenuItem:` | Enable `removeHost:`/`addHost:` per seleksi |
| 1099 | 1115 | `_selectParentFromSelection` | Pindah seleksi tree ke node parent (atau bersihkan) |
| 1117 | 1134 | `_selectFirstChildOfParentNode` | Expand + pilih child host pertama |
| 1139 | 1143 | `closeErrorsSheet:` | Akhiri + order out sheet error |

### Notifications (1145–1167)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1154 | 1162 | `contextWillSave:` | Inject `self` sebagai `userManager` ke MO SPUser/Privileges tersisip |
| 1164 | 1167 | `contextDidChange:` | Reload outline saat tak initializing (**tak pernah diregistrasi — dead**) |

### Core data notifications (1169–1499)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1172 | 1249 | `updateUser:` | Rename user/host, ganti password (`ALTER USER` atau `SET PASSWORD`), else resource+priv |
| 1251 | 1264 | `deleteUser:` | `DROP USER 'user'@'host'`; skip baris hostless/display-only |
| 1266 | 1345 | `insertUser:` | `CREATE USER` varian IDENTIFIED BY/AS, lalu resource+priv |
| 1347 | 1350 | `grantDbPrivilegesWithPrivilege:` | Wrapper convenience, `skippingRevoke:NO` |
| 1355 | 1400 | `grantDbPrivilegesWithPrivilege:skippingRevoke:` | Diff key berubah, grant lalu opsional revoke per-schema |
| 1405 | 1438 | `updateResourcesForUser:` | `ALTER USER ... WITH MAX_*`, fallback UPDATE langsung, lalu warn |
| 1440 | 1443 | `grantPrivilegesToUser:` | Wrapper convenience, `skippingRevoke:NO` |
| 1448 | 1499 | `grantPrivilegesToUser:skippingRevoke:` | Diff priv global berubah, grant/revoke, rekursi schema priv |

### Private API (1501–1727)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1508 | 1526 | `_fetchUserWithUserName:` | Fetch parent SPUser by user |
| 1528 | 1562 | `_fetchPrivsWithUser:schema:host:` | Fetch MO Privileges predikat empat-arah |
| 1567 | 1570 | `_createNewSPUser` | Insert SPUser baru ke MOC |
| 1575 | 1624 | `_grantPrivileges:onDatabase:forUser:host:` | Bangun statement GRANT; shortcut `GRANT ALL`; batch grant-option |
| 1629 | 1662 | `_revokePrivileges:onDatabase:forUser:host:` | Bangun statement REVOKE; `REVOKE ALL` + `REVOKE GRANT OPTION` |
| 1667 | 1681 | `_checkAndDisplayMySqlError` | Append ke sheet error bila saving, else alert |
| 1683 | 1707 | `_checkAndDisplayMySqlErrorForPrivilegeOperation:privileges:onDatabase:forUser:host:statement:` | Tampilkan error spesifik priv + hint MariaDB |
| 1717 | 1727 | `_renameUserFrom:host:to:host:` | Statement `RENAME USER` + cek error |

### TableView Delegate Methods (1729–1829)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1733 | 1790 | `tableViewSelectionDidChange:` | Seleksi schema: bangun ulang list priv granted/available, enable tombol |
| 1792 | 1829 | `tableView:willDisplayCell:forTableColumn:row:` | Gray baris "all databases"; tint kuning schema ber-priv |

### Tab View Delegate methods (1831–1893)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1834 | 1871 | `tabView:shouldSelectTabViewItem:` | Blokir tab tanpa user/host, commit editing, warn user tanpa host |
| 1873 | 1893 | `tabView:didSelectTabViewItem:` | Ganti seleksi: General→parent, lain→child pertama |

### Outline view Delegate Methods (1895–1977)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1898 | 1908 | `outlineView:willDisplayCell:forTableColumn:item:` | Ikon user vs network per parent/child |
| 1910 | 1913 | `outlineView:isGroupItem:` | Selalu NO |
| 1915 | 1918 | `outlineView:shouldSelectItem:` | Selalu YES |
| 1920 | 1923 | `outlineView:shouldEditTableColumn:item:` | Editable hanya bila item tanpa child |
| 1925 | 1950 | `outlineViewSelectionDidChange:` | Auto-switch tab, default host `%`, deselect tabel schema |
| 1952 | 1977 | `selectionShouldChangeInOutlineView:` | Tolak nama user/host duplikat |

### SPUserManagerDataSource (1979–1993)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1981 | 1983 | `numberOfRowsInTableView:` | Jumlah baris = count `schemas` |
| 1985 | 1993 | `tableView:objectValueForTableColumn:row:` | Nama schema, lokalisasi placeholder `""`/`"%"` |

### Tail (1995)
| Start | End | Method | Tujuan |
| --- | --- | --- | --- |
| 1997 | 1999 | `dealloc` | Hapus semua observer notifikasi |

## 4. Temuan menonjol / UNKNOWN

**Hook validasi Core Data**
- `SPUserMO` meng-override `validateForInsert:`/`validateForDelete:`/`validateForUpdate:` dan meneruskan ke manager di key `userManager` — `SPUserMO.m:88–113` (`[mgr insertUser:self]` 94, `deleteUser:` 103, `updateUser:` 112). Return NO menggagalkan save Core Data.
- Back-reference `userManager` di-inject **hanya untuk objek tersisip** di `contextWillSave:` (1154–1162; komentar 1156). Objek yang dimuat dari DB bergantung pada injeksi ini di batas store-save. **UNKNOWN**: tak ada penanganan bila hook berjalan untuk MO yang tak pernah menerima `userManager` (nil-receiver → NO).
- Karena hook berjalan di dalam `[moc save:]`, semua SQL satu klik Apply di-issue dari `doApply:` (773) dan diakumulasi di `errorsString` (764, 777) selagi `isSaving == YES` (769/775) — ini sebab `_checkAndDisplayMySqlError*` (1667–1707) bercabang pada `isSaving` alih-alih alert.
- `isInitializing` menekan SQL saat load: `deleteUser:` (1254), `insertUser:` (1269).

**Generasi grant/revoke**
- **Shortcut "All privileges"**: bila count diminta == count didukung *dan* statement ber-scope database, `_grantPrivileges:` mengemit `GRANT ALL ON db.* ... WITH GRANT OPTION` (1581–1589) dan `_revokePrivileges:` mengemit `REVOKE ALL PRIVILEGES` + `REVOKE GRANT OPTION` (1637–1661). Predikat `SPUserManagerShouldUseAllPrivilegesShortcut` (`.h:133–136`). Scope global tak pernah pakai shortcut.
- **Split grant-option:** priv dinamis MySQL 8 yang nama grant-nya harus tetap underscore (`.h:90–100`) dipisah ke batch GRANT kedua bawa `WITH GRANT OPTION` (1591–1621; `_shouldPreserveMySQLDynamicGrantOptionForPrivilege:` 315–325 berbasis `mySQLDynamicPrivilegeGrantOptionsByAccount` 262–271).
- **Key → grant name:** `SPUserManagerGrantNameForPrivilegeKey` (`.h:102–110`) potong `_priv`, `_`→spasi kecuali 6 key underscore-sensitive; diproses `SAUserManagerPrivilegeNormalizer.normalizedGrantName:isMariaDB:` (`SAUserManagerPrivilegeNormalizer.swift:10–18`) yang menulis ulang `delete versioning rows`→`delete history` hanya di MariaDB.
- **Diffing:** kedua jalur grant hanya mempertimbangkan key di `changedValues` (1363, 1455); priv tak berubah tak muncul di granted maupun revoked.
- **Rename sebelum kerja priv:** `updateUser:` rename user (1179–1188)/host (1231–1237) *sebelum* resource+priv, selalu pakai `originalhost` fallback `host` (1184).

**Dekode bit MariaDB**
- `mysql.global_priv.Priv` adalah JSON; kode **tidak** parse JSON — cari substring `"access"` pertama lalu `NSScanner` maju ke kolon pertama lalu `unsigned long long` pertama (208–215). **UNKNOWN**: tak ada jaminan MariaDB menaruh `access` pertama atau unquoted.
- Jalur gagal: bukan MariaDB, versi <10.4.0, atau query error → `@{}` + flag NO (189–198); flag memangkas 10 key global-only dari map dukungan (`.h:154–170`) via `_initializeUsers` 431–433.
- Bit map (284–296): `binlog_monitor`=20, `set_user`=30, `federated_admin`=31, `connection_admin`=32, `read_only_admin`=33, `replication_slave_admin`=34, `replication_master_admin`=35, `binlog_admin`=36, `binlog_replay`=37, `replica_monitor`=38, `show_create_routine`=39. Hanya di-set YES; **tak pernah dibersihkan ke NO** (301–303). **UNKNOWN**: bit di luar 20–39 tak didekode di sini (diharapkan dari kolom `mysql.user`).
- `show_create_routine` ditangani lewat key dukungan sintetik `show_create_routine_global_priv` (`.h:128–131`, 154–162), di-remap untuk entitas `SPUser` di `_supportedPrivilegeKeysForEntityName:` 170–172.
- Sinonim kolom MariaDB/MySQL di-collapse table-driven di `privColumnToGrantMap` (115–129).

**Lain-lain**
- **Hint error MariaDB:** `SPUserManagerPrivilegeOperationErrorMessageForServerError` (`.h:38–88`) menambah hint `SHOW CREATE ROUTINE` hanya bila server mendukung priv itu (via `privsSupportedByServer[@"show_create_routine_priv"]` 1694) dan statement gagal mungkin melibatkannya.
- **Password:** `requiresPost576PasswordHandling` dari absennya kolom `Password` (389); field password di-seed sentinel `"sequelpro_dummy_password"` (514). `insertUser:` pakai hash `authentication_string` tersimpan untuk parent existing dan plaintext UI hanya untuk user baru (1284–1319).
- **`contextDidChange:` (1164–1167) tak terjangkau:** grep seluruh repo hanya menemukan definisi; tak pernah diregistrasi (hanya `_initializeSchemaPrivs` 137–140 dan `contextWillSave:` 719–722) dan absen dari extension (54–81). **UNKNOWN**: dianggap sisa; reload outline-nya dead.
- **`_clearData` (1076–1080) tanpa pemanggil**; `refresh:` melakukan reset inline (998).
- **Selector notifikasi nol-argumen:** `_initializeSchemaPrivs` diregistrasi `@selector(_initializeSchemaPrivs)` (138) tapi didefinisikan tanpa parameter (577) — hanya bekerja via dukungan legacy zero-arg. Fragilitas laten.
- **Recycle store manual di `refresh:`** menghapus semua persistent store lalu re-add `NSInMemoryStoreType` (1008–1017) sebelum `_initializeUsers` (1020).
- **Escape-heavy schema key:** nama schema di-escape `_`→`\_` sebelum fetch priv (1047, 1361, 1751, 1809) dan di-unescape saat dibaca kembali (660).
- **Side-effect UI di delegate:** `outlineViewSelectionDidChange:` menulis ulang host nil menjadi `"%"` (1940–1944) — mutasi model dari callback seleksi.
- **Macro legacy `NS_DURING/NS_HANDLER`** menelan exception untuk key `_priv` tak termodel di `_initializeChild:` (598–619), `grantDbPrivilegesWithPrivilege:skippingRevoke:` (1373–1381), `grantPrivilegesToUser:skippingRevoke:` (1467–1475) — gagal senyap.
