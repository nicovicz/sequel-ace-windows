# Koneksi, favorites, dan keychain

Kontrak source untuk window Connection: form koneksi, tipe koneksi, state
machine koneksi, favorites (penyimpanan/urut/jumlah), deteksi duplikat,
import/export, keychain, dan parsing `mysql://`. Sitasi `file:line` merujuk
checkout `02c910932`.

File utama:

| File | Peran |
| --- | --- |
| `Source/Controllers/MainViewControllers/ConnectionView/SPConnectionController.m/.h` (5438 baris) | Controller window Connection (ObjC): form, favorites outline, mulai koneksi |
| `.../ConnectionView/SAConnectionWindowController.swift` | Window controller modern |
| `.../ConnectionView/SAConnectionFormModel.swift`, `SAConnectionFormView.swift`, `SAConnectionFormHelpers.swift`, `SAConnectionDetailsValidator.swift` | Model form, view SwiftUI, helper, validasi |
| `.../ConnectionView/SAFavorite*.swift` | Model/nav/daftar/search favorites |
| `Source/Controllers/DataControllers/SPFavoritesController.m/.h` | Muat/simpan `Favorites.plist` |
| `Source/Model/SAConnectionInfo.swift`, `SAConnectionInfo+Favorite.swift`, `SAConnectionInfo+ConnectionString.swift`, `ConnectionStringParser.swift` | Model koneksi value-type + favorite + connection string |
| `Source/Other/Keychain/*` | Akses keychain |
| `Source/Other/Utility/SAConnectionService.swift`, `SAConnectionAttemptSequence.swift`, `SAConnectionLostAlert.swift`, `SAConnectionLostSheetCopy.swift` | Layanan koneksi + attempt + lost-connection |

Enum tipe koneksi (`Source/Other/Data/SPConstants.h:57-63`):
`SPTCPIPConnection=0`, `SPSocketConnection=1`, `SPSSHTunnelConnection=2`,
`SPAWSIAMConnection=3`, `SPVaultConnection=4`. Mirror Swift `SAConnectionType`
(`SAConnectionInfo.swift`).

Mode zona waktu `SPConnectionTimeZoneMode` (`SPConnectionController.h:58-62`):
`UseServerTZ/UseSystemTZ/UseFixedTZ`.

## 1. Model favorites

### 1.1 Penyimpanan

- File: `~/Library/Application Support/Sequel Ace/Data/Favorites.plist` (XML
  plist), ditulis atomik dengan backup/restore. Path dari `SPDataSupportFolder =
  "Data"` (`SPConstants.m:102`) + `SPFavoritesDataFile = "Favorites.plist"`
  (`SPConstants.m:89`) via `applicationSupportDirectoryForSubDirectory:` —
  `SPFavoritesController.m:255,263` (load), `:417,425` (save).
- Save dijalankan di thread terpisah; plist diserialisasi **sebelum** operasi
  file; file lama dipindah ke backup UUID, direstore bila write gagal
  (`SPFavoritesController.m:395-460`).

### 1.2 Struktur plist

- Top-level: satu key `SPFavoritesRootKey = "Favorites Root"` (`SPConstants.m:344`)
  → dict **group** `{Name, IsExpanded, Children}` (`SPConstants.m:345-347`).
  Default file dibuat di `SPFavoritesController.m:270` (`Name:"Favorites"`, `Children:[]`).
- Saat load, node root sintetis dibuat dengan satu child group (nama di-uppercase
  `"FAVORITES"`) berisi semua favorite/group — `SPFavoritesController.m:308-343`.
- Sebuah dict adalah **group** iff punya `Name` **dan** `Children`; selain itu
  favorite — `SPFavoritesController.m:356`.
- Serialisasi: `SPTreeNode.dictionaryRepresentation` menyalin dict favorite apa
  adanya, atau mengemit `{Name, IsExpanded, Children}` untuk group — `SPTreeNode.m:258-291`.

### 1.3 Field favorite

Key didefinisikan di `SPConstants.m:348-388` dan di-inline ulang di
`SAConnectionInfo+Favorite.swift:36-71` (harus sinkron). Set lengkap:

