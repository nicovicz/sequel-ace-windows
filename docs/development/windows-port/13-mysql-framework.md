# Framework MySQL (SPMySQLFramework)

Kontrak source untuk `Frameworks/SPMySQLFramework/` — pembungkus ObjC atas
`libmysqlclient` yang melakukan seluruh I/O protokol MySQL. Sitasi `file:line`
relatif terhadap `Frameworks/SPMySQLFramework/` dan merujuk checkout `02c910932`.

Bundled client: libmysqlclient **8.4.11** (`MySQL Client Libraries/include/mysql_version.h:12,24`;
`lib/libmysqlclient.24.dylib`), protokol V10 (`mysql_version.h:11`).

## 1. Inventaris file per peran

**Connection core**: `Source/SPMySQLConnection.{h,m}` (239/1716 baris) — init, connect/disconnect,
state, cipher list; `SPMySQL Private APIs.h`. Kategori (`Source/SPMySQLConnection Categories/`):
`Querying & Preparation`, `Databases & Tables`, `Encoding`, `Server Info`, `Ping & KeepAlive`,
`Max Packet Size`, `Locking`, `Conversion`, `Delegate & Proxy`, `Copying`. Protokol:
`SPMySQLConnectionDelegate.h`, `SPMySQLConnectionProxy.h`; `SPMySQLConstants.h`, `SPMySQLDataTypes.h`.

**Result set**: `SPMySQLResult.{h,m}`, `SPMySQLEmptyResult.{h,m}`; kategori
`SPMySQLResult Categories/` (`Data Conversion`, `Field Definitions`, `Convenience Methods`).

**Streaming/store**: `SPMySQLStreamingResult.{h,m}`, `SPMySQLFastStreamingResult.{h,m}`,
`SPMySQLStreamingResultStore.{h,m}` + `SPMySQLStreamingResultStoreDelegate.h`.

**Data/helper**: `SPMySQLGeometryData.{h,m}`, `SPMySQLStringAdditions.{h,m}`, `SPMySQLArrayAdditions`,
`SPMySQLMutableDictionaryAdditions`, `SPMySQLUtilities.h`, `SPMySQLKeepAliveTimer.{h,m}`, `SPMySQL.h`.

**Kelas pendukung Swift** (`Source/*.swift`): `SAConnectionEscaper`+`SAOfflineEscapingHandle`,
`SAConnectionSessionAccess`, `SADatabaseAssertion`, `SAConnectionCharacterSets`,
`SAByteStringDecoder`, `SASessionStartupPlan`, `SASessionStateTracking`,
`SASessionTimeZoneRestorer`, `SAProxyReconnectCoordinator`, `SACleartextAuthPolicy`.

Tidak ada kode TLS/encoding/greeting milik framework sendiri: TLS, escaping, dan auth plugin
didelegasikan ke libmysqlclient (`SPMySQLConnection.m:1046, 1039-1067`) + `SAOfflineEscapingHandle.swift`.

## 2. Siklus hidup koneksi

**Default** (`SPMySQLConnection.m:344-441`): port 3306 (`:354`), `useSocket = YES` (`:359`),
`timeout = 30` (`:372`), `useKeepAlive = YES` + `keepAliveInterval = 60` (`:373-374`),
encoding `utf8mb4`/`NSUTF8StringEncoding` (`:382-383`), `delegateQueryLogging = YES` (`:394`),
`retryQueriesOnConnectionFailure = YES` (`:430`), `maxQuerySize = 1048576` (`:423`),
timer keep-alive tiap 10 s (`:435`). Client flag default = compression + interactive + multi-results
(`:75-78`; konstanta `SPMySQLConstants.h:78-82`).

