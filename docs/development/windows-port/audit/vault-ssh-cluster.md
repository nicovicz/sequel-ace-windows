# Audit: Vault + SSH-tunnel cluster

Audit method-by-method klaster Vault + SSH tunnel. `Start`/`End` = nomor baris di file
masing-masing. Baseline commit `02c910932`. (`VaultOIDCHandler.swift` sudah diaudit terpisah.)

File: `Vault/VaultAuthManager.swift` (482), `Vault/VaultClient.swift` (320),
`Vault/SAVaultRoleListController.swift` (247), `SSHTunnel/SASSHTunnelAuthMessage.swift` (207),
`SSHTunnel/SASSHTunnelPeerValidator.swift` (257), `SSHTunnel/SASSHTunnelSocketServer.swift` (301).

---

## 1. `VaultAuthManager.swift` (482)

Koordinasi validasi token Vault, fallback login OIDC, dan generate kredensial DB ephemeral; diekspos ke ObjC via `@objcMembers`.

### Tipe & konstanta
`@objc enum VaultAuthError: Int, Error, LocalizedError` (13–34: invalidConfiguration/loginCancelled/loginFailed/credentialsFailed/emptyCredentials); `@objcMembers final class VaultAuthManager: NSObject` (36–482); `private struct CacheEntry` (44–49).
`log` (38), `errorDomain = "VaultAuthErrorDomain"` (39), `cacheExpirationMargin = 30s` (40), mount default literal `"oidc"` (114, 368).

### `// MARK:`
`42 (Credential cache (internal for testability)), 119 (Token helpers), 129 (Main entry point (ObjC-compatible))`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 20–33 | | `VaultAuthError.errorDescription` | Teks terlokalisasi per case |
| 38–52 | | `log`/`errorDomain`/`cacheExpirationMargin`/`CacheEntry.*`/`credentialCache`/`cacheLock` | Konstanta + cache |
| 62–73 | | `inFlightKeys`/`inFlightCondition`/`cacheKey(baseURL:oidcMount:credPath:)` | Coalescing + key |
| 77–117 | | `cachedCredentials(for:matchingToken:)` / `setCachedCredentials(...)` / `clearCachedCredentials(for:)` / `clearCachedCredentials(host:port:oidcMount:credPath:)` | Cache |
| 123–127 | | `isAuthorized(baseURL:mount:)` | Cek token cache + `tokenLookupSelf` |
| 134–363 | | `generateCredentials(...error:)` / `generateCredentials(...loginIdentifier:...error:)` + helper nested `failCancelled`/`finishInFlight` | Alur penuh cache/coalesce/token/OIDC/credential |
| 366–378 | | `effectiveOIDCMount(_:)` / `isLoginCancellation(_ error:)` | Helper |
| 388–463 | | `listRoles(...confirmBrowserLogin:...error:)` | Resolve token lalu LIST `<mount>/roles` |
| 468–481 | | `isAuthorized(host:port:)` / `isAuthorized(host:port:oidcMount:)` | Cek authorized berbasis string |