`id`, `name`, `type`, `host`, `socket`, `port`, `user`, `colorIndex`, `database`,
`useCompression`, `timeZoneMode`, `timeZone`, `allowDataLocalInfile`,
`enableClearTextPlugin`, `requestServerPublicKey`, `useSSL`,
`sslKeyFileLocationEnabled`, `sslKeyFileLocation`, `sslCertificateFileLocationEnabled`,
`sslCertificateFileLocation`, `sslCACertFileLocationEnabled`, `sslCACertFileLocation`,
`sshHost`, `sshUser`, `sshKeyLocationEnabled`, `sshKeyLocation`, `sshPort`,
`sshRemoteSocketPath`, `useAWSIAMAuth`, `awsRegion`, `awsProfile`, `awsAccessKey`,
`awsUseProfile`, `vaultHost`, `vaultPort`, `vaultOIDCMount`, `vaultCredentialsPath`.
Key group: `Name`, `IsExpanded`, `Children`.

- Template favorite baru `SAConnectionInfoObjC.defaultNewFavoriteDictionary(withID:)`
  (`SAConnectionInfo+Favorite.swift`) — kuirk dipertahankan: tanpa key `useCompression`,
  tanpa key path SSL, `vaultPort`/`vaultOIDCMount` `""`, `awsProfile` `"default"`,
  `colorIndex` `-1`.
- Duplikat `duplicatedFavoriteDictionaryFromFavorite:withID:` — salin semua nilai,
  `id` baru, nama diberi sufiks `"%@ Copy"`.

### 1.4 Urutan

- Default unsorted (`SPFavoritesSortUnsorted`); sort pilihan: Name/Host/Type/ColorIndex.
  Item terpilih disimpan di `NSUserDefaults` `SPFavoritesSortedBy` (+
  `SPFavoritesSortedInReverse`), di-set `SPConnectionController.m:1610-1616`,
  dimuat `:4841`.
- `_sortFavorites` memetakan item sort→key (`SPConnectionController.m:3098-3118`);
  rekursif `_sortTreeNode:usingKey:` `:3142-3162`; comparator `_compareFavoritesUsingKey`
  `:3173+` — untuk key selain name, **group diurutkan ke atas** (`:3184-3189`).

### 1.5 Warna

- `colorIndex` int; **-1 = tanpa warna** (`SAConnectionInfo+Favorite.swift:121`,
  template `:217`).
- Palet = `SPFavoriteColorSupport.userColorList`; pada macOS ≥10.13 daftar tetap 7
  warna `favoriteRed/Orange/Yellow/Green/Blue/Purple/Graphite`
  (`SPFavoriteColorSupport.m:80-89`); warna lama dibaca dari `NSUserDefaults` key
  `SPFavoriteColorList` (`SPFavoriteColorSupport.m:92-102`).

### 1.6 Deteksi duplikat

`SAFavoriteDuplicateMatcher.favorite(_:isDuplicateOfHost:user:database:port:typeString:modeSpecificFields:)`:

- **Match dasar**: `host` **DAN** `user` **DAN** `database` **DAN** port
  ternormalisasi (sama) **DAN** `existingType == candidateType`.
- `normalizedPort(_:typeString:)`: NSNumber/String dinormalisasi; kosong → `"3306"`
  **kecuali socket**, di mana kosong tetap kosong.
- Mapping tipe `typeTag(forString:)`: `"SPSocketConnection"`→1, `"SPSSHTunnelConnection"`→2,
  `"SPAWSIAMConnection"`→3, **selain itu (termasuk Vault)→0 (TCP/IP)**. Kuirk sengaja:
  favorite Vault dibandingkan sebagai TCP/IP dua arah.
- **Mode-specific** (bila `modeSpecificFields` disuplai): sshTunnel →
  sshHost/sshUser/sshPort/sshRemoteSocketPath; socket → socket; awsIAM →
  awsRegion/awsProfile; TCP/IP → selalu match.
- Tree walk di ObjC `findDuplicateFavoriteForHost:...modeSpecificFields:`
  mengiterasi `favoritesRoot.allChildLeafs`, melewati group (`SPConnectionController.m:2506-2534`).

## 2. Keychain

### 2.1 Bentuk item (`SAKeychain.swift`)