**connect/reconnect/disconnect**:
- `-connect` set `userTriggeredDisconnect=NO` lalu `-_connect` (`:485-491`); `-reconnect` → `-_reconnectAllowingRetries:YES` (`:503-508`); `-disconnect` (`:513-518`).
- `-_connect` (`:761-889`): raise `NSInternalInconsistencyException` "Attempted to connect a connection that is not disconnected (SPMySQLConnectionState=%d)." (`:773`); lock; `_makeRawMySQLConnectionWithEncoding:isMasterConnection:` (`:786`); track socket via `SAConnectionSessionAccess trackSocket:serverThreadID:error:` (`:808`); state `SPMySQLConnected` (`:818`); rekam session charset (`:821-827`); `initialConnectTime`, `mysqlConnectionThreadId`, `lastConnectionUsedTime` (`:830-835`); baca versi (`:844-846`); SSL state + fallback notification (`:848-854`); `_updateConnectionVariables` (`:868`); reset retry (`:883`); `_updateMaxQuerySize` (`:886`).
- `-_reconnectAllowingRetries:` bungkus `_performReconnectAllowingRetries:` (`:1158-1362`): simpan `encodingToRestore`/`databaseToRestore`/timezone (`:1187-1197`); teardown proxy + connect dengan timeout + runloop pumping (`:1214-1293`); `_connect` (`:1300`); restore session (`:1307-1310`); gagal → retry ≤5 kali lalu disconnect (`:1336-1343`).
- Network wait `-_waitForNetworkConnectionWithTimeout:` pakai `SCNetworkReachabilityCreateWithName`, poll 0.5 s (`:1419`), budget 10 s (`:1209`).
- Disconnect `-_disconnect` → `-_disconnectPreservingProxyReconnect:NO` (`:1432-1496`): cancel query aktif (`:1458`), state `SPMySQLDisconnecting` (`:1460`), tunggu ≤10 s lock (`:1464-1470`), cancel keep-alive (`:1473`), `mysql_close` hanya bila `!net.reading_or_writing && net.io && net.buff` (`:1479-1483`), reset `serverVersionNumber`, `forgetSession` escaper (`:1484-1489`).