### Temuan
- **Cache kredensial = static global proses**, bukan per-koneksi: `credentialCache` (51), `inFlightKeys` (62) — dibagi semua window; hanya key yang memisahkan identitas.
- **Key sengaja tanpa role/username** — hanya `host:port@oidcMount/credPath` (69–73); rasional (65–68): dua favorite, host DB + cred path sama, OIDC mount beda.
- **Margin expiry 30 s** (81, 40). **Eviksi token-identity**: mismatch `matchingToken` hanya membuang entry bila token tersimpan non-kosong (85–88).
- **Coalescing in-flight / lock ordering** terdokumentasi `inFlightCondition` SEBELUM `VaultOIDCHandler.loginLock` (54–61), diimplementasi 207–232 dengan poll `0.1s` (214) + cek cancellation di loop (209–213).
- ⚠️ **Coalescing tanpa batas waktu**: loop `while inFlightKeys.contains(key)` (208) tanpa deadline global → bila leader tak pernah `finishInFlight()`, waiter poll selamanya. `finishInFlight()` dipanggil di setiap jalur keluar leader (244, 260, 273, 281, 284, 299, 307, 320, 328, 334, 346, 355, 358).
- **Error jaringan sengaja dibedakan dari token invalid** (248–251, 289–300): `tokenLookupSelf` throw → abort `loginFailed` tanpa buka browser; hanya 400/403 return false (`VaultClient.swift:160-162`).
- **Guard main-thread hanya `assert`, bukan runtime check** (124, 160, 397, 469, 478) — hilang di release.
- `isAuthorized(baseURL:mount:)` melakukan I/O jaringan nyata via `tokenLookupSelf` (126), bukan pure cache.
- **UNKNOWN**: kenapa overload ObjC `generateCredentials` (134–145) kirim `loginIdentifier: ""` (cancellation tak pernah berlaku di sana).
- **Asimetri**: `listRoles` tanpa coalescing in-flight → dua refresh konkuren bisa masing-masing mulai login OIDC (dimigasi di layer UI, `SAVaultRoleListController.swift:56,160,189`).

---

## 2. `VaultClient.swift` (320)

Wrapper REST sinkron untuk HashiCorp Vault; semua panggilan memblok pemanggil via `DispatchSemaphore` agar thread koneksi background dapat memakainya.

### Tipe & konstanta
`enum VaultClientError` (13–35: invalidBaseURL/networkError/httpError/parseError); `struct VaultCredentials` (37–41); `final class VaultClient` (43–320, semua static).
`log` (45); port default `"443"` (53); skema `https://` (54). Endpoint: `v1/auth/token/lookup-self` (145), `v1/auth/<mount>/oidc/auth_url` (177), `v1/auth/<mount>/oidc/callback` (125), `v1/<credPath>` (227), `v1/<mount>/roles` (274). Mount default `"oidc"` (111); lease default `3600` (66). Timeout: 10 s lookup-self (146), 10 s auth_url (178), 15 s callback (204), 15 s generateCredentials (228), 15 s list roles (280). Header `X-Vault-Token` (148, 230, 282).

### `// MARK:`
`47 (URL helpers (static, testable)), 57 (Response parsers (static, testable)), 107 (Helpers), 136 (Network calls), 304 (Synchronous helper)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 19–41 | | `VaultClientError.errorDescription` / `VaultCredentials.*` | Error + model |
| 49–55 | | `buildBaseURL(host:port:)` | Compose `https://host:port`, default 443 |
| 59–105 | | `parseCredentials` / `parseOIDCAuthURL` / `parseVaultErrors` / `parseToken` / `parseRoleList` | Parser respons |
| 109–134 | | `effectiveMount(_:)` / `oidcAuthURLRequestBody(...)` / `oidcCallbackURL(...)` | Helper |
| 144–247 | | `tokenLookupSelf(baseURL:token:)` / `oidcAuthURL(...)` / `oidcCallback(...)` / `generateCredentials(baseURL:credPath:token:)` | Network |
| 262–302 | | `rolesURL(baseURL:mount:)` / `listDatabaseRoles(baseURL:mount:token:)` | Roles |
| 306 | 319 | `synchronousDataTask(with:)` | URLSession + semaphore blocking bridge |

