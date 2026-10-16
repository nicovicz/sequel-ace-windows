# Server tools: user manager, process list, server variables, go-to-database

Kontrak source untuk jendela/sheet server-tools. Sitasi `file:line` merujuk
checkout `02c910932`.

Entry point (dari `SPDatabaseDocument`):

| Sheet | Dibuka oleh | File:line |
| --- | --- | --- |
| User manager | `-[SPDatabaseDocument showUserManager]`; pre-check `SELECT user FROM mysql.user LIMIT 1` | `SPDatabaseDocument.m:2065, 2075, 2083` |
| Process list | `-[SPDatabaseDocument showServerProcesses]` | `SPDatabaseDocument.m:913-920` |
| Server variables | `-[SPDatabaseDocument showServerVariables]` | `SPDatabaseDocument.m:900-907` |
| Go to Database | `-[SPDatabaseDocument showGotoDatabase]` | `SPDatabaseDocument.m:1023-1042` |

Semua mendapat `mySQLConnection` live di-inject (`SPDatabaseDocument.m:904, 917, 2070`).

## 1. User manager

File: `SPUserManager.h/.m` (2001 baris); `SAUserManagerPrivilegeNormalizer.swift`;
`Source/Model/CoreData/SPUserManager.xcdatamodel/contents`; `SPUserMO.{h,m}`, `SPPrivilegesMO.{h,m}`.

### 1.1 Model Core Data (in-memory saja)

`SPUserManager.xcdatamodel/contents` mendefinisikan dua entity: **`SPUser`** (kelas `SPUserMO`)
dan **`Privileges`** (kelas `SPPrivilegesMO`). Store `NSInMemoryStoreType`, tak ada persistensi
disk (`SPUserManager.m:697, 1017`).

**Entity `SPUser`** (`contents:29`): `user` String (`:89`); `host` String default `%` (`:54`);
`originaluser`/`originalhost` String (pelacakan edit) (`:65,:61`); `originalpassword` (`:62`);
`password` (`:68`); `plugin` (`:69`); `authentication_string` (`:33`);
`max_connections`/`max_questions`/`max_updates` Integer 32 (`:58-60`); transient `userManager` (`:90`).
Boolean `*_priv` global (`:30-88`), termasuk priv dinamis MySQL 8 (`binlog_admin_priv`,
`connection_admin_priv`, `read_only_admin_priv`, `replication_slave_admin_priv`,
`set_any_definer_priv`, `allow_nonexistent_definer_priv`, `create_role_priv`, `drop_role_priv`,
`create_tablespace_priv`, `federated_admin_priv`, `binlog_monitor_priv`, `binlog_replay_priv`,
`replica_monitor_priv`, `replication_master_admin_priv`, `set_user_priv`) dan MariaDB
(`show_create_routine_priv`, `delete_versioning_rows_priv`). Relasi: `children` self-ref (toMany,
cascade) ↔ `parent` (`:91,:92`) — parent = username, child = satu baris host;
`schema_privileges` toMany → `Privileges` (`:93`).

**Entity `Privileges`** (`contents:3`): `db` String max64 indexed (`:10`), transient `userManager` (`:26`),
relasi `user` → SPUser inverse `schema_privileges` (`:27`); Boolean `*_priv` scope DB:
`select_priv`, `insert_priv`, `update_priv`, `delete_priv`, `create_priv`, `drop_priv`,
`references_priv`, `index_priv`, `alter_priv`, `create_view_priv`, `show_view_priv`,
`create_routine_priv`, `alter_routine_priv`, `execute_priv`, `trigger_priv`, `event_priv`,
`lock_tables_priv`, `create_temporary_tables_priv`, `grant_option_priv`,
`delete_versioning_rows_priv`, `show_create_routine_priv` (`contents:4-25`).

Model meneruskan validasi ke controller via transient key `userManager` (`SPUserManager.m:1154-1162`):
`SPUserMO validateForInsert/Delete/Update → insertUser:/deleteUser:/updateUser:`;
`SPPrivilegesMO validateForUpdate → grantDbPrivilegesWithPrivilege:`. Tree: parent = user,
child = host (`_initializeTree` `:446`; `SPUserMO displayName`).

