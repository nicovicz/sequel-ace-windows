# Integrasi: SSH tunnel, AWS, Vault, secure bookmark, updater, analytics

Kontrak source untuk subsistem integrasi OS/jaringan. Sitasi `file:line` merujuk
checkout `02c910932`.

## 1. SSH tunnel (`Source/Other/SSHTunnel/`)

### 1.1 Pembentukan — invokasi openssh & argumen

- Objek tunnel dibuat via `initToHost:port:login:tunnellingToPort:onHost:` (`SPSSHTunnel.m:97`); menyimpan `sshHost/sshLogin/sshPort/remoteHost/remotePort`, `useHostFallback` bila host target == host tunnel (`:108`).
- Biner default `/usr/bin/ssh` (`:461`); dapat dioverride pref `SSHClientPath` (`:462-465`, `SPConstants.m:287`). Biner kustom menulis banner debug (`:468-470`).
- Argumen (`:477-624`): `-v` (`:486`), `-N` (`:489`), muxing `-o ControlMaster=auto` + `ControlPath=…/SPSSH-<8hex>` bila aktif (`:493-503`), else `-S none` + `-o ControlMaster=no` (`:506-507`); `-o ExitOnForwardFailure=yes` (`:511`); `-o ConnectTimeout=<pref>` (`:514-516`); `-o NumberOfPasswordPrompts=3` (`:519`). Muxing digerbangi `SPSSHEnableMuxingPreference = @"SSHMultiplexingEnabled"` (`:123`; `SPConstants.m:286`).
- Known-hosts: kecuali pref `SSHConfigContainsUserKnownHostsFile` (`SPConstants.m:97`), pakai path writable kustom — default `~/.keys/ssh_known_hosts_strict` (`SPConstants.m:96`) — lewat `-o UserKnownHostsFile="…"` (`:570`); validasi writability/quote-in-path, abort + alert bila gagal (`:529-568`). Config file pref `SPSSHConfigFile` (`@"ssh_config"`, `SPConstants.m:94`) atau resource bundle, lewat `-F` (`:579-586`). Identity `-i <path>` (`:593-595`); keepalive (`:598-603`); port `-p` (`:606-608`); tujuan `login@host`/`host` (`:609-614`).
- **Forwarding/socket remote** (`:615-624`): bila `remoteSocketPath` diset → `-L localPort:<remoteSocketPath>` (`:246`); else bila host fallback → dua aturan `-L`; else `-L localPort:remoteHost:remotePort`. Port lokal acak via `getRandomPort()` (`:1225`).

### 1.2 Penanganan password/key

- Tiga mode `SPSSHTunnelPasswordMode` (`SPConstants.h:132-137`): `UsesKeychain=0`, `AsksUI=1`, `None=2`; dipilih lewat env `SP_PASSWORD_METHOD` (`:643-653`).
- Nama+account keychain **sengaja TIDAK** ditaruh di env (komentar `:644-647`); assistant bertanya ke app lewat koneksi, app resolve item "saat ask time". `setPassword:` menyimpan di memori (`:227`, menolak bila mode keychain `:229`); `setPasswordKeychainName:account:` (`:255`); `setKeyFilePath:` expand `~` + wajib ada (`:238-243`).
- Env ke ssh (`:630-654`): `SSH_ASKPASS` = exe `SequelAceTunnelAssistant` (`:629,635`), `DISPLAY=":0"` (`:636`, wajib agar OpenSSH pakai askpass — komentar `:697-700`), `SP_CONNECTION_NAME` (`:637`), `SP_CONNECTION_VERIFY_HASH` (`:638`), transport key + socket path (`:639-642`), `SP_PASSWORD_METHOD` (`:648/650/652`). `SHELL` dihapus agar ProxyJump tetap di sandbox (`:633`).

### 1.3 IPC app ↔ assistant