### Temuan
- **HTTP client**: `URLSession.shared` + `DispatchSemaphore(value: 0)` ad-hoc (310–317) — tanpa timeout pada `semaphore.wait()`; mengandalkan `timeoutInterval` per-request.
- **Kontrak tri-state `tokenLookupSelf`** load-bearing: `200 → true` (159), `400/403 → false` (162), lain throw `httpError` (163–168); kolaps akan memicu login browser palsu.
- **`parseCredentials` default lease 3600 s bila absen** (66). `parseRoleList` sort `localizedCaseInsensitiveCompare` (104) — urutan bergantung locale.
- **Hardening traversal `rolesURL`** (249–275): tolak segmen kosong/`.`/`..` + scalar non-printable-ASCII (270–272), setelah `SAVaultCredentialsPath.normalizeMount` (263); komentar klaim `%2e%2e` tetap literal karena re-encoding (258–261).
- `oidcAuthURLRequestBody` kirim `client_nonce`, bukan `nonce` (118), query callback juga `client_nonce` (130) — mirror vault-plugin-auth-jwt CLIHandler (173–174).
- `generateCredentials` strip leading `/` dari credPath (226) sebelum `appendingPathComponent`.
- `oidcCallback` **tanpa** `X-Vault-Token` (204–205) — callback memang unauthenticated.
- `buildBaseURL` **tanpa validasi port** (49–55) — port invalid → `URL(string:)` nil → degrade "invalid configuration".
- `cachePolicy: .reloadIgnoringLocalCacheData` di setiap request (146, 178, 204, 228, 280).

---

## 3. `SAVaultRoleListController.swift` (247)

Memiliki perilaku combo box "Role" Vault — LIST background via OIDC, urutan fuzzy, debounce, invalidasi, dan buang respons stale.

### Tipe & konstanta
`@objc protocol SAVaultRoleListControllerDelegate: AnyObject` (16–31, 9 method); `@objc final class SAVaultRoleListController: NSObject` (33–247). Referensi eksternal: `SAVaultRoleFilter.orderedRoles/separator` (77, 114), `VaultAuthManager.isLoginCancellation` (237), `VaultOIDCHandler.prepareActiveLogin/cancelActiveLogin/clearPreparedActiveLogin` (180, 149, 199). Debounce `0.15s` (88).