### 1.2 Query pembacaan user/host/privilege (SQL persis)

Di `_initializeUsers` (`:363-440`):
- `SELECT * FROM mysql.user ORDER BY user` — `:372`. `requiresPost576PasswordHandling = !containsObject:@"Password"` (`:389`). Bila hasil tak punya `Password` maupun `authentication_string`, warn + abort (`:375-387`).
- `SHOW PRIVILEGES` — `:397`; iterasi, skip `Usage`, map nama→key, tandai `privsSupportedByServer[privKey]=@YES` (`:400-412`).
- Fallback bila `SHOW PRIVILEGES` kosong: `SHOW COLUMNS FROM mysql.user` — `:415`, kumpulkan tiap kolom `*_priv` (`:419-428`).

Di `_initializeTree` (`:446-548`):
- `SELECT * FROM mysql.db` — `:451`; baris dikelompokkan per `User`; `Db` `""`/`"%"` masuk daftar schema (`:464-471`).

Priv global MariaDB (`_mariaDBGlobalPrivilegeAccessDataByAccount` `:187-222`):
- `SELECT User, Host, Priv FROM mysql.global_priv` — `:194`, hanya MariaDB ≥ 10.4 (`:192`). Parse JSON `Priv` untuk key `"access"`, catat nilai unsigned-long-long per akun (`:208-218`). Bit didekode di `_initializeMariaDBGlobalPrivsForChild` (`:279-305`), mis. `binlog_monitor_priv`=bit20, `set_user_priv`=30, `show_create_routine_priv`=39.

Priv dinamis MySQL 8 (`_mySQLDynamicPrivilegeDataByAccount` `:224-277`):
- `` SELECT `USER` AS User, `HOST` AS Host, `PRIV` AS Priv, `WITH_GRANT_OPTION` AS GrantOption FROM mysql.global_grants `` — `:232`, hanya `isMySQL8` (`:230`). Simpan hanya key termodel (`:251`); `WITH_GRANT_OPTION = 'Y'` (`:263-271`).

**Parsing/normalisasi grant**: `privColumnToGrantMap` memetakan kolom mysql → key model:
`grant_priv→grant_option_priv`, `show_db_priv→show_databases_priv`,
`create_tmp_table_priv→create_temporary_tables_priv`,
`repl_slave_priv`/`replication_replica_priv`/`repl_replica_priv→replication_slave_priv`,
`repl_client_priv→replication_client_priv`, `repl_slave_admin_priv`, `repl_master_admin_priv`,
`slave_monitor_priv→replica_monitor_priv`, `truncate_versioning_priv`/`delete_history_priv→delete_versioning_rows_priv`,
`show_create_routine_priv` (`:115-129`). Reverse (key→sintaks GRANT) `SPUserManagerGrantNameForPrivilegeKey`
memotong `_priv`, `_`→spasi — **kecuali** 6 key dinamis MySQL dalam
`SPUserManagerMySQLDynamicPrivilegeKeysRequiringUnderscoreGrantNames()` yang tetap underscore —
lalu `SAUserManagerPrivilegeNormalizer.normalizedGrantName:isMariaDB:` menulis ulang
`"delete versioning rows"` → `"delete history"` di MariaDB (`SAUserManagerPrivilegeNormalizer.swift:9-20`).

> Grant dibaca **langsung dari tabel grant mysql**, bukan dari `SHOW GRANTS` (yang hanya muncul di string error).

### 1.3 Add/rename/delete user

**Add** `addUser:` (`:839-856`): parent `SPUserMO` + child host default `localhost` (`:850`). `insertUser:`
(`:1266-1345`): post-5.7.6 → `CREATE USER 'user'@'host' IDENTIFIED WITH '<plugin>' AS '<hash>'`
(bila parent sudah ada, `:1293`) atau `... IDENTIFIED BY '<plaintext>'` (`:1298`); legacy →
`CREATE USER 'user'@'host' IDENTIFIED BY [PASSWORD ]'<pw>'` (`:1319,:1322`); user tanpa password →
`CREATE USER 'user'@'host'` (`:1328`); lalu `updateResourcesForUser:` + `grantPrivilegesToUser:skippingRevoke:YES`.