- **Verification hash**: `tunnelConnectionName = "NKQ4HJ66PX.sequel-ace.SequelAce-<hash>"`, `tunnelConnectionVerifyHash = <hash>` (`:126-127`).
- **Auth service** — `SASSHTunnelAuthService` satu-satunya objek yang divend, 3 panggilan (`SASSHTunnelAuthService.swift:47`; `SPSSHTunnel.m:132-133, 951-964`). `password(verificationHash:)` wajib `verificationHash == source.verificationHash` (`:77`).
- **Wire protocol** — JSON line-delimited, key diurut, berversi `v=1` (`SASSHTunnelAuthMessage.swift:97-131`). Request: `question(text)`, `password(hash)`, `query(text,hash)` (`:44-50, 103-111`); response: `answer(bool)`, `secret(string)`, `refused` (`:73-82, 114-122`). Decode type-strict (`:192-206`). `mayPromptTheUser` false hanya untuk `password` (`:65-70`).
- **Transport** — `SASSHTunnelTransport` `.distributedObjects`/`.socket` (`:47-58`); pref tersembunyi `SPSSHTunnelUseSocketTransport` (`:63`); **default `.distributedObjects`** setelah issue #2689 (`:71`); tetap untuk seumur tunnel (`SPSSHTunnel.m:144`); socket gagal → fallback ke DO (`:145-161`).
- **Distributed Objects**: `NSConnection` didaftarkan dengan nama tunnel + `authService` root (`:128-139`). Tunnel sendiri tidak diekspos (komentar `:52-60`).
- **UNIX socket** (`SASSHTunnelSocketServer.swift`): per-tunnel di tmp container sandbox, `AF_UNIX`/`SOCK_STREAM` (`:76`), nama `s-<8hex>.sock` (`:91-93`), chmod `0600` (`:211`), backlog 32 (`:214`), timeout request 10 s tapi reply tanpa timeout (`:59-61`). Client `SASSHTunnelSocketClient.swift` (`:82-107`). Env key `SP_CONNECTION_TRANSPORT`/`SP_CONNECTION_SOCKET_PATH` (`SASSHTunnelSocketIO.swift:49-58`).
- **Peer auth**: socket peer divalidasi via audit token `getsockopt(LOCAL_PEERTOKEN)` + `SecCodeCopyGuestWithAttributes` (`SASSHTunnelPeerValidator.swift:201-231`); wajib `anchor apple generic` **dan** team ID proses ini (`:138-163`); app juga butuh signing identifier assistant, dibaca dari biner karena nama terkirim berhash (`:60-72, 110-128`). Build tanpa team ID mencatat sekali + menerima (`:142-153`). `SP_CONNECTION_VERIFY_HASH` jadi shared secret cadangan.
- **Askpass** (`SequelAceTunnelAssistant.m` + `SASSHTunnelAskpass.swift`): klasifikasi argumen ssh: `" (yes/no"` → pertanyaan yes/no (`:73-92`); `"password:"` → `getPasswordWithVerificationHash:` (`:96-136`); else passphrase key/SecurID (`:139-166`). Transport error fail-closed, **hanya `refused` eksplisit** sampai ke fallback GUI (`SASSHTunnelAskpass.swift:108-123`); "never prints an empty line for ssh to read as an empty password" (`:43`).

### 1.4 Sheet pertanyaan/password

- Parent window wajib; nib `SSHQuestionDialog` dimuat lazy (`SPSSHTunnel.m:217-220`).
- Sheet yes/no memblok thread koneksi pada `answerAvailableLock` (`:998-1048`); teardown menjawab "no" (`:1039-1046`). Sheet passphrase (`:1069-1139`) parse `Enter passphrase for key '<key>':` via regex (`:1107`) untuk checkbox simpan keychain; disimpan dengan nama keychain `"SSH"` (`:1164-1166`).
- `SASSHTunnelPromptCoordinator` memutuskan apakah prompt boleh tampil + menutup prompt in-flight saat teardown (`:49-133`); prompt ditolak setelah ssh keluar (`:79-83`, `SPSSHTunnel.m:1025, 1100`).

### 1.5 Penanganan kegagalan

- `NSPipe` pada stderr (`:664-670`); `standardErrorHandler:` parse baris (`:848`). Transisi: `"Local forwarding listening on"`/`"mux_client_request_session: master session id: "` → `SPMySQLProxyConnected` (`:869-872`); `"Connection established"` → `SPMySQLProxyWaitingForAuth` (`:876-879`).
- String error terminal → `lastError` (`:883-911`): `"bind: Address already in use"` (`:883`, termasuk pertanyaan fallback localhost), `"closed by remote host."` (`:890`), `"Permission denied ("`/`"No more authentication methods to try"` (`:896`), `"connect failed: Connection refused"` → `SPMySQLProxyForwardingFailed` (`:902`), `"Operation timed out"` (`:906`).
- `SASSHStderrDrainCoordinator` (`SASSHTunnelFailure.swift:28`) mengoordinasi batas drain: coalescing request (`.start/.queued/.ignored` `:74-91`), batas 5 s (`:36`), urutan snapshot akhir agar reconnect tidak menghapus diagnostik (`:139-172`).