- Berbasis `SecItem*`; menulis ke **file-based login keychain** (tidak pernah
  `kSecUseDataProtectionKeychain`), jadi item lama dibaca/diupdate/dihapus in place.
- Tipe `kSecClassGenericPassword`; `kSecAttrService` = nama; `kSecAttrAccount` =
  account; `kSecAttrLabel` = nama; `kSecAttrGeneric` = `Data("application password")`
  — `SAKeychain.swift:61,76-81`. Query by (service, account) `:94-104`.
- Perilaku: `add` no-op diam bila item ada; `updateItem` pulih dari
  `errSecItemNotFound` (add baru) dan `errSecDuplicateItem` (hapus tujuan, retry
  sekali); update password kosong membuat ulang item.

### 2.2 Aturan nama/account (`SAKeychainNaming.swift`)

- `name(favoriteName:id:)` → **`"Sequel Ace : <favoriteName> (<id>)"`** (`:53-55`).
- `account(user:host:database:)` → **`"<user>@<host>/<database ?? "">"`** (`:59-61`).
- `sshName(favoriteName:id:)` → **`"Sequel Ace SSHTunnel : <favoriteName> (<id>)"`** (`:65-67`).
- `sshAccount(user:host:)` → **`"<user>@<host>"`** (`:71-73`).
- Koersi id `longLongValue`: NSNumber→`int64Value`, String→`(as NSString).longLongValue`
  (`:81-86`).
- Untuk favorite **socket**, host dinormalisasi `"localhost"` untuk save dan lookup:
  `SPConnectionController.m:1771,2242`.

### 2.3 Yang disimpan

- **Hanya** password MySQL (`connectionKeychainItemName`/`...Account`) dan password
  SSH (`connectionSSHKeychainItemName`/`...Account`). Disimpan via
  `savePassword:forFavorite:` dan `saveSSHPassword:forFavorite:`
  (`SPConnectionController.m:2228-2271`), dibaca di `_resolvedMySQLPassword`
  (`:3709-3715`) dan `_resolvedSSHPassword` (`:3723-3729`).
- Sentinel placeholder `SAConnectionInfoObjC.keychainPasswordPlaceholder =
  "SequelAceSecretPassword"`: bila field sama dengan ini dan ada item name, nilai
  asli diambil dari keychain (`SPConnectionController.m:527-542,3710-3712`).
- SSL key, token AWS, kredensial Vault **tidak** disimpan di keychain oleh subsistem ini.

### 2.4 Fallback keychain nonaktif