**Rename** `_renameUserFrom:host:to:host:` (`:1717-1727`): `RENAME USER 'olduser'@'oldhost' TO 'newuser'@'newhost'`
(dari `updateUser:` untuk ganti username `:1183-1187` atau host `:1231-1237`).

**Delete** `deleteUser:` (`:1251-1264`): `DROP USER 'user'@'host'` (dilewati saat init / tanpa host `:1254`).

**Password**: post-5.7.6 satu statement semua host — `ALTER USER 'user'@'host' IDENTIFIED WITH '<plugin>' BY '<plaintext>'`
(`:1196-1208`); legacy per host — `SET PASSWORD FOR 'user'@'host' = PASSWORD('<pw>')` (`:1217-1223`).

**Resource limits** `updateResourcesForUser:` (`:1405-1438`):
`ALTER USER 'user'@'host' WITH MAX_QUERIES_PER_HOUR <n> MAX_UPDATES_PER_HOUR <n> MAX_CONNECTIONS_PER_HOUR <n>`;
fallback `UPDATE mysql.user SET max_questions = <n>, max_updates = <n>, max_connections = <n> WHERE User = 'user' AND Host = 'host'`.

### 1.4 Tingkat privilege

Dua tingkat saja:
- **Global Privileges** (`UserManagerView.xib:300`) → Boolean SPUser → `*.*`.
- **Schema Privileges** (`UserManagerView.xib:1155`) → `Privileges` per `db` → `` `db`.* ``.

**Tidak ada scope tabel/kolom**: tidak ada SQL `mysql.tables_priv`/`mysql.columns_priv` yang dibaca/ditulis
di `SPUserManager.m` (hanya `mysql.user`, `mysql.db`, `mysql.global_priv`, `mysql.global_grants`).

Aksi: `checkAllPrivileges:`/`uncheckAllPrivileges:` (`:801-834`); `grantPrivilegesToUser:[skippingRevoke:]`
(`:1440-1499`) — hanya key berubah (`changedValues`) → GRANT/REVOKE, lalu rekursi ke tiap `schema_privileges`;
`addSchemaPriv:`/`removeSchemaPriv:` (`:930-1074`); `grantDbPrivilegesWithPrivilege:[skippingRevoke:]` (`:1347-1400`).

### 1.5 GRANT/REVOKE/DDL yang dibentuk (persis)

`_grantPrivileges:onDatabase:forUser:host:` (`:1575-1624`): shortcut "All" bila semua priv terpilih (`:1580`) →
`GRANT ALL ON `<db>`.* TO '<user>'@'<host>' WITH GRANT OPTION` (global `ON *.*`, `:1582-1585`); else →
`GRANT <PRIV1>, <PRIV2> ON `<db>`.* TO '<user>'@'<host>'` ( atau `ON *.*`), dengan ` WITH GRANT OPTION`
ditambahkan untuk batch priv dinamis MySQL 8 (`:1612-1617`).

`_revokePrivileges:onDatabase:forUser:host:` (`:1629-1662`): shortcut All →
`REVOKE ALL PRIVILEGES ON `<db>`.* FROM '<user>'@'<host>'` + `REVOKE GRANT OPTION ON `<db>`.* FROM '<user>'@'<host>'`;
else → `REVOKE <PRIV1>, <PRIV2> ON `<db>`.* FROM '<user>'@'<host>'`.

Quoting identifier: `` `db` `` via `backtickQuotedString` (`SPStringAdditions.m:233`); user/host via
`tickQuotedString` (`:244`).

### 1.6 FLUSH PRIVILEGES

Dijalankan sekali per **Apply**, setelah save Core Data, apa pun hasilnya — `doApply:` `:779`:
`[connection queryString:@"FLUSH PRIVILEGES"]`.

### 1.7 String user manager