## 2. AWS IAM (`Source/Other/AWS/`)

### 2.1 Alur token

Entry `AWSIAMAuthManager.generateAuthToken(...)` (`:147`). Region: eksplisit → `RDSIAMAuthentication.regionFromHostname` → `"us-east-1"` (`:158-169`). Profile: trim, `"default"` bila kosong (`:173-174`). accessKey/secretKey manual **diabaikan** — profile-only (`:171-172`).

Presedensi kredensial `loadCredentialsFromProfile` (`:201-268`): SSO/Identity Center (`:218-220`) → console sign-in (`aws login`, `:224-226`) → role assumption bila `requiresRoleAssumption` (kredensial cache dipakai ulang untuk hindari MFA berulang `:239-242`; MFA `assumeRoleWithMFA` `:246-253` atau non-MFA `:256-259`) → else base langsung. Cache per profile, margin 300 s (`:119-126`); durasi clamp 1-3600 s (`:121-123`); `clearCachedCredentials` saat tutup (`:341-350`).

- **STS SigV4** (`AWSSTSClient.swift`): `AWS4-HMAC-SHA256` (`:69`); endpoint `sts.<region>.<partition-suffix>` dengan tabel partition (us-isob/us-iso/cn/us-gov `:77-103`); body `Action=AssumeRole,Version=2011-06-15,...` (`:156-168`); durasi clamp 900-43200 (`:145-147`); session name `SequelAce-<epoch>` (`:142-143`); signature HMAC-SHA256 CommonCrypto (`:186-231, 448-491`).
- **RDS IAM token** (`RDSIAMAuthentication.swift`): get-query token service `rds-db`, action `connect`, expiry 900 s (`:57-61`); `X-Amz-Security-Token` saat temporary (`:176-178`); URL-encode user + credential scope (`:162, 170`). `isRDSHostname` wajib sufiks `.rds.amazonaws.com`/`.cn` (`:318-324`).

### 2.2 Region/profile

- Katalog region dari `https://ip-ranges.amazonaws.com/ip-ranges.json` (`:70`); cache TTL 7 hari (`:75-77`); fallback list (`:78-113`); parsing `prefixes`+`ipv6_prefixes` (`:592-619`). Combo region menunda popup pertama agar startup tanpa jaringan (`:773-967`).
- Profile dari `~/.aws/credentials` + `~/.aws/config` (`AWSCredentials.swift:227-238`), `[profile xyz]` vs `[default]` (`:353-402`), resolusi `source_profile` dengan deteksi siklus (`:244-325`), pull-through `[sso-session NAME]` (`:297-333`), `sso-session` dikeluarkan dari daftar profile (`:429-431`). "default" diurut pertama (`:232-235`).

### 2.3 Bookmark `~/.aws`

- Path menghormati env `AWS_SHARED_CREDENTIALS_FILE`/`AWS_CONFIG_FILE` (`:175-189`); else `AWSDirectoryBookmarkManager.shared.awsDirectoryBasePath`.
- `AWSDirectoryBookmarkManager` pakai security-scoped bookmark (`.withSecurityScope` `:42-44`) untuk sandbox; base `~/<home>/.aws` (`:61-63`); refcount access (`:139-211`); baca via `readAWSFileContents` (`:308-364`). Stale bookmark revoke (`:96-97, 126-134`).
- Write-access untuk renewal `aws login`: `SAAWSDirectoryWriteAccessPrompt.requestWriteAccessIfNeeded` (`:42-76`); decline di-latch per sesi (`:38, 69`).

### 2.4 Pemetaan error token