- `SAKeychainAccess.make()` mengembalikan `SAKeychainDisabled()` (null object) bila
  env var **`LIBMYSQL_ENABLE_CLEARTEXT_PLUGIN`** diset (issue #2437); selain itu
  `SAKeychain()`. `SAKeychainDisabled` membuat semua op no-op, `password`→nil,
  `passwordExists`→false, helper nama→nil. Instance dibuat di `SPConnectionController.m:4802`.

## 3. State penting `SPConnectionController`

- `cancellingConnection` (`.h:78`): user cancel / doc tutup; NO di awal `.m:508`,
  YES `.m:781`; **tetap YES setelah cancel** (komentar `.m:1518`).
- `isConnecting` (`.h:79`): YES saat connect (`.m:507`), NO saat established/restore/fail
  (`.m:3566,3290,3866,3919`).
- `isEditingConnection` (`.h:80`): form diedit vs committed.
- `isTestingConnection` (`.h:81`): sender `testConnectButton` (`.m:418`).
- `connectionAttemptID` (`.h:82`): fence monotonik; ++ saat start (`.m:509`), cancel
  (`.m:782`), doc close (`.m:3529`).
- `errorShowing` (`.m:86`), `localNetworkPermissionDeniedForCurrentAttempt` (`.m:87`),
  `failedAttemptInfo` (`.m:89`).
- Detail: `previousType` (`.h:85`), `type` (`.h:86`), `name/host/user/password/database/socket/port`
  (`.h:87-93`), `colorIndex` (`.h:94`), `useCompression` (`.h:95`).
- Zona waktu: `timeZoneMode` (`.h:98`), `timeZoneIdentifier` (`.h:99`).
- Setting khusus: `allowDataLocalInfile` (`.h:102`), `enableClearTextPlugin` (`.h:105`),
  `requestServerPublicKey` (`.h:108`).
- AWS: `useAWSIAMAuth/awsRegion/awsProfile` (`.h:111-113`), map error token
  `awsIAMTokenErrorsByConnection` + lock (`.h:72-73`).
- Vault: `vaultHost/vaultPort/vaultOIDCMount` (`.h:117-119`), `vaultLoginIdentifier`
  (`.h:120`), `vaultMount/vaultCredentialsRole` (`.h:122-123`).
- SSL: `useSSL` + 3 pasang enabled/path (`.h:127-133`).
- SSH: `sshHost/sshUser/sshPassword/sshKeyLocationEnabled/sshKeyLocation/sshPort/sshRemoteSocketPath`
  (`.h:136-142`).
- Keychain identity: `connectionKeychainID/ItemName/ItemAccount` + varian SSH (`.h:144-148`).

## 4. Alur `initiateConnection:` (`SPConnectionController.m:399-769`)

1. Guard double-click outline: bila `clickedRow <= 0` return (`403`).
2. Guard refresh role Vault sedang jalan → warning "Vault role refresh in progress", return (`410-415`).
3. `isTestingConnection = (sender == testConnectButton)`; reset `localNetworkPermissionDeniedForCurrentAttempt = NO` (`418-419`).
4. **Pra-validasi** via `SAConnectionDetailsValidator` (`427-440`); saat gagal, bersihkan toggle/path yang cocok lalu alert dan return (`442-469`).
5. Cek AWS: bila IAM dan `!isAWSDirectoryAuthorized` → "AWS Authorization Required", return (`476-481`). Selain itu `SAAWSDirectoryWriteAccessPrompt requestWriteAccessIfNeededForProfile:` (`483-485`).
6. Prasyarat Vault (host, credentials path, database host) → alert "Insufficient connection details", return (`487-504`).
7. Masuk state connecting: `isConnecting=YES`, `cancellingConnection=NO`, `connectionAttemptID++`, `errorShowing=NO` (`507-510`); disable outline; sembunyikan help, disable Connect/Test, start progress (`515-522`).
8. Higiene password: bila item keychain diset dan bukan testing dan field masih sama, ganti field dengan `keychainPasswordPlaceholder` (`527-542`).
9. Notifikasi delegate `connectionControllerInitiatingConnection:` (`545-547`).
10. Trim host (`550`). Untuk SSH, trim sshHost dan jalankan `-_validateSSHConfigFile` (NO → abort) (`553-558`).
11. Bangun `SAConnectionInfoObjC *info = [self _buildConnectionInfo]` (`562`).
12. Pilih mode password: `deferMySQLPasswordToDelegate` = `shouldDeferMySQLPasswordToDelegateForInfo:password:delegateAvailable:` (`563-565`); `resolvedPassword` nil bila deferring/IAM, selain itu `-_resolvedMySQLPassword` (`570`); `resolvedSSHPassword = -_resolvedSSHPassword` (`573`).
13. `preferences = [SAConnectionPreferences fromUserDefaults]` (`576`); set progress text "Testing SSH…"/"Testing MySQL…" atau "SSH connecting…"/"MySQL connecting…" (`579-587`).
14. Tombol Connect jadi "Cancel" dengan aksi `cancelConnection:` (`589-593`).
15. Blok `connectCompletion` (`597-666`): userCancelled → restore; simpan `sshTunnel`; `databaseSelectionFailed` → set connection, bila testing "Invalid database" else `failConnectionWithTitle:@"Unable to connect"` + `"Connected but unable to select database '%@'."`; gagal → pilih title/message (override: SSH debug → "SSH port forwarding failed"; `lastErrorID==1045` → "Access denied!"; socket → "Socket not found!"/"Socket connection failed!"), lalu `_failConnectionWithTitle:…attemptInfo:info`; sukses → `mySQLConnectionEstablished`.
16. Dispatch:
    - **IAM** → `connectAWSIAMWithController:info:preferences:region:profile:attemptID:sshPassword:parentWindow:completion:` (`668-675`).
    - **Vault** → trim vault host; default port `"443"`, mount `"oidc"` (`681-683`); `vaultLoginIdentifier = [VaultOIDCHandler prepareActiveLogin]` (`686-687`); dispatch global queue, cek ulang cancel/attemptID; `VaultAuthManager generateCredentialsWithHost:port:oidcMount:credPath:loginIdentifier:username:password:error:` (`700-707`); gagal → "Vault Authentication Failed"; sukses → set `info.user`, `connectWith:...`.
    - **Lain** → `connectWith:info preferences:preferences password:resolvedPassword sshPassword:resolvedSSHPassword parentWindow:… completion:connectCompletion` (`763-768`).

**Error**: semua kegagalan lewat `failConnectionWithTitle:errorMessage:detail:` (`3804`): guard `errorShowing`; `_restoreConnectionInterface`; disconnect `sshTunnel`, deteksi "bind" → tawarkan "Use Standard Connection" (ubah tipe→TCP/IP, port dari regex `LOCALHOST:(\d+)`, host `127.0.0.1`, reconnect setelah 0.5 s) (`3821-3913`); notifikasi delegate; message = detail + "\n" + errorMessage; bisa menampilkan alert LAN-privacy (`3860-3871`); selain itu alert modal (`3873-3922`).

**Cancel** (`774-805`): disable Connect; "Cancelling…"; `cancellingConnection=YES`, `connectionAttemptID++`; cancel login Vault OIDC + role controller; `[connectionService cancel]`; nil delegate `mySQLConnection` + `sshTunnel`; `_restoreConnectionInterface`.

**Sukses** (`3563-3609`): `isConnecting=NO`; bila testing/cancelled → `cancelConnection:` (+ "Connection succeeded"); selain itu "Connected", re-enable outline, warning SSL-mismatch bila `useSSL`/IAM dan bukan `isConnectedViaSSL`, lalu `addConnectionToDocument`.

## 5. Tipe koneksi: apa yang berubah

- **Container/resize** `resizeTabViewToConnectionType:` (`1561-1587`): TCP/IP→`standardConnectionFormContainer`, IAM→`awsIAMConnectionFormContainer` (footer 29), Vault→`vaultConnectionFormContainer` (footer 49), Socket→`socketConnectionFormContainer`, SSH→`sshConnectionFormContainer`.
- **Wajib host** `_shouldRequireMySQLHost` (`1188-1196`): wajib untuk TCP/IP, SSH, IAM; SSH dengan remote-socket path dikecualikan.
- **AWS IAM** (`1185`): paksa `useSSL` off (`1212-1214`), toggle kontrol auth/profile/region (`1231-1253`), disable password field dengan placeholder (`1220-1223`).
- **Vault** (`1200`): penanganan keychain terpisah (tanpa password SQL/SSH `1765-1768,1797-1800`); field SSH dibuang saat save (`2921-2935`).
- **Socket**: host keychain dinormalisasi `"localhost"` (`1771,2242,2571-2572,2942-2943,3404`); socket-help window auto-open saat tab dipilih (`4277-4334`).
- **SSH**: `initiateConnection:` trim sshHost + `_validateSSHConfigFile`; progress "SSH…".
- Post-check SSL (`3597`): TCP/IP, Socket, IAM, Vault (bukan SSH).

## 6. Default

| Nilai | Lokasi | Arti |
| --- | --- | --- |
| `3306` | `.m:321` | port DB default token AWS IAM |
| `3306` | `.m:3974` | port probe LAN untuk TCP/IP, Vault, IAM |
| `22` | `.m:3977` | port probe LAN untuk SSH |
| `443` | `.m:682` | port Vault default |
| `oidc` | `.m:683` | mount OIDC Vault default |
| `127.0.0.1` | `.m:69,3907` | `SPLocalhostAddress` — host reconnect "Use Standard Connection" |
| `localhost` | `.m:2330` | host import default bila connection string tanpa host |
| `"default"` | `.m:325` | profile AWS fallback |
| 0.75 s | `.m:3987` | timeout probe LAN (`SALocalNetworkPermissionChecker`) |
| 0.5 s | `.m:3913` | delay sebelum reconnect via standard connection |

Timeout MySQL/SSH sesungguhnya ada di `SAConnectionService`/SPMySQL/SPSSHTunnel
(**)UNKNOWN** di file ini).