### `// MARK:`
`70 (Dropdown population), 124 (OIDC login lifecycle), 153 (Refresh)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 17–30 | | 9 requirement protokol | Host/port/mount/role + veto defer + teardown + commit + enable controls |
| 35–60 | | `delegate` / outlet / `availableRoles` / `reloadToken` / `activeLoginIdentifier` / `isRefreshing` / `isRefreshInFlight()` | State |
| 62–68 | | `init(comboBox:refreshButton:progressIndicator:)` | Designated ObjC init |
| 73–122 | | `reloadItems()` / `scheduleDebouncedReload()` / `invalidateRoles()` / `handleComboSelectionChange()` | Dropdown |
| 131–151 | | `confirmBrowserLoginOnMainThread()` / `runBrowserLoginConfirm()` / `cancelActiveLogin()` | Lifecycle login |
| 155 | 246 | `refresh()` | Validasi konteks, guard, spawn LIST background, terapkan hasil |

### Temuan
- **Port callback OIDC fixed = crux konkurensi**: refresh bisa memegang port, jadi Connect/Test harus disabled (52–56, 183–189) + connect/test in-flight men-defer refresh (158–160).
- **Lifecycle refresh**: gate `vaultRoleListShouldDeferForActiveConnection()` (160) → alert mount kosong (163–167) → `prepareActiveLogin()` (180) → set `isRefreshing`/disable controls (183–189) → background global queue .userInitiated (191).
- `isRefreshing` dibersihkan **sebelum** setiap early return setelah LIST (202–207) — sengaja agar start koneksi tak diblokir.
- **Buang respons stale** bandingkan empat field konteks (host/port/oidcMount/mount) dengan yang di-capture (223–227).
- **Guard torn-down** berjalan setelah clear `isRefreshing` tapi sebelum sentuh `progressIndicator` (209–213).
- **Cancellation senyap**: `isLoginCancellation` menekan alert kegagalan (237–243).
- **`invalidateRoles()` cancel login in-flight** (104), rasional 94–102 (login terikat host/port/mount stale; bebaskan port callback + re-enable controls).
- ⚠️ `confirmBrowserLoginOnMainThread` pakai `DispatchQueue.main.sync` off-main (131–134) — risiko deadlock main-thread bila dipanggil dari konteks yang memblokir main queue. **UNKNOWN**: di sini hanya dijangkau dari thread LIST background.
- **Daftar role kosong → alert warning** (233–236); role tetap disimpan dulu (231).
- `reloadItems()` baca field editor live (76) agar tak pakai `stringValue` stale saat mengetik.
- Debounce pakai perbandingan `reloadToken` di closure (86–91) — closure telat jadi no-op, bukan cancelled.
- **UNKNOWN**: semantik `SAVaultRoleFilter.orderedRoles`/`separator` (fuzzy ordering + baris separator) di luar scope.

---

## 4. `SASSHTunnelAuthMessage.swift` (207)

Kontrak wire JSON line-delimited antara SSH tunnel dan asisten askpass; Foundation murni, dikompilasi ke app + assistant + test.

### Tipe & konstanta
`enum SASSHTunnelAuthRequest: Equatable` (44–71: question/password(verificationHash:)/query); `enum SASSHTunnelAuthResponse: Equatable` (73–82: answer/secret/refused); `enum SASSHTunnelAuthWireError: Error, Equatable` (84–89: malformed/unsupportedVersion/unknownKind/missingField); `enum SASSHTunnelAuthWire` (97–207, namespace static).
`version = 1` (99); kind request `"question"`/`"password"`/`"query"` (106, 108, 110); kind response `"answer"`/`"secret"`/`"refused"` (117, 119, 121); key `kind`/`text`/`hash`/`answer`/`secret`/`v`; terminator byte `0x0A` (130).

### `// MARK:`
`101 (Encoding), 134 (Decoding)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 65–70 | | `SASSHTunnelAuthRequest.mayPromptTheUser` | True untuk question/query; false untuk password |
| 103–132 | | `encode(_ request:)` / `encode(_ response:)` / `encode(_ fields:)` | Map case → JSON line |
| 136–162 | | `decodeRequest(_:)` / `decodeResponse(_:)` | Parse line → request/response |
| 164–175 | | `decodeObject(_:)` | Trim CR/LF, parse object, enforce version |
| 177–206 | | `kind(of:)` / `string(_:in:)` / `bool(_:in:)` / `int(_:in:)` | Reader field |

### Temuan
- **Encoding byte-stable**: `.sortedKeys, .withoutEscapingSlashes` + `0x0A` (129–131) agar fixture test bisa pin byte (92–93).
- ⚠️ **Gagal encode ditelan**: `try? … ?? Data()` (129) — komentar mengklaim hanya string/bool/Int masuk sehingga "tidak bisa gagal"; kegagalan akan senyap menghasilkan newline kosong.
- **Versi wajib saat baca**: `v` hilang/non-Int → `missingField`; mismatch → `unsupportedVersion` (172–173); reader tak pernah menebak (95–96).
- **Disiplin tipe vs bridging NSNumber**: `bool` butuh `CFBooleanGetTypeID()` (192–196), `int` tolak bool + float via `CFNumberIsFloatType` (199–205) — cegah koersi `1`/`true` (186–190).
- **Strip newline** loop `0x0A` + `0x0D` (166) — tangani CRLF.
- **`password` vs `query`** relevan keamanan: hanya `password` bebas-prompt/idempoten sehingga hanya itu boleh di-retry lewat transport lain (issue #2689) (52–64, 65–70).
- `refused` ada agar asisten exit non-zero alih-alih mencetak baris kosong untuk ssh (78–81).
- **UNKNOWN**: apakah field `text` `query`/`question` membawa output ssh tak-terpercaya; tak ada sanitasi/length-limit.

---

## 5. `SASSHTunnelPeerValidator.swift` (257)

Menggantikan shared secret environment sebagai cek utama dengan mengidentifikasi peer socket dari audit token + validasi code signature.

### Tipe & konstanta
`enum SASSHTunnelPeerValidator` (57–257, static); `struct Identity: Equatable` (76–79); `enum Failure: Error, Equatable` (81–88).
`assistantIdentifier = "SequelAceTunnelAssistant"` (72, default yang dipin test, **bukan** kewajiban peer); `baseRequirement = "anchor apple generic"` (74). Opsi socket `SOL_LOCAL` + `LOCAL_PEERTOKEN` (204); guest attr `kSecGuestAttributeAudit` (227).

### `// MARK:`
`90 (Policies), 170 (Checks), 210 (This process), 218 (Security framework plumbing)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 110–133 | | `assistantPeerPolicy(assistantPath:log:)` / `signingIdentifier(ofBinaryAt:)` / `appPeerPolicy()` | Kebijakan peer |
| 138–168 | | `policy(ownTeamIdentifier:expectedIdentifier:baseRequirement:log:)` / `requirement(identifier:base:)` | Bangun closure + requirement |
| 174–208 | | `validatePeer(on:requirementText:teamIdentifier:)` / `validatePeerSignature(on:requirementText:)` / `auditToken(ofPeerOn:)` | Cek peer |
| 212–256 | | `ownIdentity()` / `peerCode(on:)` / `check(_:against:)` / `identity(of:)` / `identity(ofStaticCode:)` | Plumbing Security |

### Temuan
- **Model validasi peer**: berbasis socket, bebas race pid via `getsockopt(LOCAL_PEERTOKEN)` (38–41, 201–208), lalu `SecCodeCopyGuestWithAttributes(kSecGuestAttributeAudit:)` (227).
- **Cek dua bagian**: requirement terpenuhi (179) lalu team equality dari signing info (184–186) — team sengaja **bukan** `subject.OU` agar jalan lintas Developer ID/App Store/dev cert (44–47).
- **Ekspektasi diturunkan dari signature, bukan literal** (49–52): identifier app-side dari binary asisten on-disk (112, 123–128); team sendiri dari `ownIdentity()` (116, 212–216).
- ⚠️ **Fail-open tanpa team identifier**: build ad-hoc/unsigned log sekali (guard `warned` + `NSLock`, 142–154) + **terima semua peer**, sisakan verification hash sebagai satu-satunya guard (51–54, 136–137).
- ⚠️ **Fail-open saat identifier asisten tak terbaca**: buang setengah identifier, tetap `anchor apple generic` + team sendiri (101–106, 113–115).
- **Regresi terdokumentasi**: mewajibkan `assistantIdentifier` bare mematahkan semua tunnel karena asisten di-re-sign dengan sufiks hash (`SequelAceTunnelAssistant-<40 hex>`), issue #2689 (63–71).
- **UNKNOWN/risiko**: jalur fail-open (142–154, 113–115) → kesalahan build/packaging senyap menurunkan ke team-only (atau tanpa) validasi peer; sinyal hanya satu baris log. Tak ada mode fail-closed.
- **UNKNOWN**: default `log` `NSLog` (111, 141) — caller in-app diharapkan inject sink ke tunnel debug window (108–109).

---

## 6. `SASSHTunnelSocketServer.swift` (301)

Transport socket app (IPC plan Step 3): UNIX domain socket per-tunnel di container sandbox untuk asisten askpass, satu request per koneksi; transport saja, keputusan di `SASSHTunnelAuthService`.

### Tipe & konstanta
`@objc final class SASSHTunnelSocketServer: NSObject` (43–301); nested `enum Error` (52–57: noUsableDirectory/socketFailed/bindFailed/listenFailed); nested `final class LivePaths` (112–130); `typealias Handler` (45) / `PeerPolicy` (50).
`requestTimeout = 10` s untuk kirim baris request; reply tanpa timeout (59–61); backlog `32` (214); mode socket `0o600` (209–211); queue label accept (75) / serve (76); nama socket `s-XXXXXXXX.sock` (91–93); regex nama sendiri `^(s-[0-9a-f]{8}|ssh-[0-9a-f]{10})\.sock$` (100); env var `SP_CONNECTION_SOCKET_PATH` (63).

### `// MARK:`
`251 (Accepting), 268 (Serving one connection)`.