Enum → string: `AWSIAMAuthError` (`:36-60`), `AWSCredentialsError` (`:34-55`), `AWSSTSClientError` (`:35-62`), `AWSSSOClientError` (`:35-62`), `AWSLoginAuthError` (`:35-53`), `SAAWSLoginRefreshError` (`:33-66`). `presentableError` (`:753-762`). `SAAWSSignInCommand.command(for:)` memetakan error → `aws login`/`aws sso login` (`:55-84`).
- **SSO portal** (`AWSSSOClient.swift`): token bearer dari `~/.aws/sso/cache/<sha1(key)>.json` (`:117-137`); token kedaluwarsa ditolak (`:140-151`); `GetRoleCredentials` di `https://portal.sso.<region>.<suffix>/federation/credentials` header `x-amz-sso_bearer_token` (`:156-191`); status 401→tokenExpired, 403→accessDenied (`:211-221`).
- **Console sign-in** (`AWSLoginCredentialsProvider.swift`): `~/.aws/login/cache/<sha256(login_session)>.json` (`:216-232`); refresh dalam window 5 menit (`SAAWSLoginRefresh.swift:176`); DPoP ES256 (RFC 9449) dari PEM (`:113-146`); URL token `https://<region>.<signin-domain>/v1/token` (`:73-96`); atomic replace cache (`:436-524`).

## 3. Vault (`Source/Other/Vault/`)

### 3.1 Alur login OIDC

`VaultOIDCHandler.login` (`:252`): klaim slot login eksklusif proses-wide (percobaan kedua gagal cepat `.loginInProgress` `:257, 69-82`); bind `NWListener` loopback-only port **8250** (`:247, 262-269, 478-546`); redirectURI `http://localhost:8250/oidc/callback` (`:275`); state+nonce dari `SecRandomCopyBytes` (`:279-280, 578-589`); POST `v1/auth/<mount>/oidc/auth_url` (`VaultClient.swift:175-199`); buka browser (`:295-299`); tunggu callback ≤120 s (`:46, 305-306`); GET `v1/auth/<mount>/oidc/callback?state&client_nonce&code` → token (`:334-340`; `VaultClient.swift:202-222`). Guard `callbackState == state` **sengaja dimatikan** — Vault punya state opaque sendiri + integritas di server (`:317-330`). Token dipersist di Keychain (service `Sequel Ace Vault Token`) di-key `"<baseURL>|oidc_mount=<mount>"` (`:342-356, 361-444`).

### 3.2 Generasi kredensial

`VaultAuthManager.generateCredentials(...)` (`:134/150`): assert background thread (`:160`); `buildBaseURL` (`VaultClient.swift:49-55`, default port 443); cache key `host:port@oidcMount/credPath` (`:69-73`); hormati cancellation + cached creds (eviction saat token berubah) (`:186-201`); **in-flight coalescing** per key (`:203-232`); validasi token via `tokenLookupSelf` (`:252-301`); else OIDC login (`:303-329`); lalu GET `v1/<path>` (`VaultClient.swift:225-247`) parse `data.username`/`data.password` + `lease_duration` default 3600 (`:59-68`); tolak creds kosong (`:349-356`). Margin cache 30 s (`:40`). `tokenLookupSelf` true(200)/false(400/403→re-auth)/throw(429/5xx) (`VaultClient.swift:159-168`).

### 3.3 Daftar role

`VaultAuthManager.listRoles` (`:388-463`): resolve token valid (cached else OIDC), panggil `confirmBrowserLogin` sinkron hanya saat browser benar-benar diperlukan (`:410-436`); LIST `<mount>/roles` via `VaultClient.listDatabaseRoles` (`:454-462`; `VaultClient.swift:278-302`, HTTP `LIST`, header `X-Vault-Token`). 404 disurfacekan karena "mount hilang" dan "list kosong" tak terbedakan (`:292-300`). `SAVaultRoleListController` debounce reorder 0.15 s (`:85-92`), invalidate saat konteks berubah + abort login scoped (`:103-107, 147-151`), buang respons stale (`:223-228`), disable Connect/Test selama refresh (hindari bentrok port 8250) (`:22-30, 160, 186-189`).

### 3.4 Parsing path & filter role

- `SAVaultCredentialsPath` memecah `<mount>/creds/<role>` pada `/creds/` **terakhir** (`:13, 27-31, 42-46`); `normalizeMount` trim whitespace+slash (`:19-22`); `credPath(mount:role:)` (`:50-61`).
- `SAVaultRoleFilter` fuzzy subsequence scoring (word-boundary +10, consecutive +5) (`:20-41`); ordered list = match lalu separator non-selectable `"──────────"` lalu sisanya (`:15, 47-68`).