## 7. Validasi & string

Validasi pra-koneksi lewat `SAConnectionDetailsValidator.validate(...)` urut:
1. `.hostMissing` (tcpIP, sshTunnel tanpa remote socket, awsIAM) — judul "Insufficient connection details"; pesan `"Insufficient details provided to establish a connection. Please enter at least the hostname."` (`:92-95`).
2. `.sshHostMissing` — pesan `"Insufficient details provided to establish a connection. Please enter the hostname for the SSH Tunnel, or disable the SSH Tunnel."` (`:104-107`).
3. `.sshKeyFileMissing` — judul "SSH Key not found"; `"A SSH key location was specified, but no file was found in the specified location.  Please re-select the key and try again."` (`:113-120`).
4. SSL key/cert/CA — judul "SSL Key File not found"/"SSL Certificate File not found"/"SSL Certificate Authority File not found" + pesan masing-masing (`:139-175`).

Pemeriksaan PEM (`SAConnectionFileValidator`): marker `"PRIVATE KEY-----"` untuk SSL key, `"CERTIFICATE-----"` untuk cert; `"“%@” is not a valid private key file."` (`:271`), `"“%@” is not a valid client certificate file."`, `.pub` untuk sshKey → `"“%@” is a public key."` + `"Choose the private key instead — the file without the .pub extension."` (`:297-301`).