`"Resultset from mysql.user contains neither 'Password' nor 'authentication_string' column."` (`:377`);
`"User Data Error"` (`:381`); `"Unsaved changes"`/`"Changes have been made, which will be lost if this window is closed. Are you sure you want to continue"` (`:984-985`);
`"Resource Limits are not supported for your version of MySQL. Any Resouce Limits you specified have been discarded and not saved. MySQL said: %@"` (`:1433`);
`"An error occurred whilst trying to perform the operation.\n\nMySQL said: %@"` (`:1674`);
`"Unable to remove host"` (`:923`); `"User has no hosts"` (`:1857`); `"Duplicate User"`/`"A user with the name '%@' already exists"` (`:1961`);
`"Duplicate Host"` (`:1970`); `"All Databases"`/`"All Databases (%)"` (`:1988,:1990`);
`"Could not %@ %@ on %@ for %@.\n\nMySQL said: %@"` (`SPUserManager.h`);
`"Unable to get list of users"`/`"An error occurred while trying to get the list of users. Please make sure you have the necessary privileges to perform user management, including access to the mysql.user table."` (`SPDatabaseDocument.m:2079`).
XIB: judul window `"User Managment"` **[sic, salah eja di source]** (`UserManagerView.xib:34`); header `"Accounts"` (`:67`);
tab `"General"`/`"Global Privileges"`/`"Schema Privileges"`/`"Resources"` (`:220,300,1155,1340`);
box `"Login Information"`/`"Views and Procedures"`/`"Database and Tables"`/`"Administration"`/`"Replication"`/`"Resource Limits"` (`:225,309,431,612,1066,1345`);
field `"Username:"`/`"Password:"`/`"Max Updates:"`/`"Max Connections:"`/`"Max Questions:"` (`:235,244,1355,1364,1373`);
tombol `"Check All"`/`"Uncheck All"` (`:604,423`); context menu `"Add Host"`/`"Remove Host"`/`"Refresh"`/`"Remove User"` (`:119,124,130,1582,1593,1600`).

## 2. Process list

File: `SPProcessListController.{h,m}`, `DatabaseProcessList.xib`, serializer
`SPProcessListRowSerializer` (`StringExtension.swift:238`).

- **SQL**: `SPMySQLResult *processList = (showFullProcessList) ? [connection queryString:@"SHOW FULL PROCESSLIST"] : [connection listProcesses];`. Jalur non-FULL = `mysql_list_processes()` (`Server Info.m:104-118`). `showFullProcessList` dari pref `ProcessListShowFullProcessList`.
- **Auto-refresh**: pref `SPProcessListEnableAutoRefresh` (`SPConstants.m:210`) + `SPProcessListAutoRrefreshInterval` (`SPConstants.m:211`, **salah eja**). Interval menu 2/5/10/30/60 s + `Custom…` (`DatabaseProcessList.xib:263-298`).
- **Filter**: `controlTextDidChange:` → match case-insensitive pada `Id, User, Host, db, Command, Time, State, Info` (dan `Progress` bila kolom terlihat). Label count `"Showing %lu of %lu processes"`.
- **Kill**: `_killProcessQueryWithId:` → `KILL QUERY <id>` (atau `KILL TIDB QUERY <id>` bila versi memuat `TiDB`); `_killProcessConnectionWithId:` → `KILL CONNECTION <id>` (atau `KILL TIDB CONNECTION <id>`).
- **Kolom**: `Id`→"ID" (`:51-52`), `User`→"User", `Host`→"Host", `db`→"Database", `Command`→"Command", `Time`→"Time", `State`→"State", `Info`→"Info", `Progress`→"Progress" (`:155-156`, MariaDB saja; disembunyikan bila `![connection isMariaDB]`). `Id` digerbangi pref `ProcessListShowProcessID` (`SPConstants.m:208`); `Info` dipotong 100 char + `…` bila FULL off.
- **Copy/save** serialisasi `Id User Host db Command Time State Info [Progress]` join spasi (`StringExtension.swift:262-284`).