## 4. Secure bookmark (`Source/Controllers/Other/SecureBookmark*.swift`)

- Bookmark security-scoped via `URLBookmarkCreationWithSecurityScope` (`SecureBookmarkManager.swift:30`). Model `SecureBookmarkData` (NSSecureCoding: `bookmarkData`, `options`, `bookmarkURL`) (`SecureBookmarkData.swift:11-53`); dibungkus via `NSKeyedArchiver`/`NSSecureCoding` (`SecureBookmark.swift:27, 42-62`).
- Penyimpanan `UserDefaults` key: `SASecureBookmarks` (`@"SPSecureBookmarks"`, `SPConstants.m:236`), `SPStaleSecureBookmarks` (`:237`), `SPKnownHostsBookmarks` (`:238`), backup lama `SPSecureBookmarksOldFormat` (`:239`), flag migrasi `SPSecureBookmarksHaveBeenMigrated` (`:240`). Konsumen: known-hosts SSH, direktori `~/.aws`, files/preferences.
- **Stale** = `bookmarkDataIsStale` OR gagal resolve OR `startAccessingSecurityScopedResource` gagal (`:199-217`); coba **refresh** via `refreshedEncodedBookmarkData` (`:202-207, 267-296`); gagal → `staleBookmarks` (`:206`). Volume root hilang (`/Volumes/...`) → simpan bookmark, skip aktivasi (`:230-252`). Alert stale saat launch: `SPAppController.m:207` (`NSAlertExtension.swift:52-68`).

## 5. Updater (GitHub) & analytics

### 5.1 Aturan keputusan updater
- Config dari `BundleExtension.checkForNewVersion` — repo `Sequel-Ace/Sequel-Ace`, `includePrerelease` = beta, `appVariant` beta/production; **dilewati seluruhnya untuk build MAS** (`BundleExtension.swift:76-89`; `GitHub.swift:26-35`).
- Endpoint `https://api.github.com/repos/<user>/<project>/releases?per_page=100` (`GitHubReleaseManager.swift:18, 98-99`); paginasi ≤10 halaman mengikuti `rel="next"` terbatas `https://api.github.com` (`GitHub.swift:423-466`).
- Gate: `SAGitHubReleaseCheckPolicy.shouldCheck` = `!isAppStoreInstall && (isUserInitiated || automaticChecksEnabled)` (`GitHub.swift:26-34`), automatic = pref `SPShowUpdateAvailable` (`SPConstants.m:146`). Draft/prerelease difilter (`GitHubReleaseManager.swift:218-228`); rilis tanpa zip app kompatibel dibuang (`:244`); **settling window** 15 menit (`GitHub.swift:84-85, 246-253`); **phased rollout** persen `[1,2,5,10,20,50,100]` selama 7 hari berbasis seed instalasi (`:269-362`), cek manual melewati rollout (`:295-297`). Pref supresi `SPSkipNewReleaseAvailable` (`SPConstants.m:463`).

### 5.2 Analytics consent
- Pref `SaveApplicationUsageAnalytics` (`SAAnalyticsConsentPolicy.swift:17`). Firebase dikonfigurasi **hanya bila** enabled (`:20-23`). `requestConsentIfNeeded` (`:27-39`) dipanggil di finish-launch (`SPAppController.m:197`) lalu `applyAnalyticsConsent` (`:198`).
- Ketika off → disable collection, set semua consent denied, hapus laporan terkirim; ketika on → `FirebaseApp.configure()`, hanya grant `analyticsStorage` (ad consent tetap denied), set user properties `distribution`/`release_channel` (`SAAnalyticsConsentPolicy+Firebase.swift:22-76`); TestFlight dideteksi via `sandboxReceipt` (`:63-67`).
- Endpoint privacy policy `https://moballo.com/privacy-policy/` (`SAAnalyticsConsentPolicy.swift:18`).

## 6. SADragPasteboard (`Source/Other/Utility/SADragPasteboard.swift`)

Memiliki **lima** UTI kustom (reverse-DNS karena `NSPasteboardItem`/`NSPasteboardWriting` menolak nama non-UTI — `:23-29`):