Alert jalur connect: "AWS Authorization Required" (`477-478`); "Insufficient connection details" varian Vault (`488-495,500-501`); "Invalid database" (`619`); "Unable to connect"/"Connected but unable to select database '%@'." (`621-622`); "Access denied!"/"Please check your username and password and try again." (`645-646`); "Socket not found!"/"Socket connection failed!" (`649/651`); SSL mismatch (`3599`); "Vault Authentication Failed"/"Vault returned empty credentials." (`727-728`).

Alert import/export: "Invalid Connection String"/"The connection string is not valid." (`2297-2298`); "Import from Clipboard or File?" (`2139`) + tombol "Import from Clipboard"/"Choose File..."/"Cancel" (`2186-2188`); "Duplicate Connection Found" (`2424`); "Duplicate Connections Found" (`4535`); "Copy Connection String" + peringatan plaintext (`2670-2674`).

## 8. Import/export favorites (dua format)

**A. Favorites (plist, bukan .spf)**
- Export `SPFavoritesExporter` menulis `{ "SPConnectionFavorites": [ nodeDict, … ] }`:
  root key `SPFavoritesDataRootKey = "SPConnectionFavorites"` (`SPConstants.m:398`;
  `SPFavoritesExporter.m:79`); tiap elemen `SPTreeNode.dictionaryRepresentation`
  (`:72-77`); node yang merupakan turunan node terpilih lain dilewati (`:74-76`).
  Nama default `"SequelAceFavorites.plist"` (`SPConnectionController.m:68`).
- Import `SPFavoritesImporter`: baca plist; key `"SPConnectionFavorites"` → NSArray;
  **fallback legacy key `"favorites"`** (`SPFavoritesImporter.m:59,77,83-85`);
  merge dengan ID baru (`SPConnectionController.m:4629-4646`); duplikat diselesaikan
  via `SPDuplicateImportViewController` (Update/Skip/Create New).

**B. Dokumen `.spf`/`.spfs`**
- `.spf` top-level (lihat dok 04): `version`=@1, `format`=`"connection"`,
  `rdbms_type`="mysql", `rdbms_version`, `auto_connect`, `encrypted`, `"data"`
  (blob `NSKeyedArchiver` polos atau terenkripsi `e_string`).
- `.spfs` (connection bundle): ekstensi `"spfs"` (`SPConstants.m:76`); plist bundle
  `format`=`"connection bundle"`, `version`=@1 (`SPAppController.m:429-430`).

## 9. Model form koneksi & `mysql://`

Field & default (`SAConnectionFormModel.blankFormInfo` = decoder
`SAConnectionInfo+Favorite.swift fromFavoriteDictionary(nil)`):