**Handshake/autentikasi** (`-_makeRawMySQLConnectionWithEncoding:isMasterConnection:` `:895-1121`):
- `mysql_init` (`:900`), `MYSQL_OPT_RECONNECT=FALSE` (`:909-910`), `MYSQL_OPT_PROTOCOL` socket vs TCP (`:913-919`), `MYSQL_OPT_CONNECT_TIMEOUT` dari `timeout` (`:922`), `MYSQL_SET_CHARSET_NAME` (`:925-926`).
- Auth: plugin default dipaksa `mysql_native_password` (`:929`, issue #141); `MYSQL_PLUGIN_DIR` ke PlugIns framework (`:936-939`, issue #1036); `MYSQL_ENABLE_CLEARTEXT_PLUGIN` bila `enableClearTextPlugin` (`:947-950`, issue #368); `MYSQL_OPT_GET_SERVER_PUBLIC_KEY` bila `requestServerPublicKey` (`:952-955`); `MYSQL_OPT_LOCAL_INFILE` (`:942-944`, issue #245).
- Kredensial: password dari property else delegate `keychainPasswordForConnection:` (`:976-998`); delegate `credentialErrorMessageForConnection:` dapat membatalkan attempt tanpa menyentuh server (`:982-995`).
- Socket path: user path else `+findSocketPath` (`:1002-1013`); kandidat termasuk MAMP/XAMPP/MacPorts/Homebrew (`:689-715`).
- `mysql_real_connect` (`:1069`); retry non-TLS sekali bila diizinkan `SACleartextAuthPolicy allowsRetryWithoutTLS…` (`:1072-1076`); gagal → rekam message/`mysql_errno`/`mysql_sqlstate` hanya pada master connection (`:1079-1114`); `theConnection->reconnect = 0` (`:1117`).
- Auth yang didukung: apa pun dari libmysqlclient 8.4 (native, caching_sha2 + public-key request, sha256, cleartext plugin, plugin dir untuk ed25519 MariaDB). Deteksi MariaDB `(^.*)-[mariadb].*` (`:674-684`), MariaDB103 (`:661-672`); TiDB via string `KILL TIDB` (`:763`).

**Negosiasi SSL**:
- Diminta via `useSSL` + path key/cert/CA (`:1016-1034`); `mysql_ssl_set(...)` dengan cipher pre-TLS1.3 gabungan (`:1046`), lalu `MYSQL_OPT_TLS_CIPHERSUITES` dengan suite TLS1.3 bawaan; gagal → log `"Failed to set default TLS 1.3 cipher suites; continuing with libmysqlclient defaults."` (`:1047-1049`).
- `MYSQL_OPT_SSL_MODE` = `SSL_MODE_REQUIRED` bila `SACleartextAuthPolicy requiresTLSWithCleartextPluginEnabled:sslRequested:` else `SSL_MODE_PREFERRED` (`:1054-1055`); jika REQUIRED tak bisa diterapkan → gagal dengan `"libmysqlclient is missing support for MYSQL_OPT_SSL_MODE"`, `CR_SSL_CONNECTION_ERROR`, `"HY000"` (`:1057-1067`).
- Post-connect: `connectedWithSSL = mysql_get_ssl_cipher(...) != NULL` (`:849`); bila diminta tapi tak dipakai → delegate `connectionFellBackToNonSSL:` (`:850-854`), `-isConnectedViaSSL` (`:542-545`).
- Cipher default `+defaultSSLCipherList` (`:158-193`), `+legacySSLCipherList` (`:194-254`), `+_defaultTLSSuiteListString` (`:267`).

**Timeout** — connect `timeout` (`:922`); ping default 30 s atau `timeout` (`Ping & KeepAlive.m:174-175`), hard-kill `pingTimeout+2` (`:216-220`); disconnect lock wait 10 s (`:1466`); keep-alive cancel wait 10 s (`Ping & KeepAlive.m:341`). Reading/writing timeout sengaja **tidak** diset (`Ping & KeepAlive.m:141-145`).

**Ping/keep-alive/reconnect**:
- `-_keepAlive` tiap 10 s (timer `:435`), skip bila interval-1 belum lewat (`Ping & KeepAlive.m:64-69`), tandai `SPMySQLConnectionLostInBackground` setelah 3 gagal (`:72-75`).
- `-_threadedKeepAlive` (`:93-135`): bila ping terakhir blocked / ≥3 gagal → background reconnect bila dipakai dalam 15 menit (`:110-111`) else state lost (`:116`); else `_pingConnectionUsingLoopDelay:10000` (`:124-129`); log `"warning: overwriting existing keepAliveThread: %@, results may be unpredictable!"` (`:98`).
- `-_pingConnectionUsingLoopDelay:` (`:150-240`): lock, pthread `_backgroundPingTask` (`:193`), cancel di timeout, `pthread_kill(SIGUSR1)` + `keepAliveLastPingBlocked` di +2 s (`:216-220`); `_backgroundPingTask` panggil `mysql_ping` (`:273`).
- `-checkConnection` (`:557-604`): NO kecuali connected; bila busy dan tak ada ping thread → YES (`:569-573`); else tunggu ping thread (`:578-580`); ping loop delay 400 (`:586`) dan gagal → reconnect (`:592-595`).
- `-checkConnectionIfNecessary` (`:617-633`): lost-in-background → reconnect; skip bila dipakai < 30 s lalu (`:627`).
- `-isConnected` memicu reconnect saat `SPMySQLConnectionLostInBackground` (`:528-537`).
- Thread registration: `-_validateThreadSetup` (`mysql_thread_init` + observer `NSThreadWillExitNotification` `:1688-1704`); `+initialize` panggil `mysql_library_init` + catat ia memasang SIGPIPE handler MySQL (`:144-156`).

**Cancel/kill**: `-cancelCurrentQuery` (`Querying & Preparation.m:741-792`) membuka koneksi **terpisah** (`:751`), jalankan `KILL [TIDB] QUERY <thread_id>` (`:762-767`), set `lastQueryWasCancelled = YES` (`:747,782`). Helper `-killQueryOnThreadID:` (`Server Info.m:136-153`). Keputusan delegate `connectionLost:` (`SPMySQLConnectionDelegate.h:116`) ditanya di main thread dengan guard modal (`Delegate & Proxy.m:153-198`).

**Kontrak delegate** (`SPMySQLConnectionDelegate.h`, semua `@optional`): `willQueryString:connection:` (`:42`),
`queryGaveError:connection:` (`:51`), `showErrorWithTitle:message:` (`:62`),
`keychainPasswordForConnection:` (`:73`), `credentialErrorMessageForConnection:` (`:85`),
`noConnectionAvailable:` (`:97`), `connectionFellBackToNonSSL:` (`:106`), `connectionLost:` (`:116`).
Delegate result-store: `resultStoreDidFinishLoadingData:` (`SPMySQLStreamingResultStoreDelegate.h:41`).

## 3. Eksekusi query

**Entry point** (`Querying & Preparation.h:43-61`): `queryString:`, `streamingQueryString:`,
`streamingQueryString:useLowMemoryBlockingStreaming:` (YES→`SPMySQLStreamingResult`, NO→`SPMySQLFastStreamingResult`, `:299-307`),
`resultStoreFromQueryString:`, generik `queryString:usingEncoding:withResultType:[assertingDatabase:|assertingDatabaseContext:]`.

Mapping result store di `_queryString:` (`:543-569`): `SPMySQLResultAsResult`→`mysql_store_result`+`SPMySQLResult`;
low-mem→`mysql_use_result`+`SPMySQLStreamingResult`; fast→`SPMySQLFastStreamingResult`;
store→`SPMySQLStreamingResultStore` (download deferred); tanpa field-count → `SPMySQLEmptyResult` (`:577`).

**Serialisasi**: semua jalur lewat `SAConnectionSessionAccess performQuery:recover:` (`:348-356`).

**Jalur sinkron** `_queryString:` (`:377-615`): nil bila `userTriggeredDisconnect` (`:390`) atau
disconnected/connecting dengan delegate `queryGaveError:@"No connection available!"` + `noConnectionAvailable:` (`:394-404`);
`_validateThreadSetup` (`:407`); `checkConnectionIfNecessary` (`:410`); restore max-packet (`:413-415`);
`willQueryString:` (`:418-420`); buffer byte (`:423`); oversize → `_attemptMaxQuerySizeIncreaseTo:` (`:430-436`);
retry = 1 + `retryQueriesOnConnectionFailure` (`:439-440`); lock (`:444`); database assertion (`:460-475`);
`mysql_real_query` (`:478`); timing (`:480-481`); `mysql_affected_rows` (`:488`);
error `mysql_error`/`mysql_errno`/`mysql_sqlstate` (ASCII/latin1, `:507-510`);
retry hanya bila tak dicancel dan `+isErrorIDConnectionError:` (`:514-516`).

**Affected rows/last insert id**: `rowsAffectedByLastQuery` (`:655-658`) dari `lastQueryAffectedRowCount`
(set `:609`, atau `mysql_affected_rows` `:550`; init `~0` `:449`); `lastInsertID` (`:664-667`) dari `insert_id` (`:582-584`).

**Error surfacing**: `queryErrored` (`:675-678`), `lastErrorMessage` (`:684-688`), `lastSqlstate` (`:690-694`),
`lastErrorID` (`:700-703`); setter `_updateLastErrorMessage:` (`:845-861`), `_updateLastErrorID:` (`:869-879`),
`_updateLastSqlstate:` (`:889-901`). Klasifikasi error koneksi `+isErrorIDConnectionError:` (`:708-731`):
2001-2007, 2009, 2012, 2013, 2027, 2032, 2047, 2048, 2050, 2055. Cancellation menimpa error dengan
`"Query cancelled."` / 1317 / `"70100"` (`:586-592`).

**Multi-result**: connect dengan `CLIENT_MULTI_RESULTS` (`SPMySQLConnection.m:78`); result yang tak diambil
di-flush tiap unlock: `mysql_more_results` → `_flushMultipleResultSets` (`Locking.m:91-102`), loop
`mysql_next_result`/`mysql_use_result`/`mysql_fetch_row`/`mysql_free_result` (`Querying & Preparation.m:807-827`),
log `SPLog(@"SPMySQLConnection: Discarding unretrieved results. This is currently normal when using CALL.")` (`Locking.m:100`).

**Streaming semantics**:
- `SPMySQLStreamingResult`: `numberOfRows` = NSNotFound hingga selesai (`:110-115`); `seekToRow:` raise `"Seeking is not supported in streaming SPMySQL result sets."` (`:123-126`); `cancelResultLoad` men-drain semua baris lalu unlock (`:184-208`); enumerasi linear else `NSRangeException` `"SPMySQLStreamingResult results can only be accessed linearly"` (`:225`); `dealloc` raise `"Parent connection remains locked after SPMySQLStreamingResult use"` bila unlock dua kali (`:96-99`).
- `SPMySQLFastStreamingResult`: cache linked-list, `pthread_mutex_t dataLock` (`FastStreamingResult.h:34-41`), background `_downloadAllData` (`:318`).
- `SPMySQLStreamingResultStore`: `startDownload` alokasi malloc zone `"SPMySQLStreamingResultStore_Heap"`, kapasitas 100 baris (`:241-260`); thread download `:668-808` menyalin baris ke single-block (layout `:695-699`), pilih lebar metadata char/short/long (`:711-717`), lock saat insert (`:763-781`), truncate baris basi (`:786-792`), `_updateLastErrorInfos` + unlock + `dataDownloaded` + delegate (`:795-806`). Accessor di luar bound raise `NSRangeException` (`:305,340,422`); editing raise `"Streaming SPMySQL result editing is currently only supported once loading is complete."` (`:525,556`).

## 4. Konversi data / encoding / escaping

**Type→object** (`Data Conversion.m:67-95`): DECIMAL/TINY/SHORT/LONG/FLOAT/DOUBLE/LONGLONG/INT24/DATE/TIME/DATETIME/YEAR/NEWDATE/VARCHAR/TIMESTAMP/JSON/NEWDECIMAL/ENUM/SET → string; `MYSQL_TYPE_BIT` → bit string; BLOB family → blob; VAR_STRING/STRING → string-or-blob; GEOMETRY → `SPMySQLGeometryData`; NULL → `NSNull`.
- **Angka dan tanggal dikembalikan sebagai `NSString`**, bukan `NSNumber`/`NSDate`.
- **NULL** → `NSNull` bersama (`:125-127, 169-170`); `SPMySQLStreamingResultStore` menyimpan null map BOOL terpisah (`:698-699,732`) via `cellIsNullAtRow:column:`.
- **Blob** → `NSData` (`:158`); dengan preview length, dipotong + sufiks `"..."` (`:149-157`). `returnDataAsStrings` memaksa BLOB→string (`:133-135`).
- **String vs blob** (`_processorForField` `:186-214`): STRING/VAR_STRING dengan `BINARY_FLAG` → blob; tipe blob tanpa `BINARY_FLAG` → string.
- **Bit** → string biner zero-padded dengan `MIN(bytes*8, field.length)` (`:165-166`).
- **Geometry** → `SPMySQLGeometryData dataWithBytes:length:version:` (`:162`).
- **Tipe tak dikenal** → `NSLog(@"SPMySQLResult processing encountered an unknown field type (%d), falling back to NSData handling", …)` (`:173`).
- **Preview truncation** menghitung *karakter* per encoding: UTF-16 (`:279-299`), UTF-8 (`:304-320`), CP932, EUC-JP, EUC-KR/CP949/GBK/EUC-CN, GB18030, Shift-JIS, 4-byte, 2-byte Big5/GB2312/UCS-2, else 1 byte/char (`:429-432`); menambah `"..."` (`:451`).
- **Byte→string**: `+[NSString stringForDataBytes:length:encoding:]` → `SAByteStringDecoder` dengan fallback ISO-Latin-1 yang tak bisa gagal.

**Row shape**: default `SPMySQLResultRowAsDictionary` (`SPMySQLResult.m:76`), array rows via `insertObject:atIndex:`; dict di-key `fieldNames`; fast enumeration batch 128 baris (`:279-283`); `getAllRows` mengembalikan nil (bukan array kosong) bila tanpa baris (`Convenience Methods.m:63-66`). Metadata field via `-fieldDefinitions` dengan key `name/org_name/table/org_table/db/byte_length/char_length/max_byte_length/flags/null/*_FLAG/decimals/charsetnr/charset_name/charset_collation/type/typegrouping` (`Field Definitions.m:343-438`).

**Encoding**: `+stringEncodingForMySQLCharset:` (`Encoding.m:241-376`) — `latin1`→**CP1252, bukan ISO-8859-1** (`:251`), `euckr`→CP949 (`:275-277`), `gb2312`→EUC-CN (`:281-283`), `latin5`→ISO-8859-9; `binary`→UTF8 (`:323-324`); tak cocok → log + fallback UTF-8 (`:374-375`). Reverse `+mySQLCharsetForStringEncoding:` (`:383-457`). Connection-side: `-setEncoding:` jalankan `SET NAMES` (`:89-152`); mode transport Latin1 `SET CHARACTER_SET_RESULTS/CLIENT=latin1` (`:165-197`); variabel post-connect dari `SHOW VARIABLES`, utamakan `character_set_results` lalu `character_set` lalu `character_set_client` (`SPMySQLConnection.m:1502-1535`).

**Escaping/quoting**:
- `escapeString:includingQuotes:` (`Querying & Preparation.m:75-177`): nil passthrough; butuh koneksi live, else delegate `noConnectionAvailable:` + nil (`:81-86`); konversi lossy ke `stringEncoding` (`:95`); buffer 2n+2; escape via `SAConnectionEscaper escapeBytes:length:into:characterSetOnRecord:sessionIsBeingReplaced:` terhadap charset transport (`:122-129`); gagal (`-1`) → `SPLog(@"[escapeString:includingQuotes]: the value could not be escaped for character set %@", encoding)` + nil (`:138-145`); quote byte `'` mentah; konversi balik `initWithData:encoding:stringEncoding` (`:172`).
- `escapeAndQuoteString:` = sama dengan quote (`:62-65`).
- `escapeData:includingQuotes:YES` → `X'<hex>'` via `mysql_hex_string` (`:196-230, 211, 218-220`).
- Helper teks `mySQLTickQuotedString` (`'`→`''`) dan `mySQLBacktickQuotedString` (`` ` ``→`` `` ``) di `SPMySQLStringAdditions.m:41-54`.
- Escaper nyata `SAConnectionEscaper` (`SAOfflineEscapingHandle.swift:110`): pilih charset dari handshake/session report/staleness (`:372-379`), pakai `mysql_real_escape_string` + fallback `mysql_real_escape_string_quote` saat `CR_INSECURE_API_ERR` (yaitu `NO_BACKSLASH_ESCAPES`, quote digandakan) (`:57-68`), buat handle offline `mysql_init` + `mysql_set_character_set` + `server_status` (`:36-46`).

## 5. Deteksi capability & versi server

- Versi diambil setelah connect (`mysql_get_server_info`, `mysql_get_server_version` `SPMySQLConnection.m:844-846`), di-decode Latin1 (`:837-843`); reset 0 saat disconnect (`:1485`).
- `serverVersionString` (`Server Info.m:42`), `serverMajorVersion = ver/10000` (`:54-57`), `serverMinorVersion` (`:63-67`), `serverReleaseVersion` (`:72-76`), `serverVersionIsGreaterThanOrEqualTo:minorVersion:releaseVersion:` (`:85-90`). `serverMajorVersion` diteruskan ke tiap result untuk parsing geometry (`SPMySQLResult.m:98`).
- Deteksi MariaDB (`:674-684`), MariaDB ≥10.3 (`:661-672`), TiDB via `serverVersionString` memuat `"TiDB"` (`:763`), ProxySQL via `select @@version_comment limit 1` lowercase mengandung `"ProxySQL"` (`:1633-1647`).
- Probing runtime (bukan handshake flag): `max_allowed_packet` (`Max Packet Size.m:124-146`, varian `SELECT @@global.max_allowed_packet` lalu `SHOW VARIABLES LIKE`, floor `>= 34`, komentar "see #2653"); probe editability `SET GLOBAL max_allowed_packet = @@global.max_allowed_packet` (`:152-157`).
- Server status bitfield → `SPMySQLServerStatusBits` (`Server Info.m:172-202`).
- Variabel yang menggerakkan perilaku: `interactive_timeout`<300 → `SET interactive_timeout=600`/`wait_timeout=600` (`SPMySQLConnection.m:1604-1613`); `information_schema_stats_expiry` != 0 → `SET information_schema_stats_expiry=0`, skip di ProxySQL (`:1616-1627`, issue #2006).

## 6. String error yang terlihat pengguna (dikutip persis)

- `"\n\ntime_zone will be set to SYSTEM."` (`SPMySQLConnection.m:739`), judul `"Error"` (`:740`), log `"Failed to set time_zone. Error: %@"` (`:733`).
- `"This connection has the cleartext authentication plugin enabled, which sends the password in plain text, so it is only made over TLS. TLS could not be established with the server and no password was sent."` (`:1109`).
- `"User triggered disconnection"` (`:1347`).
- `"The connection cannot be used while you are being asked what to do about it. Answer that question, then try again."` (`Querying & Preparation.m:370`), id 2013 / `"HY000"`.
- `"Query cancelled."` (`:589`), id 1317 / `"70100"`.
- `"The query length of %lu bytes is larger than max_allowed_packet size (%lu)."` (`Max Packet Size.m:176`), judul `"Error"`, tombol `"OK"`.
- `"libmysqlclient is missing support for MYSQL_OPT_SSL_MODE"` (`SPMySQLConnection.m:1059`).
- `"No connection available!"` (ke delegate `queryGaveError:connection:` `Querying & Preparation.m:398`).

Exception/log (console-facing): `"Attempted to connect a connection that is not disconnected (SPMySQLConnectionState=%d)."` (`SPMySQLConnection.m:773`);
`"SPMySQLFullStreamingResults should not be init'd as SPMySQLResults; …"` (`SPMySQLStreamingResult.m:56`);
`"Parent connection remains locked after SPMySQLStreamingResult use"` (`:98`);
`"Seeking is not supported in streaming SPMySQL result sets."` (`:125`);
`"SPMySQLStreamingResult results can only be accessed linearly"` (`:225`);
`"Data storage has already been assigned or created"` (`SPMySQLStreamingResultStore.m:137`);
`"Data download has already been started"` (`:244`);
`"Streaming SPMySQL result editing is currently only supported once loading is complete."` (`:525,556`);
`"Requested storage index (%llu) beyond bounds (%llu)"` (`:305`);
`"SPMySQLResult processing encountered an unknown field type (%d), falling back to NSData handling"` (`Data Conversion.m:173`);
`"Unhandled field type when processing SPMySQLResults"` (`:177`);
`"SPMySQL Framework has encountered the MySQL encoding '%s' which it is unable to process correctly; falling back to UTF8 mapping."` (`Encoding.m:374`);
`"warning: overwriting existing keepAliveThread: %@, results may be unpredictable!"` (`Ping & KeepAlive.m:98`);
`"SPMySQLConnection: Discarding unretrieved results. This is currently normal when using CALL."` (`Locking.m:100`).

## 7. Celah/UNKNOWN

- Tidak ada API prepared statement (`mysql_stmt_*`), `mysql_change_user`, atau wrapper `mysql_autocommit/commit/rollback` — kontrol transaksi lewat SQL teks oleh caller. **UNKNOWN** apakah disengaja atau sekadar tak dipakai.
- `mysqlConnectionThreadId` mencerminkan `mySQLConnection->thread_id`, tapi jalur cancel memakai `serverThreadID` milik `SAConnectionSessionAccess` — mana yang menang diputuskan di `SAConnectionSessionAccess.cancelQuery(usingKill:)` (tak dilacak lebih jauh).
- `-isNotMariadb103` nama/komentar bertentangan (regex mengembalikan false untuk 10.3+); perlakukan sebagai pasangan yang namanya membingungkan.
- `SPMySQLConnection.h:165` mendeklarasikan `timeZoneIdentifier` readonly tapi `.m:717-744` menugaskannya; deklarasi backing tak terlihat di header publik.