### Definisi
| Start | End | Name | Tujuan |
| --- | --- | --- | --- |
| 84–110 | | `candidateDirectories()` / `socketFileName()` / `isOwnSocketName(_:)` / `liveSocketPaths` | Path + nama |
| 113–129 | | `LivePaths.lock/paths/insert/remove/contains` | Registry path live |
| 136 | 154 | `sweepStaleSockets(in:)` | Unlink socket nama-sendiri yang menolak koneksi |
| 165–234 | | `init(service:assistantPath:diagnosticSink:)` (convenience ObjC) / `init(directories:handler:peerPolicy:log:)` | Pilih path, sweep, bind, chmod, listen, read source |
| 236–249 | | `deinit` / `close()` | Tutup socket + hapus file |
| 253–266 | | `acceptPending()` | Drain accept loop di accept queue |
| 270 | 300 | `serve(_:handler:peerPolicy:log:)` | Cek policy, baca line, decode, handle, reply |

### Temuan
- **Protokol socket**: satu request per koneksi; koneksi refused/undecodable ditutup tanpa reply → asisten baca EOF + fail closed (38–42, 276–279, 291–294).
- **Validasi peer sebelum baca apa pun** (276–279); rejection di-log dengan sink user-facing (69–73, 277) — konteks issue #2689.
- **Asimetri timeout**: `SO_RCVTIMEO` dari `requestTimeout` pada klien diterima (281–282); reply tanpa timeout karena menunggu user (59–61).
- **Nama socket sengaja pendek** agar muat `sun_path` (103 byte) (88–93); regex match lebar eksak agar file tak terkait tak pernah di-unlink (95–101).
- **Sweep socket stale** (132–154): enumerasi dir → skip path live sendiri (140) → probe `connect()` → unlink hanya pada `ECONNREFUSED` (150–152). Komentar: skip live-path cegah app konek ke server sehatnya sendiri + memicu rejection peer-policy (103–109).
- **`LivePaths` dijaga `NSLock` sendiri** (113–129), didaftarkan saat init (222) + dihapus di cancel handler (230).
- **Pemilihan direktori**: direktori pertama di mana `SASSHTunnelSocketIO.address(for:)` sukses (184–193), else `Error.noUsableDirectory`; hanya `NSTemporaryDirectory()` ditawarkan (81–86).
- **Hardening**: `chmod(path, 0o600)` setelah bind (209–211). Bind gagal → tutup fd + throw (204–208); listen gagal → tutup fd + unlink + throw (214–219). **UNKNOWN**: tanpa retry pada collision nama random (32-bit) — muncul sebagai `bindFailed`.
- **fd listening non-blocking, klien blocking**: `setBlocking(fd, false)` (220) untuk dispatch source; klien diterima di-set blocking sebelum dispatch (260–261).
- **Loop accept keluar pada error apa pun non-EINTR**, termasuk EWOULDBLOCK (254–259), mengandalkan read source re-fire.
- **`close()` idempoten di bawah `stateLock`** (242–249); fd close + `unlink` aktual terjadi di cancel handler (227–231) — deferred, jadi path dihapus asinkron. **UNKNOWN**: apakah caller mengasumsikan file hilang sinkron setelah `close()`.
- **UNKNOWN**: `SASSHTunnelSocketIO` (makeSocket/address/config/setBlocking/readLine/writeAll) di luar file; framing/limit `readLine` tak diketahui. `SASSHTunnelAuthService.handle` (169) juga eksternal.

## Ringkasan
- Model Vault bertumpu pada **state static proses-global** (cache + in-flight) dan **port callback fixed** — dua sumber serialisasi implisit.
- SSH tunnel: validasi peer fail-open saat build unsigned + wire protocol version-1 byte-stable; socket satu-request-per-koneksi dengan penolakan fail-closed.