| Field | Default form kosong |
| --- | --- |
| type | `.tcpIP` |
| string kosong | `""` (placeholder port UI `"3306"`) |
| colorIndex | **-1** (`:121`) |
| useCompression | **true** (`:124`) |
| timeZoneMode | `.useServerTZ` |
| awsProfile | **`"default"`** (`:143`) |
| useAWSIAMAuth | diturunkan dari tipe (1 iff awsIAM) |
| vaultPort/vaultOIDCMount | `""` (sengaja tidak didekode) |

- `SAConnectionFormModel` membungkus `info` (`@Published`); `load(favorite:)` pakai
  decoder D1 dan **tidak membawa password**; `canAttemptConnection`: socket selalu
  true; sshTunnel butuh host atau remote socket; tcpIP/awsIAM butuh host; vault
  butuh host + vaultHost + credentials path.
- Nama tergenerasi (`SAConnectionFormHelpers.generateName`): socket→`"localhost"`;
  non-socket host kosong→nil; selain itu `host[/database]`; **vault** →
  `"<vaultHost|"vault">/<role lastPathComponent>"`.

**Parse `mysql://` (`ConnectionStringParser.parse`)**:
- Skema harus `"mysql"` (case-insensitive).
- host: `url.host` percent-decoded; absen/kosong → `"127.0.0.1"`.
- user/password decoded; **kehadiran password men-set `autoConnect = true`**.
- port: `url.port` → string. database: `url.pathComponents[1]` decoded bila non-kosong.
- Whitelist query `validQueryParameters`: `type, socket, ssh_host, ssh_port, ssh_user,
  ssh_password, ssh_keyLocationEnabled, ssh_keyLocation, ssh_remote_socket_path,
  aws_region, aws_profile, autoConnect, enable_cleartext_plugin,
  get_server_public_key, request_server_public_key`. Key lain → `invalidParameters`.
- `autoConnect` truthy = `"true"`/`"1"`. `type` → `socket`→SPSocketConnection,
  `ssh`→SPSSHTunnelConnection, `aws_iam`/`awsiam`→SPAWSIAMConnection,
  `tcpip`/`tcp`→SPTCPIPConnection.
- Inferensi tipe bila `type` absen: awsRegion/awsProfile → awsIAM; socket → socket;
  ssh_host/ssh_remote_socket_path → sshTunnel; selain itu tcpIP.

**Generate `toConnectionString`: format `mysql://[user[:password]@]host[:port][/database][?params]`**.
- socket → `type=socket`(+`socket`); ssh → `type=ssh`(+ssh_*); awsIAM → `type=aws_iam`(+aws_*);
  **tcpIP dan vault tidak mengemit `type`** (Vault round-trip sebagai tcpip — kuirk sama).
- Flag `enableClearTextPlugin`/`requestServerPublicKey` mengemit `=1`.

## 10. Pencarian favorites

- `SAFavoriteSearchMatcher`: query lowercase dipisah whitespace → `tokens` (`:33-36`);
  `isActive` = tokens non-kosong; `matches(name:host:)` = `tokens.allSatisfy {
  lowerName.contains($0) || lowerHost.contains($0) }` (`:48-54`) — **setiap token
  harus substring case-insensitive dari name ATAU host**.
- `SAFavoriteSearchTreeWalker.visibleNodes`: leaf disertakan iff match; group
  disertakan iff ada minimal satu descendant match (**nama group sendiri tidak
  dicocokkan**).
- `SAFavoriteItem.filtered(using:)` SwiftUI mirror: quickConnect selalu disimpan;
  group disimpan iff ≥1 child disimpan.

## 11. UNKNOWN

- Struktur sub-dictionary `data` `.spf` belum dibaca penuh (hanya envelope
  `SPDatabaseDocument.m:2746-2815`) — **UNKNOWN**.
- Nilai string key `SPFavoritesSortedBy`/`SPFavoritesSortedInReverse`/`SPFavoriteColorList`
  (nilai simbol `SPConstants.m`) tidak dicetak satu-satu.
- Nomor baris Swift untuk `SAConnectionFormModel.swift`, `SAConnectionFormView.swift`,
  `SAFavoriteItem*.swift`, `SAFavoriteDuplicateMatcher.swift`, `ConnectionStringParser.swift`
  tidak dienumerasi baris-per-baris (dibaca utuh) — sitasi via file+simbol.