| Konstanta | Nilai | Fungsi |
| --- | --- | --- |
| `tableRowType` (`:32`) | `com.sequel-ace.pasteboard.table-row` | reorder indeks baris |
| `sslCipherType` (`:35`) | `com.sequel-ace.pasteboard.ssl-cipher` | reorder cipher SSL di pref jaringan |
| `dragRowType` (`:42`) | `com.sequel-ace.pasteboard.drag-row` | item per-baris marker-only agar drag-out dapat image per-baris tanpa payload |
| `navigatorSchemaPathsType` (`:47`) | `com.sequel-ace.pasteboard.navigator-schema-paths` | schema path keyed-archived |
| `navigatorTableDataType` (`:52`) | `com.sequel-ace.pasteboard.navigator-table-data` | `CREATE TABLE …` satu tabel navigator |

Alasan marker-only `dragRowType` (`:100-110`): `.string` digabung antar item dengan `\n`, sedangkan `.tabularText`/custom/plist hanya dari **item pertama** — jadi item per-baris harus bebas payload. Payload whole-drag dilampirkan ke item pertama via `attach(dragString:…)`/`attach(string/propertyList/data:…)` (`:131-168`).

## 7. String yang terlihat pengguna (terpilih, dikutip persis)

**SSH**: `"SSH Tunnel started without a parent window.  A parent window must be present."` (`:426`); `"No local port could be allocated for the SSH Tunnel."` (`:449`); `"Custom SSH binary enabled. Disable in Preferences to rule out incompatibilities!"` (`:469`); `"The SSH Tunnel has unexpectedly closed."` (`:753`); `"The SSH Tunnel was unable to bind to the local port. … Would you like to fall back to a standard connection to localhost…"` (`:887`); `"The SSH Tunnel was closed 'by the remote host'.…"` (`:894`); `"The SSH Tunnel could not authenticate with the remote host. Please check your password and ensure you still have access."` (`:900`); `"Enter your password for the SSH key\n\"%@\""` (`:1111`).

**AWS**: `"AWS credentials not found"` (`:47`); `"Failed to assume IAM role"` (`:54`); `"AWS profile not found"` (`AWSCredentials.swift:44`); `"The cached AWS SSO token has expired."` (`AWSSSOClient.swift:49`); `"AWS MFA Authentication Required"` (`AWSMFATokenDialog.swift:85`); `"Please enter a valid 6-digit MFA code."` (`:149`); `"Run `%@` in Terminal, then try again."` (`SAAWSSignInCommand.swift:94`).

**Vault**: `"Vault host or credentials path is missing."` (`:23`); `"Vault login was cancelled."` (`:25`); `"Another Vault sign-in is already in progress. Finish or cancel it (including in other windows) and try again."` (`VaultOIDCHandler.swift:31`); `"Vault OIDC login timed out. Please try again."` (`:33`); `"Sign in to Vault?"` (`SAVaultRoleListController.swift:138`); `"No database roles were returned for this mount.…"` (`:235`).

**Bookmark**: `"Sequel Ace found 1 stale secure bookmark. You can continue launching now, or open Files preferences to refresh or revoke it."` (`NSAlertExtension.swift:54-57`).

**Updater**: `"You are currently running the latest release."` (`GitHubReleaseManager.swift:300`); `"Version %@ is available. You are currently running %@"` (`:389`); `"Would you like to quit Sequel Ace now so you can complete the installation?"` (`:653-657`); `"Failed to unpack the update. Please try downloading it manually."` (`:677`).

**Analytics**: `"Help Improve Sequel Ace"` (`SAAnalyticsConsentPolicy.swift:80`); `"Share Analytics"` (`:91`); `"Don't Share"` (`:97`); `"Privacy Policy…"` (`:103`).

## 8. UNKNOWN

- **UNKNOWN**: plist Google mana yang menyuplai konfigurasi Firebase (tak ada DSN di file yang diperiksa).
- **UNKNOWN**: judul/body notifikasi `SANotificationCenter` (didefinisikan di call site luar himpunan yang diperiksa).
- Default transport SSH `.distributedObjects` di kode meski rilis sebelumnya mengirim `.socket`; jalur socket tetap terjangkau via pref tersembunyi + fallback assistant.