String: `"Server Processes on %@"`; `"Kill query?"`/`"Are you sure you want to kill the current query executing on connection ID %lld?\n\n…"`; `"Kill connection?"`/`"…connection ID %lld?…"`; `"Unable to kill query"`/`"An error occurred while attempting to kill the query associated with connection %lld.\n\nMySQL said: %@"`; `"Unable to kill connection"`; `"Showing %lu of %lu processes"`.
XIB: window `"Processes"` (`:27`); `"Filter"` (`:193`); tooltip `"Refresh process list (⌘R)"` (`:217`); `"Auto refresh"` (`:311`); menu `"Show Process ID"` (`:240`), `"Show FULL Process List"` (`:251`), `"Auto Refresh Interval"` (`:259`), `"2 Seconds"`…`"Custom..."` (`:263-294`); context menu `"Copy"`, `"Kill Query"`, `"Kill Connection"` (`:341,348,354`); `"Custom Interval"` (`:366`).

## 3. Server variables

File: `SPServerVariablesController.{h,m}`, `DatabaseServerVariables.xib`.

- **SQL**: `SPMySQLResult *serverVariables = [connection queryString:@"SHOW VARIABLES"];` lalu `setReturnDataAsStrings:YES`.
- **Filter**: `controlTextDidChange:` → match case-insensitive pada `Variable_name` dan `Value`; count `"%lu of %lu"`; tombol save berubah `"Save As..."`/`"Save View As..."`.
- **Save**: `NSSavePanel` `.cnf` (`UTType typeWithFilenameExtension:@"cnf"`), nama default `"ServerVariables"`, header `# MySQL server variables for <host>` lalu `Variable_name = Value` per baris.
- **Copy**: `copy:` (nama=value), `copyServerVariableName:`, `copyServerVariableValue:`.

XIB: window `"Variables"` (`:20`); kolom `"Variable Name"`/`"Value"` (`:45,:57`); `"Close"`/`"Save As..."` (`:93,:107`); header `"Server Variables"` (`:118`); `"Filter"` (`:127`); menu `"Copy"`, `"Copy Variable Name"`, `"Copy Variable Value"` (`:154,161,167`).

## 4. Go to Database

File: `SPGotoDatabaseController.{h,m}`, `GotoDatabaseDialog.xib`.

- **Tidak mengeksekusi SQL sendiri** — caller menyuplai list (`setDatabaseList:`) dan mengonsumsi (`selectedDatabase`). `showGotoDatabase` membangun list dari `allSystemDatabaseNames` + `allDatabaseNames` (`SPDatabaseDocument.m:1028-1031`); bila nama memuat `.`, dipecah → `selectDatabase:<first> item:<last>`, else `selectDatabase:<name> item:nil` (`:1033-1041`).
- **Filtering** `_buildFilterList:didFindExactMatch:`: word search bila term dibungkus quote (`StringQualifiesForWordSearch`); default subsequence match `nonConsecutivelySearchString:matchingRanges:`. Hasil diurut by jumlah match group lalu alfabetis. Dengan `allowCustomNames` (default YES `:88`) dan tanpa exact match, nama mentah ditambahkan sebagai item kustom (quote-stripped, ditandai `⁈` + underline).
- **Keyboard**: ESC tutup, ↑/↓ pindah seleksi, Return = OK; `toggleWordSearch:` membungkus/melepas quote; double-click = OK.

XIB: window `"Go to Database"` (`:20`); placeholder `"Database Name"` (`:60`); `"OK"`/`"Cancel"` (`:32,:46`); header `"Database"` (`:86`); menu `"Search as word"` (`:121`).

## 5. UNKNOWN

- `SPProcessListController` memakai `showFullProcessList = [prefs boolForKey:…]` sebelum `prefs` di-assign di `init` — perilaku bergantung semantik `+[NSUserDefaults standardUserDefaults]`.
- Scope grant level tabel/kolom absen by design (§1.4); **UNKNOWN** apakah subsistem lain mengelola `tables_priv`/`columns_priv`.
- `GotoDatabaseController` tak query apa pun; semua pencocokan in-memory.
- Posisi bit akses MariaDB (bit 20-39) dan rename `delete versioning rows`→`delete history` mengkodekan spesifik versi server.
